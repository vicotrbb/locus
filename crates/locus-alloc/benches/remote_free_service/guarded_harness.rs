#![allow(missing_docs)]

use criterion::{black_box, Criterion};
use locus_alloc::{
    RemoteFreeServiceRetuneCandidate, RemoteFreeServiceRetuneGuard,
    RemoteFreeServiceRetuneGuardDecision,
};

use crate::remote_free_service_harness::{
    assert_service_telemetry, format_milli, run_service_case, CounterSummary, ServiceTelemetryCase,
    ServiceTelemetryStats, BLOCKS_PER_OWNER, BYTES_PER_BLOCK, OWNERS, SAMPLES,
};

const GUARDED_STABLE_WINDOWS: u64 = 2;
const GUARDED_MAX_MUTATIONS: u64 = 2;

#[derive(Debug, Clone, Copy)]
enum GuardedSequenceKind {
    ConfirmingCandidates,
    RollbackFailedCandidate,
}

#[derive(Debug, Clone, Copy)]
enum GuardedStep {
    Case(ServiceTelemetryCase),
    PendingCandidate,
}

#[derive(Debug, Clone, Copy)]
struct GuardedExpectedStep {
    candidate: RemoteFreeServiceRetuneCandidate,
    decision: GuardedDecisionKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GuardedDecisionKind {
    Hold,
    Apply,
    Confirmed,
    Rollback,
    MutationLimitReached,
}

#[derive(Debug, Clone, Copy)]
struct GuardedSequenceStats {
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
    hold_decisions: u64,
    apply_decisions: u64,
    confirmed_decisions: u64,
    rollback_decisions: u64,
    mutation_limit_decisions: u64,
    drain_earlier_apply_decisions: u64,
    combined_apply_decisions: u64,
    max_wait_bursts: u64,
    total_wait_bursts: u64,
    final_pending_candidate: Option<RemoteFreeServiceRetuneCandidate>,
    final_applied_mutations: u64,
    final_confirmed_mutations: u64,
    final_rollbacks: u64,
}

pub fn benchmark_guarded_sequences(c: &mut Criterion) {
    benchmark_guarded_sequence_kind(c, GuardedSequenceKind::ConfirmingCandidates);
    benchmark_guarded_sequence_kind(c, GuardedSequenceKind::RollbackFailedCandidate);
}

fn benchmark_guarded_sequence_kind(c: &mut Criterion, kind: GuardedSequenceKind) {
    print_guarded_sequence_sample(kind);
    print_guarded_sequence_sample_summary(kind);

    c.bench_function(kind.bench_label(), |bench| {
        bench.iter(|| {
            let stats = run_guarded_sequence(kind);
            assert_guarded_sequence(kind, stats);
            black_box(stats);
        });
    });
}

fn print_guarded_sequence_sample(kind: GuardedSequenceKind) {
    let stats = run_guarded_sequence(kind);
    assert_guarded_sequence(kind, stats);
    let label = kind.sample_label();

    println!(
        "{label} windows={} stable_windows={GUARDED_STABLE_WINDOWS} max_mutations={GUARDED_MAX_MUTATIONS} submitted_count={} drained_count={} released_bytes={} policy_drains={} observed_reports={} reports_needing_retune={} max_pending_over_target={} max_queued_bytes_over_budget={} queue_backpressure_reports={} hold_decisions={} apply_decisions={} confirmed_decisions={} rollback_decisions={} mutation_limit_decisions={} drain_earlier_apply_decisions={} combined_apply_decisions={} max_wait_bursts={} mean_wait_bursts={} final_pending_candidate={} final_applied_mutations={} final_confirmed_mutations={} final_rollbacks={}",
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
        stats.hold_decisions,
        stats.apply_decisions,
        stats.confirmed_decisions,
        stats.rollback_decisions,
        stats.mutation_limit_decisions,
        stats.drain_earlier_apply_decisions,
        stats.combined_apply_decisions,
        stats.max_wait_bursts,
        format_milli(stats.mean_wait_milli()),
        option_candidate_label(stats.final_pending_candidate),
        stats.final_applied_mutations,
        stats.final_confirmed_mutations,
        stats.final_rollbacks
    );
}

fn print_guarded_sequence_sample_summary(kind: GuardedSequenceKind) {
    let mut reports_needing_retune = CounterSummary::new();
    let mut max_pending_over_target = CounterSummary::new();
    let mut max_queued_bytes_over_budget = CounterSummary::new();
    let mut queue_backpressure_reports = CounterSummary::new();
    let mut apply_decisions = CounterSummary::new();
    let mut confirmed_decisions = CounterSummary::new();
    let mut rollback_decisions = CounterSummary::new();
    let mut max_wait = CounterSummary::new();
    let mut mean_wait = CounterSummary::new();
    let mut final_pending_candidate = None;
    let mut final_applied_mutations = 0;
    let mut final_confirmed_mutations = 0;
    let mut final_rollbacks = 0;

    for _ in 0..SAMPLES {
        let stats = run_guarded_sequence(kind);
        assert_guarded_sequence(kind, stats);

        reports_needing_retune.observe(stats.reports_needing_retune);
        max_pending_over_target.observe(stats.max_pending_over_target);
        max_queued_bytes_over_budget.observe(stats.max_queued_bytes_over_budget);
        queue_backpressure_reports.observe(stats.queue_backpressure_reports);
        apply_decisions.observe(stats.apply_decisions);
        confirmed_decisions.observe(stats.confirmed_decisions);
        rollback_decisions.observe(stats.rollback_decisions);
        max_wait.observe(stats.max_wait_bursts);
        mean_wait.observe(stats.mean_wait_milli());
        final_pending_candidate = stats.final_pending_candidate;
        final_applied_mutations = stats.final_applied_mutations;
        final_confirmed_mutations = stats.final_confirmed_mutations;
        final_rollbacks = stats.final_rollbacks;
    }

    let label = kind.summary_label();
    println!(
        "{label} windows={} stable_windows={GUARDED_STABLE_WINDOWS} max_mutations={GUARDED_MAX_MUTATIONS} samples={SAMPLES} reports_needing_retune_min={} reports_needing_retune_max={} reports_needing_retune_mean={} max_pending_over_target_min={} max_pending_over_target_max={} max_pending_over_target_mean={} max_queued_bytes_over_budget_min={} max_queued_bytes_over_budget_max={} max_queued_bytes_over_budget_mean={} queue_backpressure_reports_min={} queue_backpressure_reports_max={} queue_backpressure_reports_mean={} apply_decisions_min={} apply_decisions_max={} apply_decisions_mean={} confirmed_decisions_min={} confirmed_decisions_max={} confirmed_decisions_mean={} rollback_decisions_min={} rollback_decisions_max={} rollback_decisions_mean={} max_wait_min={} max_wait_max={} max_wait_mean={} mean_wait_min={} mean_wait_max={} mean_wait_mean={} final_pending_candidate={} final_applied_mutations={} final_confirmed_mutations={} final_rollbacks={}",
        kind.expected_windows(),
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
        apply_decisions.min,
        apply_decisions.max,
        format_milli(apply_decisions.mean_milli(SAMPLES)),
        confirmed_decisions.min,
        confirmed_decisions.max,
        format_milli(confirmed_decisions.mean_milli(SAMPLES)),
        rollback_decisions.min,
        rollback_decisions.max,
        format_milli(rollback_decisions.mean_milli(SAMPLES)),
        max_wait.min,
        max_wait.max,
        format_milli(max_wait.mean_milli(SAMPLES)),
        format_milli(mean_wait.min),
        format_milli(mean_wait.max),
        format_milli(mean_wait.mean_milli(SAMPLES) / 1000),
        option_candidate_label(final_pending_candidate),
        final_applied_mutations,
        final_confirmed_mutations,
        final_rollbacks
    );
}

fn run_guarded_sequence(kind: GuardedSequenceKind) -> GuardedSequenceStats {
    let mut guard =
        RemoteFreeServiceRetuneGuard::try_new(GUARDED_STABLE_WINDOWS, GUARDED_MAX_MUTATIONS)
            .expect("guarded service retune planner");
    let mut sequence_stats = GuardedSequenceStats::new();

    for (step, expected_step) in kind.steps().into_iter().zip(kind.expected_steps()) {
        let case = match step {
            GuardedStep::Case(case) => case,
            GuardedStep::PendingCandidate => candidate_case(
                guard
                    .pending_validation()
                    .expect("pending candidate before validation step"),
            ),
        };
        let stats = run_service_case(case);
        assert_service_telemetry(case, stats);

        let decision = guard.observe_summary(stats.summary);
        assert_guarded_decision(expected_step, decision);
        sequence_stats.observe_window(stats, decision);
    }

    sequence_stats.final_pending_candidate = guard.pending_validation();
    sequence_stats.final_applied_mutations = guard.applied_mutations();
    sequence_stats.final_confirmed_mutations = guard.confirmed_mutations();
    sequence_stats.final_rollbacks = guard.rollbacks();
    sequence_stats
}

fn candidate_case(candidate: RemoteFreeServiceRetuneCandidate) -> ServiceTelemetryCase {
    match candidate {
        RemoteFreeServiceRetuneCandidate::DrainEarlier => {
            ServiceTelemetryCase::planner_candidate_drain_earlier()
        }
        RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier => {
            ServiceTelemetryCase::planner_candidate_capacity_and_drain_earlier()
        }
        _ => panic!("candidate cannot be applied by guarded benchmark: {candidate:?}"),
    }
}

fn assert_guarded_decision(
    expected_step: GuardedExpectedStep,
    decision: RemoteFreeServiceRetuneGuardDecision,
) {
    let observed = GuardedDecisionKind::from_decision(decision);
    assert_eq!(observed, expected_step.decision);

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

impl GuardedSequenceKind {
    fn bench_label(self) -> &'static str {
        match self {
            Self::ConfirmingCandidates => "remote_free_service_telemetry_guarded_confirming",
            Self::RollbackFailedCandidate => "remote_free_service_telemetry_guarded_rollback",
        }
    }

    fn sample_label(self) -> &'static str {
        match self {
            Self::ConfirmingCandidates => "remote_free_service_guarded_confirming_sample",
            Self::RollbackFailedCandidate => "remote_free_service_guarded_rollback_sample",
        }
    }

