#![allow(missing_docs)]

use criterion::{black_box, Criterion};
use locus_alloc::{RemoteFreeServiceRetuneCandidate, RemoteFreeServiceRetuneDryRunPlanner};

use crate::remote_free_service_harness::{
    assert_service_telemetry, format_milli, run_service_case, CounterSummary, ServiceTelemetryCase,
    ServiceTelemetryStats, BLOCKS_PER_OWNER, BYTES_PER_BLOCK, OWNERS, SAMPLES,
};

const DRY_RUN_STABLE_WINDOWS: u64 = 2;

#[derive(Debug, Clone, Copy)]
struct DryRunSequenceStats {
    service_windows: u64,
    submitted_count: u64,
    drained_count: u64,
    released_bytes: u64,
    policy_drains: u64,
    observed_reports: u64,
    reports_needing_retune: u64,
    max_pending_over_target: u64,
    max_queued_bytes_over_budget: u64,
    queue_backpressure_reports: u64,
    keep_config_candidate_windows: u64,
    drain_earlier_candidate_windows: u64,
    combined_candidate_windows: u64,
    would_apply_drain_earlier_windows: u64,
    would_apply_combined_windows: u64,
    max_wait_bursts: u64,
    total_wait_bursts: u64,
    final_candidate: RemoteFreeServiceRetuneCandidate,
    final_streak: u64,
    final_would_apply: Option<RemoteFreeServiceRetuneCandidate>,
}

#[derive(Debug, Clone, Copy)]
struct DryRunExpectedStep {
    candidate: RemoteFreeServiceRetuneCandidate,
    streak: u64,
    would_apply: Option<RemoteFreeServiceRetuneCandidate>,
}

pub fn benchmark_dry_run_sequence(c: &mut Criterion) {
    print_dry_run_sequence_sample();
    print_dry_run_sequence_sample_summary();

    c.bench_function("remote_free_service_telemetry_dry_run_sequence", |b| {
        b.iter(|| {
            let stats = run_dry_run_sequence();
            assert_dry_run_sequence(stats);
            black_box(stats);
        });
    });
}

fn print_dry_run_sequence_sample() {
    let stats = run_dry_run_sequence();
    assert_dry_run_sequence(stats);

    println!(
        "remote_free_service_dry_run_sample windows={} stable_windows={DRY_RUN_STABLE_WINDOWS} submitted_count={} drained_count={} released_bytes={} policy_drains={} observed_reports={} reports_needing_retune={} max_pending_over_target={} max_queued_bytes_over_budget={} queue_backpressure_reports={} keep_config_candidate_windows={} drain_earlier_candidate_windows={} combined_candidate_windows={} would_apply_drain_earlier_windows={} would_apply_combined_windows={} max_wait_bursts={} mean_wait_bursts={} final_candidate={} final_streak={} final_would_apply={}",
        stats.service_windows,
        stats.submitted_count,
        stats.drained_count,
        stats.released_bytes,
        stats.policy_drains,
        stats.observed_reports,
        stats.reports_needing_retune,
        stats.max_pending_over_target,
        stats.max_queued_bytes_over_budget,
        stats.queue_backpressure_reports,
        stats.keep_config_candidate_windows,
        stats.drain_earlier_candidate_windows,
        stats.combined_candidate_windows,
        stats.would_apply_drain_earlier_windows,
        stats.would_apply_combined_windows,
        stats.max_wait_bursts,
        format_milli(stats.mean_wait_milli()),
        stats.final_candidate.as_str(),
        stats.final_streak,
        option_candidate_label(stats.final_would_apply)
    );
}

