#![allow(missing_docs)]

use criterion::{black_box, Criterion};
use locus_alloc::{
    RemoteFreeDrainPolicy, RemoteFreeOwnerRuntime, RemoteFreeOwnerRuntimeApplyOutcome,
    RemoteFreeServiceRetuneCandidate, RemoteFreeServiceRetuneGuard,
    RemoteFreeServiceRetuneGuardDecision, RemoteFreeServiceRetunePolicyApplicator,
    RemoteFreeServiceRuntimeOwnerId, RemoteFreeServiceRuntimeRetuneCoordinator,
    RemoteFreeServiceRuntimeRetuneOutcome, RemoteFreeServiceRuntimeRetuneOwners,
};

use crate::remote_free_service_application_harness::{
    run_runtime_owner_window_with_summary, run_runtime_owner_window_with_summary_and_block_bytes,
    service_config, RuntimeApplicationStats, RuntimeTraceBlock, QUEUE_CAPACITY_GROWTH_FACTOR,
    RUNTIME_INITIAL_QUEUE_CAPACITY,
};
use crate::remote_free_service_harness::{
    format_milli, CounterSummary, BYTES_PER_BLOCK, QUEUE_CAPACITY, SAMPLES,
};

const REGISTRY_STABLE_WINDOWS: u64 = 2;
const REGISTRY_MAX_MUTATIONS: u64 = 2;
const REGISTRY_OWNERS: u64 = 3;
const REGISTRY_OWNERS_USIZE: usize = 3;
const REGISTRY_WINDOWS: u64 = 8;
const REGISTRY_ROLLBACK_VALIDATION_BYTES: u64 = BYTES_PER_BLOCK * 2 + 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RegistryDecisionKind {
    Hold,
    Apply,
    Confirmed,
    Rollback,
    MutationLimitReached,
}

#[derive(Debug, Clone, Copy)]
struct RegistryStats {
    runtime: RuntimeApplicationStats,
    registered_owners: usize,
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
    runtime_no_change_decisions: u64,
    missing_owner_checks: u64,
    final_guard_applied_mutations: u64,
    final_guard_confirmed_mutations: u64,
    final_guard_rollbacks: u64,
    final_guard_pending_candidate: Option<RemoteFreeServiceRetuneCandidate>,
}

pub(crate) fn benchmark_runtime_registry_sequence(c: &mut Criterion) {
    print_registry_sample();
    print_registry_sample_summary();

    c.bench_function("remote_free_service_runtime_registry_sequence", |bench| {
        bench.iter(|| {
            let stats = run_runtime_registry_sequence();
            assert_registry_stats(stats);
            black_box(stats);
        });
    });
}

