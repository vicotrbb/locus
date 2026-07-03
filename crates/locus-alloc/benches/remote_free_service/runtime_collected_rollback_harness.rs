#![allow(missing_docs)]

use criterion::{black_box, Criterion};
use locus_alloc::{
    RemoteFreeDrainPolicy, RemoteFreeOwnerRuntime, RemoteFreeOwnerRuntimeApplyOutcome,
    RemoteFreeOwnerRuntimeRollbackOutcome, RemoteFreeServiceRetuneCandidate,
    RemoteFreeServiceRetuneGuard, RemoteFreeServiceRetuneGuardDecision,
    RemoteFreeServiceRetunePolicyApplicator, RemoteFreeServiceRetuneSummary,
};

use crate::remote_free_service_application_harness::{
    run_runtime_owner_window_with_summary, run_runtime_owner_window_with_summary_and_block_bytes,
    service_config, RuntimeApplicationStats, RuntimeTraceBlock, QUEUE_CAPACITY_GROWTH_FACTOR,
    RUNTIME_INITIAL_QUEUE_CAPACITY,
};
use crate::remote_free_service_harness::{format_milli, CounterSummary, BYTES_PER_BLOCK, SAMPLES};
use crate::remote_free_service_sample_filter::should_print_sample;
use crate::remote_free_service_sample_output::print_sample_line;

const RUNTIME_ROLLBACK_STABLE_WINDOWS: u64 = 2;
const RUNTIME_ROLLBACK_MAX_MUTATIONS: u64 = 2;
const RUNTIME_ROLLBACK_WINDOWS: u64 = 3;
const RUNTIME_ROLLBACK_VALIDATION_BYTES: u64 = BYTES_PER_BLOCK * 2 + 1;
const RUNTIME_ROLLBACK_BENCHMARK: &str = "remote_free_service_runtime_collected_rollback";
const RUNTIME_ROLLBACK_SAMPLE: &str = "remote_free_service_runtime_collected_rollback_sample";
const RUNTIME_ROLLBACK_SAMPLE_SUMMARY: &str =
    "remote_free_service_runtime_collected_rollback_sample_summary";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeRollbackDecisionKind {
    Hold,
    Apply,
    Rollback,
}

#[derive(Debug, Clone, Copy)]
struct RuntimeRollbackExpectedStep {
    candidate: RemoteFreeServiceRetuneCandidate,
    observed_candidate: Option<RemoteFreeServiceRetuneCandidate>,
    decision: RuntimeRollbackDecisionKind,
}

#[derive(Debug, Clone, Copy)]
struct RuntimeRollbackStats {
    runtime: RuntimeApplicationStats,
    observed_reports: u64,
    reports_needing_retune: u64,
    max_pending_over_target: u64,
    max_queued_bytes_over_budget: u64,
    queue_backpressure_reports: u64,
    hold_decisions: u64,
    apply_decisions: u64,
    rollback_decisions: u64,
    runtime_no_change_decisions: u64,
    final_guard_applied_mutations: u64,
    final_guard_confirmed_mutations: u64,
    final_guard_rollbacks: u64,
    final_guard_pending_candidate: Option<RemoteFreeServiceRetuneCandidate>,
}

pub(crate) fn benchmark_runtime_collected_rollback(c: &mut Criterion) {
    print_runtime_rollback_sample();
    print_runtime_rollback_sample_summary();

    c.bench_function(RUNTIME_ROLLBACK_BENCHMARK, |bench| {
        bench.iter(|| {
            let stats = run_runtime_collected_rollback();
            assert_runtime_rollback_stats(stats);
            black_box(stats);
        });
    });
}