    fn summary_label(self) -> &'static str {
        match self {
            Self::ConfirmingCandidates => "remote_free_service_guarded_confirming_sample_summary",
            Self::RollbackFailedCandidate => "remote_free_service_guarded_rollback_sample_summary",
        }
    }

    fn steps(self) -> Vec<GuardedStep> {
        match self {
            Self::ConfirmingCandidates => vec![
                GuardedStep::Case(ServiceTelemetryCase::fixed_policy_all_clean()),
                GuardedStep::Case(ServiceTelemetryCase::one_end_drain_owner()),
                GuardedStep::Case(ServiceTelemetryCase::one_end_drain_owner()),
                GuardedStep::PendingCandidate,
                GuardedStep::Case(ServiceTelemetryCase::one_capacity128_end_drain_owner()),
                GuardedStep::Case(ServiceTelemetryCase::one_capacity128_end_drain_owner()),
                GuardedStep::PendingCandidate,
            ],
            Self::RollbackFailedCandidate => vec![
                GuardedStep::Case(ServiceTelemetryCase::fixed_policy_all_clean()),
                GuardedStep::Case(ServiceTelemetryCase::one_end_drain_owner()),
                GuardedStep::Case(ServiceTelemetryCase::one_end_drain_owner()),
                GuardedStep::Case(ServiceTelemetryCase::one_end_drain_owner()),
            ],
        }
    }

    fn expected_steps(self) -> Vec<GuardedExpectedStep> {
        match self {
            Self::ConfirmingCandidates => vec![
                GuardedExpectedStep {
                    candidate: RemoteFreeServiceRetuneCandidate::KeepConfig,
                    decision: GuardedDecisionKind::Hold,
                },
                GuardedExpectedStep {
                    candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
                    decision: GuardedDecisionKind::Hold,
                },
                GuardedExpectedStep {
                    candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
                    decision: GuardedDecisionKind::Apply,
                },
                GuardedExpectedStep {
                    candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
                    decision: GuardedDecisionKind::Confirmed,
                },
                GuardedExpectedStep {
                    candidate:
                        RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier,
                    decision: GuardedDecisionKind::Hold,
                },
                GuardedExpectedStep {
                    candidate:
                        RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier,
                    decision: GuardedDecisionKind::Apply,
                },
                GuardedExpectedStep {
                    candidate:
                        RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier,
                    decision: GuardedDecisionKind::Confirmed,
                },
            ],
            Self::RollbackFailedCandidate => vec![
                GuardedExpectedStep {
                    candidate: RemoteFreeServiceRetuneCandidate::KeepConfig,
                    decision: GuardedDecisionKind::Hold,
                },
                GuardedExpectedStep {
                    candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
                    decision: GuardedDecisionKind::Hold,
                },
                GuardedExpectedStep {
                    candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
                    decision: GuardedDecisionKind::Apply,
                },
                GuardedExpectedStep {
                    candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
                    decision: GuardedDecisionKind::Rollback,
                },
            ],
        }
    }

    fn expected_windows(self) -> u64 {
        match self {
            Self::ConfirmingCandidates => 7,
            Self::RollbackFailedCandidate => 4,
        }
    }

    fn expected_policy_drains(self) -> u64 {
        match self {
            Self::ConfirmingCandidates => 96,
            Self::RollbackFailedCandidate => 52,
        }
    }

    fn expected_reports_needing_retune(self) -> u64 {
        match self {
            Self::ConfirmingCandidates => 24,
            Self::RollbackFailedCandidate => 18,
        }
    }

    fn expected_queue_backpressure_reports(self) -> u64 {
        match self {
            Self::ConfirmingCandidates => 8,
            Self::RollbackFailedCandidate => 0,
        }
    }

    fn expected_apply_decisions(self) -> u64 {
        match self {
            Self::ConfirmingCandidates => 2,
            Self::RollbackFailedCandidate => 1,
        }
    }

    fn expected_confirmed_decisions(self) -> u64 {
        match self {
            Self::ConfirmingCandidates => 2,
            Self::RollbackFailedCandidate => 0,
        }
    }

    fn expected_rollback_decisions(self) -> u64 {
        match self {
            Self::ConfirmingCandidates => 0,
            Self::RollbackFailedCandidate => 1,
        }
    }

    fn expected_max_queued_bytes_over_budget(self) -> u64 {
        match self {
            Self::ConfirmingCandidates | Self::RollbackFailedCandidate => 786_432,
        }
    }

    fn expected_mean_wait_milli(self) -> u64 {
        match self {
            Self::ConfirmingCandidates => 1_821,
            Self::RollbackFailedCandidate => 2_062,
        }
    }
}

