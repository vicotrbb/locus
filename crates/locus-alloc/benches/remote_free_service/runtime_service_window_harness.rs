#![allow(missing_docs)]

use std::convert::Infallible;

use criterion::{black_box, Criterion};
use locus_alloc::{
    RemoteFreeDrainPolicy, RemoteFreeOwnerRuntime, RemoteFreeServiceRetuneCandidate,
    RemoteFreeServiceRetuneGuard, RemoteFreeServiceRetunePolicyApplicator,
    RemoteFreeServiceRetuneSummary, RemoteFreeServiceRuntimeDirtyOwners,
    RemoteFreeServiceRuntimeOwnerId, RemoteFreeServiceRuntimeRetuneCoordinator,
    RemoteFreeServiceRuntimeRetuneOwners, RemoteFreeServiceRuntimeWindowObservation,
    RemoteFreeServiceRuntimeWindowStats,
};

use crate::remote_free_service_application_harness::{
    run_runtime_owner_window_with_summary, run_runtime_owner_window_with_summary_and_block_bytes,
    service_config, RuntimeApplicationStats, RuntimeTraceBlock, QUEUE_CAPACITY_GROWTH_FACTOR,
    RUNTIME_INITIAL_QUEUE_CAPACITY,
};
use crate::remote_free_service_harness::{
    format_milli, CounterSummary, BYTES_PER_BLOCK, QUEUE_CAPACITY, SAMPLES,
};

