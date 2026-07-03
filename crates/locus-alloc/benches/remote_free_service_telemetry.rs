#![allow(missing_docs)]

use std::thread;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use locus_alloc::{
    RemoteFreeDrainController, RemoteFreeDrainPolicy, RemoteFreeQueue,
    RemoteFreeQueuedByteDrainConfig, RemoteFreeQueuedByteDriftReport,
    RemoteFreeQueuedByteRetuneAction, RemoteFreeServiceRetuneCandidate,
    RemoteFreeServiceRetuneSummary, RemoteFreeTryEnqueueErrorKind,
};

const OWNERS: usize = 4;
const BLOCKS_PER_OWNER: u64 = 256;
const BURSTS: u64 = 8;
const BURST_BLOCKS: u64 = 32;
const BYTES_PER_BLOCK: u64 = 4096;
const QUEUE_CAPACITY: usize = 256;
const BATCH_LIMIT: usize = 64;
const TARGET_PENDING_BLOCKS: u64 = 64;
const SAMPLES: u64 = 8;

#[derive(Debug, Clone, Copy)]
struct ServiceTelemetryCase {
    label: &'static str,
    drifting_owner: Option<usize>,
    expected: ExpectedServiceTelemetry,
}

#[derive(Debug, Clone, Copy)]
struct ExpectedServiceTelemetry {
    observed_reports: u64,
    reports_needing_retune: u64,
    max_pending_over_target: u64,
    max_queued_bytes_over_budget: u64,
    queue_backpressure_reports: u64,
    keep_config_reports: u64,
    drain_earlier_reports: u64,
    retune_candidate: RemoteFreeServiceRetuneCandidate,
}

#[derive(Debug)]
struct TraceBlock {
    submit_burst: u64,
    allocation: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
struct ServiceTelemetryStats {
    submitted_count: u64,
    drained_count: u64,
    released_bytes: u64,
    policy_drains: u64,
    drain_rounds: u64,
    max_wait_bursts: u64,
    total_wait_bursts: u64,
    summary: RemoteFreeServiceRetuneSummary,
}

#[derive(Debug, Clone, Copy)]
struct CounterSummary {
    min: u64,
    max: u64,
    sum: u64,
}

impl ServiceTelemetryCase {
    fn fixed_policy_all_clean() -> Self {
        Self {
            label: "fixed_policy_all_clean",
            drifting_owner: None,
            expected: ExpectedServiceTelemetry {
                observed_reports: 32,
                reports_needing_retune: 0,
                max_pending_over_target: 0,
                max_queued_bytes_over_budget: 0,
                queue_backpressure_reports: 0,
                keep_config_reports: 32,
                drain_earlier_reports: 0,
                retune_candidate: RemoteFreeServiceRetuneCandidate::KeepConfig,
            },
        }
    }

    fn one_end_drain_owner() -> Self {
        Self {
            label: "one_end_drain_owner",
            drifting_owner: Some(0),
            expected: ExpectedServiceTelemetry {
                observed_reports: 32,
                reports_needing_retune: 6,
                max_pending_over_target: 192,
                max_queued_bytes_over_budget: 786_432,
                queue_backpressure_reports: 0,
                keep_config_reports: 26,
                drain_earlier_reports: 6,
                retune_candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
            },
        }
    }