fn print_registry_sample() {
    let stats = run_runtime_registry_sequence();
    assert_registry_stats(stats);

    println!(
        "remote_free_service_runtime_registry_sample owners={REGISTRY_OWNERS} windows={REGISTRY_WINDOWS} stable_windows={REGISTRY_STABLE_WINDOWS} max_mutations={REGISTRY_MAX_MUTATIONS} rollback_validation_bytes={REGISTRY_ROLLBACK_VALIDATION_BYTES} submitted_count={} drained_count={} released_bytes={} policy_drains={} drain_rounds={} registered_owners={} observed_reports={} reports_needing_retune={} max_pending_over_target={} max_queued_bytes_over_budget={} queue_backpressure_reports={} hold_decisions={} apply_decisions={} confirmed_decisions={} rollback_decisions={} mutation_limit_decisions={} runtime_install_count={} runtime_confirm_count={} runtime_rollback_count={} runtime_no_change_decisions={} missing_owner_checks={} max_wait_bursts={} mean_wait_bursts={} final_queue_capacity={} final_previous_config_present={} final_guard_pending_candidate={} final_guard_applied_mutations={} final_guard_confirmed_mutations={} final_guard_rollbacks={}",
        stats.runtime.submitted_count,
        stats.runtime.drained_count,
        stats.runtime.released_bytes,
        stats.runtime.policy_drains,
        stats.runtime.drain_rounds,
        stats.registered_owners,
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
        stats.missing_owner_checks,
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

fn print_registry_sample_summary() {
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
        let stats = run_runtime_registry_sequence();
        assert_registry_stats(stats);

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
        "remote_free_service_runtime_registry_sample_summary owners={REGISTRY_OWNERS} windows={REGISTRY_WINDOWS} samples={SAMPLES} policy_drains_min={} policy_drains_max={} policy_drains_mean={} drain_rounds_min={} drain_rounds_max={} drain_rounds_mean={} reports_needing_retune_min={} reports_needing_retune_max={} reports_needing_retune_mean={} apply_decisions_min={} apply_decisions_max={} apply_decisions_mean={} confirmed_decisions_min={} confirmed_decisions_max={} confirmed_decisions_mean={} rollback_decisions_min={} rollback_decisions_max={} rollback_decisions_mean={} mutation_limit_decisions_min={} mutation_limit_decisions_max={} mutation_limit_decisions_mean={} max_wait_min={} max_wait_max={} max_wait_mean={} mean_wait_min={} mean_wait_max={} mean_wait_mean={}",
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

fn run_runtime_registry_sequence() -> RegistryStats {
    let mut owners =
        RemoteFreeServiceRuntimeRetuneOwners::new(RemoteFreeServiceRuntimeRetuneCoordinator::new(
            RemoteFreeServiceRetuneGuard::try_new(REGISTRY_STABLE_WINDOWS, REGISTRY_MAX_MUTATIONS)
                .expect("guard"),
            RemoteFreeServiceRetunePolicyApplicator::try_new(QUEUE_CAPACITY_GROWTH_FACTOR)
                .expect("applicator"),
        ));

    let confirmed = owners.register_owner(runtime_with_initial_empty_policy(QUEUE_CAPACITY));
    let rolled_back = owners.register_owner(runtime_with_initial_empty_policy(
        RUNTIME_INITIAL_QUEUE_CAPACITY,
    ));
    let mutation_limited = owners.register_owner(runtime_with_initial_empty_policy(QUEUE_CAPACITY));

    let mut stats = RegistryStats::new();
    stats.registered_owners = owners.len();

    run_confirmed_owner(&mut owners, confirmed, &mut stats);
    run_rolled_back_owner(&mut owners, rolled_back, &mut stats);
    run_mutation_limited_owner(&mut owners, mutation_limited, &mut stats);

    assert!(owners
        .owner(RemoteFreeServiceRuntimeOwnerId::new(usize::MAX))
        .is_none());
    stats.missing_owner_checks = stats.missing_owner_checks.saturating_add(1);

    let guard = owners.coordinator().guard();
    stats.final_guard_applied_mutations = guard.applied_mutations();
    stats.final_guard_confirmed_mutations = guard.confirmed_mutations();
    stats.final_guard_rollbacks = guard.rollbacks();
    stats.final_guard_pending_candidate = guard.pending_validation();
    stats
}

fn run_confirmed_owner(
    owners: &mut RemoteFreeServiceRuntimeRetuneOwners<RuntimeTraceBlock>,
    owner_id: RemoteFreeServiceRuntimeOwnerId,
    stats: &mut RegistryStats,
) {
    observe_window(
        owners,
        owner_id,
        stats,
        RegistryDecisionKind::Hold,
        RemoteFreeServiceRetuneCandidate::DrainEarlier,
        None,
        BYTES_PER_BLOCK,
    );
    observe_window(
        owners,
        owner_id,
        stats,
        RegistryDecisionKind::Apply,
        RemoteFreeServiceRetuneCandidate::DrainEarlier,
        None,
        BYTES_PER_BLOCK,
    );
    observe_window(
        owners,
        owner_id,
        stats,
        RegistryDecisionKind::Confirmed,
        RemoteFreeServiceRetuneCandidate::DrainEarlier,
        None,
        BYTES_PER_BLOCK,
    );

    let runtime = owners.owner(owner_id).expect("confirmed owner");
    stats.runtime.final_queue_capacity = runtime.config().queue_capacity();
    stats.runtime.final_previous_config_present = runtime.previous_config().is_some();
}

fn run_rolled_back_owner(
    owners: &mut RemoteFreeServiceRuntimeRetuneOwners<RuntimeTraceBlock>,
    owner_id: RemoteFreeServiceRuntimeOwnerId,
    stats: &mut RegistryStats,
) {
    observe_window(
        owners,
        owner_id,
        stats,
        RegistryDecisionKind::Hold,
        RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier,
        None,
        BYTES_PER_BLOCK,
    );
    observe_window(
        owners,
        owner_id,
        stats,
        RegistryDecisionKind::Apply,
        RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier,
        None,
        BYTES_PER_BLOCK,
    );
    observe_window(
        owners,
        owner_id,
        stats,
        RegistryDecisionKind::Rollback,
        RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier,
        Some(RemoteFreeServiceRetuneCandidate::ReviewQueuedByteBudget),
        REGISTRY_ROLLBACK_VALIDATION_BYTES,
    );

    let runtime = owners.owner(owner_id).expect("rolled-back owner");
    assert_eq!(
        runtime.config().queue_capacity(),
        RUNTIME_INITIAL_QUEUE_CAPACITY
    );
    assert_eq!(runtime.previous_config(), None);
}

fn run_mutation_limited_owner(
    owners: &mut RemoteFreeServiceRuntimeRetuneOwners<RuntimeTraceBlock>,
    owner_id: RemoteFreeServiceRuntimeOwnerId,
    stats: &mut RegistryStats,
) {
    observe_window(
        owners,
        owner_id,
        stats,
        RegistryDecisionKind::Hold,
        RemoteFreeServiceRetuneCandidate::DrainEarlier,
        None,
        BYTES_PER_BLOCK,
    );
    observe_window(
        owners,
        owner_id,
        stats,
        RegistryDecisionKind::MutationLimitReached,
        RemoteFreeServiceRetuneCandidate::DrainEarlier,
        None,
        BYTES_PER_BLOCK,
    );

    let runtime = owners.owner(owner_id).expect("mutation-limited owner");
    stats.runtime.final_queue_capacity = runtime.config().queue_capacity();
    stats.runtime.final_previous_config_present = runtime.previous_config().is_some();
}

fn runtime_with_initial_empty_policy(
    queue_capacity: usize,
) -> RemoteFreeOwnerRuntime<RuntimeTraceBlock> {
    RemoteFreeOwnerRuntime::new_with_drain_policy(
        service_config(queue_capacity),
        RemoteFreeDrainPolicy::new(),
    )
    .expect("runtime")
}

fn observe_window(
    owners: &mut RemoteFreeServiceRuntimeRetuneOwners<RuntimeTraceBlock>,
    owner_id: RemoteFreeServiceRuntimeOwnerId,
    stats: &mut RegistryStats,
    expected_decision: RegistryDecisionKind,
    expected_candidate: RemoteFreeServiceRetuneCandidate,
    expected_observed_candidate: Option<RemoteFreeServiceRetuneCandidate>,
    block_bytes: u64,
) {
    let summary = {
        let runtime = owners.owner_mut(owner_id).expect("owner for window");
        if block_bytes == BYTES_PER_BLOCK {
            run_runtime_owner_window_with_summary(runtime, &mut stats.runtime)
        } else {
            run_runtime_owner_window_with_summary_and_block_bytes(
                runtime,
                &mut stats.runtime,
                block_bytes,
            )
        }
    };

    stats.observe_summary(summary);
    let outcome = owners
        .observe_owner_summary(owner_id, summary)
        .expect("registry outcome");
    assert_registry_outcome(
        outcome,
        expected_decision,
        expected_candidate,
        expected_observed_candidate,
    );
    stats.observe_outcome(outcome);
}

fn assert_registry_outcome(
    outcome: RemoteFreeServiceRuntimeRetuneOutcome,
    expected_decision: RegistryDecisionKind,
    expected_candidate: RemoteFreeServiceRetuneCandidate,
    expected_observed_candidate: Option<RemoteFreeServiceRetuneCandidate>,
) {
    assert_eq!(
        RegistryDecisionKind::from_decision(outcome.decision()),
        expected_decision
    );
    assert_decision_candidate(
        outcome.decision(),
        expected_candidate,
        expected_observed_candidate,
    );

    match (expected_decision, outcome) {
        (
            RegistryDecisionKind::Hold | RegistryDecisionKind::MutationLimitReached,
            RemoteFreeServiceRuntimeRetuneOutcome::NoChange {
                runtime: RemoteFreeOwnerRuntimeApplyOutcome::NoChange { .. },
                ..
            },
        )
        | (
            RegistryDecisionKind::Apply,
            RemoteFreeServiceRuntimeRetuneOutcome::Applied {
                runtime: RemoteFreeOwnerRuntimeApplyOutcome::Installed { .. },
                ..
            },
        )
        | (
            RegistryDecisionKind::Confirmed,
            RemoteFreeServiceRuntimeRetuneOutcome::Confirmed { .. },
        )
        | (
            RegistryDecisionKind::Rollback,
            RemoteFreeServiceRuntimeRetuneOutcome::RolledBack { .. },
        ) => {}
        _ => panic!("unexpected registry runtime outcome"),
    }
}

fn assert_decision_candidate(
    decision: RemoteFreeServiceRetuneGuardDecision,
    expected_candidate: RemoteFreeServiceRetuneCandidate,
    expected_observed_candidate: Option<RemoteFreeServiceRetuneCandidate>,
) {
    match decision {
        RemoteFreeServiceRetuneGuardDecision::Hold { candidate, .. }
        | RemoteFreeServiceRetuneGuardDecision::Apply { candidate }
        | RemoteFreeServiceRetuneGuardDecision::Confirmed { candidate }
        | RemoteFreeServiceRetuneGuardDecision::MutationLimitReached { candidate } => {
            assert_eq!(candidate, expected_candidate);
            assert_eq!(expected_observed_candidate, None);
        }
        RemoteFreeServiceRetuneGuardDecision::Rollback {
            candidate,
            observed_candidate,
        } => {
            assert_eq!(candidate, expected_candidate);
            assert_eq!(Some(observed_candidate), expected_observed_candidate);
        }
        RemoteFreeServiceRetuneGuardDecision::CollectTelemetry => {}
    }
}

impl RegistryDecisionKind {
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

impl RegistryStats {
    fn new() -> Self {
        Self {
            runtime: RuntimeApplicationStats::new(),
            registered_owners: 0,
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
            runtime_no_change_decisions: 0,
            missing_owner_checks: 0,
            final_guard_applied_mutations: 0,
            final_guard_confirmed_mutations: 0,
            final_guard_rollbacks: 0,
            final_guard_pending_candidate: None,
        }
    }

    fn observe_summary(&mut self, summary: locus_alloc::RemoteFreeServiceRetuneSummary) {
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

    fn observe_outcome(&mut self, outcome: RemoteFreeServiceRuntimeRetuneOutcome) {
        match outcome {
            RemoteFreeServiceRuntimeRetuneOutcome::NoChange { decision, .. } => {
                self.runtime_no_change_decisions =
                    self.runtime_no_change_decisions.saturating_add(1);
                match decision {
                    RemoteFreeServiceRetuneGuardDecision::CollectTelemetry
                    | RemoteFreeServiceRetuneGuardDecision::Hold { .. } => {
                        self.hold_decisions = self.hold_decisions.saturating_add(1);
                    }
                    RemoteFreeServiceRetuneGuardDecision::MutationLimitReached { .. } => {
                        self.mutation_limit_decisions =
                            self.mutation_limit_decisions.saturating_add(1);
                    }
                    _ => {}
                }
            }
            RemoteFreeServiceRuntimeRetuneOutcome::Applied { .. } => {
                self.apply_decisions = self.apply_decisions.saturating_add(1);
                self.runtime.install_count = self.runtime.install_count.saturating_add(1);
            }
            RemoteFreeServiceRuntimeRetuneOutcome::Confirmed { .. } => {
                self.confirmed_decisions = self.confirmed_decisions.saturating_add(1);
                self.runtime.confirm_count = self.runtime.confirm_count.saturating_add(1);
            }
            RemoteFreeServiceRuntimeRetuneOutcome::RolledBack { .. } => {
                self.rollback_decisions = self.rollback_decisions.saturating_add(1);
                self.runtime.rollback_count = self.runtime.rollback_count.saturating_add(1);
            }
        }
    }
}

fn assert_registry_stats(stats: RegistryStats) {
    assert_eq!(stats.runtime.submitted_count, REGISTRY_WINDOWS * 256);
    assert_eq!(stats.runtime.drained_count, stats.runtime.submitted_count);
    assert_eq!(
        stats.runtime.released_bytes,
        1_792_u64
            .saturating_mul(BYTES_PER_BLOCK)
            .saturating_add(256_u64.saturating_mul(REGISTRY_ROLLBACK_VALIDATION_BYTES))
    );
    assert_eq!(stats.runtime.policy_drains, 12);
    assert_eq!(stats.runtime.drain_rounds, 36);
    assert_eq!(stats.runtime.install_count, 2);
    assert_eq!(stats.runtime.confirm_count, 1);
    assert_eq!(stats.runtime.rollback_count, 1);
    assert_eq!(stats.runtime.max_wait_bursts, 8);
    assert_eq!(stats.runtime.mean_wait_milli(), 3_312);
    assert_eq!(stats.runtime.final_queue_capacity, QUEUE_CAPACITY);
    assert!(!stats.runtime.final_previous_config_present);

    assert_eq!(stats.registered_owners, REGISTRY_OWNERS_USIZE);
    assert_eq!(stats.observed_reports, REGISTRY_WINDOWS * 8);
    assert_eq!(stats.reports_needing_retune, 46);
    assert_eq!(stats.max_pending_over_target, 192);
    assert_eq!(stats.max_queued_bytes_over_budget, 786_432);
    assert_eq!(stats.queue_backpressure_reports, 12);
    assert_eq!(stats.hold_decisions, 3);
    assert_eq!(stats.apply_decisions, 2);
    assert_eq!(stats.confirmed_decisions, 1);
    assert_eq!(stats.rollback_decisions, 1);
    assert_eq!(stats.mutation_limit_decisions, 1);
    assert_eq!(stats.runtime_no_change_decisions, 4);
    assert_eq!(stats.missing_owner_checks, 1);
    assert_eq!(stats.final_guard_applied_mutations, 2);
    assert_eq!(stats.final_guard_confirmed_mutations, 1);
    assert_eq!(stats.final_guard_rollbacks, 1);
    assert_eq!(stats.final_guard_pending_candidate, None);
}

fn option_candidate_label(candidate: Option<RemoteFreeServiceRetuneCandidate>) -> &'static str {
    candidate.map_or("none", RemoteFreeServiceRetuneCandidate::as_str)
}
