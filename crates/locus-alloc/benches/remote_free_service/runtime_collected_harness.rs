#![allow(missing_docs)]

use criterion::{black_box, Criterion};
use locus_alloc::{
    RemoteFreeDrainPolicy, RemoteFreeOwnerRuntime, RemoteFreeOwnerRuntimeApplyOutcome,
    RemoteFreeOwnerRuntimeConfirmOutcome, RemoteFreeServiceRetuneCandidate,
    RemoteFreeServiceRetuneGuard, RemoteFreeServiceRetuneGuardDecision,
    RemoteFreeServiceRetunePolicyApplicator, RemoteFreeServiceRetuneSummary,
};

use crate::remote_free_service_application_harness::{
    run_runtime_owner_window_with_summary, service_config, RuntimeApplicationStats,
    RuntimeTraceBlock, QUEUE_CAPACITY_GROWTH_FACTOR,
};
use crate::remote_free_service_harness::{
    format_milli, CounterSummary, BYTES_PER_BLOCK, QUEUE_CAPACITY, SAMPLES,
};
use crate::remote_free_service_sample_filter::should_print_sample;
use crate::remote_free_service_sample_output::print_sample_line;

const RUNTIME_COLLECTED_STABLE_WINDOWS: u64 = 2;
const RUNTIME_COLLECTED_MAX_MUTATIONS: u64 = 2;
const RUNTIME_COLLECTED_WINDOWS: u64 = 3;
const RUNTIME_COLLECTED_BENCHMARK: &str = "remote_free_service_runtime_collected_guarded_confirm";
const RUNTIME_COLLECTED_SAMPLE: &str =
    "remote_free_service_runtime_collected_guarded_confirm_sample";
const RUNTIME_COLLECTED_SAMPLE_SUMMARY: &str =
    "remote_free_service_runtime_collected_guarded_confirm_sample_summary";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeCollectedDecisionKind {
    Hold,
    Apply,
    Confirmed,
}

#[derive(Debug, Clone, Copy)]
struct RuntimeCollectedExpectedStep {
    candidate: RemoteFreeServiceRetuneCandidate,
    decision: RuntimeCollectedDecisionKind,
}

#[derive(Debug, Clone, Copy)]
struct RuntimeCollectedStats {
    runtime: RuntimeApplicationStats,
    observed_reports: u64,
    reports_needing_retune: u64,
    max_pending_over_target: u64,
    max_queued_bytes_over_budget: u64,
    queue_backpressure_reports: u64,
    hold_decisions: u64,
    apply_decisions: u64,
    confirmed_decisions: u64,
    runtime_no_change_decisions: u64,
    final_guard_applied_mutations: u64,
    final_guard_confirmed_mutations: u64,
    final_guard_rollbacks: u64,
    final_guard_pending_candidate: Option<RemoteFreeServiceRetuneCandidate>,
}

pub(crate) fn benchmark_runtime_collected_guarded_confirm(c: &mut Criterion) {
    print_runtime_collected_sample();
    print_runtime_collected_sample_summary();

    c.bench_function(RUNTIME_COLLECTED_BENCHMARK, |bench| {
        bench.iter(|| {
            let stats = run_runtime_collected_guarded_confirm();
            assert_runtime_collected_stats(stats);
            black_box(stats);
        });
    });
}