fn print_dry_run_sequence_sample_summary() {
    let mut reports_needing_retune = CounterSummary::new();
    let mut max_pending_over_target = CounterSummary::new();
    let mut max_queued_bytes_over_budget = CounterSummary::new();
    let mut queue_backpressure_reports = CounterSummary::new();
    let mut would_apply_drain_earlier_windows = CounterSummary::new();
    let mut would_apply_combined_windows = CounterSummary::new();
    let mut max_wait = CounterSummary::new();
    let mut mean_wait = CounterSummary::new();
    let mut final_candidate = RemoteFreeServiceRetuneCandidate::CollectTelemetry;
    let mut final_would_apply = None;
    let mut final_streak = 0;

    for _ in 0..SAMPLES {
        let stats = run_dry_run_sequence();
        assert_dry_run_sequence(stats);

        reports_needing_retune.observe(stats.reports_needing_retune);
        max_pending_over_target.observe(stats.max_pending_over_target);
        max_queued_bytes_over_budget.observe(stats.max_queued_bytes_over_budget);
        queue_backpressure_reports.observe(stats.queue_backpressure_reports);
        would_apply_drain_earlier_windows.observe(stats.would_apply_drain_earlier_windows);
        would_apply_combined_windows.observe(stats.would_apply_combined_windows);
        max_wait.observe(stats.max_wait_bursts);
        mean_wait.observe(stats.mean_wait_milli());
        final_candidate = stats.final_candidate;
        final_would_apply = stats.final_would_apply;
        final_streak = stats.final_streak;
    }

    println!(
        "remote_free_service_dry_run_sample_summary windows=6 stable_windows={DRY_RUN_STABLE_WINDOWS} samples={SAMPLES} reports_needing_retune_min={} reports_needing_retune_max={} reports_needing_retune_mean={} max_pending_over_target_min={} max_pending_over_target_max={} max_pending_over_target_mean={} max_queued_bytes_over_budget_min={} max_queued_bytes_over_budget_max={} max_queued_bytes_over_budget_mean={} queue_backpressure_reports_min={} queue_backpressure_reports_max={} queue_backpressure_reports_mean={} would_apply_drain_earlier_windows_min={} would_apply_drain_earlier_windows_max={} would_apply_drain_earlier_windows_mean={} would_apply_combined_windows_min={} would_apply_combined_windows_max={} would_apply_combined_windows_mean={} max_wait_min={} max_wait_max={} max_wait_mean={} mean_wait_min={} mean_wait_max={} mean_wait_mean={} final_candidate={} final_streak={} final_would_apply={}",
        reports_needing_retune.min,
        reports_needing_retune.max,
        format_milli(reports_needing_retune.mean_milli(SAMPLES)),
        max_pending_over_target.min,
        max_pending_over_target.max,
        format_milli(max_pending_over_target.mean_milli(SAMPLES)),
        max_queued_bytes_over_budget.min,
        max_queued_bytes_over_budget.max,
        max_queued_bytes_over_budget.mean_milli(SAMPLES) / 1000,
        queue_backpressure_reports.min,
        queue_backpressure_reports.max,
        format_milli(queue_backpressure_reports.mean_milli(SAMPLES)),
        would_apply_drain_earlier_windows.min,
        would_apply_drain_earlier_windows.max,
        format_milli(would_apply_drain_earlier_windows.mean_milli(SAMPLES)),
        would_apply_combined_windows.min,
        would_apply_combined_windows.max,
        format_milli(would_apply_combined_windows.mean_milli(SAMPLES)),
        max_wait.min,
        max_wait.max,
        format_milli(max_wait.mean_milli(SAMPLES)),
        format_milli(mean_wait.min),
        format_milli(mean_wait.max),
        format_milli(mean_wait.mean_milli(SAMPLES) / 1000),
        final_candidate.as_str(),
        final_streak,
        option_candidate_label(final_would_apply)
    );
}

fn run_dry_run_sequence() -> DryRunSequenceStats {
    let cases = [
        ServiceTelemetryCase::fixed_policy_all_clean(),
        ServiceTelemetryCase::one_end_drain_owner(),
        ServiceTelemetryCase::one_end_drain_owner(),
        ServiceTelemetryCase::one_capacity128_end_drain_owner(),
        ServiceTelemetryCase::one_capacity128_end_drain_owner(),
        ServiceTelemetryCase::fixed_policy_all_clean(),
    ];
    let expected_steps = [
        DryRunExpectedStep {
            candidate: RemoteFreeServiceRetuneCandidate::KeepConfig,
            streak: 0,
            would_apply: None,
        },
        DryRunExpectedStep {
            candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
            streak: 1,
            would_apply: None,
        },
        DryRunExpectedStep {
            candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
            streak: 2,
            would_apply: Some(RemoteFreeServiceRetuneCandidate::DrainEarlier),
        },
        DryRunExpectedStep {
            candidate: RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier,
            streak: 1,
            would_apply: None,
        },
        DryRunExpectedStep {
            candidate: RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier,
            streak: 2,
            would_apply: Some(
                RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier,
            ),
        },
        DryRunExpectedStep {
            candidate: RemoteFreeServiceRetuneCandidate::KeepConfig,
            streak: 0,
            would_apply: None,
        },
    ];
    let mut planner = RemoteFreeServiceRetuneDryRunPlanner::try_new(DRY_RUN_STABLE_WINDOWS)
        .expect("dry-run planner");
    let mut sequence_stats = DryRunSequenceStats::new();

    for (case, expected_step) in cases.into_iter().zip(expected_steps) {
        let stats = run_service_case(case);
        assert_service_telemetry(case, stats);

        let candidate = planner.observe_summary(stats.summary);
        assert_eq!(candidate, expected_step.candidate);
        assert_eq!(
            planner.consecutive_candidate_windows(),
            expected_step.streak
        );
        assert_eq!(planner.would_apply_candidate(), expected_step.would_apply);

        sequence_stats.observe_window(stats, candidate, planner.would_apply_candidate());
    }

    sequence_stats.final_candidate = planner.current_candidate();
    sequence_stats.final_streak = planner.consecutive_candidate_windows();
    sequence_stats.final_would_apply = planner.would_apply_candidate();
    sequence_stats
}

