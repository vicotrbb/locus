#![allow(missing_docs)]

use criterion::{black_box, Criterion};
use locus_alloc::{
    RemoteFreeDrainObservation, RemoteFreeOwnerRuntime, RemoteFreeOwnerRuntimeApplyOutcome,
    RemoteFreeOwnerRuntimeConfirmOutcome, RemoteFreeOwnerRuntimeRollbackOutcome,
    RemoteFreeQueueStats, RemoteFreeQueuedByteDriftReport, RemoteFreeServiceRetuneCandidate,
    RemoteFreeServiceRetuneGuard, RemoteFreeServiceRetuneGuardDecision,
    RemoteFreeServiceRetunePolicyApplicator, RemoteFreeServiceRetuneSummary,
};

use crate::remote_free_service_application_harness::{
    run_runtime_owner_window, service_config, RuntimeApplicationStats, RuntimeTraceBlock,
    QUEUE_CAPACITY_GROWTH_FACTOR, RUNTIME_INITIAL_QUEUE_CAPACITY,
};
use crate::remote_free_service_harness::{format_milli, CounterSummary, BYTES_PER_BLOCK, SAMPLES};

const GUARDED_RUNTIME_STABLE_WINDOWS: u64 = 2;
const GUARDED_RUNTIME_MAX_MUTATIONS: u64 = 2;
const GUARDED_RUNTIME_WINDOWS: u64 = 9;
const GUARDED_RUNTIME_WINDOWS_USIZE: usize = 9;

