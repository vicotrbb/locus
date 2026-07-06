#![allow(missing_docs)]

//! LOCUS-EVAL v1: locus contenders (chunk mailboxes and the naive
//! shared per-handle queue) over the four suite workloads.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use criterion::{criterion_group, criterion_main, Criterion};
use locus_alloc::NodeId;
use locus_alloc::{ChunkMailbox, KvBlockHandle, KvBlockPool, RemoteFreeQueue};

#[path = "locus_eval/workloads.rs"]
mod workloads;

use workloads::{
    Workload, BLOCK_SIZE, CHURN_CHUNK_BLOCKS, CHURN_LIVE_CHUNKS, CHURN_POOL_BLOCKS, CHURN_STEPS,
    PREFILL_BLOCKS, REQUESTS, TRACE_POOL_BLOCKS, TRACE_WORKLOADS, WORKERS,
};

const SHARED_CAPACITY: usize = 1024;
const SHARED_BATCH_LIMIT: usize = 64;

#[derive(Clone, Copy)]
enum Contender {
    Mailbox,
    Shared,
}

impl Contender {
    fn name(self) -> &'static str {
        match self {
            Contender::Mailbox => "locus_mailbox",
            Contender::Shared => "locus_shared",
        }
    }
}

enum WorkerCommand {
    Free(Vec<KvBlockHandle>),
    Stop,
}

struct Request {
    blocks: Vec<KvBlockHandle>,
    decode_steps_left: usize,
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

struct Queues {
    shared: Vec<RemoteFreeQueue<KvBlockHandle>>,
    mailboxes: Vec<ChunkMailbox<Vec<KvBlockHandle>>>,
    submitted: Arc<AtomicUsize>,
    drained: usize,
}

impl Queues {
    fn new(contender: Contender) -> Self {
        match contender {
            Contender::Shared => Self {
                shared: vec![RemoteFreeQueue::new(SHARED_CAPACITY, SHARED_BATCH_LIMIT)
                    .expect("shared queue configuration")],
                mailboxes: Vec::new(),
                submitted: Arc::new(AtomicUsize::new(0)),
                drained: 0,
            },
            Contender::Mailbox => Self {
                shared: Vec::new(),
                mailboxes: (0..WORKERS).map(|_| ChunkMailbox::new()).collect(),
                submitted: Arc::new(AtomicUsize::new(0)),
                drained: 0,
            },
        }
    }

    fn worker_free_paths(&self) -> Vec<Box<dyn FnMut(Vec<KvBlockHandle>) + Send>> {
        if self.mailboxes.is_empty() {
            (0..WORKERS)
                .map(|_| {
                    let sink = self.shared[0].sink();
                    let submitted = Arc::clone(&self.submitted);
                    Box::new(move |handles: Vec<KvBlockHandle>| {
                        submitted.fetch_add(handles.len(), Ordering::Relaxed);
                        for handle in handles {
                            sink.enqueue(handle).expect("owner queue stays alive");
                        }
                    }) as Box<dyn FnMut(Vec<KvBlockHandle>) + Send>
                })
                .collect()
        } else {
            self.mailboxes
                .iter()
                .map(|mailbox| {
                    let sender = mailbox.sender();
                    let submitted = Arc::clone(&self.submitted);
                    Box::new(move |handles: Vec<KvBlockHandle>| {
                        submitted.fetch_add(handles.len(), Ordering::Relaxed);
                        sender.push(handles);
                    }) as Box<dyn FnMut(Vec<KvBlockHandle>) + Send>
                })
                .collect()
        }
    }

    fn drain(&mut self, pool: &mut KvBlockPool) -> usize {
        let mut freed = 0_usize;
        for queue in &mut self.shared {
            let _ = queue.drain_batch(|handle| {
                pool.free(handle).expect("drained handle frees");
                freed += 1;
            });
        }
        for mailbox in &self.mailboxes {
            mailbox.take_all(|handles| {
                for handle in handles {
                    pool.free(handle).expect("drained handle frees");
                    freed += 1;
                }
            });
        }
        self.drained += freed;
        freed
    }

