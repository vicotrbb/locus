#![allow(missing_docs)]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use criterion::{criterion_group, criterion_main, Criterion};
use locus_alloc::{ChunkMailbox, KvBlockHandle, KvBlockPool, RemoteFreeQueue};
use locus_core::NodeId;

const BLOCK_SIZE: usize = 4096;
const POOL_BLOCKS: usize = 4096;
const REQUESTS: usize = 64;
const ARRIVALS_PER_STEP: usize = 4;
const PREFILL_BLOCKS: usize = 16;
const CANCEL_DECODE_STEPS: usize = 8;
const WORKERS: usize = 4;
const SHARED_CAPACITY: usize = 1024;
const SHARED_BATCH_LIMIT: usize = 64;
const CHUNK_CAPACITY: usize = 8;

enum WorkerCommand {
    Free(Vec<KvBlockHandle>),
    Stop,
}

enum FreePath {
    SharedPerHandle,
    ShardedChunk,
    ShardedMailbox,
}

struct Request {
    blocks: Vec<KvBlockHandle>,
    decode_steps_left: usize,
}

fn decode_steps(request_index: usize) -> usize {
    if request_index % 4 == 0 {
        CANCEL_DECODE_STEPS
    } else {
        16 + (request_index % 3) * 16
    }
}

struct WorkerPool {
    commands: Vec<SyncSender<WorkerCommand>>,
    workers: Vec<JoinHandle<()>>,
}

impl WorkerPool {
    fn spawn(free_paths: Vec<Box<dyn FnMut(Vec<KvBlockHandle>) + Send>>) -> Self {
        let mut commands = Vec::with_capacity(free_paths.len());
        let mut workers = Vec::with_capacity(free_paths.len());

        for mut free_path in free_paths {
            let (command_sender, command_receiver): (
                SyncSender<WorkerCommand>,
                Receiver<WorkerCommand>,
            ) = sync_channel(REQUESTS);
            commands.push(command_sender);
            workers.push(thread::spawn(move || {
                while let Ok(WorkerCommand::Free(handles)) = command_receiver.recv() {
                    free_path(handles);
                }
            }));
        }

        Self { commands, workers }
    }

    fn dispatch(&self, worker: usize, handles: Vec<KvBlockHandle>) {
        self.commands[worker]
            .send(WorkerCommand::Free(handles))
            .expect("worker accepts free command");
    }

    fn stop(self) {
        for command in &self.commands {
            command
                .send(WorkerCommand::Stop)
                .expect("worker accepts stop command");
        }
        for worker in self.workers {
            worker.join().expect("worker thread joins");
        }
    }
}

struct TraceQueues {
    per_handle: Vec<RemoteFreeQueue<KvBlockHandle>>,
    chunk: Vec<RemoteFreeQueue<Vec<KvBlockHandle>>>,
    mailboxes: Vec<ChunkMailbox<Vec<KvBlockHandle>>>,
    mailbox_submitted: Arc<AtomicUsize>,
    mailbox_drained: usize,
}

impl TraceQueues {
    fn new(path: &FreePath) -> Self {
        match path {
            FreePath::SharedPerHandle => Self {
                per_handle: vec![RemoteFreeQueue::new(SHARED_CAPACITY, SHARED_BATCH_LIMIT)
                    .expect("shared queue configuration")],
                chunk: Vec::new(),
                mailboxes: Vec::new(),
                mailbox_submitted: Arc::new(AtomicUsize::new(0)),
                mailbox_drained: 0,
            },
            FreePath::ShardedChunk => Self {
                per_handle: Vec::new(),
                chunk: (0..WORKERS)
                    .map(|_| {
                        RemoteFreeQueue::new(CHUNK_CAPACITY, CHUNK_CAPACITY)
                            .expect("chunk queue configuration")
                    })
                    .collect(),
                mailboxes: Vec::new(),
                mailbox_submitted: Arc::new(AtomicUsize::new(0)),
                mailbox_drained: 0,
            },
            FreePath::ShardedMailbox => Self {
                per_handle: Vec::new(),
                chunk: Vec::new(),
                mailboxes: (0..WORKERS).map(|_| ChunkMailbox::new()).collect(),
                mailbox_submitted: Arc::new(AtomicUsize::new(0)),
                mailbox_drained: 0,
            },
        }
    }

    fn worker_free_paths(&self) -> Vec<Box<dyn FnMut(Vec<KvBlockHandle>) + Send>> {
        if !self.mailboxes.is_empty() {
            return self
                .mailboxes
                .iter()
                .map(|mailbox| {
                    let sender = mailbox.sender();
                    let submitted = Arc::clone(&self.mailbox_submitted);
                    Box::new(move |handles: Vec<KvBlockHandle>| {
                        submitted.fetch_add(handles.len(), Ordering::Relaxed);
                        sender.push(handles);
                    }) as Box<dyn FnMut(Vec<KvBlockHandle>) + Send>
                })
                .collect();
        }
        if self.chunk.is_empty() {
            (0..WORKERS)
                .map(|_| {
                    let sink = self.per_handle[0].sink();
                    Box::new(move |handles: Vec<KvBlockHandle>| {
                        for handle in handles {
                            sink.enqueue(handle).expect("owner queue stays alive");
                        }
                    }) as Box<dyn FnMut(Vec<KvBlockHandle>) + Send>
                })
                .collect()
        } else {
            self.chunk
                .iter()
                .map(|queue| {
                    let sink = queue.sink();
                    Box::new(move |handles: Vec<KvBlockHandle>| {
                        sink.enqueue(handles).expect("owner queue stays alive");
                    }) as Box<dyn FnMut(Vec<KvBlockHandle>) + Send>
                })
                .collect()
        }
    }