impl DryRunSequenceStats {
    fn new() -> Self {
        Self {
            service_windows: 0,
            submitted_count: 0,
            drained_count: 0,
            released_bytes: 0,
            policy_drains: 0,
            observed_reports: 0,
            reports_needing_retune: 0,
            max_pending_over_target: 0,
            max_queued_bytes_over_budget: 0,
            queue_backpressure_reports: 0,
            keep_config_candidate_windows: 0,
            drain_earlier_candidate_windows: 0,
            combined_candidate_windows: 0,
            would_apply_drain_earlier_windows: 0,
            would_apply_combined_windows: 0,
            max_wait_bursts: 0,
            total_wait_bursts: 0,
            final_candidate: RemoteFreeServiceRetuneCandidate::CollectTelemetry,
            final_streak: 0,
            final_would_apply: None,
        }
    }

    fn observe_window(
        &mut self,
        stats: ServiceTelemetryStats,
        candidate: RemoteFreeServiceRetuneCandidate,
        would_apply: Option<RemoteFreeServiceRetuneCandidate>,
    ) {
        self.service_windows = self.service_windows.saturating_add(1);
        self.submitted_count = self.submitted_count.saturating_add(stats.submitted_count);
        self.drained_count = self.drained_count.saturating_add(stats.drained_count);
        self.released_bytes = self.released_bytes.saturating_add(stats.released_bytes);
        self.policy_drains = self.policy_drains.saturating_add(stats.policy_drains);
        self.observed_reports = self
            .observed_reports
            .saturating_add(stats.summary.observed_reports());
        self.reports_needing_retune = self
            .reports_needing_retune
            .saturating_add(stats.summary.reports_needing_retune());
        self.max_pending_over_target = self
            .max_pending_over_target
            .max(stats.summary.max_pending_items_over_target());
        self.max_queued_bytes_over_budget = self
            .max_queued_bytes_over_budget
            .max(stats.summary.max_queued_bytes_over_budget());
        self.queue_backpressure_reports = self
            .queue_backpressure_reports
            .saturating_add(stats.summary.queue_backpressure_reports());
        self.max_wait_bursts = self.max_wait_bursts.max(stats.max_wait_bursts);
        self.total_wait_bursts = self
            .total_wait_bursts
            .saturating_add(stats.total_wait_bursts);

        match candidate {
            RemoteFreeServiceRetuneCandidate::KeepConfig => {
                self.keep_config_candidate_windows =
                    self.keep_config_candidate_windows.saturating_add(1);
            }
            RemoteFreeServiceRetuneCandidate::DrainEarlier => {
                self.drain_earlier_candidate_windows =
                    self.drain_earlier_candidate_windows.saturating_add(1);
            }
            RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier => {
                self.combined_candidate_windows = self.combined_candidate_windows.saturating_add(1);
            }
            RemoteFreeServiceRetuneCandidate::CollectTelemetry
            | RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacity
            | RemoteFreeServiceRetuneCandidate::ReviewQueuedByteBudget => {}
        }

        match would_apply {
            Some(RemoteFreeServiceRetuneCandidate::DrainEarlier) => {
                self.would_apply_drain_earlier_windows =
                    self.would_apply_drain_earlier_windows.saturating_add(1);
            }
            Some(RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier) => {
                self.would_apply_combined_windows =
                    self.would_apply_combined_windows.saturating_add(1);
            }
            _ => {}
        }
    }

    fn mean_wait_milli(self) -> u64 {
        if self.drained_count == 0 {
            return 0;
        }

        self.total_wait_bursts.saturating_mul(1000) / self.drained_count
    }
}

fn assert_dry_run_sequence(stats: DryRunSequenceStats) {
    assert_eq!(stats.service_windows, 6);
    assert_eq!(stats.submitted_count, BLOCKS_PER_OWNER * OWNERS as u64 * 6);
    assert_eq!(stats.drained_count, stats.submitted_count);
    assert_eq!(
        stats.released_bytes,
        stats.submitted_count * BYTES_PER_BLOCK
    );
    assert_eq!(stats.policy_drains, 80);
    assert_eq!(stats.observed_reports, 192);
    assert_eq!(stats.reports_needing_retune, 24);
    assert_eq!(stats.max_pending_over_target, 192);
    assert_eq!(stats.max_queued_bytes_over_budget, 786_432);
    assert_eq!(stats.queue_backpressure_reports, 8);
    assert_eq!(stats.keep_config_candidate_windows, 2);
    assert_eq!(stats.drain_earlier_candidate_windows, 2);
    assert_eq!(stats.combined_candidate_windows, 2);
    assert_eq!(stats.would_apply_drain_earlier_windows, 1);
    assert_eq!(stats.would_apply_combined_windows, 1);
    assert_eq!(stats.max_wait_bursts, 8);
    assert_eq!(stats.mean_wait_milli(), 1_875);
    assert_eq!(
        stats.final_candidate,
        RemoteFreeServiceRetuneCandidate::KeepConfig
    );
    assert_eq!(stats.final_streak, 0);
    assert_eq!(stats.final_would_apply, None);
}

fn option_candidate_label(candidate: Option<RemoteFreeServiceRetuneCandidate>) -> &'static str {
    candidate.map_or("none", RemoteFreeServiceRetuneCandidate::as_str)
}