    fn assert_balanced(&self) {
        assert_eq!(self.submitted.load(Ordering::Relaxed), self.drained);
        for stats in self.shared.iter().map(RemoteFreeQueue::stats) {
            assert_eq!(stats.submitted_count, stats.drained_count);
            assert_eq!(stats.pending_count, 0);
            assert_eq!(stats.disconnected_count, 0);
        }
    }
}

fn allocate_with_backpressure(pool: &mut KvBlockPool, queues: &mut Queues) -> KvBlockHandle {
    loop {
        if let Ok(handle) = pool.allocate() {
            return handle;
        }
        if queues.drain(pool) == 0 {
            thread::yield_now();
        }
    }
}

struct TraceOutcome {
    allocated: usize,
    freed: usize,
    peak_outstanding: usize,
}

fn run_trace(
    workload: Workload,
    pool: &mut KvBlockPool,
    queues: &mut Queues,
    worker_pool: &WorkerPool,
) -> TraceOutcome {
    let mut active: Vec<Request> = Vec::with_capacity(REQUESTS);
    let mut arrived = 0_usize;
    let mut allocated_blocks = 0_usize;
    let mut freed_blocks = 0_usize;
    let mut peak_outstanding = 0_usize;
    let mut next_worker = 0_usize;

    while arrived < REQUESTS || !active.is_empty() {
        freed_blocks += queues.drain(pool);

        let arriving = workload.arrivals_per_step().min(REQUESTS - arrived);
        for _ in 0..arriving {
            let mut blocks = Vec::with_capacity(PREFILL_BLOCKS + workload.decode_steps(arrived));
            for _ in 0..PREFILL_BLOCKS {
                blocks.push(allocate_with_backpressure(pool, queues));
            }
            allocated_blocks += PREFILL_BLOCKS;
            active.push(Request {
                blocks,
                decode_steps_left: workload.decode_steps(arrived),
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

        freed_blocks += queues.drain(pool);
        peak_outstanding = peak_outstanding.max(allocated_blocks - freed_blocks);
    }

    while freed_blocks < allocated_blocks {
        let freed = queues.drain(pool);
        freed_blocks += freed;
        if freed == 0 {
            thread::yield_now();
        }
    }

    TraceOutcome {
        allocated: allocated_blocks,
        freed: freed_blocks,
        peak_outstanding,
    }
}

fn run_churn(
    pool: &mut KvBlockPool,
    queues: &mut Queues,
    worker_pool: &WorkerPool,
    live: &mut VecDeque<Vec<KvBlockHandle>>,
    fill: u8,
) -> TraceOutcome {
    let mut allocated_blocks = 0_usize;
    let mut freed_blocks = 0_usize;
    let mut peak_in_flight = 0_usize;
    let mut dispatched = 0_usize;
    let mut next_worker = 0_usize;

    for _ in 0..CHURN_STEPS {
        let oldest = live.pop_front().expect("steady-state chunk available");
        dispatched += oldest.len();
        worker_pool.dispatch(next_worker, oldest);
        next_worker = (next_worker + 1) % WORKERS;

        let mut chunk = Vec::with_capacity(CHURN_CHUNK_BLOCKS);
        for _ in 0..CHURN_CHUNK_BLOCKS {
            let handle = allocate_with_backpressure(pool, queues);
            pool.block_mut(handle)
                .expect("allocated block is addressable")
                .fill(fill);
            chunk.push(handle);
        }
        allocated_blocks += CHURN_CHUNK_BLOCKS;
        live.push_back(chunk);

        freed_blocks += queues.drain(pool);
        peak_in_flight = peak_in_flight.max(dispatched - freed_blocks);
    }

    while freed_blocks < dispatched {
        let freed = queues.drain(pool);
        freed_blocks += freed;
        if freed == 0 {
            thread::yield_now();
        }
    }

    TraceOutcome {
        allocated: allocated_blocks,
        freed: freed_blocks,
        peak_outstanding: CHURN_LIVE_CHUNKS * CHURN_CHUNK_BLOCKS + peak_in_flight,
    }
}

fn churn_steady_state(pool: &mut KvBlockPool) -> VecDeque<Vec<KvBlockHandle>> {
    let mut live = VecDeque::with_capacity(CHURN_LIVE_CHUNKS);
    for _ in 0..CHURN_LIVE_CHUNKS {
        let mut chunk = Vec::with_capacity(CHURN_CHUNK_BLOCKS);
        for _ in 0..CHURN_CHUNK_BLOCKS {
            let handle = pool.allocate().expect("steady-state block allocates");
            pool.block_mut(handle)
                .expect("allocated block is addressable")
                .fill(0xA5);
            chunk.push(handle);
        }
        live.push_back(chunk);
    }
    live
}

fn release_churn_state(
    pool: &mut KvBlockPool,
    queues: &mut Queues,
    live: &mut VecDeque<Vec<KvBlockHandle>>,
) {
    while let Some(chunk) = live.pop_front() {
        for handle in chunk {
            pool.free(handle).expect("steady-state block frees");
        }
    }
    let _ = queues.drain(pool);
}

fn bench_workload(c: &mut Criterion, workload: Workload, contender: Contender) {
    let pool_blocks = if workload == Workload::ChurnTouch {
        CHURN_POOL_BLOCKS
    } else {
        TRACE_POOL_BLOCKS
    };
    let mut pool =
        KvBlockPool::new(NodeId(0), BLOCK_SIZE, pool_blocks).expect("pool configuration");
    let mut queues = Queues::new(contender);
    let worker_pool = WorkerPool::spawn(queues.worker_free_paths());

    let mut live = if workload == Workload::ChurnTouch {
        churn_steady_state(&mut pool)
    } else {
        VecDeque::new()
    };

    let mut peak_outstanding = 0_usize;
    let mut totals = (0_usize, 0_usize);
    for round in 0..4_u8 {
        let outcome = if workload == Workload::ChurnTouch {
            run_churn(&mut pool, &mut queues, &worker_pool, &mut live, round)
        } else {
            run_trace(workload, &mut pool, &mut queues, &worker_pool)
        };
        assert_eq!(outcome.allocated, outcome.freed);
        assert_eq!(outcome.allocated, workload.blocks_per_trace());
        totals.0 += outcome.allocated;
        totals.1 += outcome.freed;
        peak_outstanding = peak_outstanding.max(outcome.peak_outstanding);
    }
    println!(
        "locus_eval_sample workload={} contender={} traces=4 allocated={} freed={} \
         peak_outstanding={} theoretical_peak={}",
        workload.name(),
        contender.name(),
        totals.0,
        totals.1,
        peak_outstanding,
        workload.theoretical_peak_live(),
    );

    let name = format!(
        "locus_eval_{}_{}_w{WORKERS}",
        workload.name(),
        contender.name()
    );
    c.bench_function(&name, |b| {
        b.iter(|| {
            let outcome = if workload == Workload::ChurnTouch {
                run_churn(&mut pool, &mut queues, &worker_pool, &mut live, 0x5A)
            } else {
                run_trace(workload, &mut pool, &mut queues, &worker_pool)
            };
            assert_eq!(outcome.allocated, workload.blocks_per_trace());
        });
    });

    if workload == Workload::ChurnTouch {
        release_churn_state(&mut pool, &mut queues, &mut live);
    }
    worker_pool.stop();
    let _ = queues.drain(&mut pool);
    queues.assert_balanced();
}

fn bench_locus_eval(c: &mut Criterion) {
    for contender in [Contender::Mailbox, Contender::Shared] {
        for workload in TRACE_WORKLOADS {
            bench_workload(c, workload, contender);
        }
        bench_workload(c, Workload::ChurnTouch, contender);
    }
}

criterion_group!(benches, bench_locus_eval);
criterion_main!(benches);