fn print_runtime_rollback_sample() {
    if !should_print_sample(RUNTIME_ROLLBACK_SAMPLE, RUNTIME_ROLLBACK_BENCHMARK) {
        return;
    }

    let stats = run_runtime_collected_rollback();
    assert_runtime_rollback_stats(stats);

    print_sample_line(
        RUNTIME_ROLLBACK_BENCHMARK,
        format_args!(
        "{RUNTIME_ROLLBACK_SAMPLE} windows={RUNTIME_ROLLBACK_WINDOWS} stable_windows={RUNTIME_ROLLBACK_STABLE_WINDOWS} max_mutations={RUNTIME_ROLLBACK_MAX_MUTATIONS} validation_bytes={RUNTIME_ROLLBACK_VALIDATION_BYTES} submitted_count={} drained_count={} released_bytes={} policy_drains={} drain_rounds={} observed_reports={} reports_needing_retune={} max_pending_over_target={} max_queued_bytes_over_budget={} queue_backpressure_reports={} hold_decisions={} apply_decisions={} rollback_decisions={} runtime_install_count={} runtime_confirm_count={} runtime_rollback_count={} runtime_no_change_decisions={} max_wait_bursts={} mean_wait_bursts={} final_queue_capacity={} final_previous_config_present={} final_guard_pending_candidate={} final_guard_applied_mutations={} final_guard_confirmed_mutations={} final_guard_rollbacks={}",
        stats.runtime.submitted_count,
        stats.runtime.drained_count,
        stats.runtime.released_bytes,
        stats.runtime.policy_drains,
        stats.runtime.drain_rounds,
        stats.observed_reports,
        stats.reports_needing_retune,
        stats.max_pending_over_target,
        stats.max_queued_bytes_over_budget,
        stats.queue_backpressure_reports,
        stats.hold_decisions,
        stats.apply_decisions,
        stats.rollback_decisions,
        stats.runtime.install_count,
        stats.runtime.confirm_count,
        stats.runtime.rollback_count,
        stats.runtime_no_change_decisions,
        stats.runtime.max_wait_bursts,
        format_milli(stats.runtime.mean_wait_milli()),
        stats.runtime.final_queue_capacity,
        stats.runtime.final_previous_config_present,
        option_candidate_label(stats.final_guard_pending_candidate),
        stats.final_guard_applied_mutations,
        stats.final_guard_confirmed_mutations,
        stats.final_guard_rollbacks,
    ));
}

fn print_runtime_rollback_sample_summary() {
    if !should_print_sample(RUNTIME_ROLLBACK_SAMPLE_SUMMARY, RUNTIME_ROLLBACK_BENCHMARK) {
        return;
    }

    let mut policy_drains = CounterSummary::new();
    let mut drain_rounds = CounterSummary::new();
    let mut reports_needing_retune = CounterSummary::new();
    let mut apply_decisions = CounterSummary::new();
    let mut rollback_decisions = CounterSummary::new();
    let mut max_wait = CounterSummary::new();
    let mut mean_wait = CounterSummary::new();

    for _ in 0..SAMPLES {
        let stats = run_runtime_collected_rollback();
        assert_runtime_rollback_stats(stats);

        policy_drains.observe(stats.runtime.policy_drains);
        drain_rounds.observe(stats.runtime.drain_rounds);
        reports_needing_retune.observe(stats.reports_needing_retune);
        apply_decisions.observe(stats.apply_decisions);
        rollback_decisions.observe(stats.rollback_decisions);
        max_wait.observe(stats.runtime.max_wait_bursts);
        mean_wait.observe(stats.runtime.mean_wait_milli());
    }

    print_sample_line(
        RUNTIME_ROLLBACK_BENCHMARK,
        format_args!(
        "{RUNTIME_ROLLBACK_SAMPLE_SUMMARY} windows={RUNTIME_ROLLBACK_WINDOWS} samples={SAMPLES} policy_drains_min={} policy_drains_max={} policy_drains_mean={} drain_rounds_min={} drain_rounds_max={} drain_rounds_mean={} reports_needing_retune_min={} reports_needing_retune_max={} reports_needing_retune_mean={} apply_decisions_min={} apply_decisions_max={} apply_decisions_mean={} rollback_decisions_min={} rollback_decisions_max={} rollback_decisions_mean={} max_wait_min={} max_wait_max={} max_wait_mean={} mean_wait_min={} mean_wait_max={} mean_wait_mean={}",
        policy_drains.min,
        policy_drains.max,
        format_milli(policy_drains.mean_milli(SAMPLES)),
        drain_rounds.min,
        drain_rounds.max,
        format_milli(drain_rounds.mean_milli(SAMPLES)),
        reports_needing_retune.min,
        reports_needing_retune.max,
        format_milli(reports_needing_retune.mean_milli(SAMPLES)),
        apply_decisions.min,
        apply_decisions.max,
        format_milli(apply_decisions.mean_milli(SAMPLES)),
        rollback_decisions.min,
        rollback_decisions.max,
        format_milli(rollback_decisions.mean_milli(SAMPLES)),
        max_wait.min,
        max_wait.max,
        format_milli(max_wait.mean_milli(SAMPLES)),
        format_milli(mean_wait.min),
        format_milli(mean_wait.max),
        format_milli(mean_wait.mean_milli(SAMPLES) / 1000),
    ));
}

