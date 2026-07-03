#![allow(missing_docs)]

use std::thread;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use locus_alloc::{
    RemoteFreeDrainController, RemoteFreeDrainPolicy, RemoteFreeQueue, RemoteFreeQueuedByteBudget,
    RemoteFreeQueuedByteDrainConfig, RemoteFreeQueuedByteDriftReport,
    RemoteFreeQueuedByteRetuneHint, RemoteFreeTryEnqueueErrorKind,
};

const TRACE_BURSTS: u64 = 8;
const TRACE_BURST_BLOCKS: u64 = 32;
const TRACE_BLOCKS: u64 = TRACE_BURSTS * TRACE_BURST_BLOCKS;
const TRACE_SIZES: [usize; 8] = [4096, 4096, 8192, 4096, 16_384, 4096, 32_768, 8192];
const TRACE_SIZES_U64: [u64; 8] = [4096, 4096, 8192, 4096, 16_384, 4096, 32_768, 8192];
const TRACE_PATTERN_BYTES: u64 = 4096 + 4096 + 8192 + 4096 + 16_384 + 4096 + 32_768 + 8192;
const TRACE_TOTAL_BYTES: u64 = TRACE_PATTERN_BYTES * (TRACE_BLOCKS / 8);
const TRACE_TWO_BURST_BYTES: u64 = 655_360;

#[derive(Debug, Clone, Copy)]
struct DriftCase {
    label: &'static str,
    queue_capacity: usize,
    drain_batch_limit: usize,
    config: RemoteFreeQueuedByteDrainConfig,
    expected: ExpectedDrift,
}

#[derive(Debug, Clone, Copy)]
struct ExpectedDrift {
    max_pending_over_target: u64,
    max_queued_bytes_over_budget: u64,
    queue_backpressure_observed: u64,
    full_count: u64,
    retune_hint: RemoteFreeQueuedByteRetuneHint,
}

#[derive(Debug)]
struct TraceBlock {
    submit_burst: u64,
    allocation: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
struct DriftTraceStats {
    submitted_count: u64,
    drained_count: u64,
    full_count: u64,
    forced_drains: u64,
    drain_rounds: u64,
    max_pending_count: u64,
    queued_bytes: u64,
    max_queued_bytes: u64,
    released_bytes: u64,
    max_pending_over_target: u64,
    max_queued_bytes_over_budget: u64,
    queue_backpressure_observed: u64,
    retune_hint: RemoteFreeQueuedByteRetuneHint,
}

#[derive(Debug, Clone, Copy)]
struct CounterSummary {
    min: u64,
    max: u64,
    sum: u64,
}

const DRIFT_CASES: [fn() -> DriftCase; 4] = [
    matched_end_drain_case,
    pending_drift_case,
    queued_byte_drift_case,
    backpressure_drift_case,
];

fn matched_end_drain_case() -> DriftCase {
    DriftCase {
        label: "matched_end_drain",
        queue_capacity: 256,
        drain_batch_limit: 256,
        config: config_from_raw_budget(256, 256, 256, TRACE_TOTAL_BYTES),
        expected: ExpectedDrift {
            max_pending_over_target: 0,
            max_queued_bytes_over_budget: 0,
            queue_backpressure_observed: 0,
            full_count: 0,
            retune_hint: RemoteFreeQueuedByteRetuneHint::KeepConfig,
        },
    }
}

fn pending_drift_case() -> DriftCase {
    DriftCase {
        label: "pending_target64_budget_total",
        queue_capacity: 256,
        drain_batch_limit: 256,
        config: config_from_raw_budget(256, 256, 64, TRACE_TOTAL_BYTES),
        expected: ExpectedDrift {
            max_pending_over_target: 192,
            max_queued_bytes_over_budget: 0,
            queue_backpressure_observed: 0,
            full_count: 0,
            retune_hint: RemoteFreeQueuedByteRetuneHint::ReviewDrainCadence,
        },
    }
}

fn queued_byte_drift_case() -> DriftCase {
    DriftCase {
        label: "pending_target256_budget640kib",
        queue_capacity: 256,
        drain_batch_limit: 256,
        config: config_from_raw_budget(256, 256, 256, TRACE_TWO_BURST_BYTES),
        expected: ExpectedDrift {
            max_pending_over_target: 0,
            max_queued_bytes_over_budget: TRACE_TOTAL_BYTES - TRACE_TWO_BURST_BYTES,
            queue_backpressure_observed: 0,
            full_count: 0,
            retune_hint: RemoteFreeQueuedByteRetuneHint::ReviewQueuedByteBudget,
        },
    }
}

fn backpressure_drift_case() -> DriftCase {
    DriftCase {
        label: "capacity64_backpressure",
        queue_capacity: 64,
        drain_batch_limit: 64,
        config: config_from_raw_budget(64, 64, 64, TRACE_TWO_BURST_BYTES),
        expected: ExpectedDrift {
            max_pending_over_target: 0,
            max_queued_bytes_over_budget: 0,
            queue_backpressure_observed: 1,
            full_count: 3,
            retune_hint: RemoteFreeQueuedByteRetuneHint::IncreaseQueueCapacity,
        },
    }
}

fn config_from_raw_budget(
    queue_capacity: usize,
    drain_batch_limit: usize,
    target_pending_items: u64,
    queued_byte_budget: u64,
) -> RemoteFreeQueuedByteDrainConfig {
    let budget = RemoteFreeQueuedByteBudget::try_new(queued_byte_budget).expect("budget");
    RemoteFreeQueuedByteDrainConfig::new(
        queue_capacity,
        drain_batch_limit,
        target_pending_items,
        budget,
    )
    .expect("config")
}

impl DriftTraceStats {
    fn new() -> Self {
        Self {
            submitted_count: 0,
            drained_count: 0,
            full_count: 0,
            forced_drains: 0,
            drain_rounds: 0,
            max_pending_count: 0,
            queued_bytes: 0,
            max_queued_bytes: 0,
            released_bytes: 0,
            max_pending_over_target: 0,
            max_queued_bytes_over_budget: 0,
            queue_backpressure_observed: 0,
            retune_hint: RemoteFreeQueuedByteRetuneHint::KeepConfig,
        }
    }

