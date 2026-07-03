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

const MULTI_OWNER_RUNTIME_STABLE_WINDOWS: u64 = 2;
const MULTI_OWNER_RUNTIME_MAX_MUTATIONS: u64 = 2;
const MULTI_OWNER_RUNTIME_OWNERS: u64 = 3;
const MULTI_OWNER_RUNTIME_WINDOWS: u64 = 8;
const MULTI_OWNER_RUNTIME_BENCHMARK: &str =
    "remote_free_service_runtime_collected_multi_owner_mutation_limit";
const MULTI_OWNER_RUNTIME_SAMPLE: &str =
    "remote_free_service_runtime_collected_multi_owner_mutation_limit_sample";
const MULTI_OWNER_RUNTIME_SAMPLE_SUMMARY: &str =
    "remote_free_service_runtime_collected_multi_owner_mutation_limit_sample_summary";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MultiOwnerDecisionKind {
    Hold,
    Apply,
    Confirmed,
    MutationLimitReached,
}

#[derive(Debug, Clone, Copy)]
struct MultiOwnerRuntimeStats {
    runtime: RuntimeApplicationStats,
    observed_reports: u64,
    reports_needing_retune: u64,
    max_pending_over_target: u64,
    max_queued_bytes_over_budget: u64,
    queue_backpressure_reports: u64,
    hold_decisions: u64,
    apply_decisions: u64,
    confirmed_decisions: u64,
    mutation_limit_decisions: u64,
    runtime_no_change_decisions: u64,
    final_guard_applied_mutations: u64,
    final_guard_confirmed_mutations: u64,
    final_guard_rollbacks: u64,
    final_guard_pending_candidate: Option<RemoteFreeServiceRetuneCandidate>,
}

pub(crate) fn benchmark_runtime_collected_multi_owner_mutation_limit(c: &mut Criterion) {
    print_multi_owner_sample();
    print_multi_owner_sample_summary();

    c.bench_function(MULTI_OWNER_RUNTIME_BENCHMARK, |bench| {
        bench.iter(|| {
            let stats = run_multi_owner_runtime_collected_sequence();
            assert_multi_owner_runtime_stats(stats);
            black_box(stats);
        });
    });
}

fn print_multi_owner_sample() {
    if !should_print_sample(MULTI_OWNER_RUNTIME_SAMPLE, MULTI_OWNER_RUNTIME_BENCHMARK) {
        return;
    }

    let stats = run_multi_owner_runtime_collected_sequence();
    assert_multi_owner_runtime_stats(stats);

    println!(
        "{MULTI_OWNER_RUNTIME_SAMPLE} owners={MULTI_OWNER_RUNTIME_OWNERS} windows={MULTI_OWNER_RUNTIME_WINDOWS} stable_windows={MULTI_OWNER_RUNTIME_STABLE_WINDOWS} max_mutations={MULTI_OWNER_RUNTIME_MAX_MUTATIONS} submitted_count={} drained_count={} released_bytes={} policy_drains={} drain_rounds={} observed_reports={} reports_needing_retune={} max_pending_over_target={} max_queued_bytes_over_budget={} queue_backpressure_reports={} hold_decisions={} apply_decisions={} confirmed_decisions={} mutation_limit_decisions={} runtime_install_count={} runtime_confirm_count={} runtime_rollback_count={} runtime_no_change_decisions={} max_wait_bursts={} mean_wait_bursts={} final_queue_capacity={} final_previous_config_present={} final_guard_pending_candidate={} final_guard_applied_mutations={} final_guard_confirmed_mutations={} final_guard_rollbacks={}",
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
        stats.mutation_limit_decisions,
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
    );
}