fn run_runtime_collected_rollback() -> RuntimeRollbackStats {
    let mut guard = RemoteFreeServiceRetuneGuard::try_new(
        RUNTIME_ROLLBACK_STABLE_WINDOWS,
        RUNTIME_ROLLBACK_MAX_MUTATIONS,
    )
    .expect("guard");
    let mut runtime = RemoteFreeOwnerRuntime::new_with_drain_policy(
        service_config(RUNTIME_INITIAL_QUEUE_CAPACITY),
        RemoteFreeDrainPolicy::new(),
    )
    .expect("runtime");
    let applicator = RemoteFreeServiceRetunePolicyApplicator::try_new(QUEUE_CAPACITY_GROWTH_FACTOR)
        .expect("applicator");
    let mut stats = RuntimeRollbackStats::new();

    for expected_step in runtime_rollback_expected_steps() {
        let summary = match expected_step.decision {
            RuntimeRollbackDecisionKind::Rollback => {
                run_runtime_owner_window_with_summary_and_block_bytes(
                    &mut runtime,
                    &mut stats.runtime,
                    RUNTIME_ROLLBACK_VALIDATION_BYTES,
                )
            }
            RuntimeRollbackDecisionKind::Hold | RuntimeRollbackDecisionKind::Apply => {
                run_runtime_owner_window_with_summary(&mut runtime, &mut stats.runtime)
            }
        };

        stats.observe_summary(summary);
        let decision = guard.observe_summary(summary);
        assert_runtime_rollback_decision(expected_step, decision);
        stats.observe_decision(decision);
        apply_runtime_rollback_decision(&mut runtime, applicator, decision, &mut stats);
    }

    stats.runtime.final_queue_capacity = runtime.config().queue_capacity();
    stats.runtime.final_previous_config_present = runtime.previous_config().is_some();
    stats.final_guard_applied_mutations = guard.applied_mutations();
    stats.final_guard_confirmed_mutations = guard.confirmed_mutations();
    stats.final_guard_rollbacks = guard.rollbacks();
    stats.final_guard_pending_candidate = guard.pending_validation();
    stats
}

fn runtime_rollback_expected_steps() -> [RuntimeRollbackExpectedStep; 3] {
    [
        RuntimeRollbackExpectedStep {
            candidate: RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier,
            observed_candidate: None,
            decision: RuntimeRollbackDecisionKind::Hold,
        },
        RuntimeRollbackExpectedStep {
            candidate: RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier,
            observed_candidate: None,
            decision: RuntimeRollbackDecisionKind::Apply,
        },
        RuntimeRollbackExpectedStep {
            candidate: RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier,
            observed_candidate: Some(RemoteFreeServiceRetuneCandidate::ReviewQueuedByteBudget),
            decision: RuntimeRollbackDecisionKind::Rollback,
        },
    ]
}