const SERVICE_WINDOW_STABLE_WINDOWS: u64 = 2;
const SERVICE_WINDOW_MAX_MUTATIONS: u64 = 2;
const SERVICE_WINDOW_OWNERS: u64 = 3;
const SERVICE_WINDOW_OWNERS_USIZE: usize = 3;
const SERVICE_WINDOW_WINDOWS: u64 = 8;
const SERVICE_WINDOW_ROLLBACK_VALIDATION_BYTES: u64 = BYTES_PER_BLOCK * 2 + 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ServiceWindowDecisionKind {
    Hold,
    Apply,
    Confirmed,
    Rollback,
    MutationLimitReached,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ServiceWindowRunnerMode {
    RoutedObservation,
    CollectedOwnerBorrow,
    DirtyOwnerBorrow,
}

#[derive(Debug, Clone, Copy)]
struct ServiceWindowSequenceStats {
    runtime: RuntimeApplicationStats,
    registered_owners: usize,
    window: RemoteFreeServiceRuntimeWindowStats,
    missing_owner_checks: u64,
    final_guard_applied_mutations: u64,
    final_guard_confirmed_mutations: u64,
    final_guard_rollbacks: u64,
    final_guard_pending_candidate: Option<RemoteFreeServiceRetuneCandidate>,
}

pub(crate) fn benchmark_runtime_service_window_sequence(c: &mut Criterion) {
    print_service_window_sample(
        ServiceWindowRunnerMode::RoutedObservation,
        "remote_free_service_runtime_service_window_sample",
    );
    print_service_window_sample_summary(
        ServiceWindowRunnerMode::RoutedObservation,
        "remote_free_service_runtime_service_window_sample_summary",
    );

    c.bench_function(
        "remote_free_service_runtime_service_window_sequence",
        |bench| {
            bench.iter(|| {
                let stats =
                    run_runtime_service_window_sequence(ServiceWindowRunnerMode::RoutedObservation);
                assert_service_window_stats(&stats);
                black_box(stats);
            });
        },
    );
}

pub(crate) fn benchmark_runtime_window_collection_sequence(c: &mut Criterion) {
    print_service_window_sample(
        ServiceWindowRunnerMode::CollectedOwnerBorrow,
        "remote_free_service_runtime_window_collection_sample",
    );
    print_service_window_sample_summary(
        ServiceWindowRunnerMode::CollectedOwnerBorrow,
        "remote_free_service_runtime_window_collection_sample_summary",
    );

    c.bench_function(
        "remote_free_service_runtime_window_collection_sequence",
        |bench| {
            bench.iter(|| {
                let stats = run_runtime_service_window_sequence(
                    ServiceWindowRunnerMode::CollectedOwnerBorrow,
                );
                assert_service_window_stats(&stats);
                black_box(stats);
            });
        },
    );
}

pub(crate) fn benchmark_runtime_dirty_window_collection_sequence(c: &mut Criterion) {
    print_service_window_sample(
        ServiceWindowRunnerMode::DirtyOwnerBorrow,
        "remote_free_service_runtime_dirty_window_collection_sample",
    );
    print_service_window_sample_summary(
        ServiceWindowRunnerMode::DirtyOwnerBorrow,
        "remote_free_service_runtime_dirty_window_collection_sample_summary",
    );

    c.bench_function(
        "remote_free_service_runtime_dirty_window_collection_sequence",
        |bench| {
            bench.iter(|| {
                let stats =
                    run_runtime_service_window_sequence(ServiceWindowRunnerMode::DirtyOwnerBorrow);
                assert_service_window_stats(&stats);
                black_box(stats);
            });
        },
    );
}

fn print_service_window_sample(mode: ServiceWindowRunnerMode, sample_name: &str) {
    let stats = run_runtime_service_window_sequence(mode);
    assert_service_window_stats(&stats);

    println!(
        "{sample_name} owners={SERVICE_WINDOW_OWNERS} windows={SERVICE_WINDOW_WINDOWS} stable_windows={SERVICE_WINDOW_STABLE_WINDOWS} max_mutations={SERVICE_WINDOW_MAX_MUTATIONS} rollback_validation_bytes={SERVICE_WINDOW_ROLLBACK_VALIDATION_BYTES} submitted_count={} drained_count={} released_bytes={} policy_drains={} drain_rounds={} registered_owners={} service_window_observations={} observed_reports={} reports_needing_retune={} max_pending_over_target={} max_queued_bytes_over_budget={} queue_backpressure_reports={} hold_decisions={} apply_decisions={} confirmed_decisions={} rollback_decisions={} mutation_limit_decisions={} runtime_install_count={} runtime_confirm_count={} runtime_rollback_count={} runtime_no_change_outcomes={} missing_owner_checks={} max_wait_bursts={} mean_wait_bursts={} final_queue_capacity={} final_previous_config_present={} final_guard_pending_candidate={} final_guard_applied_mutations={} final_guard_confirmed_mutations={} final_guard_rollbacks={}",
        stats.runtime.submitted_count,
        stats.runtime.drained_count,
        stats.runtime.released_bytes,
        stats.runtime.policy_drains,
        stats.runtime.drain_rounds,
        stats.registered_owners,
        stats.window.owner_observations(),
        stats.window.observed_reports(),
        stats.window.reports_needing_retune(),
        stats.window.max_pending_items_over_target(),
        stats.window.max_queued_bytes_over_budget(),
        stats.window.queue_backpressure_reports(),
        stats.window.hold_decisions(),
        stats.window.apply_decisions(),
        stats.window.confirmed_decisions(),
        stats.window.rollback_decisions(),
        stats.window.mutation_limit_decisions(),
        stats.runtime.install_count,
        stats.runtime.confirm_count,
        stats.runtime.rollback_count,
        stats.window.no_change_outcomes(),
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

fn print_service_window_sample_summary(mode: ServiceWindowRunnerMode, sample_name: &str) {
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
        let stats = run_runtime_service_window_sequence(mode);
        assert_service_window_stats(&stats);

        policy_drains.observe(stats.runtime.policy_drains);
        drain_rounds.observe(stats.runtime.drain_rounds);
        reports_needing_retune.observe(stats.window.reports_needing_retune());
        apply_decisions.observe(stats.window.apply_decisions());
        confirmed_decisions.observe(stats.window.confirmed_decisions());
        rollback_decisions.observe(stats.window.rollback_decisions());
        mutation_limit_decisions.observe(stats.window.mutation_limit_decisions());
        max_wait.observe(stats.runtime.max_wait_bursts);
        mean_wait.observe(stats.runtime.mean_wait_milli());
    }

    println!(
        "{sample_name} owners={SERVICE_WINDOW_OWNERS} windows={SERVICE_WINDOW_WINDOWS} samples={SAMPLES} policy_drains_min={} policy_drains_max={} policy_drains_mean={} drain_rounds_min={} drain_rounds_max={} drain_rounds_mean={} reports_needing_retune_min={} reports_needing_retune_max={} reports_needing_retune_mean={} apply_decisions_min={} apply_decisions_max={} apply_decisions_mean={} confirmed_decisions_min={} confirmed_decisions_max={} confirmed_decisions_mean={} rollback_decisions_min={} rollback_decisions_max={} rollback_decisions_mean={} mutation_limit_decisions_min={} mutation_limit_decisions_max={} mutation_limit_decisions_mean={} max_wait_min={} max_wait_max={} max_wait_mean={} mean_wait_min={} mean_wait_max={} mean_wait_mean={}",
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

fn run_runtime_service_window_sequence(
    mode: ServiceWindowRunnerMode,
) -> ServiceWindowSequenceStats {
    let mut owners =
        RemoteFreeServiceRuntimeRetuneOwners::new(RemoteFreeServiceRuntimeRetuneCoordinator::new(
            RemoteFreeServiceRetuneGuard::try_new(
                SERVICE_WINDOW_STABLE_WINDOWS,
                SERVICE_WINDOW_MAX_MUTATIONS,
            )
            .expect("guard"),
            RemoteFreeServiceRetunePolicyApplicator::try_new(QUEUE_CAPACITY_GROWTH_FACTOR)
                .expect("applicator"),
        ));

    let confirmed = owners.register_owner(runtime_with_initial_empty_policy(QUEUE_CAPACITY));
    let rolled_back = owners.register_owner(runtime_with_initial_empty_policy(
        RUNTIME_INITIAL_QUEUE_CAPACITY,
    ));
    let mutation_limited = owners.register_owner(runtime_with_initial_empty_policy(QUEUE_CAPACITY));

    let mut stats = ServiceWindowSequenceStats::new();
    stats.registered_owners = owners.len();

    run_confirmed_owner(&mut owners, confirmed, &mut stats, mode);
    run_rolled_back_owner(&mut owners, rolled_back, &mut stats, mode);
    run_mutation_limited_owner(&mut owners, mutation_limited, &mut stats, mode);

    let missing = RemoteFreeServiceRuntimeOwnerId::new(usize::MAX);
    assert_missing_owner(&mut owners, missing, mode);
    stats.missing_owner_checks = stats.missing_owner_checks.saturating_add(1);

    stats.runtime.install_count = stats.window.applied_outcomes();
    stats.runtime.confirm_count = stats.window.confirmed_outcomes();
    stats.runtime.rollback_count = stats.window.rolled_back_outcomes();

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
    stats: &mut ServiceWindowSequenceStats,
    mode: ServiceWindowRunnerMode,
) {
    observe_window(
        owners,
        owner_id,
        stats,
        ServiceWindowDecisionKind::Hold,
        BYTES_PER_BLOCK,
        mode,
    );
    observe_window(
        owners,
        owner_id,
        stats,
        ServiceWindowDecisionKind::Apply,
        BYTES_PER_BLOCK,
        mode,
    );
    observe_window(
        owners,
        owner_id,
        stats,
        ServiceWindowDecisionKind::Confirmed,
        BYTES_PER_BLOCK,
        mode,
    );

    let runtime = owners.owner(owner_id).expect("confirmed owner");
    stats.runtime.final_queue_capacity = runtime.config().queue_capacity();
    stats.runtime.final_previous_config_present = runtime.previous_config().is_some();
}

fn run_rolled_back_owner(
    owners: &mut RemoteFreeServiceRuntimeRetuneOwners<RuntimeTraceBlock>,
    owner_id: RemoteFreeServiceRuntimeOwnerId,
    stats: &mut ServiceWindowSequenceStats,
    mode: ServiceWindowRunnerMode,
) {
    observe_window(
        owners,
        owner_id,
        stats,
        ServiceWindowDecisionKind::Hold,
        BYTES_PER_BLOCK,
        mode,
    );
    observe_window(
        owners,
        owner_id,
        stats,
        ServiceWindowDecisionKind::Apply,
        BYTES_PER_BLOCK,
        mode,
    );
    observe_window(
        owners,
        owner_id,
        stats,
        ServiceWindowDecisionKind::Rollback,
        SERVICE_WINDOW_ROLLBACK_VALIDATION_BYTES,
        mode,
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
    stats: &mut ServiceWindowSequenceStats,
    mode: ServiceWindowRunnerMode,
) {
    observe_window(
        owners,
        owner_id,
        stats,
        ServiceWindowDecisionKind::Hold,
        BYTES_PER_BLOCK,
        mode,
    );
    observe_window(
        owners,
        owner_id,
        stats,
        ServiceWindowDecisionKind::MutationLimitReached,
        BYTES_PER_BLOCK,
        mode,
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
    stats: &mut ServiceWindowSequenceStats,
    expected_decision: ServiceWindowDecisionKind,
    block_bytes: u64,
    mode: ServiceWindowRunnerMode,
) {
    let window_stats = match mode {
        ServiceWindowRunnerMode::RoutedObservation => {
            let summary = {
                let runtime = owners.owner_mut(owner_id).expect("owner for window");
                collect_runtime_summary(runtime, &mut stats.runtime, block_bytes)
            };

            owners
                .observe_service_window([RemoteFreeServiceRuntimeWindowObservation::new(
                    owner_id, summary,
                )])
                .expect("service window stats")
        }
        ServiceWindowRunnerMode::CollectedOwnerBorrow => owners
            .collect_service_window([owner_id], |_, runtime| {
                Ok::<_, Infallible>(collect_runtime_summary(
                    runtime,
                    &mut stats.runtime,
                    block_bytes,
                ))
            })
            .expect("collected service window stats"),
        ServiceWindowRunnerMode::DirtyOwnerBorrow => {
            let mut dirty_owners = RemoteFreeServiceRuntimeDirtyOwners::new();
            assert!(dirty_owners.mark_dirty(owner_id));
            assert!(!dirty_owners.mark_dirty(owner_id));
            let stats = owners
                .collect_dirty_service_window(&mut dirty_owners, |_, runtime| {
                    Ok::<_, Infallible>(collect_runtime_summary(
                        runtime,
                        &mut stats.runtime,
                        block_bytes,
                    ))
                })
                .expect("dirty service window stats");
            assert!(dirty_owners.is_empty());
            stats
        }
    };

    assert_one_window_decision(window_stats, expected_decision);
    stats.window.merge(window_stats);
}

fn collect_runtime_summary(
    runtime: &mut RemoteFreeOwnerRuntime<RuntimeTraceBlock>,
    stats: &mut RuntimeApplicationStats,
    block_bytes: u64,
) -> RemoteFreeServiceRetuneSummary {
    if block_bytes == BYTES_PER_BLOCK {
        run_runtime_owner_window_with_summary(runtime, stats)
    } else {
        run_runtime_owner_window_with_summary_and_block_bytes(runtime, stats, block_bytes)
    }
}

fn assert_missing_owner(
    owners: &mut RemoteFreeServiceRuntimeRetuneOwners<RuntimeTraceBlock>,
    missing: RemoteFreeServiceRuntimeOwnerId,
    mode: ServiceWindowRunnerMode,
) {
    match mode {
        ServiceWindowRunnerMode::RoutedObservation => {
            let error = owners
                .observe_service_window([RemoteFreeServiceRuntimeWindowObservation::new(
                    missing,
                    RemoteFreeServiceRetuneSummary::new(),
                )])
                .expect_err("missing owner");
            assert_eq!(error.owner_id(), missing);
        }
        ServiceWindowRunnerMode::CollectedOwnerBorrow => {
            let error = owners
                .collect_service_window([missing], |_, _| {
                    Ok::<_, Infallible>(RemoteFreeServiceRetuneSummary::new())
                })
                .expect_err("missing owner");
            assert_eq!(error.owner_id(), missing);
        }
        ServiceWindowRunnerMode::DirtyOwnerBorrow => {
            let mut dirty_owners = RemoteFreeServiceRuntimeDirtyOwners::new();
            assert!(dirty_owners.mark_dirty(missing));
            let error = owners
                .collect_dirty_service_window(&mut dirty_owners, |_, _| {
                    Ok::<_, Infallible>(RemoteFreeServiceRetuneSummary::new())
                })
                .expect_err("missing owner");
            assert_eq!(error.owner_id(), missing);
            assert_eq!(dirty_owners.owner_ids(), &[missing]);
        }
    }
}

fn assert_one_window_decision(
    stats: RemoteFreeServiceRuntimeWindowStats,
    expected: ServiceWindowDecisionKind,
) {
    assert_eq!(stats.owner_observations(), 1);
    assert_eq!(stats.observed_reports(), 8);

    match expected {
        ServiceWindowDecisionKind::Hold => {
            assert_eq!(stats.hold_decisions(), 1);
            assert_eq!(stats.no_change_outcomes(), 1);
        }
        ServiceWindowDecisionKind::Apply => {
            assert_eq!(stats.apply_decisions(), 1);
            assert_eq!(stats.applied_outcomes(), 1);
        }
        ServiceWindowDecisionKind::Confirmed => {
            assert_eq!(stats.confirmed_decisions(), 1);
            assert_eq!(stats.confirmed_outcomes(), 1);
        }
        ServiceWindowDecisionKind::Rollback => {
            assert_eq!(stats.rollback_decisions(), 1);
            assert_eq!(stats.rolled_back_outcomes(), 1);
        }
        ServiceWindowDecisionKind::MutationLimitReached => {
            assert_eq!(stats.mutation_limit_decisions(), 1);
            assert_eq!(stats.no_change_outcomes(), 1);
        }
    }
}

impl ServiceWindowSequenceStats {
    fn new() -> Self {
        Self {
            runtime: RuntimeApplicationStats::new(),
            registered_owners: 0,
            window: RemoteFreeServiceRuntimeWindowStats::new(),
            missing_owner_checks: 0,
            final_guard_applied_mutations: 0,
            final_guard_confirmed_mutations: 0,
            final_guard_rollbacks: 0,
            final_guard_pending_candidate: None,
        }
    }
}

fn assert_service_window_stats(stats: &ServiceWindowSequenceStats) {
    assert_eq!(stats.runtime.submitted_count, SERVICE_WINDOW_WINDOWS * 256);
    assert_eq!(stats.runtime.drained_count, stats.runtime.submitted_count);
    assert_eq!(
        stats.runtime.released_bytes,
        1_792_u64
            .saturating_mul(BYTES_PER_BLOCK)
            .saturating_add(256_u64.saturating_mul(SERVICE_WINDOW_ROLLBACK_VALIDATION_BYTES))
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

    assert_eq!(stats.registered_owners, SERVICE_WINDOW_OWNERS_USIZE);
    assert_eq!(stats.window.owner_observations(), SERVICE_WINDOW_WINDOWS);
    assert_eq!(stats.window.observed_reports(), SERVICE_WINDOW_WINDOWS * 8);
    assert_eq!(stats.window.reports_needing_retune(), 46);
    assert_eq!(stats.window.max_pending_items_over_target(), 192);
    assert_eq!(stats.window.max_queued_bytes_over_budget(), 786_432);
    assert_eq!(stats.window.queue_backpressure_reports(), 12);
    assert_eq!(stats.window.collect_telemetry_decisions(), 0);
    assert_eq!(stats.window.hold_decisions(), 3);
    assert_eq!(stats.window.apply_decisions(), 2);
    assert_eq!(stats.window.confirmed_decisions(), 1);
    assert_eq!(stats.window.rollback_decisions(), 1);
    assert_eq!(stats.window.mutation_limit_decisions(), 1);
    assert_eq!(stats.window.no_change_outcomes(), 4);
    assert_eq!(stats.window.applied_outcomes(), 2);
    assert_eq!(stats.window.confirmed_outcomes(), 1);
    assert_eq!(stats.window.rolled_back_outcomes(), 1);
    assert_eq!(stats.missing_owner_checks, 1);
    assert_eq!(stats.final_guard_applied_mutations, 2);
    assert_eq!(stats.final_guard_confirmed_mutations, 1);
    assert_eq!(stats.final_guard_rollbacks, 1);
    assert_eq!(stats.final_guard_pending_candidate, None);
}

fn option_candidate_label(candidate: Option<RemoteFreeServiceRetuneCandidate>) -> &'static str {
    candidate.map_or("none", RemoteFreeServiceRetuneCandidate::as_str)
}
