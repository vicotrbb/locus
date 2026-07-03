#![allow(missing_docs)]

use std::thread;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use locus_alloc::{RemoteFreeQueue, RemoteFreeTryEnqueueErrorKind};

const TRACE_BURSTS: u64 = 8;
const TRACE_BURST_BLOCKS: u64 = 32;
const TRACE_BLOCKS: u64 = TRACE_BURSTS * TRACE_BURST_BLOCKS;
const TRACE_SIZES: [usize; 8] = [4096, 4096, 8192, 4096, 16384, 4096, 32768, 8192];
const TRACE_SIZE_COUNT: u64 = 8;
const TRACE_PATTERN_BYTES: u64 = 4096 + 4096 + 8192 + 4096 + 16384 + 4096 + 32768 + 8192;
const TRACE_TOTAL_BYTES: u64 = TRACE_PATTERN_BYTES * (TRACE_BLOCKS / TRACE_SIZE_COUNT);

#[derive(Debug, Clone, Copy)]
enum DrainPolicy {
    EndOfTrace,
    MaxWaitBursts(u64),
}

#[derive(Debug)]
struct TraceBlock {
    submit_burst: u64,
    allocation: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
struct TraceStats {
    submitted_count: u64,
    drained_count: u64,
    full_count: u64,
    forced_drains: u64,
    policy_drains: u64,
    drain_rounds: u64,
    max_pending_count: u64,
    queued_bytes: u64,
    max_queued_bytes: u64,
    released_bytes: u64,
    max_wait_bursts: u64,
    total_wait_bursts: u64,
}

impl DrainPolicy {
    fn label(self) -> &'static str {
        match self {
            Self::EndOfTrace => "end_drain",
            Self::MaxWaitBursts(2) => "max_wait2",
            Self::MaxWaitBursts(_) => "max_wait_custom",
        }
    }

    fn should_drain_after_burst(self, burst: u64) -> bool {
        match self {
            Self::EndOfTrace | Self::MaxWaitBursts(0) => false,
            Self::MaxWaitBursts(max_wait) => {
                let completed_bursts = burst.saturating_add(1);
                completed_bursts % max_wait == 0
            }
        }
    }
}

impl TraceStats {
    fn new() -> Self {
        Self {
            submitted_count: 0,
            drained_count: 0,
            full_count: 0,
            forced_drains: 0,
            policy_drains: 0,
            drain_rounds: 0,
            max_pending_count: 0,
            queued_bytes: 0,
            max_queued_bytes: 0,
            released_bytes: 0,
            max_wait_bursts: 0,
            total_wait_bursts: 0,
        }
    }

    fn mean_wait_milli(self) -> u64 {
        if self.drained_count == 0 {
            return 0;
        }

        self.total_wait_bursts.saturating_mul(1000) / self.drained_count
    }
}

fn remote_free_mixed_size_end_drain_capacity256_batch64(c: &mut Criterion) {
    let policy = DrainPolicy::EndOfTrace;
    print_trace_sample(policy, 256, 64);
    remote_free_mixed_size_policy(
        c,
        "remote_free_mixed_size_trace_capacity256_batch64_end_drain",
        256,
        64,
        policy,
    );
}

fn remote_free_mixed_size_max_wait2_capacity256_batch64(c: &mut Criterion) {
    let policy = DrainPolicy::MaxWaitBursts(2);
    print_trace_sample(policy, 256, 64);
    remote_free_mixed_size_policy(
        c,
        "remote_free_mixed_size_trace_capacity256_batch64_max_wait2",
        256,
        64,
        policy,
    );
}

fn remote_free_mixed_size_policy(
    c: &mut Criterion,
    name: &'static str,
    capacity: usize,
    batch_limit: usize,
    policy: DrainPolicy,
) {
    c.bench_function(name, |bench| {
        bench.iter(|| {
            let stats = run_trace_sample(capacity, batch_limit, policy);
            assert_eq!(stats.submitted_count, TRACE_BLOCKS);
            assert_eq!(stats.drained_count, TRACE_BLOCKS);
            assert_eq!(stats.queued_bytes, 0);
            assert_eq!(stats.released_bytes, TRACE_TOTAL_BYTES);
            black_box(stats);
        });
    });
}

