#![allow(missing_docs)]

use std::sync::mpsc::{sync_channel, SyncSender};
use std::thread::{self, JoinHandle};

use criterion::{criterion_group, criterion_main, Criterion};
use locus::NodeId;
use locus::{KvBlockHandle, KvBlockPool, RemoteFreeQueue, RemoteFreeSink};

const BLOCK_SIZE: usize = 4096;
const BLOCKS: usize = 256;
const QUEUE_CAPACITY: usize = 256;
const PRODUCER_COUNTS: [usize; 3] = [1, 2, 4];
const BATCH_LIMITS: [usize; 3] = [8, 64, 256];

enum ProducerCommand {
    Run(Vec<KvBlockHandle>),
    Stop,
}

struct ProducerPool {
    commands: Vec<SyncSender<ProducerCommand>>,
    workers: Vec<JoinHandle<()>>,
}

impl ProducerPool {
    fn spawn(count: usize, sink: &RemoteFreeSink<KvBlockHandle>) -> Self {
        let mut commands = Vec::with_capacity(count);
        let mut workers = Vec::with_capacity(count);

        for _ in 0..count {
            let (command_sender, command_receiver) = sync_channel::<ProducerCommand>(1);
            let producer_sink = sink.clone();
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

fn run_concurrent_cycle(
    pool: &mut KvBlockPool,
    queue: &mut RemoteFreeQueue<KvBlockHandle>,
    producer_pool: &ProducerPool,
    producers: usize,
) {
    let chunks = allocate_chunks(pool, producers);
    producer_pool.run(chunks);

    let mut received = 0_usize;
    while received < BLOCKS {
        let drain = queue.drain_batch(|handle| {
            pool.free(handle).expect("drained handle frees");
        });
        received += drain.drained;
        if drain.drained == 0 {
            thread::yield_now();
        }
    }
}

fn print_case_stats(producers: usize, batch_limit: usize) {
    let mut pool = KvBlockPool::new(NodeId(0), BLOCK_SIZE, BLOCKS).expect("pool configuration");
    let mut queue = RemoteFreeQueue::new(QUEUE_CAPACITY, batch_limit).expect("queue configuration");
    let producer_pool = ProducerPool::spawn(producers, &queue.sink());

    for _ in 0..8 {
        run_concurrent_cycle(&mut pool, &mut queue, &producer_pool, producers);
    }

    producer_pool.stop();
    let stats = queue.stats();
    println!(
        "concurrent_sample producers={producers} batch_limit={batch_limit} \
         submitted={} drained={} pending={} full={} disconnected={}",
        stats.submitted_count,
        stats.drained_count,
        stats.pending_count,
        stats.full_count,
        stats.disconnected_count,
    );
    assert_eq!(stats.submitted_count, stats.drained_count);
    assert_eq!(stats.pending_count, 0);
}

fn bench_remote_free_concurrent(c: &mut Criterion) {
    for producers in PRODUCER_COUNTS {
        for batch_limit in BATCH_LIMITS {
            print_case_stats(producers, batch_limit);

            let mut pool =
                KvBlockPool::new(NodeId(0), BLOCK_SIZE, BLOCKS).expect("pool configuration");
            let mut queue =
                RemoteFreeQueue::new(QUEUE_CAPACITY, batch_limit).expect("queue configuration");
            let producer_pool = ProducerPool::spawn(producers, &queue.sink());

            let name =
                format!("kv_remote_free_concurrent_p{producers}_batch{batch_limit}_{BLOCKS}x4k");
            c.bench_function(&name, |b| {
                b.iter(|| {
                    run_concurrent_cycle(&mut pool, &mut queue, &producer_pool, producers);
                });
            });

            producer_pool.stop();
        }
    }
}

criterion_group!(benches, bench_remote_free_concurrent);
criterion_main!(benches);