fn print_multi_owner_sample_summary() {
    if !should_print_sample(
        MULTI_OWNER_RUNTIME_SAMPLE_SUMMARY,
        MULTI_OWNER_RUNTIME_BENCHMARK,
    ) {
        return;
    }

    let mut policy_drains = CounterSummary::new();
    let mut drain_rounds = CounterSummary::new();
    let mut reports_needing_retune = CounterSummary::new();
    let mut apply_decisions = CounterSummary::new();
    let mut confirmed_decisions = CounterSummary::new();
    let mut mutation_limit_decisions = CounterSummary::new();
    let mut max_wait = CounterSummary::new();
    let mut mean_wait = CounterSummary::new();

    for _ in 0..SAMPLES {
        let stats = run_multi_owner_runtime_collected_sequence();
        assert_multi_owner_runtime_stats(stats);

        policy_drains.observe(stats.runtime.policy_drains);
        drain_rounds.observe(stats.runtime.drain_rounds);
        reports_needing_retune.observe(stats.reports_needing_retune);
        apply_decisions.observe(stats.apply_decisions);
        confirmed_decisions.observe(stats.confirmed_decisions);
        mutation_limit_decisions.observe(stats.mutation_limit_decisions);
        max_wait.observe(stats.runtime.max_wait_bursts);
        mean_wait.observe(stats.runtime.mean_wait_milli());
    }

    println!(
        "{MULTI_OWNER_RUNTIME_SAMPLE_SUMMARY} owners={MULTI_OWNER_RUNTIME_OWNERS} windows={MULTI_OWNER_RUNTIME_WINDOWS} samples={SAMPLES} policy_drains_min={} policy_drains_max={} policy_drains_mean={} drain_rounds_min={} drain_rounds_max={} drain_rounds_mean={} reports_needing_retune_min={} reports_needing_retune_max={} reports_needing_retune_mean={} apply_decisions_min={} apply_decisions_max={} apply_decisions_mean={} confirmed_decisions_min={} confirmed_decisions_max={} confirmed_decisions_mean={} mutation_limit_decisions_min={} mutation_limit_decisions_max={} mutation_limit_decisions_mean={} max_wait_min={} max_wait_max={} max_wait_mean={} mean_wait_min={} mean_wait_max={} mean_wait_mean={}",
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

fn run_multi_owner_runtime_collected_sequence() -> MultiOwnerRuntimeStats {
    let mut guard = RemoteFreeServiceRetuneGuard::try_new(
        MULTI_OWNER_RUNTIME_STABLE_WINDOWS,
        MULTI_OWNER_RUNTIME_MAX_MUTATIONS,
    )
    .expect("guard");
    let applicator = RemoteFreeServiceRetunePolicyApplicator::try_new(QUEUE_CAPACITY_GROWTH_FACTOR)
        .expect("applicator");
    let mut stats = MultiOwnerRuntimeStats::new();

    run_apply_confirm_owner(&mut guard, applicator, &mut stats);
    run_apply_confirm_owner(&mut guard, applicator, &mut stats);
    run_mutation_limit_owner(&mut guard, applicator, &mut stats);

    stats.final_guard_applied_mutations = guard.applied_mutations();
    stats.final_guard_confirmed_mutations = guard.confirmed_mutations();
    stats.final_guard_rollbacks = guard.rollbacks();
    stats.final_guard_pending_candidate = guard.pending_validation();
    stats
}

fn run_apply_confirm_owner(
    guard: &mut RemoteFreeServiceRetuneGuard,
    applicator: RemoteFreeServiceRetunePolicyApplicator,
    stats: &mut MultiOwnerRuntimeStats,
) {
    let mut runtime = runtime_with_initial_empty_policy();

    let hold_summary = run_runtime_owner_window_with_summary(&mut runtime, &mut stats.runtime);
    observe_multi_owner_step(
        guard,
        &mut runtime,
        applicator,
        hold_summary,
        MultiOwnerDecisionKind::Hold,
        stats,
    );

    let apply_summary = run_runtime_owner_window_with_summary(&mut runtime, &mut stats.runtime);
    observe_multi_owner_step(
        guard,
        &mut runtime,
        applicator,
        apply_summary,
        MultiOwnerDecisionKind::Apply,
        stats,
    );

    let confirm_summary = run_runtime_owner_window_with_summary(&mut runtime, &mut stats.runtime);
    observe_multi_owner_step(
        guard,
        &mut runtime,
        applicator,
        confirm_summary,
        MultiOwnerDecisionKind::Confirmed,
        stats,
    );

    stats.runtime.final_queue_capacity = runtime.config().queue_capacity();
    stats.runtime.final_previous_config_present = runtime.previous_config().is_some();
}

fn run_mutation_limit_owner(
    guard: &mut RemoteFreeServiceRetuneGuard,
    applicator: RemoteFreeServiceRetunePolicyApplicator,
    stats: &mut MultiOwnerRuntimeStats,
) {
    let mut runtime = runtime_with_initial_empty_policy();

    let hold_summary = run_runtime_owner_window_with_summary(&mut runtime, &mut stats.runtime);
    observe_multi_owner_step(
        guard,
        &mut runtime,
        applicator,
        hold_summary,
        MultiOwnerDecisionKind::Hold,
        stats,
    );

    let mutation_limit_summary =
        run_runtime_owner_window_with_summary(&mut runtime, &mut stats.runtime);
    observe_multi_owner_step(
        guard,
        &mut runtime,
        applicator,
        mutation_limit_summary,
        MultiOwnerDecisionKind::MutationLimitReached,
        stats,
    );

    stats.runtime.final_queue_capacity = runtime.config().queue_capacity();
    stats.runtime.final_previous_config_present = runtime.previous_config().is_some();
}

fn runtime_with_initial_empty_policy() -> RemoteFreeOwnerRuntime<RuntimeTraceBlock> {
    RemoteFreeOwnerRuntime::new_with_drain_policy(
        service_config(QUEUE_CAPACITY),
        RemoteFreeDrainPolicy::new(),
    )
    .expect("runtime")
}

fn observe_multi_owner_step(
    guard: &mut RemoteFreeServiceRetuneGuard,
    runtime: &mut RemoteFreeOwnerRuntime<RuntimeTraceBlock>,
    applicator: RemoteFreeServiceRetunePolicyApplicator,
    summary: RemoteFreeServiceRetuneSummary,
    expected_decision: MultiOwnerDecisionKind,
    stats: &mut MultiOwnerRuntimeStats,
) {
    stats.observe_summary(summary);
    let decision = guard.observe_summary(summary);

    assert_eq!(
        MultiOwnerDecisionKind::from_decision(decision),
        expected_decision
    );
    assert_decision_candidate(decision, RemoteFreeServiceRetuneCandidate::DrainEarlier);
    stats.observe_decision(decision);
    apply_multi_owner_runtime_decision(runtime, applicator, decision, stats);
}

fn apply_multi_owner_runtime_decision(
    runtime: &mut RemoteFreeOwnerRuntime<RuntimeTraceBlock>,
    applicator: RemoteFreeServiceRetunePolicyApplicator,
    decision: RemoteFreeServiceRetuneGuardDecision,
    stats: &mut MultiOwnerRuntimeStats,
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
            panic!("multi-owner mutation-limit sequence should not roll back");
        }
    }
}