#[derive(Debug, Clone, Copy)]
enum RuntimeSummaryKind {
    Clean,
    DrainEarlier,
    Combined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeGuardDecisionKind {
    Hold,
    Apply,
    Confirmed,
    Rollback,
    MutationLimitReached,
}

#[derive(Debug, Clone, Copy)]
struct RuntimeGuardStep {
    summary: RuntimeSummaryKind,
    candidate: RemoteFreeServiceRetuneCandidate,
    decision: RuntimeGuardDecisionKind,
}

#[derive(Debug, Clone, Copy)]
struct GuardedRuntimeStats {
    runtime: RuntimeApplicationStats,
    observed_reports: u64,
    reports_needing_retune: u64,
    max_pending_over_target: u64,
    max_queued_bytes_over_budget: u64,
    queue_backpressure_reports: u64,
    hold_decisions: u64,
    apply_decisions: u64,
    confirmed_decisions: u64,
    rollback_decisions: u64,
    mutation_limit_decisions: u64,
    drain_earlier_apply_decisions: u64,
    combined_apply_decisions: u64,
    runtime_no_change_decisions: u64,
    final_guard_applied_mutations: u64,
    final_guard_confirmed_mutations: u64,
    final_guard_rollbacks: u64,
    final_guard_pending_candidate: Option<RemoteFreeServiceRetuneCandidate>,
}

pub(crate) fn benchmark_guarded_runtime_sequence(c: &mut Criterion) {
    print_guarded_runtime_sample();
    print_guarded_runtime_sample_summary();
    c.bench_function("remote_free_service_guarded_runtime_sequence", |b| {
        b.iter(|| {
            let stats = run_guarded_runtime_sequence();
            assert_guarded_runtime_stats(stats);
            black_box(stats);
        });
    });
}

fn print_guarded_runtime_sample() {
    let stats = run_guarded_runtime_sequence();
    assert_guarded_runtime_stats(stats);

    println!(
        "remote_free_service_guarded_runtime_sample windows={GUARDED_RUNTIME_WINDOWS} stable_windows={GUARDED_RUNTIME_STABLE_WINDOWS} max_mutations={GUARDED_RUNTIME_MAX_MUTATIONS} submitted_count={} drained_count={} released_bytes={} policy_drains={} drain_rounds={} observed_reports={} reports_needing_retune={} max_pending_over_target={} max_queued_bytes_over_budget={} queue_backpressure_reports={} hold_decisions={} apply_decisions={} confirmed_decisions={} rollback_decisions={} mutation_limit_decisions={} runtime_install_count={} runtime_confirm_count={} runtime_rollback_count={} runtime_no_change_decisions={} drain_earlier_apply_decisions={} combined_apply_decisions={} max_wait_bursts={} mean_wait_bursts={} final_queue_capacity={} final_previous_config_present={} final_guard_pending_candidate={} final_guard_applied_mutations={} final_guard_confirmed_mutations={} final_guard_rollbacks={}",
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
        stats.rollback_decisions,
        stats.mutation_limit_decisions,
        stats.runtime.install_count,
        stats.runtime.confirm_count,
        stats.runtime.rollback_count,
        stats.runtime_no_change_decisions,
        stats.drain_earlier_apply_decisions,
        stats.combined_apply_decisions,
        stats.runtime.max_wait_bursts,
        format_milli(stats.runtime.mean_wait_milli()),
        stats.runtime.final_queue_capacity,
        stats.runtime.final_previous_config_present,
        option_candidate_label(stats.final_guard_pending_candidate),
        stats.final_guard_applied_mutations,
        stats.final_guard_confirmed_mutations,
        stats.final_guard_rollbacks,
    );
}

fn print_guarded_runtime_sample_summary() {
    let mut policy_drains = CounterSummary::new();
    let mut drain_rounds = CounterSummary::new();
    let mut reports_needing_retune = CounterSummary::new();
    let mut apply_decisions = CounterSummary::new();
    let mut confirmed_decisions = CounterSummary::new();
    let mut rollback_decisions = CounterSummary::new();
    let mut mutation_limit_decisions = CounterSummary::new();
    let mut max_wait = CounterSummary::new();
    let mut mean_wait = CounterSummary::new();

    for _ in 0..SAMPLES {
        let stats = run_guarded_runtime_sequence();
        assert_guarded_runtime_stats(stats);

        policy_drains.observe(stats.runtime.policy_drains);
        drain_rounds.observe(stats.runtime.drain_rounds);
        reports_needing_retune.observe(stats.reports_needing_retune);
        apply_decisions.observe(stats.apply_decisions);
        confirmed_decisions.observe(stats.confirmed_decisions);
        rollback_decisions.observe(stats.rollback_decisions);
        mutation_limit_decisions.observe(stats.mutation_limit_decisions);
        max_wait.observe(stats.runtime.max_wait_bursts);
        mean_wait.observe(stats.runtime.mean_wait_milli());
    }

    println!(
        "remote_free_service_guarded_runtime_sample_summary windows={GUARDED_RUNTIME_WINDOWS} samples={SAMPLES} policy_drains_min={} policy_drains_max={} policy_drains_mean={} drain_rounds_min={} drain_rounds_max={} drain_rounds_mean={} reports_needing_retune_min={} reports_needing_retune_max={} reports_needing_retune_mean={} apply_decisions_min={} apply_decisions_max={} apply_decisions_mean={} confirmed_decisions_min={} confirmed_decisions_max={} confirmed_decisions_mean={} rollback_decisions_min={} rollback_decisions_max={} rollback_decisions_mean={} mutation_limit_decisions_min={} mutation_limit_decisions_max={} mutation_limit_decisions_mean={} max_wait_min={} max_wait_max={} max_wait_mean={} mean_wait_min={} mean_wait_max={} mean_wait_mean={}",
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
        rollback_decisions.min,
        rollback_decisions.max,
        format_milli(rollback_decisions.mean_milli(SAMPLES)),
        mutation_limit_decisions.min,
        mutation_limit_decisions.max,
        format_milli(mutation_limit_decisions.mean_milli(SAMPLES)),
        max_wait.min,
        max_wait.max,
        format_milli(max_wait.mean_milli(SAMPLES)),
        format_milli(mean_wait.min),
        format_milli(mean_wait.max),
        format_milli(mean_wait.mean_milli(SAMPLES) / 1000),
    );
}

fn run_guarded_runtime_sequence() -> GuardedRuntimeStats {
    let mut guard = RemoteFreeServiceRetuneGuard::try_new(
        GUARDED_RUNTIME_STABLE_WINDOWS,
        GUARDED_RUNTIME_MAX_MUTATIONS,
    )
    .expect("guard");
    let mut runtime = RemoteFreeOwnerRuntime::new(service_config(RUNTIME_INITIAL_QUEUE_CAPACITY))
        .expect("runtime");
    let applicator = RemoteFreeServiceRetunePolicyApplicator::try_new(QUEUE_CAPACITY_GROWTH_FACTOR)
        .expect("applicator");
    let mut stats = GuardedRuntimeStats::new();

    for step in guarded_runtime_steps() {
        run_runtime_owner_window(&mut runtime, &mut stats.runtime);

        let summary = summary_for_kind(step.summary);
        stats.observe_summary(summary);
        let decision = guard.observe_summary(summary);
        assert_runtime_guard_decision(step, decision);
        stats.observe_decision(decision);
        apply_runtime_decision(&mut runtime, applicator, decision, &mut stats);
    }

    stats.runtime.final_queue_capacity = runtime.config().queue_capacity();
    stats.runtime.final_previous_config_present = runtime.previous_config().is_some();
    stats.final_guard_applied_mutations = guard.applied_mutations();
    stats.final_guard_confirmed_mutations = guard.confirmed_mutations();
    stats.final_guard_rollbacks = guard.rollbacks();
    stats.final_guard_pending_candidate = guard.pending_validation();
    stats
}

fn guarded_runtime_steps() -> [RuntimeGuardStep; GUARDED_RUNTIME_WINDOWS_USIZE] {
    [
        RuntimeGuardStep {
            summary: RuntimeSummaryKind::Clean,
            candidate: RemoteFreeServiceRetuneCandidate::KeepConfig,
            decision: RuntimeGuardDecisionKind::Hold,
        },
        RuntimeGuardStep {
            summary: RuntimeSummaryKind::DrainEarlier,
            candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
            decision: RuntimeGuardDecisionKind::Hold,
        },
        RuntimeGuardStep {
            summary: RuntimeSummaryKind::DrainEarlier,
            candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
            decision: RuntimeGuardDecisionKind::Apply,
        },
        RuntimeGuardStep {
            summary: RuntimeSummaryKind::Clean,
            candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
            decision: RuntimeGuardDecisionKind::Confirmed,
        },
        RuntimeGuardStep {
            summary: RuntimeSummaryKind::Combined,
            candidate: RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier,
            decision: RuntimeGuardDecisionKind::Hold,
        },
        RuntimeGuardStep {
            summary: RuntimeSummaryKind::Combined,
            candidate: RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier,
            decision: RuntimeGuardDecisionKind::Apply,
        },
        RuntimeGuardStep {
            summary: RuntimeSummaryKind::DrainEarlier,
            candidate: RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier,
            decision: RuntimeGuardDecisionKind::Rollback,
        },
        RuntimeGuardStep {
            summary: RuntimeSummaryKind::DrainEarlier,
            candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
            decision: RuntimeGuardDecisionKind::Hold,
        },
        RuntimeGuardStep {
            summary: RuntimeSummaryKind::DrainEarlier,
            candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
            decision: RuntimeGuardDecisionKind::MutationLimitReached,
        },
    ]
}

fn summary_for_kind(kind: RuntimeSummaryKind) -> RemoteFreeServiceRetuneSummary {
    let mut summary = RemoteFreeServiceRetuneSummary::new();
    summary.observe_report(report_for_kind(kind));
    summary
}

fn report_for_kind(kind: RuntimeSummaryKind) -> RemoteFreeQueuedByteDriftReport {
    match kind {
        RuntimeSummaryKind::Clean => report(64, 262_144, 0),
        RuntimeSummaryKind::DrainEarlier => report(96, 524_288, 0),
        RuntimeSummaryKind::Combined => report(96, 524_288, 1),
    }
}

fn report(
    pending_count: u64,
    queued_bytes: u64,
    full_count: u64,
) -> RemoteFreeQueuedByteDriftReport {
    RemoteFreeQueuedByteDriftReport::from_observation(
        service_config(RUNTIME_INITIAL_QUEUE_CAPACITY),
        RemoteFreeQueueStats {
            capacity: RUNTIME_INITIAL_QUEUE_CAPACITY,
            batch_limit: 64,
            submitted_count: pending_count,
            pending_count,
            full_count,
            disconnected_count: 0,
            drained_count: 0,
        },
        RemoteFreeDrainObservation::new(pending_count, queued_bytes, 1),
    )
}

fn apply_runtime_decision(
    runtime: &mut RemoteFreeOwnerRuntime<RuntimeTraceBlock>,
    applicator: RemoteFreeServiceRetunePolicyApplicator,
    decision: RemoteFreeServiceRetuneGuardDecision,
    stats: &mut GuardedRuntimeStats,
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
    }
}

