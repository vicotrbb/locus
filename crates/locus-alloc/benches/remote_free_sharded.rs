#![allow(missing_docs)]

use std::sync::mpsc::{sync_channel, SyncSender};
use std::thread::{self, JoinHandle};

use criterion::{criterion_group, criterion_main, Criterion};
use locus_alloc::NodeId;
use locus_alloc::{KvBlockHandle, KvBlockPool, RemoteFreeQueue, RemoteFreeSink};

const BLOCK_SIZE: usize = 4096;
const BLOCKS: usize = 256;
const BATCH_LIMIT: usize = 64;
const PRODUCER_COUNTS: [usize; 2] = [2, 4];

enum ProducerCommand {
    Run(Vec<KvBlockHandle>),
    Stop,
}

struct ProducerPool {
    commands: Vec<SyncSender<ProducerCommand>>,
    workers: Vec<JoinHandle<()>>,
}

impl ProducerPool {
    fn spawn(sinks: Vec<RemoteFreeSink<KvBlockHandle>>) -> Self {
        let mut commands = Vec::with_capacity(sinks.len());
        let mut workers = Vec::with_capacity(sinks.len());

        for producer_sink in sinks {
            let (command_sender, command_receiver) = sync_channel::<ProducerCommand>(1);
            commands.push(command_sender);
            workers.push(thread::spawn(move || {
                while let Ok(ProducerCommand::Run(handles)) = command_receiver.recv() {
                    for handle in handles {
                        producer_sink
                            .enqueue(handle)
                            .expect("owner queue stays alive during run");
                    }
                }
            }));
        }

        Self { commands, workers }
    }

    fn run(&self, chunks: Vec<Vec<KvBlockHandle>>) {
        assert_eq!(chunks.len(), self.commands.len());
        for (command, chunk) in self.commands.iter().zip(chunks) {
            command
                .send(ProducerCommand::Run(chunk))
                .expect("producer accepts run command");
        }
    }

    fn stop(self) {
        for command in &self.commands {
            command
                .send(ProducerCommand::Stop)
                .expect("producer accepts stop command");
        }
        for worker in self.workers {
            worker.join().expect("producer thread joins");
        }
    }
}

fn allocate_chunks(pool: &mut KvBlockPool, producers: usize) -> Vec<Vec<KvBlockHandle>> {
    let per_producer = BLOCKS / producers;
    (0..producers)
        .map(|_| {
            (0..per_producer)
                .map(|_| pool.allocate().expect("pool has free blocks"))
                .collect()
        })
        .collect()
}

fn run_cycle(
    pool: &mut KvBlockPool,
    queues: &mut [RemoteFreeQueue<KvBlockHandle>],
    producer_pool: &ProducerPool,
    producers: usize,
) {
    let chunks = allocate_chunks(pool, producers);
    producer_pool.run(chunks);

    let mut received = 0_usize;
    while received < BLOCKS {
        let mut round_drained = 0_usize;
        for queue in queues.iter_mut() {
            let drain = queue.drain_batch(|handle| {
                pool.free(handle).expect("drained handle frees");
            });
            round_drained += drain.drained;
        }
        received += round_drained;
        if round_drained == 0 {
            thread::yield_now();
        }
    }
}

fn build_queues(producers: usize, sharded: bool) -> Vec<RemoteFreeQueue<KvBlockHandle>> {
    if sharded {
        (0..producers)
            .map(|_| {
                RemoteFreeQueue::new(BLOCKS / producers, BATCH_LIMIT)
                    .expect("shard queue configuration")
            })
            .collect()
    } else {
        vec![RemoteFreeQueue::new(BLOCKS, BATCH_LIMIT).expect("shared queue configuration")]
    }
}

fn sinks_for(
    queues: &[RemoteFreeQueue<KvBlockHandle>],
    producers: usize,
) -> Vec<RemoteFreeSink<KvBlockHandle>> {
    if queues.len() == 1 {
        (0..producers).map(|_| queues[0].sink()).collect()
    } else {
        queues.iter().map(RemoteFreeQueue::sink).collect()
    }
}

fn print_case_stats(producers: usize, sharded: bool) {
    let mut pool = KvBlockPool::new(NodeId(0), BLOCK_SIZE, BLOCKS).expect("pool configuration");
    let mut queues = build_queues(producers, sharded);
    let producer_pool = ProducerPool::spawn(sinks_for(&queues, producers));

    for _ in 0..8 {
        run_cycle(&mut pool, &mut queues, &producer_pool, producers);
    }

    producer_pool.stop();
    let mode = if sharded { "sharded" } else { "shared" };
    let mut total_submitted = 0_u64;
    let mut total_drained = 0_u64;
    for (index, queue) in queues.iter().enumerate() {
        let stats = queue.stats();
        println!(
            "sharded_sample mode={mode} producers={producers} queue={index} \
             submitted={} drained={} pending={} full={} disconnected={}",
            stats.submitted_count,
            stats.drained_count,
            stats.pending_count,
            stats.full_count,
            stats.disconnected_count,
        );
        assert_eq!(stats.submitted_count, stats.drained_count);
        assert_eq!(stats.pending_count, 0);
        total_submitted += stats.submitted_count;
        total_drained += stats.drained_count;
    }
    assert_eq!(total_submitted, total_drained);
    assert_eq!(total_submitted, 8 * BLOCKS as u64);
}

fn bench_remote_free_sharded(c: &mut Criterion) {
    for producers in PRODUCER_COUNTS {
        for sharded in [false, true] {
            print_case_stats(producers, sharded);

            let mut pool =
                KvBlockPool::new(NodeId(0), BLOCK_SIZE, BLOCKS).expect("pool configuration");
            let mut queues = build_queues(producers, sharded);
            let producer_pool = ProducerPool::spawn(sinks_for(&queues, producers));

            let mode = if sharded { "sharded" } else { "shared" };
            let name = format!("kv_remote_free_{mode}_p{producers}_batch{BATCH_LIMIT}_{BLOCKS}x4k");
            c.bench_function(&name, |b| {
                b.iter(|| {
                    run_cycle(&mut pool, &mut queues, &producer_pool, producers);
                });
            });

            producer_pool.stop();
        }
    }
}

criterion_group!(benches, bench_remote_free_sharded);
criterion_main!(benches);