    fn drain(&mut self, pool: &mut KvBlockPool) -> usize {
        let mut freed = 0_usize;
        for queue in &mut self.per_handle {
            let _ = queue.drain_batch(|handle| {
                pool.free(handle).expect("drained handle frees");
                freed += 1;
            });
        }
        for queue in &mut self.chunk {
            let _ = queue.drain_batch(|handles| {
                for handle in handles {
                    pool.free(handle).expect("drained handle frees");
                    freed += 1;
                }
            });
        }
        let mut mailbox_freed = 0_usize;
        for mailbox in &self.mailboxes {
            mailbox.take_all(|handles| {
                for handle in handles {
                    pool.free(handle).expect("drained handle frees");
                    mailbox_freed += 1;
                }
            });
        }
        self.mailbox_drained += mailbox_freed;
        freed + mailbox_freed
    }

    fn assert_balanced(&self) {
        if !self.mailboxes.is_empty() {
            assert_eq!(
                self.mailbox_submitted.load(Ordering::Relaxed),
                self.mailbox_drained
            );
        }
        for queue in self
            .per_handle
            .iter()
            .map(RemoteFreeQueue::stats)
            .chain(self.chunk.iter().map(RemoteFreeQueue::stats))
        {
            assert_eq!(queue.submitted_count, queue.drained_count);
            assert_eq!(queue.pending_count, 0);
            assert_eq!(queue.disconnected_count, 0);
        }
    }
}

fn allocate_with_backpressure(pool: &mut KvBlockPool, queues: &mut TraceQueues) -> KvBlockHandle {
    loop {
        if let Ok(handle) = pool.allocate() {
            return handle;
        }
        if queues.drain(pool) == 0 {
            thread::yield_now();
        }
    }
}

fn run_trace(
    pool: &mut KvBlockPool,
    queues: &mut TraceQueues,
    worker_pool: &WorkerPool,
) -> (usize, usize) {
    let mut active: Vec<Request> = Vec::with_capacity(REQUESTS);
    let mut arrived = 0_usize;
    let mut allocated_blocks = 0_usize;
    let mut freed_blocks = 0_usize;
    let mut next_worker = 0_usize;

    while arrived < REQUESTS || !active.is_empty() {
        freed_blocks += queues.drain(pool);

        let arriving = ARRIVALS_PER_STEP.min(REQUESTS - arrived);
        for _ in 0..arriving {
            let mut blocks = Vec::with_capacity(PREFILL_BLOCKS + decode_steps(arrived));
            for _ in 0..PREFILL_BLOCKS {
                blocks.push(allocate_with_backpressure(pool, queues));
            }
            allocated_blocks += PREFILL_BLOCKS;
            active.push(Request {
                blocks,
                decode_steps_left: decode_steps(arrived),
            });
            arrived += 1;
        }

        let mut index = 0;
        while index < active.len() {
            if active[index].decode_steps_left == 0 {
                let request = active.swap_remove(index);
                worker_pool.dispatch(next_worker, request.blocks);
                next_worker = (next_worker + 1) % WORKERS;
                continue;
            }
            let handle = allocate_with_backpressure(pool, queues);
            active[index].blocks.push(handle);
            allocated_blocks += 1;
            active[index].decode_steps_left -= 1;
            index += 1;
        }
    }

    while freed_blocks < allocated_blocks {
        let freed = queues.drain(pool);
        freed_blocks += freed;
        if freed == 0 {
            thread::yield_now();
        }
    }

    (allocated_blocks, freed_blocks)
}

fn path_name(path: &FreePath) -> &'static str {
    match path {
        FreePath::SharedPerHandle => "shared_per_handle",
        FreePath::ShardedChunk => "sharded_chunk",
        FreePath::ShardedMailbox => "sharded_mailbox",
    }
}

fn print_case_stats(path: &FreePath) {
    let mut pool =
        KvBlockPool::new(NodeId(0), BLOCK_SIZE, POOL_BLOCKS).expect("pool configuration");
    let mut queues = TraceQueues::new(path);
    let worker_pool = WorkerPool::spawn(queues.worker_free_paths());

    let mut totals = (0_usize, 0_usize);
    for _ in 0..4 {
        let (allocated, freed) = run_trace(&mut pool, &mut queues, &worker_pool);
        assert_eq!(allocated, freed);
        totals.0 += allocated;
        totals.1 += freed;
    }

    worker_pool.stop();
    queues.assert_balanced();
    println!(
        "mixed_lifetime_sample path={} traces=4 allocated={} freed={}",
        path_name(path),
        totals.0,
        totals.1,
    );
}

fn bench_remote_free_mixed_lifetime_trace(c: &mut Criterion) {
    for path in [
        FreePath::SharedPerHandle,
        FreePath::ShardedChunk,
        FreePath::ShardedMailbox,
    ] {
        print_case_stats(&path);

        let mut pool =
            KvBlockPool::new(NodeId(0), BLOCK_SIZE, POOL_BLOCKS).expect("pool configuration");
        let mut queues = TraceQueues::new(&path);
        let worker_pool = WorkerPool::spawn(queues.worker_free_paths());

        let name = format!(
            "kv_remote_free_mixed_lifetime_{}_w{WORKERS}_{REQUESTS}req",
            path_name(&path)
        );
        c.bench_function(&name, |b| {
            b.iter(|| {
                let (allocated, freed) = run_trace(&mut pool, &mut queues, &worker_pool);
                assert_eq!(allocated, freed);
            });
        });

        worker_pool.stop();
        queues.assert_balanced();
    }
}

criterion_group!(benches, bench_remote_free_mixed_lifetime_trace);
criterion_main!(benches);