fn assert_runtime_guard_decision(
    expected_step: RuntimeGuardStep,
    decision: RemoteFreeServiceRetuneGuardDecision,
) {
    assert_eq!(
        RuntimeGuardDecisionKind::from_decision(decision),
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

impl RuntimeGuardDecisionKind {
    fn from_decision(decision: RemoteFreeServiceRetuneGuardDecision) -> Self {
        match decision {
            RemoteFreeServiceRetuneGuardDecision::CollectTelemetry
            | RemoteFreeServiceRetuneGuardDecision::Hold { .. } => Self::Hold,
            RemoteFreeServiceRetuneGuardDecision::Apply { .. } => Self::Apply,
            RemoteFreeServiceRetuneGuardDecision::Confirmed { .. } => Self::Confirmed,
            RemoteFreeServiceRetuneGuardDecision::Rollback { .. } => Self::Rollback,
            RemoteFreeServiceRetuneGuardDecision::MutationLimitReached { .. } => {
                Self::MutationLimitReached
            }
        }
    }
}

impl GuardedRuntimeStats {
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
            rollback_decisions: 0,
            mutation_limit_decisions: 0,
            drain_earlier_apply_decisions: 0,
            combined_apply_decisions: 0,
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
            RemoteFreeServiceRetuneGuardDecision::Apply { candidate } => {
                self.apply_decisions = self.apply_decisions.saturating_add(1);
                match candidate {
                    RemoteFreeServiceRetuneCandidate::DrainEarlier => {
                        self.drain_earlier_apply_decisions =
                            self.drain_earlier_apply_decisions.saturating_add(1);
                    }
                    RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier => {
                        self.combined_apply_decisions =
                            self.combined_apply_decisions.saturating_add(1);
                    }
                    _ => {}
                }
            }
            RemoteFreeServiceRetuneGuardDecision::Confirmed { .. } => {
                self.confirmed_decisions = self.confirmed_decisions.saturating_add(1);
            }
            RemoteFreeServiceRetuneGuardDecision::Rollback { .. } => {
                self.rollback_decisions = self.rollback_decisions.saturating_add(1);
            }
            RemoteFreeServiceRetuneGuardDecision::MutationLimitReached { .. } => {
                self.mutation_limit_decisions = self.mutation_limit_decisions.saturating_add(1);
            }
        }
    }
}