fn print_runtime_collected_sample() {
    if !should_print_sample(RUNTIME_COLLECTED_SAMPLE, RUNTIME_COLLECTED_BENCHMARK) {
        return;
    }

    let stats = run_runtime_collected_guarded_confirm();
    assert_runtime_collected_stats(stats);

    print_sample_line(
        RUNTIME_COLLECTED_BENCHMARK,
        format_args!(
        "{RUNTIME_COLLECTED_SAMPLE} windows={RUNTIME_COLLECTED_WINDOWS} stable_windows={RUNTIME_COLLECTED_STABLE_WINDOWS} max_mutations={RUNTIME_COLLECTED_MAX_MUTATIONS} submitted_count={} drained_count={} released_bytes={} policy_drains={} drain_rounds={} observed_reports={} reports_needing_retune={} max_pending_over_target={} max_queued_bytes_over_budget={} queue_backpressure_reports={} hold_decisions={} apply_decisions={} confirmed_decisions={} runtime_install_count={} runtime_confirm_count={} runtime_rollback_count={} runtime_no_change_decisions={} max_wait_bursts={} mean_wait_bursts={} final_queue_capacity={} final_previous_config_present={} final_guard_pending_candidate={} final_guard_applied_mutations={} final_guard_confirmed_mutations={} final_guard_rollbacks={}",
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
        stats.confirmed_decisions,
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

fn print_runtime_collected_sample_summary() {
    if !should_print_sample(
        RUNTIME_COLLECTED_SAMPLE_SUMMARY,
        RUNTIME_COLLECTED_BENCHMARK,
    ) {
        return;
    }

    let mut policy_drains = CounterSummary::new();
    let mut drain_rounds = CounterSummary::new();
    let mut reports_needing_retune = CounterSummary::new();
    let mut apply_decisions = CounterSummary::new();
    let mut confirmed_decisions = CounterSummary::new();
    let mut max_wait = CounterSummary::new();
    let mut mean_wait = CounterSummary::new();

    for _ in 0..SAMPLES {
        let stats = run_runtime_collected_guarded_confirm();
        assert_runtime_collected_stats(stats);

        policy_drains.observe(stats.runtime.policy_drains);
        drain_rounds.observe(stats.runtime.drain_rounds);
        reports_needing_retune.observe(stats.reports_needing_retune);
        apply_decisions.observe(stats.apply_decisions);
        confirmed_decisions.observe(stats.confirmed_decisions);
        max_wait.observe(stats.runtime.max_wait_bursts);
        mean_wait.observe(stats.runtime.mean_wait_milli());
    }

    print_sample_line(
        RUNTIME_COLLECTED_BENCHMARK,
        format_args!(
        "{RUNTIME_COLLECTED_SAMPLE_SUMMARY} windows={RUNTIME_COLLECTED_WINDOWS} samples={SAMPLES} policy_drains_min={} policy_drains_max={} policy_drains_mean={} drain_rounds_min={} drain_rounds_max={} drain_rounds_mean={} reports_needing_retune_min={} reports_needing_retune_max={} reports_needing_retune_mean={} apply_decisions_min={} apply_decisions_max={} apply_decisions_mean={} confirmed_decisions_min={} confirmed_decisions_max={} confirmed_decisions_mean={} max_wait_min={} max_wait_max={} max_wait_mean={} mean_wait_min={} mean_wait_max={} mean_wait_mean={}",
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
        confirmed_decisions.min,
        confirmed_decisions.max,
        format_milli(confirmed_decisions.mean_milli(SAMPLES)),
        max_wait.min,
        max_wait.max,
        format_milli(max_wait.mean_milli(SAMPLES)),
        format_milli(mean_wait.min),
        format_milli(mean_wait.max),
        format_milli(mean_wait.mean_milli(SAMPLES) / 1000),
    ));
}

fn run_runtime_collected_guarded_confirm() -> RuntimeCollectedStats {
    let mut guard = RemoteFreeServiceRetuneGuard::try_new(
        RUNTIME_COLLECTED_STABLE_WINDOWS,
        RUNTIME_COLLECTED_MAX_MUTATIONS,
    )
    .expect("guard");
    let mut runtime = RemoteFreeOwnerRuntime::new_with_drain_policy(
        service_config(QUEUE_CAPACITY),
        RemoteFreeDrainPolicy::new(),
    )
    .expect("runtime");
    let applicator = RemoteFreeServiceRetunePolicyApplicator::try_new(QUEUE_CAPACITY_GROWTH_FACTOR)
        .expect("applicator");
    let mut stats = RuntimeCollectedStats::new();

    for expected_step in runtime_collected_expected_steps() {
        let summary = run_runtime_owner_window_with_summary(&mut runtime, &mut stats.runtime);
        stats.observe_summary(summary);

        let decision = guard.observe_summary(summary);
        assert_runtime_collected_decision(expected_step, decision);
        stats.observe_decision(decision);
        apply_runtime_collected_decision(&mut runtime, applicator, decision, &mut stats);
    }

    stats.runtime.final_queue_capacity = runtime.config().queue_capacity();
    stats.runtime.final_previous_config_present = runtime.previous_config().is_some();
    stats.final_guard_applied_mutations = guard.applied_mutations();
    stats.final_guard_confirmed_mutations = guard.confirmed_mutations();
    stats.final_guard_rollbacks = guard.rollbacks();
    stats.final_guard_pending_candidate = guard.pending_validation();
    stats
}

fn runtime_collected_expected_steps() -> [RuntimeCollectedExpectedStep; 3] {
    [
        RuntimeCollectedExpectedStep {
            candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
            decision: RuntimeCollectedDecisionKind::Hold,
        },
        RuntimeCollectedExpectedStep {
            candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
            decision: RuntimeCollectedDecisionKind::Apply,
        },
        RuntimeCollectedExpectedStep {
            candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
            decision: RuntimeCollectedDecisionKind::Confirmed,
        },
    ]
}

fn apply_runtime_collected_decision(
    runtime: &mut RemoteFreeOwnerRuntime<RuntimeTraceBlock>,
    applicator: RemoteFreeServiceRetunePolicyApplicator,
    decision: RemoteFreeServiceRetuneGuardDecision,
    stats: &mut RuntimeCollectedStats,
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
        RemoteFreeServiceRetuneGuardDecision::Confirmed { .. } => {
            let previous_config = runtime.previous_config();
            assert_eq!(
                runtime.confirm(),
                Ok(RemoteFreeOwnerRuntimeConfirmOutcome {
                    confirmed_config: runtime.config(),
                    cleared_previous_config: previous_config,
                })
            );
            stats.runtime.confirm_count = stats.runtime.confirm_count.saturating_add(1);
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
        RemoteFreeServiceRetuneGuardDecision::Rollback { .. } => {
            panic!("runtime-collected confirm sequence should not roll back");
        }
    }
}

fn assert_runtime_collected_decision(
    expected_step: RuntimeCollectedExpectedStep,
    decision: RemoteFreeServiceRetuneGuardDecision,
) {
    assert_eq!(
        RuntimeCollectedDecisionKind::from_decision(decision),
        expected_step.decision
    );

    match decision {
        RemoteFreeServiceRetuneGuardDecision::Hold { candidate, .. }
        | RemoteFreeServiceRetuneGuardDecision::Apply { candidate }
        | RemoteFreeServiceRetuneGuardDecision::Confirmed { candidate }
        | RemoteFreeServiceRetuneGuardDecision::Rollback { candidate, .. }
        | RemoteFreeServiceRetuneGuardDecision::MutationLimitReached { candidate } => {
            assert_eq!(candidate, expected_step.candidate);
        }
        RemoteFreeServiceRetuneGuardDecision::CollectTelemetry => {}
    }
}

impl RuntimeCollectedDecisionKind {
    fn from_decision(decision: RemoteFreeServiceRetuneGuardDecision) -> Self {
        match decision {
            RemoteFreeServiceRetuneGuardDecision::CollectTelemetry
            | RemoteFreeServiceRetuneGuardDecision::Hold { .. } => Self::Hold,
            RemoteFreeServiceRetuneGuardDecision::Apply { .. } => Self::Apply,
            RemoteFreeServiceRetuneGuardDecision::Confirmed { .. } => Self::Confirmed,
            RemoteFreeServiceRetuneGuardDecision::Rollback { .. }
            | RemoteFreeServiceRetuneGuardDecision::MutationLimitReached { .. } => {
                panic!("unexpected runtime-collected confirm decision")
            }
        }
    }
}

impl RuntimeCollectedStats {
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
            confirmed_decisions: 0,
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
            RemoteFreeServiceRetuneGuardDecision::Confirmed { .. } => {
                self.confirmed_decisions = self.confirmed_decisions.saturating_add(1);
            }
            RemoteFreeServiceRetuneGuardDecision::Rollback { .. }
            | RemoteFreeServiceRetuneGuardDecision::MutationLimitReached { .. } => {}
        }
    }
}

fn assert_runtime_collected_stats(stats: RuntimeCollectedStats) {
    assert_eq!(
        stats.runtime.submitted_count,
        RUNTIME_COLLECTED_WINDOWS * 256
    );
    assert_eq!(stats.runtime.drained_count, stats.runtime.submitted_count);
    assert_eq!(
        stats.runtime.released_bytes,
        stats
            .runtime
            .submitted_count
            .saturating_mul(BYTES_PER_BLOCK)
    );
    assert_eq!(stats.runtime.policy_drains, 4);
    assert_eq!(stats.runtime.drain_rounds, 12);
    assert_eq!(stats.runtime.install_count, 1);
    assert_eq!(stats.runtime.confirm_count, 1);
    assert_eq!(stats.runtime.rollback_count, 0);
    assert_eq!(stats.runtime.max_wait_bursts, 8);
    assert_eq!(stats.runtime.mean_wait_milli(), 3_500);
    assert_eq!(stats.runtime.final_queue_capacity, QUEUE_CAPACITY);
    assert!(!stats.runtime.final_previous_config_present);

    assert_eq!(stats.observed_reports, RUNTIME_COLLECTED_WINDOWS * 8);
    assert_eq!(stats.reports_needing_retune, 12);
    assert_eq!(stats.max_pending_over_target, 192);
    assert_eq!(stats.max_queued_bytes_over_budget, 786_432);
    assert_eq!(stats.queue_backpressure_reports, 0);
    assert_eq!(stats.hold_decisions, 1);
    assert_eq!(stats.apply_decisions, 1);
    assert_eq!(stats.confirmed_decisions, 1);
    assert_eq!(stats.runtime_no_change_decisions, 1);
    assert_eq!(stats.final_guard_applied_mutations, 1);
    assert_eq!(stats.final_guard_confirmed_mutations, 1);
    assert_eq!(stats.final_guard_rollbacks, 0);
    assert_eq!(stats.final_guard_pending_candidate, None);
}

fn option_candidate_label(candidate: Option<RemoteFreeServiceRetuneCandidate>) -> &'static str {
    candidate.map_or("none", RemoteFreeServiceRetuneCandidate::as_str)
}