fn assert_decision_candidate(
    decision: RemoteFreeServiceRetuneGuardDecision,
    expected_candidate: RemoteFreeServiceRetuneCandidate,
) {
    match decision {
        RemoteFreeServiceRetuneGuardDecision::Hold { candidate, .. }
        | RemoteFreeServiceRetuneGuardDecision::Apply { candidate }
        | RemoteFreeServiceRetuneGuardDecision::Confirmed { candidate }
        | RemoteFreeServiceRetuneGuardDecision::Rollback { candidate, .. }
        | RemoteFreeServiceRetuneGuardDecision::MutationLimitReached { candidate } => {
            assert_eq!(candidate, expected_candidate);
        }
        RemoteFreeServiceRetuneGuardDecision::CollectTelemetry => {}
    }
}

impl MultiOwnerDecisionKind {
    fn from_decision(decision: RemoteFreeServiceRetuneGuardDecision) -> Self {
        match decision {
            RemoteFreeServiceRetuneGuardDecision::CollectTelemetry
            | RemoteFreeServiceRetuneGuardDecision::Hold { .. } => Self::Hold,
            RemoteFreeServiceRetuneGuardDecision::Apply { .. } => Self::Apply,
            RemoteFreeServiceRetuneGuardDecision::Confirmed { .. } => Self::Confirmed,
            RemoteFreeServiceRetuneGuardDecision::MutationLimitReached { .. } => {
                Self::MutationLimitReached
            }
            RemoteFreeServiceRetuneGuardDecision::Rollback { .. } => {
                panic!("unexpected multi-owner rollback decision")
            }
        }
    }
}

impl MultiOwnerRuntimeStats {
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
            mutation_limit_decisions: 0,
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
            RemoteFreeServiceRetuneGuardDecision::MutationLimitReached { .. } => {
                self.mutation_limit_decisions = self.mutation_limit_decisions.saturating_add(1);
            }
            RemoteFreeServiceRetuneGuardDecision::Rollback { .. } => {}
        }
    }
}

fn assert_multi_owner_runtime_stats(stats: MultiOwnerRuntimeStats) {
    assert_eq!(
        stats.runtime.submitted_count,
        MULTI_OWNER_RUNTIME_WINDOWS * 256
    );
    assert_eq!(stats.runtime.drained_count, stats.runtime.submitted_count);
    assert_eq!(
        stats.runtime.released_bytes,
        stats
            .runtime
            .submitted_count
            .saturating_mul(BYTES_PER_BLOCK)
    );
    assert_eq!(stats.runtime.policy_drains, 8);
    assert_eq!(stats.runtime.drain_rounds, 32);
    assert_eq!(stats.runtime.install_count, 2);
    assert_eq!(stats.runtime.confirm_count, 2);
    assert_eq!(stats.runtime.rollback_count, 0);
    assert_eq!(stats.runtime.max_wait_bursts, 8);
    assert_eq!(stats.runtime.mean_wait_milli(), 3_750);
    assert_eq!(stats.runtime.final_queue_capacity, QUEUE_CAPACITY);
    assert!(!stats.runtime.final_previous_config_present);

    assert_eq!(stats.observed_reports, MULTI_OWNER_RUNTIME_WINDOWS * 8);
    assert_eq!(stats.reports_needing_retune, 36);
    assert_eq!(stats.max_pending_over_target, 192);
    assert_eq!(stats.max_queued_bytes_over_budget, 786_432);
    assert_eq!(stats.queue_backpressure_reports, 0);
    assert_eq!(stats.hold_decisions, 3);
    assert_eq!(stats.apply_decisions, 2);
    assert_eq!(stats.confirmed_decisions, 2);
    assert_eq!(stats.mutation_limit_decisions, 1);
    assert_eq!(stats.runtime_no_change_decisions, 4);
    assert_eq!(stats.final_guard_applied_mutations, 2);
    assert_eq!(stats.final_guard_confirmed_mutations, 2);
    assert_eq!(stats.final_guard_rollbacks, 0);
    assert_eq!(stats.final_guard_pending_candidate, None);
}

fn option_candidate_label(candidate: Option<RemoteFreeServiceRetuneCandidate>) -> &'static str {
    candidate.map_or("none", RemoteFreeServiceRetuneCandidate::as_str)
}
