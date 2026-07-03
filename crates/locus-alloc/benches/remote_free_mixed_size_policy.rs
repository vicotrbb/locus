#![allow(missing_docs)]

use std::{num::NonZeroU64, thread};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use locus_alloc::{
    RemoteFreeDrainController, RemoteFreeDrainPolicy, RemoteFreeQueue,
    RemoteFreeTryEnqueueErrorKind,
};

const TRACE_BURSTS: u64 = 8;
const TRACE_BURST_BLOCKS: u64 = 32;
const TRACE_BLOCKS: u64 = TRACE_BURSTS * TRACE_BURST_BLOCKS;
const TRACE_SIZES: [usize; 8] = [4096, 4096, 8192, 4096, 16384, 4096, 32768, 8192];
const TRACE_SIZES_U64: [u64; 8] = [4096, 4096, 8192, 4096, 16384, 4096, 32768, 8192];
const TRACE_SIZE_COUNT: u64 = 8;
const TRACE_PATTERN_BYTES: u64 = 4096 + 4096 + 8192 + 4096 + 16384 + 4096 + 32768 + 8192;
const TRACE_TOTAL_BYTES: u64 = TRACE_PATTERN_BYTES * (TRACE_BLOCKS / TRACE_SIZE_COUNT);

#[derive(Debug, Clone, Copy)]
struct TracePolicy {
    label: &'static str,
    drain_policy: RemoteFreeDrainPolicy,
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

impl TracePolicy {
    fn end_drain() -> Self {
        Self {
            label: "end_drain",
            drain_policy: RemoteFreeDrainPolicy::new(),
        }
    }

    fn max_wait2() -> Self {
        Self {
            label: "max_wait2",
            drain_policy: RemoteFreeDrainPolicy::new()
                .with_max_pending_age(NonZeroU64::new(2).expect("non-zero")),
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
    let policy = TracePolicy::end_drain();
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
    let policy = TracePolicy::max_wait2();
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
    policy: TracePolicy,
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

fn print_trace_sample(policy: TracePolicy, capacity: usize, batch_limit: usize) {
    let stats = run_trace_sample(capacity, batch_limit, policy);
    let policy_label = policy.label;
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

fn run_trace_sample(capacity: usize, batch_limit: usize, policy: TracePolicy) -> TraceStats {
    let mut queue = RemoteFreeQueue::new(capacity, batch_limit).expect("queue");
    let sink = queue.sink();
    let mut stats = TraceStats::new();
    let mut size_index = 0_usize;
    let mut controller = RemoteFreeDrainController::new(policy.drain_policy);

    for burst in 0..TRACE_BURSTS {
        for _ in 0..TRACE_BURST_BLOCKS {
            let size = TRACE_SIZES[size_index % TRACE_SIZES.len()];
            let size_u64 = TRACE_SIZES_U64[size_index % TRACE_SIZES_U64.len()];
            size_index = size_index.saturating_add(1);
            let mut block = TraceBlock {
                submit_burst: burst,
                allocation: vec![0_u8; size],
            };

            loop {
                match sink.try_enqueue(block) {
                    Ok(()) => {
                        stats.submitted_count = stats.submitted_count.saturating_add(1);
                        controller.record_submit(burst, size_u64);
                        stats.queued_bytes = controller.tracker().queued_bytes();
                        stats.max_queued_bytes = stats.max_queued_bytes.max(stats.queued_bytes);
                        stats.max_pending_count = stats
                            .max_pending_count
                            .max(controller.tracker().pending_count());
                        break;
                    }
                    Err(error) if error.kind() == RemoteFreeTryEnqueueErrorKind::Full => {
                        stats.full_count = stats.full_count.saturating_add(1);
                        block = error.into_item();

                        if drain_trace_batch(&mut queue, burst, &mut stats, &mut controller) == 0 {
                            thread::yield_now();
                        } else {
                            stats.forced_drains = stats.forced_drains.saturating_add(1);
                        }
                    }
                    Err(error) => panic!("remote free enqueue failed: {error}"),
                }
            }
        }

        let completed_bursts = burst.saturating_add(1);
        let policy_report = controller
            .status_for_queue(&queue, completed_bursts)
            .expect("controller status");
        assert_eq!(policy_report.observation.queued_bytes, stats.queued_bytes);
        if policy_report.decision.should_drain()
            && drain_trace_batch(&mut queue, completed_bursts, &mut stats, &mut controller) > 0
        {
            stats.policy_drains = stats.policy_drains.saturating_add(1);
        }
    }

    while stats.drained_count < stats.submitted_count {
        if drain_trace_batch(&mut queue, TRACE_BURSTS, &mut stats, &mut controller) == 0 {
            thread::yield_now();
        }
    }

    let queue_stats = queue.stats();
    assert_eq!(queue_stats.pending_count, 0);
    assert_eq!(queue_stats.full_count, stats.full_count);
    assert!(controller.tracker().is_empty());
    stats
}

fn drain_trace_batch(
    queue: &mut RemoteFreeQueue<TraceBlock>,
    current_burst: u64,
    stats: &mut TraceStats,
    controller: &mut RemoteFreeDrainController,
) -> usize {
    let drained = queue.drain_batch(|mut block| {
        let allocation_len =
            u64::try_from(block.allocation.len()).expect("allocation len fits u64");
        let tracked = controller
            .record_drain(allocation_len)
            .expect("tracked drain");
        assert_eq!(tracked.submit_turn, block.submit_burst);
        assert_eq!(tracked.released_bytes, allocation_len);
        let wait_bursts = current_burst.saturating_sub(block.submit_burst);
        stats.queued_bytes = controller.tracker().queued_bytes();
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
