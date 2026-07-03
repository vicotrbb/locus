#![allow(missing_docs)]

use std::{sync::mpsc::sync_channel, thread};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use locus_alloc::{
    RemoteFreeQueue, RemoteFreeQueueStats, RemoteFreeSink, RemoteFreeTryEnqueueErrorKind,
};

const MIXED_TRACE_BLOCKS: u64 = 256;
const MIXED_TRACE_BURSTS: u64 = 8;
const MIXED_TRACE_BURST_BLOCKS: u64 = 32;
const MIXED_TRACE_DRAIN_EVERY_BURSTS: u64 = MIXED_TRACE_BURSTS;

enum ProducerCommand {
    Run(usize),
    Stop,
}

#[derive(Debug, Clone, Copy)]
struct CounterSummary {
    min: u64,
    max: u64,
    sum: u64,
}

#[derive(Debug)]
struct MixedTraceBlock {
    submit_burst: u64,
    allocation: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
struct MixedTraceStats {
    submitted_count: u64,
    drained_count: u64,
    full_count: u64,
    forced_drains: u64,
    drain_rounds: u64,
    max_pending_count: u64,
    max_wait_bursts: u64,
    total_wait_bursts: u64,
}

impl CounterSummary {
    fn new() -> Self {
        Self {
            min: u64::MAX,
            max: 0,
            sum: 0,
        }
    }

    fn observe(&mut self, value: u64) {
        self.min = self.min.min(value);
        self.max = self.max.max(value);
        self.sum = self.sum.saturating_add(value);
    }

