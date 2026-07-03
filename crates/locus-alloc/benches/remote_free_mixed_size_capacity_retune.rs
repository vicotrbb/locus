#![allow(missing_docs)]

use std::thread;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use locus_alloc::{
    RemoteFreeDrainController, RemoteFreeDrainPolicy, RemoteFreeQueue,
    RemoteFreeQueuedByteDrainConfig, RemoteFreeQueuedByteDriftReport,
    RemoteFreeQueuedByteRetuneAction, RemoteFreeQueuedByteRetuneHint,
    RemoteFreeTryEnqueueErrorKind,
};

const BLOCKS: u64 = 256;
const BURSTS: u64 = 8;
const BURST_BLOCKS: u64 = 32;
const BATCH_LIMIT: usize = 64;
const TARGET_PENDING_ITEMS: usize = 64;
const TRACE_SIZES: [usize; 8] = [4096, 4096, 8192, 4096, 16384, 4096, 32768, 8192];
const TRACE_SIZES_U64: [u64; 8] = [4096, 4096, 8192, 4096, 16384, 4096, 32768, 8192];
const TRACE_PATTERN_BYTES: u64 = 4096 + 4096 + 8192 + 4096 + 16384 + 4096 + 32768 + 8192;
const TOTAL_BYTES: u64 = TRACE_PATTERN_BYTES * (BLOCKS / 8);
const TARGET_QUEUED_BYTES: u64 = 655_360;

#[derive(Debug, Clone, Copy)]
struct MixedSizeCase {
    label: &'static str,
    capacity: usize,
    config: RemoteFreeQueuedByteDrainConfig,
    drain_with_policy: bool,
    expected: ExpectedMixedSizeStats,
}

#[derive(Debug, Clone, Copy)]
struct ExpectedMixedSizeStats {
    full_count: u64,
    forced_drains: u64,
    policy_drains: u64,
    drain_rounds: u64,
    max_pending_count: u64,
    max_queued_bytes: u64,
    max_pending_over_target: u64,
    max_queued_bytes_over_budget: u64,
    max_wait_bursts: u64,
    mean_wait_milli: u64,
    retune_hint: RemoteFreeQueuedByteRetuneHint,
    retune_action: RemoteFreeQueuedByteRetuneAction,
}