impl GuardedDecisionKind {
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

impl GuardedSequenceStats {
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
            hold_decisions: 0,
            apply_decisions: 0,
            confirmed_decisions: 0,
            rollback_decisions: 0,
            mutation_limit_decisions: 0,
            drain_earlier_apply_decisions: 0,
            combined_apply_decisions: 0,
            max_wait_bursts: 0,
            total_wait_bursts: 0,
            final_pending_candidate: None,
            final_applied_mutations: 0,
            final_confirmed_mutations: 0,
            final_rollbacks: 0,
        }
    }

    fn observe_window(
        &mut self,
        stats: ServiceTelemetryStats,
        decision: RemoteFreeServiceRetuneGuardDecision,
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

    fn mean_wait_milli(self) -> u64 {
        if self.drained_count == 0 {
            return 0;
        }

        self.total_wait_bursts.saturating_mul(1000) / self.drained_count
    }
}

fn assert_guarded_sequence(kind: GuardedSequenceKind, stats: GuardedSequenceStats) {
    assert_eq!(stats.service_windows, kind.expected_windows());
    assert_eq!(
        stats.submitted_count,
        BLOCKS_PER_OWNER * OWNERS as u64 * stats.service_windows
    );
    assert_eq!(stats.drained_count, stats.submitted_count);
    assert_eq!(
        stats.released_bytes,
        stats.submitted_count * BYTES_PER_BLOCK
    );
    assert_eq!(stats.policy_drains, kind.expected_policy_drains());
    assert_eq!(stats.observed_reports, stats.service_windows * 32);
    assert_eq!(
        stats.reports_needing_retune,
        kind.expected_reports_needing_retune()
    );
    assert_eq!(stats.max_pending_over_target, 192);
    assert_eq!(
        stats.max_queued_bytes_over_budget,
        kind.expected_max_queued_bytes_over_budget()
    );
    assert_eq!(
        stats.queue_backpressure_reports,
        kind.expected_queue_backpressure_reports()
    );
    assert_eq!(stats.apply_decisions, kind.expected_apply_decisions());
    assert_eq!(
        stats.confirmed_decisions,
        kind.expected_confirmed_decisions()
    );
    assert_eq!(stats.rollback_decisions, kind.expected_rollback_decisions());
    assert_eq!(stats.mutation_limit_decisions, 0);
    assert_eq!(stats.max_wait_bursts, 8);
    assert_eq!(stats.mean_wait_milli(), kind.expected_mean_wait_milli());
    assert_eq!(stats.final_pending_candidate, None);
    assert_eq!(
        stats.final_applied_mutations,
        kind.expected_apply_decisions()
    );
    assert_eq!(
        stats.final_confirmed_mutations,
        kind.expected_confirmed_decisions()
    );
    assert_eq!(stats.final_rollbacks, kind.expected_rollback_decisions());
}

fn option_candidate_label(candidate: Option<RemoteFreeServiceRetuneCandidate>) -> &'static str {
    candidate.map_or("none", RemoteFreeServiceRetuneCandidate::as_str)
}