fn apply_runtime_rollback_decision(
    runtime: &mut RemoteFreeOwnerRuntime<RuntimeTraceBlock>,
    applicator: RemoteFreeServiceRetunePolicyApplicator,
    decision: RemoteFreeServiceRetuneGuardDecision,
    stats: &mut RuntimeRollbackStats,
) {
    match decision {
        RemoteFreeServiceRetuneGuardDecision::Apply { .. } => {
            let application = applicator
                .plan(runtime.config(), decision)
                .expect("runtime application");
            let expected_current_config = application.config().expect("applied config");
            let expected_previous_config = runtime.config();
            assert_eq!(
                runtime.apply(application),
                Ok(RemoteFreeOwnerRuntimeApplyOutcome::Installed {
                    candidate: application.candidate().expect("applied candidate"),
                    previous_config: expected_previous_config,
                    current_config: expected_current_config,
                })
            );
            stats.runtime.install_count = stats.runtime.install_count.saturating_add(1);
        }
        RemoteFreeServiceRetuneGuardDecision::Rollback { .. } => {
            let replaced_config = runtime.config();
            let restored_config = runtime
                .previous_config()
                .expect("rollback config before runtime rollback");
            assert_eq!(
                runtime.rollback(),
                Ok(RemoteFreeOwnerRuntimeRollbackOutcome {
                    replaced_config,
                    restored_config,
                })
            );
            stats.runtime.rollback_count = stats.runtime.rollback_count.saturating_add(1);
        }
        RemoteFreeServiceRetuneGuardDecision::CollectTelemetry
        | RemoteFreeServiceRetuneGuardDecision::Hold { .. }
        | RemoteFreeServiceRetuneGuardDecision::MutationLimitReached { .. } => {
            let application = applicator
                .plan(runtime.config(), decision)
                .expect("runtime no-change application");
            assert_eq!(
                runtime.apply(application),
                Ok(RemoteFreeOwnerRuntimeApplyOutcome::NoChange { decision })
            );
            stats.runtime_no_change_decisions = stats.runtime_no_change_decisions.saturating_add(1);
        }
        RemoteFreeServiceRetuneGuardDecision::Confirmed { .. } => {
            panic!("runtime-collected rollback sequence should not confirm");
        }
    }
}

fn assert_runtime_rollback_decision(
    expected_step: RuntimeRollbackExpectedStep,
    decision: RemoteFreeServiceRetuneGuardDecision,
) {
    assert_eq!(
        RuntimeRollbackDecisionKind::from_decision(decision),
        expected_step.decision
    );

    match decision {
        RemoteFreeServiceRetuneGuardDecision::Hold { candidate, .. }
        | RemoteFreeServiceRetuneGuardDecision::Apply { candidate }
        | RemoteFreeServiceRetuneGuardDecision::MutationLimitReached { candidate }
        | RemoteFreeServiceRetuneGuardDecision::Confirmed { candidate } => {
            assert_eq!(candidate, expected_step.candidate);
            assert_eq!(expected_step.observed_candidate, None);
        }
        RemoteFreeServiceRetuneGuardDecision::Rollback {
            candidate,
            observed_candidate,
        } => {
            assert_eq!(candidate, expected_step.candidate);
            assert_eq!(Some(observed_candidate), expected_step.observed_candidate);
        }
        RemoteFreeServiceRetuneGuardDecision::CollectTelemetry => {}
    }
}

impl RuntimeRollbackDecisionKind {
    fn from_decision(decision: RemoteFreeServiceRetuneGuardDecision) -> Self {
        match decision {
            RemoteFreeServiceRetuneGuardDecision::CollectTelemetry
            | RemoteFreeServiceRetuneGuardDecision::Hold { .. } => Self::Hold,
            RemoteFreeServiceRetuneGuardDecision::Apply { .. } => Self::Apply,
            RemoteFreeServiceRetuneGuardDecision::Rollback { .. } => Self::Rollback,
            RemoteFreeServiceRetuneGuardDecision::Confirmed { .. }
            | RemoteFreeServiceRetuneGuardDecision::MutationLimitReached { .. } => {
                panic!("unexpected runtime-collected rollback decision")
            }
        }
    }
}

impl RuntimeRollbackStats {
    fn new() -> Self {
        Self {
            runtime: RuntimeApplicationStats::new(),
            observed_reports: 0,
            reports_needing_retune: 0,
            max_pending_over_target: 0,
            max_queued_bytes_over_budget: 0,
            queue_backpressure_reports: 0,
            hold_decisions: 0,
            apply_decisions: 0,
            rollback_decisions: 0,
            runtime_no_change_decisions: 0,
            final_guard_applied_mutations: 0,
            final_guard_confirmed_mutations: 0,
            final_guard_rollbacks: 0,
            final_guard_pending_candidate: None,
        }
    }