fn assert_guarded_runtime_stats(stats: GuardedRuntimeStats) {
    assert_eq!(stats.runtime.submitted_count, GUARDED_RUNTIME_WINDOWS * 256);
    assert_eq!(stats.runtime.drained_count, stats.runtime.submitted_count);
    assert_eq!(
        stats.runtime.released_bytes,
        stats
            .runtime
            .submitted_count
            .saturating_mul(BYTES_PER_BLOCK)
    );
    assert_eq!(stats.runtime.policy_drains, GUARDED_RUNTIME_WINDOWS * 4);
    assert_eq!(stats.runtime.drain_rounds, GUARDED_RUNTIME_WINDOWS * 4);
    assert_eq!(stats.runtime.install_count, 2);
    assert_eq!(stats.runtime.confirm_count, 1);
    assert_eq!(stats.runtime.rollback_count, 1);
    assert_eq!(stats.runtime.max_wait_bursts, 2);
    assert_eq!(stats.runtime.mean_wait_milli(), 1_500);
    assert_eq!(
        stats.runtime.final_queue_capacity,
        RUNTIME_INITIAL_QUEUE_CAPACITY
    );
    assert!(!stats.runtime.final_previous_config_present);

    assert_eq!(stats.observed_reports, GUARDED_RUNTIME_WINDOWS);
    assert_eq!(stats.reports_needing_retune, 7);
    assert_eq!(stats.max_pending_over_target, 32);
    assert_eq!(stats.max_queued_bytes_over_budget, 262_144);
    assert_eq!(stats.queue_backpressure_reports, 2);
    assert_eq!(stats.hold_decisions, 4);
    assert_eq!(stats.apply_decisions, 2);
    assert_eq!(stats.confirmed_decisions, 1);
    assert_eq!(stats.rollback_decisions, 1);
    assert_eq!(stats.mutation_limit_decisions, 1);
    assert_eq!(stats.drain_earlier_apply_decisions, 1);
    assert_eq!(stats.combined_apply_decisions, 1);
    assert_eq!(stats.runtime_no_change_decisions, 5);
    assert_eq!(stats.final_guard_applied_mutations, 2);
    assert_eq!(stats.final_guard_confirmed_mutations, 1);
    assert_eq!(stats.final_guard_rollbacks, 1);
    assert_eq!(stats.final_guard_pending_candidate, None);
}

fn option_candidate_label(candidate: Option<RemoteFreeServiceRetuneCandidate>) -> &'static str {
    candidate.map_or("none", RemoteFreeServiceRetuneCandidate::as_str)
}
