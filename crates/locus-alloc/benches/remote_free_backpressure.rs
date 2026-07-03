#![allow(missing_docs)]

use std::{sync::mpsc::sync_channel, thread};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use locus_alloc::{
    RemoteFreeQueue, RemoteFreeQueueStats, RemoteFreeSink, RemoteFreeTryEnqueueErrorKind,
};

enum ProducerCommand {
    Run(usize),
    Stop,
}

fn remote_free_try_enqueue_backpressure_batch8(c: &mut Criterion) {
    print_backpressure_sample("batch8", 8);
    remote_free_try_enqueue_backpressure(
        c,
        "remote_free_try_enqueue_backpressure_256x4k_batch8",
        8,
    );
}

fn remote_free_try_enqueue_backpressure_batch64(c: &mut Criterion) {
    print_backpressure_sample("batch64", 64);
    remote_free_try_enqueue_backpressure(
        c,
        "remote_free_try_enqueue_backpressure_256x4k_batch64",
        64,
    );
}

fn remote_free_try_enqueue_backpressure(c: &mut Criterion, name: &'static str, batch_limit: usize) {
    c.bench_function(name, |bench| {
        let mut queue = RemoteFreeQueue::new(batch_limit, batch_limit).expect("queue");
        let sink = queue.sink();
        let (command_sender, command_receiver) = sync_channel::<ProducerCommand>(1);

        let producer = thread::spawn(move || {
            while let Ok(command) = command_receiver.recv() {
                match command {
                    ProducerCommand::Run(blocks) => produce_blocks(&sink, blocks),
                    ProducerCommand::Stop => break,
                }
            }
        });

        bench.iter(|| {
            command_sender
                .send(ProducerCommand::Run(256))
                .expect("send run");

            let mut released = 0_usize;
            while released < 256 {
                let stats = queue.drain_batch(|mut block| {
                    black_box(block.as_mut_ptr());
                });
                if stats.drained == 0 {
                    thread::yield_now();
                }
                released += stats.drained;
            }

            let stats = queue.stats();
            assert_eq!(stats.pending_count, 0);
            black_box(stats);
        });

        command_sender
            .send(ProducerCommand::Stop)
            .expect("send stop");
        producer.join().expect("producer thread");
    });
}

fn print_backpressure_sample(label: &'static str, batch_limit: usize) {
    let stats = run_backpressure_sample(batch_limit);
    println!(
        "remote_free_backpressure_sample={label} blocks=256 batch_limit={batch_limit} submitted_count={} drained_count={} pending_count={} full_count={} disconnected_count={}",
        stats.submitted_count,
        stats.drained_count,
        stats.pending_count,
        stats.full_count,
        stats.disconnected_count
    );
}

fn run_backpressure_sample(batch_limit: usize) -> RemoteFreeQueueStats {
    let mut queue = RemoteFreeQueue::new(batch_limit, batch_limit).expect("queue");
    let sink = queue.sink();
    let producer = thread::spawn(move || produce_blocks(&sink, 256));

    let mut released = 0_usize;
    while released < 256 {
        let stats = queue.drain_batch(|mut block| {
            black_box(block.as_mut_ptr());
        });
        if stats.drained == 0 {
            thread::yield_now();
        }
        released += stats.drained;
    }

    producer.join().expect("producer thread");
    queue.stats()
}

fn produce_blocks(sink: &RemoteFreeSink<Vec<u8>>, blocks: usize) {
    for _ in 0..blocks {
        let mut block = vec![0_u8; 4096];

        loop {
            match sink.try_enqueue(block) {
                Ok(()) => break,
                Err(error) if error.kind() == RemoteFreeTryEnqueueErrorKind::Full => {
                    block = error.into_item();
                    thread::yield_now();
                }
                Err(error) => panic!("remote free enqueue failed: {error}"),
            }
        }
    }
}

criterion_group!(
    benches,
    remote_free_try_enqueue_backpressure_batch8,
    remote_free_try_enqueue_backpressure_batch64
);
criterion_main!(benches);