    fn observe_summary(&mut self, summary: RemoteFreeServiceRetuneSummary) {
        self.observed_reports = self
            .observed_reports
            .saturating_add(summary.observed_reports());
        self.reports_needing_retune = self
            .reports_needing_retune
            .saturating_add(summary.reports_needing_retune());
        self.max_pending_over_target = self
            .max_pending_over_target
            .max(summary.max_pending_items_over_target());
        self.max_queued_bytes_over_budget = self
            .max_queued_bytes_over_budget
            .max(summary.max_queued_bytes_over_budget());
        self.queue_backpressure_reports = self
            .queue_backpressure_reports
            .saturating_add(summary.queue_backpressure_reports());
    }

    fn observe_decision(&mut self, decision: RemoteFreeServiceRetuneGuardDecision) {
        match decision {
            RemoteFreeServiceRetuneGuardDecision::CollectTelemetry
            | RemoteFreeServiceRetuneGuardDecision::Hold { .. } => {
                self.hold_decisions = self.hold_decisions.saturating_add(1);
            }
            RemoteFreeServiceRetuneGuardDecision::Apply { .. } => {
                self.apply_decisions = self.apply_decisions.saturating_add(1);
            }
            RemoteFreeServiceRetuneGuardDecision::Rollback { .. } => {
                self.rollback_decisions = self.rollback_decisions.saturating_add(1);
            }
            RemoteFreeServiceRetuneGuardDecision::Confirmed { .. }
            | RemoteFreeServiceRetuneGuardDecision::MutationLimitReached { .. } => {}
        }
    }
}

fn assert_runtime_rollback_stats(stats: RuntimeRollbackStats) {
    assert_eq!(
        stats.runtime.submitted_count,
        RUNTIME_ROLLBACK_WINDOWS * 256
    );
    assert_eq!(stats.runtime.drained_count, stats.runtime.submitted_count);
    assert_eq!(
        stats.runtime.released_bytes,
        512_u64
            .saturating_mul(BYTES_PER_BLOCK)
            .saturating_add(256_u64.saturating_mul(RUNTIME_ROLLBACK_VALIDATION_BYTES))
    );
    assert_eq!(stats.runtime.policy_drains, 8);
    assert_eq!(stats.runtime.drain_rounds, 16);
    assert_eq!(stats.runtime.install_count, 1);
    assert_eq!(stats.runtime.confirm_count, 0);
    assert_eq!(stats.runtime.rollback_count, 1);
    assert_eq!(stats.runtime.max_wait_bursts, 4);
    assert_eq!(stats.runtime.mean_wait_milli(), 2_333);
    assert_eq!(
        stats.runtime.final_queue_capacity,
        RUNTIME_INITIAL_QUEUE_CAPACITY
    );
    assert!(!stats.runtime.final_previous_config_present);

    assert_eq!(stats.observed_reports, RUNTIME_ROLLBACK_WINDOWS * 8);
    assert_eq!(stats.reports_needing_retune, 22);
    assert_eq!(stats.max_pending_over_target, 64);
    assert_eq!(stats.max_queued_bytes_over_budget, 262_144);
    assert_eq!(stats.queue_backpressure_reports, 12);
    assert_eq!(stats.hold_decisions, 1);
    assert_eq!(stats.apply_decisions, 1);
    assert_eq!(stats.rollback_decisions, 1);
    assert_eq!(stats.runtime_no_change_decisions, 1);
    assert_eq!(stats.final_guard_applied_mutations, 1);
    assert_eq!(stats.final_guard_confirmed_mutations, 0);
    assert_eq!(stats.final_guard_rollbacks, 1);
    assert_eq!(stats.final_guard_pending_candidate, None);
}

fn option_candidate_label(candidate: Option<RemoteFreeServiceRetuneCandidate>) -> &'static str {
    candidate.map_or("none", RemoteFreeServiceRetuneCandidate::as_str)
}