    fn mean_milli(self, samples: u64) -> u64 {
        self.sum.saturating_mul(1000) / samples
    }
}

impl MixedTraceStats {
    fn mean_wait_milli(self) -> u64 {
        if self.drained_count == 0 {
            return 0;
        }

        self.total_wait_bursts.saturating_mul(1000) / self.drained_count
    }
}

fn remote_free_try_enqueue_backpressure_batch8(c: &mut Criterion) {
    print_backpressure_sample("batch8", 8, 8);
    remote_free_try_enqueue_backpressure(
        c,
        "remote_free_try_enqueue_backpressure_256x4k_batch8",
        8,
        8,
    );
}

fn remote_free_try_enqueue_backpressure_capacity8_batch64(c: &mut Criterion) {
    print_backpressure_sample("capacity8_batch64", 8, 64);
    remote_free_try_enqueue_backpressure(
        c,
        "remote_free_try_enqueue_backpressure_256x4k_capacity8_batch64",
        8,
        64,
    );
}

fn remote_free_try_enqueue_backpressure_capacity64_batch8(c: &mut Criterion) {
    print_backpressure_sample("capacity64_batch8", 64, 8);
    remote_free_try_enqueue_backpressure(
        c,
        "remote_free_try_enqueue_backpressure_256x4k_capacity64_batch8",
        64,
        8,
    );
}

fn remote_free_try_enqueue_backpressure_batch64(c: &mut Criterion) {
    print_backpressure_sample("batch64", 64, 64);
    remote_free_try_enqueue_backpressure(
        c,
        "remote_free_try_enqueue_backpressure_256x4k_batch64",
        64,
        64,
    );
}

fn remote_free_try_enqueue_backpressure_capacity128_batch64(c: &mut Criterion) {
    print_backpressure_sample("capacity128_batch64", 128, 64);
    remote_free_try_enqueue_backpressure(
        c,
        "remote_free_try_enqueue_backpressure_256x4k_capacity128_batch64",
        128,
        64,
    );
}

fn remote_free_try_enqueue_backpressure_capacity256_batch64(c: &mut Criterion) {
    print_backpressure_sample("capacity256_batch64", 256, 64);
    remote_free_try_enqueue_backpressure(
        c,
        "remote_free_try_enqueue_backpressure_256x4k_capacity256_batch64",
        256,
        64,
    );
}

fn remote_free_mixed_trace_capacity64_batch64(c: &mut Criterion) {
    print_mixed_trace_sample("capacity64_batch64", 64, 64);
    remote_free_mixed_trace(
        c,
        "remote_free_mixed_trace_256x4k_capacity64_batch64",
        64,
        64,
    );
}

fn remote_free_mixed_trace_capacity128_batch64(c: &mut Criterion) {
    print_mixed_trace_sample("capacity128_batch64", 128, 64);
    remote_free_mixed_trace(
        c,
        "remote_free_mixed_trace_256x4k_capacity128_batch64",
        128,
        64,
    );
}

fn remote_free_mixed_trace_capacity256_batch64(c: &mut Criterion) {
    print_mixed_trace_sample("capacity256_batch64", 256, 64);
    remote_free_mixed_trace(
        c,
        "remote_free_mixed_trace_256x4k_capacity256_batch64",
        256,
        64,
    );
}

fn remote_free_try_enqueue_backpressure(
    c: &mut Criterion,
    name: &'static str,
    capacity: usize,
    batch_limit: usize,
) {
    c.bench_function(name, |bench| {
        let mut queue = RemoteFreeQueue::new(capacity, batch_limit).expect("queue");
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

fn remote_free_mixed_trace(
    c: &mut Criterion,
    name: &'static str,
    capacity: usize,
    batch_limit: usize,
) {
    c.bench_function(name, |bench| {
        bench.iter(|| {
            let stats = run_mixed_trace_sample(capacity, batch_limit);
            assert_eq!(stats.submitted_count, MIXED_TRACE_BLOCKS);
            assert_eq!(stats.drained_count, MIXED_TRACE_BLOCKS);
            black_box(stats);
        });
    });
}

fn print_backpressure_sample(label: &'static str, capacity: usize, batch_limit: usize) {
    let stats = run_backpressure_sample(capacity, batch_limit);
    println!(
        "remote_free_backpressure_sample={label} blocks=256 capacity={capacity} batch_limit={batch_limit} submitted_count={} drained_count={} pending_count={} full_count={} disconnected_count={}",
        stats.submitted_count,
        stats.drained_count,
        stats.pending_count,
        stats.full_count,
        stats.disconnected_count
    );

    print_backpressure_sample_summary(label, capacity, batch_limit);
}

fn print_backpressure_sample_summary(label: &'static str, capacity: usize, batch_limit: usize) {
    const SAMPLES: u64 = 8;

    let mut full = CounterSummary::new();
    let mut pending = CounterSummary::new();

    for _ in 0..SAMPLES {
        let stats = run_backpressure_sample(capacity, batch_limit);
        full.observe(stats.full_count);
        pending.observe(stats.pending_count);
        assert_eq!(stats.submitted_count, 256);
        assert_eq!(stats.drained_count, 256);
        assert_eq!(stats.disconnected_count, 0);
    }

    println!(
        "remote_free_backpressure_sample_summary={label} blocks=256 capacity={capacity} batch_limit={batch_limit} samples={SAMPLES} full_min={} full_max={} full_mean={} pending_min={} pending_max={} pending_mean={}",
        full.min,
        full.max,
        format_milli(full.mean_milli(SAMPLES)),
        pending.min,
        pending.max,
        format_milli(pending.mean_milli(SAMPLES))
    );
}

fn print_mixed_trace_sample(label: &'static str, capacity: usize, batch_limit: usize) {
    let stats = run_mixed_trace_sample(capacity, batch_limit);
    println!(
        "remote_free_mixed_trace_sample={label} blocks={MIXED_TRACE_BLOCKS} capacity={capacity} batch_limit={batch_limit} burst_blocks={MIXED_TRACE_BURST_BLOCKS} drain_every_bursts={MIXED_TRACE_DRAIN_EVERY_BURSTS} submitted_count={} drained_count={} full_count={} forced_drains={} drain_rounds={} max_pending_count={} max_wait_bursts={} mean_wait_bursts={}",
        stats.submitted_count,
        stats.drained_count,
        stats.full_count,
        stats.forced_drains,
        stats.drain_rounds,
        stats.max_pending_count,
        stats.max_wait_bursts,
        format_milli(stats.mean_wait_milli())
    );

    assert_eq!(stats.submitted_count, MIXED_TRACE_BLOCKS);
    assert_eq!(stats.drained_count, MIXED_TRACE_BLOCKS);
}

fn format_milli(value: u64) -> String {
    format!("{}.{:03}", value / 1000, value % 1000)
}

fn run_backpressure_sample(capacity: usize, batch_limit: usize) -> RemoteFreeQueueStats {
    let mut queue = RemoteFreeQueue::new(capacity, batch_limit).expect("queue");
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

fn run_mixed_trace_sample(capacity: usize, batch_limit: usize) -> MixedTraceStats {
    let mut queue = RemoteFreeQueue::new(capacity, batch_limit).expect("queue");
    let sink = queue.sink();
    let mut stats = MixedTraceStats {
        submitted_count: 0,
        drained_count: 0,
        full_count: 0,
        forced_drains: 0,
        drain_rounds: 0,
        max_pending_count: 0,
        max_wait_bursts: 0,
        total_wait_bursts: 0,
    };

    for burst in 0..MIXED_TRACE_BURSTS {
        for _ in 0..MIXED_TRACE_BURST_BLOCKS {
            let mut block = MixedTraceBlock {
                submit_burst: burst,
                allocation: vec![0_u8; 4096],
            };

            loop {
                match sink.try_enqueue(block) {
                    Ok(()) => {
                        stats.submitted_count = stats.submitted_count.saturating_add(1);
                        stats.max_pending_count =
                            stats.max_pending_count.max(queue.stats().pending_count);
                        break;
                    }
                    Err(error) if error.kind() == RemoteFreeTryEnqueueErrorKind::Full => {
                        stats.full_count = stats.full_count.saturating_add(1);
                        block = error.into_item();

                        if drain_mixed_trace_batch(&mut queue, burst, &mut stats) == 0 {
                            thread::yield_now();
                        } else {
                            stats.forced_drains = stats.forced_drains.saturating_add(1);
                        }
                    }
                    Err(error) => panic!("remote free enqueue failed: {error}"),
                }
            }
        }
    }

    while stats.drained_count < stats.submitted_count {
        if drain_mixed_trace_batch(&mut queue, MIXED_TRACE_BURSTS, &mut stats) == 0 {
            thread::yield_now();
        }
    }

    let queue_stats = queue.stats();
    assert_eq!(queue_stats.pending_count, 0);
    assert_eq!(queue_stats.full_count, stats.full_count);
    stats
}

fn drain_mixed_trace_batch(
    queue: &mut RemoteFreeQueue<MixedTraceBlock>,
    current_burst: u64,
    stats: &mut MixedTraceStats,
) -> usize {
    let drained = queue.drain_batch(|mut block| {
        let wait_bursts = current_burst.saturating_sub(block.submit_burst);
        stats.max_wait_bursts = stats.max_wait_bursts.max(wait_bursts);
        stats.total_wait_bursts = stats.total_wait_bursts.saturating_add(wait_bursts);
        stats.drained_count = stats.drained_count.saturating_add(1);
        black_box(block.allocation.as_mut_ptr());
    });

    if drained.drained > 0 {
        stats.drain_rounds = stats.drain_rounds.saturating_add(1);
    }

    drained.drained
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
    remote_free_try_enqueue_backpressure_capacity8_batch64,
    remote_free_try_enqueue_backpressure_capacity64_batch8,
    remote_free_try_enqueue_backpressure_batch64,
    remote_free_try_enqueue_backpressure_capacity128_batch64,
    remote_free_try_enqueue_backpressure_capacity256_batch64,
    remote_free_mixed_trace_capacity64_batch64,
    remote_free_mixed_trace_capacity128_batch64,
    remote_free_mixed_trace_capacity256_batch64
);
criterion_main!(benches);