    fn observe_drift(&mut self, report: RemoteFreeQueuedByteDriftReport) {
        self.max_pending_over_target = self
            .max_pending_over_target
            .max(report.pending_items_over_target());
        self.max_queued_bytes_over_budget = self
            .max_queued_bytes_over_budget
            .max(report.queued_bytes_over_budget());
        if report.has_queue_backpressure() {
            self.queue_backpressure_observed = 1;
        }
        self.retune_hint = report.retune_hint();
    }
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

fn remote_free_drift_matrix(c: &mut Criterion) {
    print_drift_matrix_summary();

    for build_case in DRIFT_CASES {
        let case = build_case();
        let name = format!("remote_free_drift_matrix_{}", case.label);
        c.bench_function(&name, |bench| {
            bench.iter(|| {
                let stats = run_drift_case(case);
                assert_drift_case(case, stats);
                black_box(stats);
            });
        });
    }
}

fn print_drift_matrix_summary() {
    for build_case in DRIFT_CASES {
        print_drift_case_summary(build_case());
    }
}

fn print_drift_case_summary(case: DriftCase) {
    const SAMPLES: u64 = 8;

    let mut pending_over = CounterSummary::new();
    let mut bytes_over = CounterSummary::new();
    let mut backpressure = CounterSummary::new();
    let mut full = CounterSummary::new();
    let mut max_pending = CounterSummary::new();
    let mut max_queued_bytes = CounterSummary::new();

    for _ in 0..SAMPLES {
        let stats = run_drift_case(case);
        assert_drift_case(case, stats);
        pending_over.observe(stats.max_pending_over_target);
        bytes_over.observe(stats.max_queued_bytes_over_budget);
        backpressure.observe(stats.queue_backpressure_observed);
        full.observe(stats.full_count);
        max_pending.observe(stats.max_pending_count);
        max_queued_bytes.observe(stats.max_queued_bytes);
    }

    let label = case.label;
    println!(
        "remote_free_drift_matrix_sample_summary={label} blocks={TRACE_BLOCKS} bursts={TRACE_BURSTS} burst_blocks={TRACE_BURST_BLOCKS} capacity={} batch_limit={} target_pending={} queued_byte_budget={} retune_hint={} samples={SAMPLES} max_pending_over_target_min={} max_pending_over_target_max={} max_pending_over_target_mean={} max_queued_bytes_over_budget_min={} max_queued_bytes_over_budget_max={} max_queued_bytes_over_budget_mean={} queue_backpressure_observed_min={} queue_backpressure_observed_max={} queue_backpressure_observed_mean={} full_min={} full_max={} full_mean={} max_pending_min={} max_pending_max={} max_pending_mean={} max_queued_bytes_min={} max_queued_bytes_max={} max_queued_bytes_mean={}",
        case.queue_capacity,
        case.drain_batch_limit,
        case.config.target_pending_items(),
        case.config.queued_byte_budget().bytes(),
        case.expected.retune_hint.as_str(),
        pending_over.min,
        pending_over.max,
        format_milli(pending_over.mean_milli(SAMPLES)),
        bytes_over.min,
        bytes_over.max,
        bytes_over.sum / SAMPLES,
        backpressure.min,
        backpressure.max,
        format_milli(backpressure.mean_milli(SAMPLES)),
        full.min,
        full.max,
        format_milli(full.mean_milli(SAMPLES)),
        max_pending.min,
        max_pending.max,
        format_milli(max_pending.mean_milli(SAMPLES)),
        max_queued_bytes.min,
        max_queued_bytes.max,
        max_queued_bytes.sum / SAMPLES,
    );
}

fn run_drift_case(case: DriftCase) -> DriftTraceStats {
    let mut queue =
        RemoteFreeQueue::new(case.queue_capacity, case.drain_batch_limit).expect("queue");
    let sink = queue.sink();
    let mut stats = DriftTraceStats::new();
    let mut size_index = 0_usize;
    let mut controller = RemoteFreeDrainController::new(RemoteFreeDrainPolicy::new());

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
                        stats.queued_bytes = controller.queued_bytes();
                        stats.max_queued_bytes = stats.max_queued_bytes.max(stats.queued_bytes);
                        stats.max_pending_count =
                            stats.max_pending_count.max(controller.pending_count());
                        break;
                    }
                    Err(error) if error.kind() == RemoteFreeTryEnqueueErrorKind::Full => {
                        stats.full_count = stats.full_count.saturating_add(1);
                        block = error.into_item();

                        if drain_trace_batch(&mut queue, &mut stats, &mut controller) == 0 {
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
        let controller_status = controller
            .status_for_queue(&queue, completed_bursts)
            .expect("controller status");
        stats.observe_drift(RemoteFreeQueuedByteDriftReport::from_status(
            case.config,
            controller_status,
        ));
    }

    while stats.drained_count < stats.submitted_count {
        if drain_trace_batch(&mut queue, &mut stats, &mut controller) == 0 {
            thread::yield_now();
        }
    }

    let queue_stats = queue.stats();
    assert_eq!(queue_stats.pending_count, 0);
    assert_eq!(queue_stats.full_count, stats.full_count);
    assert!(controller.is_empty());
    stats
}

fn drain_trace_batch(
    queue: &mut RemoteFreeQueue<TraceBlock>,
    stats: &mut DriftTraceStats,
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
        stats.queued_bytes = controller.queued_bytes();
        stats.released_bytes = stats.released_bytes.saturating_add(allocation_len);
        stats.drained_count = stats.drained_count.saturating_add(1);
        black_box(block.allocation.as_mut_ptr());
    });

    if drained.drained > 0 {
        stats.drain_rounds = stats.drain_rounds.saturating_add(1);
    }

    drained.drained
}

fn assert_drift_case(case: DriftCase, stats: DriftTraceStats) {
    assert_eq!(stats.submitted_count, TRACE_BLOCKS);
    assert_eq!(stats.drained_count, TRACE_BLOCKS);
    assert_eq!(stats.queued_bytes, 0);
    assert_eq!(stats.released_bytes, TRACE_TOTAL_BYTES);
    assert_eq!(
        stats.max_pending_over_target,
        case.expected.max_pending_over_target
    );
    assert_eq!(
        stats.max_queued_bytes_over_budget,
        case.expected.max_queued_bytes_over_budget
    );
    assert_eq!(
        stats.queue_backpressure_observed,
        case.expected.queue_backpressure_observed
    );
    assert_eq!(stats.full_count, case.expected.full_count);
    assert_eq!(stats.retune_hint, case.expected.retune_hint);
}

fn format_milli(value: u64) -> String {
    format!("{}.{:03}", value / 1000, value % 1000)
}

criterion_group!(benches, remote_free_drift_matrix);
criterion_main!(benches);