fn print_trace_sample(policy: DrainPolicy, capacity: usize, batch_limit: usize) {
    let stats = run_trace_sample(capacity, batch_limit, policy);
    let policy_label = policy.label();
    println!(
        "remote_free_mixed_size_policy_sample={policy_label} blocks={TRACE_BLOCKS} bursts={TRACE_BURSTS} burst_blocks={TRACE_BURST_BLOCKS} capacity={capacity} batch_limit={batch_limit} submitted_count={} drained_count={} full_count={} forced_drains={} policy_drains={} drain_rounds={} max_pending_count={} max_queued_bytes={} released_bytes={} max_wait_bursts={} mean_wait_bursts={}",
        stats.submitted_count,
        stats.drained_count,
        stats.full_count,
        stats.forced_drains,
        stats.policy_drains,
        stats.drain_rounds,
        stats.max_pending_count,
        stats.max_queued_bytes,
        stats.released_bytes,
        stats.max_wait_bursts,
        format_milli(stats.mean_wait_milli())
    );
    assert_eq!(stats.released_bytes, TRACE_TOTAL_BYTES);
}

fn run_trace_sample(capacity: usize, batch_limit: usize, policy: DrainPolicy) -> TraceStats {
    let mut queue = RemoteFreeQueue::new(capacity, batch_limit).expect("queue");
    let sink = queue.sink();
    let mut stats = TraceStats::new();
    let mut size_index = 0_usize;

    for burst in 0..TRACE_BURSTS {
        for _ in 0..TRACE_BURST_BLOCKS {
            let size = TRACE_SIZES[size_index % TRACE_SIZES.len()];
            size_index = size_index.saturating_add(1);
            let mut block = TraceBlock {
                submit_burst: burst,
                allocation: vec![0_u8; size],
            };

            loop {
                match sink.try_enqueue(block) {
                    Ok(()) => {
                        stats.submitted_count = stats.submitted_count.saturating_add(1);
                        stats.queued_bytes = stats.queued_bytes.saturating_add(size as u64);
                        stats.max_queued_bytes = stats.max_queued_bytes.max(stats.queued_bytes);
                        stats.max_pending_count =
                            stats.max_pending_count.max(queue.stats().pending_count);
                        break;
                    }
                    Err(error) if error.kind() == RemoteFreeTryEnqueueErrorKind::Full => {
                        stats.full_count = stats.full_count.saturating_add(1);
                        block = error.into_item();

                        if drain_trace_batch(&mut queue, burst, &mut stats) == 0 {
                            thread::yield_now();
                        } else {
                            stats.forced_drains = stats.forced_drains.saturating_add(1);
                        }
                    }
                    Err(error) => panic!("remote free enqueue failed: {error}"),
                }
            }
        }

        if policy.should_drain_after_burst(burst)
            && drain_trace_batch(&mut queue, burst.saturating_add(1), &mut stats) > 0
        {
            stats.policy_drains = stats.policy_drains.saturating_add(1);
        }
    }

    while stats.drained_count < stats.submitted_count {
        if drain_trace_batch(&mut queue, TRACE_BURSTS, &mut stats) == 0 {
            thread::yield_now();
        }
    }

    let queue_stats = queue.stats();
    assert_eq!(queue_stats.pending_count, 0);
    assert_eq!(queue_stats.full_count, stats.full_count);
    stats
}

fn drain_trace_batch(
    queue: &mut RemoteFreeQueue<TraceBlock>,
    current_burst: u64,
    stats: &mut TraceStats,
) -> usize {
    let drained = queue.drain_batch(|mut block| {
        let allocation_len = block.allocation.len() as u64;
        let wait_bursts = current_burst.saturating_sub(block.submit_burst);
        stats.queued_bytes = stats.queued_bytes.saturating_sub(allocation_len);
        stats.released_bytes = stats.released_bytes.saturating_add(allocation_len);
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

fn format_milli(value: u64) -> String {
    format!("{}.{:03}", value / 1000, value % 1000)
}

criterion_group!(
    benches,
    remote_free_mixed_size_end_drain_capacity256_batch64,
    remote_free_mixed_size_max_wait2_capacity256_batch64
);
criterion_main!(benches);