    fn planner_candidate_drain_earlier() -> Self {
        Self {
            label: "planner_candidate_drain_earlier",
            drifting_owner: None,
            expected: ExpectedServiceTelemetry {
                observed_reports: 32,
                reports_needing_retune: 0,
                max_pending_over_target: 0,
                max_queued_bytes_over_budget: 0,
                queue_backpressure_reports: 0,
                keep_config_reports: 32,
                drain_earlier_reports: 0,
                retune_candidate: RemoteFreeServiceRetuneCandidate::KeepConfig,
            },
        }
    }
}

impl ServiceTelemetryStats {
    fn new() -> Self {
        Self {
            submitted_count: 0,
            drained_count: 0,
            released_bytes: 0,
            policy_drains: 0,
            drain_rounds: 0,
            max_wait_bursts: 0,
            total_wait_bursts: 0,
            summary: RemoteFreeServiceRetuneSummary::new(),
        }
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

fn remote_free_service_telemetry_fixed_policy(c: &mut Criterion) {
    let case = ServiceTelemetryCase::fixed_policy_all_clean();
    print_service_sample(case);
    print_service_sample_summary(case);
    bench_service_case(c, case);
}

fn remote_free_service_telemetry_one_drifting_owner(c: &mut Criterion) {
    let case = ServiceTelemetryCase::one_end_drain_owner();
    print_service_sample(case);
    print_service_sample_summary(case);
    bench_service_case(c, case);
}

fn remote_free_service_telemetry_planner_candidate_drain_earlier(c: &mut Criterion) {
    let case = ServiceTelemetryCase::planner_candidate_drain_earlier();
    print_service_sample(case);
    print_service_sample_summary(case);
    bench_service_case(c, case);
}

fn bench_service_case(c: &mut Criterion, case: ServiceTelemetryCase) {
    let label = format!("remote_free_service_telemetry_{}", case.label);
    c.bench_function(&label, |b| {
        b.iter(|| {
            let stats = run_service_case(case);
            assert_service_telemetry(case, stats);
            black_box(stats);
        });
    });
}

fn print_service_sample(case: ServiceTelemetryCase) {
    let stats = run_service_case(case);
    assert_service_telemetry(case, stats);
    let counts = stats.summary.action_counts();
    let candidate = RemoteFreeServiceRetuneCandidate::from_summary(stats.summary);
    let label = case.label;

    println!(
        "remote_free_service_telemetry_sample={label} owners={OWNERS} blocks_per_owner={BLOCKS_PER_OWNER} bursts={BURSTS} burst_blocks={BURST_BLOCKS} capacity={QUEUE_CAPACITY} batch_limit={BATCH_LIMIT} submitted_count={} drained_count={} released_bytes={} policy_drains={} drain_rounds={} max_wait_bursts={} mean_wait_bursts={} observed_reports={} reports_needing_retune={} max_pending_over_target={} max_queued_bytes_over_budget={} queue_backpressure_reports={} keep_config_reports={} drain_earlier_reports={} retune_candidate={}",
        stats.submitted_count,
        stats.drained_count,
        stats.released_bytes,
        stats.policy_drains,
        stats.drain_rounds,
        stats.max_wait_bursts,
        format_milli(stats.mean_wait_milli()),
        stats.summary.observed_reports(),
        stats.summary.reports_needing_retune(),
        stats.summary.max_pending_items_over_target(),
        stats.summary.max_queued_bytes_over_budget(),
        stats.summary.queue_backpressure_reports(),
        counts.count(RemoteFreeQueuedByteRetuneAction::KeepConfig),
        counts.count(RemoteFreeQueuedByteRetuneAction::DrainEarlier),
        candidate.as_str()
    );
}

fn print_service_sample_summary(case: ServiceTelemetryCase) {
    let mut reports_needing_retune = CounterSummary::new();
    let mut max_pending_over_target = CounterSummary::new();
    let mut max_queued_bytes_over_budget = CounterSummary::new();
    let mut keep_config_reports = CounterSummary::new();
    let mut drain_earlier_reports = CounterSummary::new();
    let mut max_wait = CounterSummary::new();
    let mut mean_wait = CounterSummary::new();
    let mut retune_candidate = case.expected.retune_candidate;

    for _ in 0..SAMPLES {
        let stats = run_service_case(case);
        assert_service_telemetry(case, stats);
        let counts = stats.summary.action_counts();

        reports_needing_retune.observe(stats.summary.reports_needing_retune());
        max_pending_over_target.observe(stats.summary.max_pending_items_over_target());
        max_queued_bytes_over_budget.observe(stats.summary.max_queued_bytes_over_budget());
        keep_config_reports.observe(counts.count(RemoteFreeQueuedByteRetuneAction::KeepConfig));
        drain_earlier_reports.observe(counts.count(RemoteFreeQueuedByteRetuneAction::DrainEarlier));
        max_wait.observe(stats.max_wait_bursts);
        mean_wait.observe(stats.mean_wait_milli());
        retune_candidate = RemoteFreeServiceRetuneCandidate::from_summary(stats.summary);
    }

    let label = case.label;
    println!(
        "remote_free_service_telemetry_sample_summary={label} owners={OWNERS} blocks_per_owner={BLOCKS_PER_OWNER} bursts={BURSTS} burst_blocks={BURST_BLOCKS} capacity={QUEUE_CAPACITY} batch_limit={BATCH_LIMIT} retune_candidate={} samples={SAMPLES} reports_needing_retune_min={} reports_needing_retune_max={} reports_needing_retune_mean={} max_pending_over_target_min={} max_pending_over_target_max={} max_pending_over_target_mean={} max_queued_bytes_over_budget_min={} max_queued_bytes_over_budget_max={} max_queued_bytes_over_budget_mean={} keep_config_reports_min={} keep_config_reports_max={} keep_config_reports_mean={} drain_earlier_reports_min={} drain_earlier_reports_max={} drain_earlier_reports_mean={} max_wait_min={} max_wait_max={} max_wait_mean={} mean_wait_min={} mean_wait_max={} mean_wait_mean={}",
        retune_candidate.as_str(),
        reports_needing_retune.min,
        reports_needing_retune.max,
        format_milli(reports_needing_retune.mean_milli(SAMPLES)),
        max_pending_over_target.min,
        max_pending_over_target.max,
        format_milli(max_pending_over_target.mean_milli(SAMPLES)),
        max_queued_bytes_over_budget.min,
        max_queued_bytes_over_budget.max,
        max_queued_bytes_over_budget.mean_milli(SAMPLES) / 1000,
        keep_config_reports.min,
        keep_config_reports.max,
        format_milli(keep_config_reports.mean_milli(SAMPLES)),
        drain_earlier_reports.min,
        drain_earlier_reports.max,
        format_milli(drain_earlier_reports.mean_milli(SAMPLES)),
        max_wait.min,
        max_wait.max,
        format_milli(max_wait.mean_milli(SAMPLES)),
        format_milli(mean_wait.min),
        format_milli(mean_wait.max),
        format_milli(mean_wait.mean_milli(SAMPLES) / 1000)
    );
}

fn run_service_case(case: ServiceTelemetryCase) -> ServiceTelemetryStats {
    let mut stats = ServiceTelemetryStats::new();

    for owner_index in 0..OWNERS {
        let use_policy = case.drifting_owner != Some(owner_index);
        run_owner_loop(use_policy, &mut stats);
    }

    stats
}

fn run_owner_loop(use_policy: bool, service_stats: &mut ServiceTelemetryStats) {
    let config = RemoteFreeQueuedByteDrainConfig::from_item_shape(
        QUEUE_CAPACITY,
        BATCH_LIMIT,
        TARGET_PENDING_BLOCKS,
        BYTES_PER_BLOCK,
    )
    .expect("service telemetry config");
    let policy = if use_policy {
        config.drain_policy()
    } else {
        RemoteFreeDrainPolicy::new()
    };
    let mut controller = RemoteFreeDrainController::new(policy);
    let mut queue = RemoteFreeQueue::new(QUEUE_CAPACITY, BATCH_LIMIT).expect("remote-free queue");
    let sink = queue.sink();

    for burst in 0..BURSTS {
        for _ in 0..BURST_BLOCKS {
            let mut block = TraceBlock {
                submit_burst: burst,
                allocation: vec![0_u8; usize::try_from(BYTES_PER_BLOCK).expect("block size")],
            };

            loop {
                match sink.try_enqueue(block) {
                    Ok(()) => {
                        controller.record_submit(burst, BYTES_PER_BLOCK);
                        service_stats.submitted_count =
                            service_stats.submitted_count.saturating_add(1);
                        break;
                    }
                    Err(error) if error.kind() == RemoteFreeTryEnqueueErrorKind::Full => {
                        block = error.into_item();
                        if drain_owner_batch(&mut queue, burst, &mut controller, service_stats) == 0
                        {
                            thread::yield_now();
                        }
                    }
                    Err(error) => panic!("remote-free enqueue failed: {error}"),
                }
            }
        }

        let completed_bursts = burst.saturating_add(1);
        let status = controller
            .status_for_queue(&queue, completed_bursts)
            .expect("controller status");
        service_stats
            .summary
            .observe_report(RemoteFreeQueuedByteDriftReport::from_status(config, status));

        if status.decision.should_drain() {
            let drained =
                drain_owner_batch(&mut queue, completed_bursts, &mut controller, service_stats);
            if drained > 0 {
                service_stats.policy_drains = service_stats.policy_drains.saturating_add(1);
            }
        }
    }

    while !controller.is_empty() {
        if drain_owner_batch(&mut queue, BURSTS, &mut controller, service_stats) == 0 {
            thread::yield_now();
        }
    }
}

fn drain_owner_batch(
    queue: &mut RemoteFreeQueue<TraceBlock>,
    current_burst: u64,
    controller: &mut RemoteFreeDrainController,
    service_stats: &mut ServiceTelemetryStats,
) -> usize {
    let drained = queue.drain_batch(|mut block| {
        let released_bytes =
            u64::try_from(block.allocation.len()).expect("allocation length fits u64");
        let tracked = controller
            .record_drain(released_bytes)
            .expect("tracked remote-free drain");
        assert_eq!(tracked.submit_turn, block.submit_burst);

        let wait_bursts = current_burst.saturating_sub(block.submit_burst);
        service_stats.drained_count = service_stats.drained_count.saturating_add(1);
        service_stats.released_bytes = service_stats.released_bytes.saturating_add(released_bytes);
        service_stats.max_wait_bursts = service_stats.max_wait_bursts.max(wait_bursts);
        service_stats.total_wait_bursts =
            service_stats.total_wait_bursts.saturating_add(wait_bursts);
        black_box(block.allocation.as_mut_ptr());
    });

    if drained.drained > 0 {
        service_stats.drain_rounds = service_stats.drain_rounds.saturating_add(1);
    }

    drained.drained
}

fn assert_service_telemetry(case: ServiceTelemetryCase, stats: ServiceTelemetryStats) {
    let counts = stats.summary.action_counts();
    let expected = case.expected;

    assert_eq!(stats.submitted_count, BLOCKS_PER_OWNER * OWNERS as u64);
    assert_eq!(stats.drained_count, stats.submitted_count);
    assert_eq!(
        stats.released_bytes,
        stats.submitted_count * BYTES_PER_BLOCK
    );
    assert_eq!(stats.summary.observed_reports(), expected.observed_reports);
    assert_eq!(
        stats.summary.reports_needing_retune(),
        expected.reports_needing_retune
    );
    assert_eq!(
        stats.summary.max_pending_items_over_target(),
        expected.max_pending_over_target
    );
    assert_eq!(
        stats.summary.max_queued_bytes_over_budget(),
        expected.max_queued_bytes_over_budget
    );
    assert_eq!(
        stats.summary.queue_backpressure_reports(),
        expected.queue_backpressure_reports
    );
    assert_eq!(
        counts.count(RemoteFreeQueuedByteRetuneAction::KeepConfig),
        expected.keep_config_reports
    );
    assert_eq!(
        counts.count(RemoteFreeQueuedByteRetuneAction::DrainEarlier),
        expected.drain_earlier_reports
    );
    assert_eq!(
        RemoteFreeServiceRetuneCandidate::from_summary(stats.summary),
        expected.retune_candidate
    );
}

fn format_milli(value: u64) -> String {
    format!("{}.{:03}", value / 1000, value % 1000)
}

criterion_group!(
    benches,
    remote_free_service_telemetry_fixed_policy,
    remote_free_service_telemetry_one_drifting_owner,
    remote_free_service_telemetry_planner_candidate_drain_earlier
);
criterion_main!(benches);
