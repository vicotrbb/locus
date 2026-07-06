#![allow(missing_docs)]

use std::sync::mpsc::{sync_channel, SyncSender};
use std::thread::{self, JoinHandle};

use criterion::{criterion_group, criterion_main, Criterion};
use locus::NodeId;
use locus::{KvBlockHandle, KvBlockPool, RemoteFreeQueue, RemoteFreeSink};

const BLOCK_SIZE: usize = 4096;
const BLOCKS: usize = 256;
const BATCH_LIMIT: usize = 64;
const CHUNK_QUEUE_CAPACITY: usize = 2;
const PRODUCER_COUNTS: [usize; 2] = [2, 4];

enum ProducerCommand {
    Run(Vec<KvBlockHandle>),
    Stop,
}

enum PublishMode {
    PerHandle,
    Chunk,
}

struct ProducerPool {
    commands: Vec<SyncSender<ProducerCommand>>,
    workers: Vec<JoinHandle<()>>,
}

impl ProducerPool {
    fn spawn_per_handle(sinks: Vec<RemoteFreeSink<KvBlockHandle>>) -> Self {
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

    fn spawn_chunk(sinks: Vec<RemoteFreeSink<Vec<KvBlockHandle>>>) -> Self {
        let mut commands = Vec::with_capacity(sinks.len());
        let mut workers = Vec::with_capacity(sinks.len());

        for producer_sink in sinks {
            let (command_sender, command_receiver) = sync_channel::<ProducerCommand>(1);
            commands.push(command_sender);
            workers.push(thread::spawn(move || {
                while let Ok(ProducerCommand::Run(handles)) = command_receiver.recv() {
                    producer_sink
                        .enqueue(handles)
                        .expect("owner queue stays alive during run");
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

fn run_per_handle_cycle(
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

fn run_chunk_cycle(
    pool: &mut KvBlockPool,
    queues: &mut [RemoteFreeQueue<Vec<KvBlockHandle>>],
    producer_pool: &ProducerPool,
    producers: usize,
) {
    let chunks = allocate_chunks(pool, producers);
    producer_pool.run(chunks);

    let mut freed = 0_usize;
    while freed < BLOCKS {
        let mut round_freed = 0_usize;
        for queue in queues.iter_mut() {
            let mut freed_in_queue = 0_usize;
            let _ = queue.drain_batch(|handles| {
                for handle in handles {
                    pool.free(handle).expect("drained handle frees");
                    freed_in_queue += 1;
                }
            });
            round_freed += freed_in_queue;
        }
        freed += round_freed;
        if round_freed == 0 {
            thread::yield_now();
        }
    }
}

fn print_case_stats(producers: usize, mode: &PublishMode) {
    let mut pool = KvBlockPool::new(NodeId(0), BLOCK_SIZE, BLOCKS).expect("pool configuration");

    match mode {
        PublishMode::PerHandle => {
            let mut queues: Vec<RemoteFreeQueue<KvBlockHandle>> = (0..producers)
                .map(|_| {
                    RemoteFreeQueue::new(BLOCKS / producers, BATCH_LIMIT)
                        .expect("shard queue configuration")
                })
                .collect();
            let producer_pool =
                ProducerPool::spawn_per_handle(queues.iter().map(RemoteFreeQueue::sink).collect());
            for _ in 0..8 {
                run_per_handle_cycle(&mut pool, &mut queues, &producer_pool, producers);
            }
            producer_pool.stop();
            report_queue_stats(
                "per_handle",
                producers,
                queues.iter().map(RemoteFreeQueue::stats),
            );
        }
        PublishMode::Chunk => {
            let mut queues: Vec<RemoteFreeQueue<Vec<KvBlockHandle>>> = (0..producers)
                .map(|_| {
                    RemoteFreeQueue::new(CHUNK_QUEUE_CAPACITY, CHUNK_QUEUE_CAPACITY)
                        .expect("chunk queue configuration")
                })
                .collect();
            let producer_pool =
                ProducerPool::spawn_chunk(queues.iter().map(RemoteFreeQueue::sink).collect());
            for _ in 0..8 {
                run_chunk_cycle(&mut pool, &mut queues, &producer_pool, producers);
            }
            producer_pool.stop();
            report_queue_stats(
                "chunk",
                producers,
                queues.iter().map(RemoteFreeQueue::stats),
            );
        }
    }
}

fn report_queue_stats(
    mode: &str,
    producers: usize,
    stats_iter: impl Iterator<Item = locus::RemoteFreeQueueStats>,
) {
    let mut total_submitted = 0_u64;
    for (index, stats) in stats_iter.enumerate() {
        println!(
            "chunk_publish_sample mode={mode} producers={producers} queue={index} \
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
    }
    let expected_items = match mode {
        "per_handle" => 8 * BLOCKS as u64,
        _ => 8 * producers as u64,
    };
    assert_eq!(total_submitted, expected_items);
}

fn bench_remote_free_chunk_publish(c: &mut Criterion) {
    for producers in PRODUCER_COUNTS {
        for mode in [PublishMode::PerHandle, PublishMode::Chunk] {
            print_case_stats(producers, &mode);

            let mut pool =
                KvBlockPool::new(NodeId(0), BLOCK_SIZE, BLOCKS).expect("pool configuration");

            match mode {
                PublishMode::PerHandle => {
                    let mut queues: Vec<RemoteFreeQueue<KvBlockHandle>> = (0..producers)
                        .map(|_| {
                            RemoteFreeQueue::new(BLOCKS / producers, BATCH_LIMIT)
                                .expect("shard queue configuration")
                        })
                        .collect();
                    let producer_pool = ProducerPool::spawn_per_handle(
                        queues.iter().map(RemoteFreeQueue::sink).collect(),
                    );
                    let name = format!(
                        "kv_remote_free_chunkcmp_per_handle_p{producers}_batch{BATCH_LIMIT}_{BLOCKS}x4k"
                    );
                    c.bench_function(&name, |b| {
                        b.iter(|| {
                            run_per_handle_cycle(&mut pool, &mut queues, &producer_pool, producers);
                        });
                    });
                    producer_pool.stop();
                }
                PublishMode::Chunk => {
                    let mut queues: Vec<RemoteFreeQueue<Vec<KvBlockHandle>>> = (0..producers)
                        .map(|_| {
                            RemoteFreeQueue::new(CHUNK_QUEUE_CAPACITY, CHUNK_QUEUE_CAPACITY)
                                .expect("chunk queue configuration")
                        })
                        .collect();
                    let producer_pool = ProducerPool::spawn_chunk(
                        queues.iter().map(RemoteFreeQueue::sink).collect(),
                    );
                    let name = format!("kv_remote_free_chunkcmp_chunk_p{producers}_{BLOCKS}x4k");
                    c.bench_function(&name, |b| {
                        b.iter(|| {
                            run_chunk_cycle(&mut pool, &mut queues, &producer_pool, producers);
                        });
                    });
                    producer_pool.stop();
                }
            }
        }
    }
}

criterion_group!(benches, bench_remote_free_chunk_publish);
criterion_main!(benches);