#[derive(Debug)]
struct TraceBlock {
    submit_burst: u64,
    allocation: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
struct MixedSizeStats {
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
    max_pending_over_target: u64,
    max_queued_bytes_over_budget: u64,
    max_wait_bursts: u64,
    total_wait_bursts: u64,
    queue_backpressure_observed: u64,
    retune_hint: RemoteFreeQueuedByteRetuneHint,
    retune_action: RemoteFreeQueuedByteRetuneAction,
}

#[derive(Debug, Clone, Copy)]
struct CounterSummary {
    min: u64,
    max: u64,
    sum: u64,
}

const CAPACITY_CASES: [fn() -> MixedSizeCase; 3] = [
    baseline_capacity64,
    candidate_capacity128,
    candidate_capacity256,
];

const POLICY_CASES: [fn() -> MixedSizeCase; 2] = [policy_capacity128, policy_capacity256];

fn baseline_capacity64() -> MixedSizeCase {
    MixedSizeCase {
        label: "baseline_capacity64",
        capacity: 64,
        config: config_for_capacity(64),
        drain_with_policy: false,
        expected: ExpectedMixedSizeStats {
            full_count: 3,
            forced_drains: 3,
            policy_drains: 0,
            drain_rounds: 4,
            max_pending_count: 64,
            max_queued_bytes: TARGET_QUEUED_BYTES,
            max_pending_over_target: 0,
            max_queued_bytes_over_budget: 0,
            max_wait_bursts: 2,
            mean_wait_milli: 1500,
            retune_hint: RemoteFreeQueuedByteRetuneHint::IncreaseQueueCapacity,
            retune_action: RemoteFreeQueuedByteRetuneAction::IncreaseQueueCapacity,
        },
    }
}

fn candidate_capacity128() -> MixedSizeCase {
    MixedSizeCase {
        label: "candidate_capacity128",
        capacity: 128,
        config: config_for_capacity(128),
        drain_with_policy: false,
        expected: ExpectedMixedSizeStats {
            full_count: 2,
            forced_drains: 2,
            policy_drains: 0,
            drain_rounds: 4,
            max_pending_count: 128,
            max_queued_bytes: 1_310_720,
            max_pending_over_target: 64,
            max_queued_bytes_over_budget: 655_360,
            max_wait_bursts: 4,
            mean_wait_milli: 3000,
            retune_hint: RemoteFreeQueuedByteRetuneHint::ReviewMultipleSignals,
            retune_action: RemoteFreeQueuedByteRetuneAction::IncreaseQueueCapacityAndDrainEarlier,
        },
    }
}

fn candidate_capacity256() -> MixedSizeCase {
    MixedSizeCase {
        label: "candidate_capacity256",
        capacity: 256,
        config: config_for_capacity(256),
        drain_with_policy: false,
        expected: ExpectedMixedSizeStats {
            full_count: 0,
            forced_drains: 0,
            policy_drains: 0,
            drain_rounds: 4,
            max_pending_count: 256,
            max_queued_bytes: TOTAL_BYTES,
            max_pending_over_target: 192,
            max_queued_bytes_over_budget: 1_966_080,
            max_wait_bursts: 8,
            mean_wait_milli: 4500,
            retune_hint: RemoteFreeQueuedByteRetuneHint::ReviewMultipleSignals,
            retune_action: RemoteFreeQueuedByteRetuneAction::DrainEarlier,
        },
    }
}

fn policy_capacity128() -> MixedSizeCase {
    MixedSizeCase {
        label: "policy_capacity128",
        capacity: 128,
        config: config_for_capacity(128),
        drain_with_policy: true,
        expected: ExpectedMixedSizeStats {
            full_count: 0,
            forced_drains: 0,
            policy_drains: 4,
            drain_rounds: 4,
            max_pending_count: 64,
            max_queued_bytes: TARGET_QUEUED_BYTES,
            max_pending_over_target: 0,
            max_queued_bytes_over_budget: 0,
            max_wait_bursts: 2,
            mean_wait_milli: 1500,
            retune_hint: RemoteFreeQueuedByteRetuneHint::KeepConfig,
            retune_action: RemoteFreeQueuedByteRetuneAction::KeepConfig,
        },
    }
}

fn policy_capacity256() -> MixedSizeCase {
    MixedSizeCase {
        label: "policy_capacity256",
        capacity: 256,
        config: config_for_capacity(256),
        drain_with_policy: true,
        expected: ExpectedMixedSizeStats {
            full_count: 0,
            forced_drains: 0,
            policy_drains: 4,
            drain_rounds: 4,
            max_pending_count: 64,
            max_queued_bytes: TARGET_QUEUED_BYTES,
            max_pending_over_target: 0,
            max_queued_bytes_over_budget: 0,
            max_wait_bursts: 2,
            mean_wait_milli: 1500,
            retune_hint: RemoteFreeQueuedByteRetuneHint::KeepConfig,
            retune_action: RemoteFreeQueuedByteRetuneAction::KeepConfig,
        },
    }
}

fn config_for_capacity(capacity: usize) -> RemoteFreeQueuedByteDrainConfig {
    RemoteFreeQueuedByteDrainConfig::from_item_sizes(
        capacity,
        BATCH_LIMIT,
        TRACE_SIZES_U64
            .iter()
            .copied()
            .cycle()
            .take(TARGET_PENDING_ITEMS),
    )
    .expect("config")
}

impl MixedSizeStats {
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
            max_pending_over_target: 0,
            max_queued_bytes_over_budget: 0,
            max_wait_bursts: 0,
            total_wait_bursts: 0,
            queue_backpressure_observed: 0,
            retune_hint: RemoteFreeQueuedByteRetuneHint::KeepConfig,
            retune_action: RemoteFreeQueuedByteRetuneAction::KeepConfig,
        }
    }

    fn observe_submit(&mut self, controller: &RemoteFreeDrainController) {
        self.queued_bytes = controller.queued_bytes();
        self.max_queued_bytes = self.max_queued_bytes.max(self.queued_bytes);
        self.max_pending_count = self.max_pending_count.max(controller.pending_count());
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
        self.retune_action = report.retune_action();
    }

    fn mean_wait_milli(self) -> u64 {
        if self.drained_count == 0 {
            return 0;
        }

        self.total_wait_bursts.saturating_mul(1000) / self.drained_count
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

fn remote_free_mixed_size_capacity_retune(c: &mut Criterion) {
    print_mixed_size_retune_summary();

    for build_case in CAPACITY_CASES {
        bench_mixed_size_case(c, build_case());
    }
    for build_case in POLICY_CASES {
        bench_mixed_size_case(c, build_case());
    }
}

fn bench_mixed_size_case(c: &mut Criterion, case: MixedSizeCase) {
    let name = format!("remote_free_mixed_size_capacity_retune_{}", case.label);
    c.bench_function(&name, |bench| {
        bench.iter(|| {
            let stats = run_mixed_size_case(case);
            assert_mixed_size_case(case, stats);
            black_box(stats);
        });
    });
}

fn print_mixed_size_retune_summary() {
    for build_case in CAPACITY_CASES {
        print_mixed_size_case_summary(build_case());
    }
    for build_case in POLICY_CASES {
        print_mixed_size_case_summary(build_case());
    }
}

fn print_mixed_size_case_summary(case: MixedSizeCase) {
    const SAMPLES: u64 = 8;

    let mut full = CounterSummary::new();
    let mut forced_drains = CounterSummary::new();
    let mut policy_drains = CounterSummary::new();
    let mut drain_rounds = CounterSummary::new();
    let mut max_pending = CounterSummary::new();
    let mut max_queued_bytes = CounterSummary::new();
    let mut max_pending_over_target = CounterSummary::new();
    let mut max_queued_bytes_over_budget = CounterSummary::new();
    let mut max_wait = CounterSummary::new();
    let mut mean_wait = CounterSummary::new();

    for _ in 0..SAMPLES {
        let stats = run_mixed_size_case(case);
        assert_mixed_size_case(case, stats);
        full.observe(stats.full_count);
        forced_drains.observe(stats.forced_drains);
        policy_drains.observe(stats.policy_drains);
        drain_rounds.observe(stats.drain_rounds);
        max_pending.observe(stats.max_pending_count);
        max_queued_bytes.observe(stats.max_queued_bytes);
        max_pending_over_target.observe(stats.max_pending_over_target);
        max_queued_bytes_over_budget.observe(stats.max_queued_bytes_over_budget);
        max_wait.observe(stats.max_wait_bursts);
        mean_wait.observe(stats.mean_wait_milli());
    }

    let label = case.label;
    println!(
        "remote_free_mixed_size_capacity_retune_sample_summary={label} blocks={BLOCKS} bursts={BURSTS} burst_blocks={BURST_BLOCKS} capacity={} batch_limit={BATCH_LIMIT} drain_with_policy={} retune_hint={} retune_action={} samples={SAMPLES} full_min={} full_max={} full_mean={} forced_drains_min={} forced_drains_max={} forced_drains_mean={} policy_drains_min={} policy_drains_max={} policy_drains_mean={} drain_rounds_min={} drain_rounds_max={} drain_rounds_mean={} max_pending_min={} max_pending_max={} max_pending_mean={} max_queued_bytes_min={} max_queued_bytes_max={} max_queued_bytes_mean={} max_pending_over_target_min={} max_pending_over_target_max={} max_pending_over_target_mean={} max_queued_bytes_over_budget_min={} max_queued_bytes_over_budget_max={} max_queued_bytes_over_budget_mean={} max_wait_min={} max_wait_max={} max_wait_mean={} mean_wait_min={} mean_wait_max={} mean_wait_mean={}",
        case.capacity,
        u64::from(case.drain_with_policy),
        case.expected.retune_hint.as_str(),
        case.expected.retune_action.as_str(),
        full.min,
        full.max,
        format_milli(full.mean_milli(SAMPLES)),
        forced_drains.min,
        forced_drains.max,
        format_milli(forced_drains.mean_milli(SAMPLES)),
        policy_drains.min,
        policy_drains.max,
        format_milli(policy_drains.mean_milli(SAMPLES)),
        drain_rounds.min,
        drain_rounds.max,
        format_milli(drain_rounds.mean_milli(SAMPLES)),
        max_pending.min,
        max_pending.max,
        format_milli(max_pending.mean_milli(SAMPLES)),
        max_queued_bytes.min,
        max_queued_bytes.max,
        max_queued_bytes.sum / SAMPLES,
        max_pending_over_target.min,
        max_pending_over_target.max,
        format_milli(max_pending_over_target.mean_milli(SAMPLES)),
        max_queued_bytes_over_budget.min,
        max_queued_bytes_over_budget.max,
        max_queued_bytes_over_budget.sum / SAMPLES,
        max_wait.min,
        max_wait.max,
        format_milli(max_wait.mean_milli(SAMPLES)),
        format_milli(mean_wait.min),
        format_milli(mean_wait.max),
        format_milli(mean_wait.sum / SAMPLES)
    );
}

fn run_mixed_size_case(case: MixedSizeCase) -> MixedSizeStats {
    let mut queue = RemoteFreeQueue::new(case.capacity, BATCH_LIMIT).expect("queue");
    let sink = queue.sink();
    let mut stats = MixedSizeStats::new();
    let drain_policy = if case.drain_with_policy {
        case.config.drain_policy()
    } else {
        RemoteFreeDrainPolicy::new()
    };
    let mut controller = RemoteFreeDrainController::new(drain_policy);
    let mut size_index = 0_usize;

    for burst in 0..BURSTS {
        for _ in 0..BURST_BLOCKS {
            let size_index_in_pattern = size_index % TRACE_SIZES.len();
            let size = TRACE_SIZES[size_index_in_pattern];
            let size_u64 = TRACE_SIZES_U64[size_index_in_pattern];
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
                        stats.observe_submit(&controller);
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
        let controller_status = controller
            .status_for_queue(&queue, completed_bursts)
            .expect("controller status");
        stats.observe_drift(RemoteFreeQueuedByteDriftReport::from_status(
            case.config,
            controller_status,
        ));
        if controller_status.decision.should_drain()
            && drain_trace_batch(&mut queue, completed_bursts, &mut stats, &mut controller) > 0
        {
            stats.policy_drains = stats.policy_drains.saturating_add(1);
        }
    }

    while stats.drained_count < stats.submitted_count {
        if drain_trace_batch(&mut queue, BURSTS, &mut stats, &mut controller) == 0 {
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
    current_burst: u64,
    stats: &mut MixedSizeStats,
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
        stats.queued_bytes = controller.queued_bytes();
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

fn assert_mixed_size_case(case: MixedSizeCase, stats: MixedSizeStats) {
    assert_eq!(stats.submitted_count, BLOCKS);
    assert_eq!(stats.drained_count, BLOCKS);
    assert_eq!(stats.queued_bytes, 0);
    assert_eq!(stats.released_bytes, TOTAL_BYTES);
    assert_eq!(stats.full_count, case.expected.full_count);
    assert_eq!(stats.forced_drains, case.expected.forced_drains);
    assert_eq!(stats.policy_drains, case.expected.policy_drains);
    assert_eq!(stats.drain_rounds, case.expected.drain_rounds);
    assert_eq!(stats.max_pending_count, case.expected.max_pending_count);
    assert_eq!(stats.max_queued_bytes, case.expected.max_queued_bytes);
    assert_eq!(
        stats.max_pending_over_target,
        case.expected.max_pending_over_target
    );
    assert_eq!(
        stats.max_queued_bytes_over_budget,
        case.expected.max_queued_bytes_over_budget
    );
    assert_eq!(stats.max_wait_bursts, case.expected.max_wait_bursts);
    assert_eq!(stats.mean_wait_milli(), case.expected.mean_wait_milli);
    assert_eq!(stats.retune_hint, case.expected.retune_hint);
    assert_eq!(stats.retune_action, case.expected.retune_action);
}

fn format_milli(value: u64) -> String {
    format!("{}.{:03}", value / 1000, value % 1000)
}

criterion_group!(benches, remote_free_mixed_size_capacity_retune);
criterion_main!(benches);
