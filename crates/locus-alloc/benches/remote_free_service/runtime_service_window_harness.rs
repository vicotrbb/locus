#![allow(missing_docs)]

use std::convert::Infallible;

use criterion::{black_box, Criterion};
use locus_alloc::{
    RemoteFreeDrainPolicy, RemoteFreeOwnerRuntime, RemoteFreeServiceRetuneCandidate,
    RemoteFreeServiceRetuneGuard, RemoteFreeServiceRetunePolicyApplicator,
    RemoteFreeServiceRetuneSummary, RemoteFreeServiceRuntimeDirtyOwnerLocalBuffer,
    RemoteFreeServiceRuntimeDirtyOwnerLocalBuffers, RemoteFreeServiceRuntimeDirtyOwnerTracker,
    RemoteFreeServiceRuntimeDirtyOwners, RemoteFreeServiceRuntimeOwnerId,
    RemoteFreeServiceRuntimeRetuneCoordinator, RemoteFreeServiceRuntimeRetuneOwners,
    RemoteFreeServiceRuntimeWindowObservation, RemoteFreeServiceRuntimeWindowStats,
};

use crate::remote_free_service_application_harness::{
    run_runtime_owner_window_with_dirty_summary_and_block_bytes,
    run_runtime_owner_window_with_local_dirty_burst_summary_and_block_bytes,
    run_runtime_owner_window_with_local_dirty_summary_and_block_bytes,
    run_runtime_owner_window_with_local_dirty_threshold_summary_and_block_bytes,
    run_runtime_owner_window_with_summary, run_runtime_owner_window_with_summary_and_block_bytes,
    service_config, RuntimeApplicationStats, RuntimeLocalDirtyFlushStats, RuntimeTraceBlock,
    QUEUE_CAPACITY_GROWTH_FACTOR, RUNTIME_INITIAL_QUEUE_CAPACITY,
};
use crate::remote_free_service_harness::{
    format_milli, CounterSummary, BURSTS, BURST_BLOCKS, BYTES_PER_BLOCK, QUEUE_CAPACITY, SAMPLES,
    TARGET_PENDING_BLOCKS,
};
use crate::remote_free_service_runtime_local_dirty_group_harness::{
    assert_bounded_missing_owner, assert_integrated_missing_owner, assert_validated_missing_owner,
    collect_runtime_local_dirty_group_window, RuntimeLocalDirtyGroupCollectionMode,
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
    DirtyEnqueueMark,
    DirtyLocalBuffer,
    DirtyLocalReusedBuffer,
    DirtyLocalBufferGroup,
    DirtyLocalBufferGroupIntegrated,
    DirtyLocalBufferGroupBounded,
    DirtyLocalBufferGroupValidated,
    DirtyLocalBurstFlush,
    DirtyLocalThresholdFlush,
}

#[derive(Debug)]
struct ServiceWindowDirtyLocalState {
    tracker: RemoteFreeServiceRuntimeDirtyOwnerTracker,
    buffers: Vec<RemoteFreeServiceRuntimeDirtyOwnerLocalBuffer>,
    flush_stats: RuntimeLocalDirtyFlushStats,
    buffer_group: RemoteFreeServiceRuntimeDirtyOwnerLocalBuffers,
    buffer_group_flush_stats: RuntimeLocalDirtyFlushStats,
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

pub(crate) fn benchmark_runtime_dirty_enqueue_collection_sequence(c: &mut Criterion) {
    print_service_window_sample(
        ServiceWindowRunnerMode::DirtyEnqueueMark,
        "remote_free_service_runtime_dirty_enqueue_collection_sample",
    );
    print_service_window_sample_summary(
        ServiceWindowRunnerMode::DirtyEnqueueMark,
        "remote_free_service_runtime_dirty_enqueue_collection_sample_summary",
    );

    c.bench_function(
        "remote_free_service_runtime_dirty_enqueue_collection_sequence",
        |bench| {
            bench.iter(|| {
                let stats =
                    run_runtime_service_window_sequence(ServiceWindowRunnerMode::DirtyEnqueueMark);
                assert_service_window_stats(&stats);
                black_box(stats);
            });
        },
    );
}

pub(crate) fn benchmark_runtime_dirty_local_collection_sequence(c: &mut Criterion) {
    print_service_window_sample(
        ServiceWindowRunnerMode::DirtyLocalBuffer,
        "remote_free_service_runtime_dirty_local_collection_sample",
    );
    print_service_window_sample_summary(
        ServiceWindowRunnerMode::DirtyLocalBuffer,
        "remote_free_service_runtime_dirty_local_collection_sample_summary",
    );

    c.bench_function(
        "remote_free_service_runtime_dirty_local_collection_sequence",
        |bench| {
            bench.iter(|| {
                let stats =
                    run_runtime_service_window_sequence(ServiceWindowRunnerMode::DirtyLocalBuffer);
                assert_service_window_stats(&stats);
                black_box(stats);
            });
        },
    );
}

pub(crate) fn benchmark_runtime_dirty_local_reused_collection_sequence(c: &mut Criterion) {
    print_service_window_sample(
        ServiceWindowRunnerMode::DirtyLocalReusedBuffer,
        "remote_free_service_runtime_dirty_local_reused_collection_sample",
    );
    print_service_window_sample_summary(
        ServiceWindowRunnerMode::DirtyLocalReusedBuffer,
        "remote_free_service_runtime_dirty_local_reused_collection_sample_summary",
    );

    c.bench_function(
        "remote_free_service_runtime_dirty_local_reused_collection_sequence",
        |bench| {
            bench.iter(|| {
                let stats = run_runtime_service_window_sequence(
                    ServiceWindowRunnerMode::DirtyLocalReusedBuffer,
                );
                assert_service_window_stats(&stats);
                black_box(stats);
            });
        },
    );
}

pub(crate) fn benchmark_runtime_dirty_local_buffer_group_collection_sequence(c: &mut Criterion) {
    print_service_window_sample(
        ServiceWindowRunnerMode::DirtyLocalBufferGroup,
        "remote_free_service_runtime_dirty_local_buffer_group_collection_sample",
    );
    print_service_window_sample_summary(
        ServiceWindowRunnerMode::DirtyLocalBufferGroup,
        "remote_free_service_runtime_dirty_local_buffer_group_collection_sample_summary",
    );

    c.bench_function(
        "remote_free_service_runtime_dirty_local_buffer_group_collection_sequence",
        |bench| {
            bench.iter(|| {
                let stats = run_runtime_service_window_sequence(
                    ServiceWindowRunnerMode::DirtyLocalBufferGroup,
                );
                assert_service_window_stats(&stats);
                black_box(stats);
            });
        },
    );
}

pub(crate) fn benchmark_runtime_dirty_local_buffer_group_integrated_collection_sequence(
    c: &mut Criterion,
) {
    print_service_window_sample(
        ServiceWindowRunnerMode::DirtyLocalBufferGroupIntegrated,
        "remote_free_service_runtime_dirty_local_buffer_group_integrated_collection_sample",
    );
    print_service_window_sample_summary(
        ServiceWindowRunnerMode::DirtyLocalBufferGroupIntegrated,
        "remote_free_service_runtime_dirty_local_buffer_group_integrated_collection_sample_summary",
    );

    c.bench_function(
        "remote_free_service_runtime_dirty_local_buffer_group_integrated_collection_sequence",
        |bench| {
            bench.iter(|| {
                let stats = run_runtime_service_window_sequence(
                    ServiceWindowRunnerMode::DirtyLocalBufferGroupIntegrated,
                );
                assert_service_window_stats(&stats);
                black_box(stats);
            });
        },
    );
}

pub(crate) fn benchmark_runtime_dirty_local_buffer_group_bounded_collection_sequence(
    c: &mut Criterion,
) {
    print_service_window_sample(
        ServiceWindowRunnerMode::DirtyLocalBufferGroupBounded,
        "remote_free_service_runtime_dirty_local_buffer_group_bounded_collection_sample",
    );
    print_service_window_sample_summary(
        ServiceWindowRunnerMode::DirtyLocalBufferGroupBounded,
        "remote_free_service_runtime_dirty_local_buffer_group_bounded_collection_sample_summary",
    );

    c.bench_function(
        "remote_free_service_runtime_dirty_local_buffer_group_bounded_collection_sequence",
        |bench| {
            bench.iter(|| {
                let stats = run_runtime_service_window_sequence(
                    ServiceWindowRunnerMode::DirtyLocalBufferGroupBounded,
                );
                assert_service_window_stats(&stats);
                black_box(stats);
            });
        },
    );
}

pub(crate) fn benchmark_runtime_dirty_local_buffer_group_validated_collection_sequence(
    c: &mut Criterion,
) {
    print_service_window_sample(
        ServiceWindowRunnerMode::DirtyLocalBufferGroupValidated,
        "remote_free_service_runtime_dirty_local_buffer_group_validated_collection_sample",
    );
    print_service_window_sample_summary(
        ServiceWindowRunnerMode::DirtyLocalBufferGroupValidated,
        "remote_free_service_runtime_dirty_local_buffer_group_validated_collection_sample_summary",
    );

    c.bench_function(
        "remote_free_service_runtime_dirty_local_buffer_group_validated_collection_sequence",
        |bench| {
            bench.iter(|| {
                let stats = run_runtime_service_window_sequence(
                    ServiceWindowRunnerMode::DirtyLocalBufferGroupValidated,
                );
                assert_service_window_stats(&stats);
                black_box(stats);
            });
        },
    );
}

pub(crate) fn benchmark_runtime_dirty_local_burst_collection_sequence(c: &mut Criterion) {
    print_service_window_sample(
        ServiceWindowRunnerMode::DirtyLocalBurstFlush,
        "remote_free_service_runtime_dirty_local_burst_collection_sample",
    );
    print_service_window_sample_summary(
        ServiceWindowRunnerMode::DirtyLocalBurstFlush,
        "remote_free_service_runtime_dirty_local_burst_collection_sample_summary",
    );

    c.bench_function(
        "remote_free_service_runtime_dirty_local_burst_collection_sequence",
        |bench| {
            bench.iter(|| {
                let stats = run_runtime_service_window_sequence(
                    ServiceWindowRunnerMode::DirtyLocalBurstFlush,
                );
                assert_service_window_stats(&stats);
                black_box(stats);
            });
        },
    );
}

pub(crate) fn benchmark_runtime_dirty_local_threshold_collection_sequence(c: &mut Criterion) {
    print_service_window_sample(
        ServiceWindowRunnerMode::DirtyLocalThresholdFlush,
        "remote_free_service_runtime_dirty_local_threshold_collection_sample",
    );
    print_service_window_sample_summary(
        ServiceWindowRunnerMode::DirtyLocalThresholdFlush,
        "remote_free_service_runtime_dirty_local_threshold_collection_sample_summary",
    );

    c.bench_function(
        "remote_free_service_runtime_dirty_local_threshold_collection_sequence",
        |bench| {
            bench.iter(|| {
                let stats = run_runtime_service_window_sequence(
                    ServiceWindowRunnerMode::DirtyLocalThresholdFlush,
                );
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
    let mut dirty_local_state = ServiceWindowDirtyLocalState::new();
    stats.registered_owners = owners.len();

    run_confirmed_owner(
        &mut owners,
        confirmed,
        &mut stats,
        mode,
        &mut dirty_local_state,
    );
    run_rolled_back_owner(
        &mut owners,
        rolled_back,
        &mut stats,
        mode,
        &mut dirty_local_state,
    );
    run_mutation_limited_owner(
        &mut owners,
        mutation_limited,
        &mut stats,
        mode,
        &mut dirty_local_state,
    );

    assert_dirty_local_sequence_state(mode, &dirty_local_state);

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

fn assert_dirty_local_sequence_state(
    mode: ServiceWindowRunnerMode,
    dirty_local_state: &ServiceWindowDirtyLocalState,
) {
    let expected_duplicate_marks = SERVICE_WINDOW_WINDOWS
        .saturating_mul(BURSTS)
        .saturating_mul(BURST_BLOCKS)
        .saturating_sub(SERVICE_WINDOW_WINDOWS);

    if mode == ServiceWindowRunnerMode::DirtyLocalReusedBuffer {
        assert_eq!(
            dirty_local_state.flush_stats.flush_count,
            SERVICE_WINDOW_WINDOWS
        );
        assert_eq!(
            dirty_local_state.flush_stats.owner_count,
            SERVICE_WINDOW_WINDOWS
        );
        assert_eq!(
            dirty_local_state.flush_stats.new_tracker_marks,
            SERVICE_WINDOW_WINDOWS
        );
        assert_eq!(
            dirty_local_state.flush_stats.duplicate_local_marks,
            expected_duplicate_marks
        );
        assert!(dirty_local_state.tracker.is_empty());
    }

    if matches!(
        mode,
        ServiceWindowRunnerMode::DirtyLocalBufferGroup
            | ServiceWindowRunnerMode::DirtyLocalBufferGroupIntegrated
            | ServiceWindowRunnerMode::DirtyLocalBufferGroupBounded
            | ServiceWindowRunnerMode::DirtyLocalBufferGroupValidated
    ) {
        assert_eq!(
            dirty_local_state.buffer_group_flush_stats.flush_count,
            SERVICE_WINDOW_WINDOWS
        );
        assert_eq!(
            dirty_local_state.buffer_group_flush_stats.owner_count,
            SERVICE_WINDOW_WINDOWS
        );
        assert_eq!(
            dirty_local_state.buffer_group_flush_stats.new_tracker_marks,
            SERVICE_WINDOW_WINDOWS
        );
        assert_eq!(
            dirty_local_state
                .buffer_group_flush_stats
                .duplicate_local_marks,
            expected_duplicate_marks
        );
        assert!(dirty_local_state.buffer_group.tracker().is_empty());
    }
}

fn run_confirmed_owner(
    owners: &mut RemoteFreeServiceRuntimeRetuneOwners<RuntimeTraceBlock>,
    owner_id: RemoteFreeServiceRuntimeOwnerId,
    stats: &mut ServiceWindowSequenceStats,
    mode: ServiceWindowRunnerMode,
    dirty_local_state: &mut ServiceWindowDirtyLocalState,
) {
    observe_window(
        owners,
        owner_id,
        stats,
        ServiceWindowDecisionKind::Hold,
        BYTES_PER_BLOCK,
        mode,
        dirty_local_state,
    );
    observe_window(
        owners,
        owner_id,
        stats,
        ServiceWindowDecisionKind::Apply,
        BYTES_PER_BLOCK,
        mode,
        dirty_local_state,
    );
    observe_window(
        owners,
        owner_id,
        stats,
        ServiceWindowDecisionKind::Confirmed,
        BYTES_PER_BLOCK,
        mode,
        dirty_local_state,
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
    dirty_local_state: &mut ServiceWindowDirtyLocalState,
) {
    observe_window(
        owners,
        owner_id,
        stats,
        ServiceWindowDecisionKind::Hold,
        BYTES_PER_BLOCK,
        mode,
        dirty_local_state,
    );
    observe_window(
        owners,
        owner_id,
        stats,
        ServiceWindowDecisionKind::Apply,
        BYTES_PER_BLOCK,
        mode,
        dirty_local_state,
    );
    observe_window(
        owners,
        owner_id,
        stats,
        ServiceWindowDecisionKind::Rollback,
        SERVICE_WINDOW_ROLLBACK_VALIDATION_BYTES,
        mode,
        dirty_local_state,
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
    dirty_local_state: &mut ServiceWindowDirtyLocalState,
) {
    observe_window(
        owners,
        owner_id,
        stats,
        ServiceWindowDecisionKind::Hold,
        BYTES_PER_BLOCK,
        mode,
        dirty_local_state,
    );
    observe_window(
        owners,
        owner_id,
        stats,
        ServiceWindowDecisionKind::MutationLimitReached,
        BYTES_PER_BLOCK,
        mode,
        dirty_local_state,
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
    dirty_local_state: &mut ServiceWindowDirtyLocalState,
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
        ServiceWindowRunnerMode::DirtyEnqueueMark => {
            collect_dirty_enqueue_window(owners, owner_id, stats, block_bytes)
        }
        ServiceWindowRunnerMode::DirtyLocalBuffer => {
            collect_dirty_local_window(owners, owner_id, stats, block_bytes)
        }
        ServiceWindowRunnerMode::DirtyLocalReusedBuffer => collect_dirty_local_reused_window(
            owners,
            owner_id,
            stats,
            block_bytes,
            dirty_local_state,
        ),
        ServiceWindowRunnerMode::DirtyLocalBufferGroup => collect_dirty_local_group_window(
            RuntimeLocalDirtyGroupCollectionMode::Manual,
            owners,
            owner_id,
            stats,
            block_bytes,
            dirty_local_state,
        ),
        ServiceWindowRunnerMode::DirtyLocalBufferGroupIntegrated => {
            collect_dirty_local_group_window(
                RuntimeLocalDirtyGroupCollectionMode::Integrated,
                owners,
                owner_id,
                stats,
                block_bytes,
                dirty_local_state,
            )
        }
        ServiceWindowRunnerMode::DirtyLocalBufferGroupBounded => collect_dirty_local_group_window(
            RuntimeLocalDirtyGroupCollectionMode::Bounded,
            owners,
            owner_id,
            stats,
            block_bytes,
            dirty_local_state,
        ),
        ServiceWindowRunnerMode::DirtyLocalBufferGroupValidated => {
            collect_dirty_local_group_window(
                RuntimeLocalDirtyGroupCollectionMode::Validated,
                owners,
                owner_id,
                stats,
                block_bytes,
                dirty_local_state,
            )
        }
        ServiceWindowRunnerMode::DirtyLocalBurstFlush => {
            collect_dirty_local_burst_window(owners, owner_id, stats, block_bytes)
        }
        ServiceWindowRunnerMode::DirtyLocalThresholdFlush => {
            collect_dirty_local_threshold_window(owners, owner_id, stats, block_bytes)
        }
    };

    assert_one_window_decision(window_stats, expected_decision);
    stats.window.merge(window_stats);
}

fn collect_dirty_enqueue_window(
    owners: &mut RemoteFreeServiceRuntimeRetuneOwners<RuntimeTraceBlock>,
    owner_id: RemoteFreeServiceRuntimeOwnerId,
    stats: &mut ServiceWindowSequenceStats,
    block_bytes: u64,
) -> RemoteFreeServiceRuntimeWindowStats {
    let tracker = RemoteFreeServiceRuntimeDirtyOwnerTracker::new();
    let summary = run_runtime_owner_window_with_dirty_summary_and_block_bytes(
        owners.owner_mut(owner_id).expect("owner for dirty sink"),
        &mut stats.runtime,
        block_bytes,
        owner_id,
        &tracker,
    );
    assert_eq!(tracker.owner_ids(), vec![owner_id]);

    let stats = owners
        .collect_tracked_dirty_service_window(&tracker, |_, _| Ok::<_, Infallible>(summary))
        .expect("tracked dirty service window stats");
    assert!(tracker.is_empty());
    stats
}

fn collect_dirty_local_window(
    owners: &mut RemoteFreeServiceRuntimeRetuneOwners<RuntimeTraceBlock>,
    owner_id: RemoteFreeServiceRuntimeOwnerId,
    stats: &mut ServiceWindowSequenceStats,
    block_bytes: u64,
) -> RemoteFreeServiceRuntimeWindowStats {
    let tracker = RemoteFreeServiceRuntimeDirtyOwnerTracker::new();
    let mut buffer = RemoteFreeServiceRuntimeDirtyOwnerLocalBuffer::new();
    let summary = run_runtime_owner_window_with_local_dirty_summary_and_block_bytes(
        owners
            .owner_mut(owner_id)
            .expect("owner for local dirty buffer"),
        &mut stats.runtime,
        block_bytes,
        owner_id,
        &mut buffer,
    );
    let expected_duplicate_marks = BURSTS.saturating_mul(BURST_BLOCKS).saturating_sub(1);
    assert_eq!(buffer.owner_ids(), &[owner_id]);
    assert_eq!(buffer.duplicate_marks(), expected_duplicate_marks);

    let flush = buffer.flush_into(&tracker);
    assert_eq!(flush.owner_count, 1);
    assert_eq!(flush.new_tracker_marks, 1);
    assert_eq!(flush.duplicate_local_marks, expected_duplicate_marks);
    assert_eq!(tracker.owner_ids(), vec![owner_id]);

    let stats = owners
        .collect_tracked_dirty_service_window(&tracker, |_, _| Ok::<_, Infallible>(summary))
        .expect("local dirty service window stats");
    assert!(tracker.is_empty());
    stats
}

fn collect_dirty_local_reused_window(
    owners: &mut RemoteFreeServiceRuntimeRetuneOwners<RuntimeTraceBlock>,
    owner_id: RemoteFreeServiceRuntimeOwnerId,
    stats: &mut ServiceWindowSequenceStats,
    block_bytes: u64,
    dirty_local_state: &mut ServiceWindowDirtyLocalState,
) -> RemoteFreeServiceRuntimeWindowStats {
    let tracker = dirty_local_state.tracker.clone();
    let previous_capacity = dirty_local_state.buffer_capacity(owner_id);
    let summary = {
        let buffer = dirty_local_state.buffer_mut(owner_id);
        run_runtime_owner_window_with_local_dirty_summary_and_block_bytes(
            owners
                .owner_mut(owner_id)
                .expect("owner for reused local dirty buffer"),
            &mut stats.runtime,
            block_bytes,
            owner_id,
            buffer,
        )
    };
    let expected_duplicate_marks = BURSTS.saturating_mul(BURST_BLOCKS).saturating_sub(1);
    {
        let buffer = dirty_local_state.buffer_mut(owner_id);
        assert_eq!(buffer.owner_ids(), &[owner_id]);
        assert_eq!(buffer.duplicate_marks(), expected_duplicate_marks);
    }

    let flush = {
        let buffer = dirty_local_state.buffer_mut(owner_id);
        buffer.flush_into(&tracker)
    };
    dirty_local_state.flush_stats.observe(flush);
    assert_eq!(flush.owner_count, 1);
    assert_eq!(flush.new_tracker_marks, 1);
    assert_eq!(flush.duplicate_local_marks, expected_duplicate_marks);
    assert!(dirty_local_state.buffer_capacity(owner_id) >= 1);
    assert!(dirty_local_state.buffer_capacity(owner_id) >= previous_capacity);
    assert_eq!(tracker.owner_ids(), vec![owner_id]);

    let stats = owners
        .collect_tracked_dirty_service_window(&tracker, |_, _| Ok::<_, Infallible>(summary))
        .expect("reused local dirty service window stats");
    assert!(tracker.is_empty());
    stats
}

fn collect_dirty_local_group_window(
    mode: RuntimeLocalDirtyGroupCollectionMode,
    owners: &mut RemoteFreeServiceRuntimeRetuneOwners<RuntimeTraceBlock>,
    owner_id: RemoteFreeServiceRuntimeOwnerId,
    stats: &mut ServiceWindowSequenceStats,
    block_bytes: u64,
    dirty_local_state: &mut ServiceWindowDirtyLocalState,
) -> RemoteFreeServiceRuntimeWindowStats {
    collect_runtime_local_dirty_group_window(
        mode,
        owners,
        owner_id,
        &mut stats.runtime,
        block_bytes,
        &mut dirty_local_state.buffer_group,
        &mut dirty_local_state.buffer_group_flush_stats,
    )
}

fn collect_dirty_local_burst_window(
    owners: &mut RemoteFreeServiceRuntimeRetuneOwners<RuntimeTraceBlock>,
    owner_id: RemoteFreeServiceRuntimeOwnerId,
    stats: &mut ServiceWindowSequenceStats,
    block_bytes: u64,
) -> RemoteFreeServiceRuntimeWindowStats {
    let tracker = RemoteFreeServiceRuntimeDirtyOwnerTracker::new();
    let (summary, flush_stats) =
        run_runtime_owner_window_with_local_dirty_burst_summary_and_block_bytes(
            owners
                .owner_mut(owner_id)
                .expect("owner for local dirty burst buffer"),
            &mut stats.runtime,
            block_bytes,
            owner_id,
            &tracker,
        );
    let expected_duplicate_marks = BURSTS.saturating_mul(BURST_BLOCKS.saturating_sub(1));
    assert_eq!(flush_stats.flush_count, BURSTS);
    assert_eq!(flush_stats.owner_count, BURSTS);
    assert_eq!(flush_stats.new_tracker_marks, 1);
    assert_eq!(flush_stats.duplicate_local_marks, expected_duplicate_marks);
    assert_eq!(tracker.owner_ids(), vec![owner_id]);

    let stats = owners
        .collect_tracked_dirty_service_window(&tracker, |_, _| Ok::<_, Infallible>(summary))
        .expect("local dirty burst service window stats");
    assert!(tracker.is_empty());
    stats
}

fn collect_dirty_local_threshold_window(
    owners: &mut RemoteFreeServiceRuntimeRetuneOwners<RuntimeTraceBlock>,
    owner_id: RemoteFreeServiceRuntimeOwnerId,
    stats: &mut ServiceWindowSequenceStats,
    block_bytes: u64,
) -> RemoteFreeServiceRuntimeWindowStats {
    let tracker = RemoteFreeServiceRuntimeDirtyOwnerTracker::new();
    let (summary, flush_stats) =
        run_runtime_owner_window_with_local_dirty_threshold_summary_and_block_bytes(
            owners
                .owner_mut(owner_id)
                .expect("owner for local dirty threshold buffer"),
            &mut stats.runtime,
            block_bytes,
            owner_id,
            &tracker,
            TARGET_PENDING_BLOCKS,
        );
    let total_marks = BURSTS.saturating_mul(BURST_BLOCKS);
    let expected_flushes = total_marks.div_ceil(TARGET_PENDING_BLOCKS);
    let expected_duplicate_marks =
        expected_threshold_duplicate_marks(total_marks, TARGET_PENDING_BLOCKS);
    assert_eq!(flush_stats.flush_count, expected_flushes);
    assert_eq!(flush_stats.owner_count, expected_flushes);
    assert_eq!(flush_stats.new_tracker_marks, 1);
    assert_eq!(flush_stats.duplicate_local_marks, expected_duplicate_marks);
    assert_eq!(tracker.owner_ids(), vec![owner_id]);

    let stats = owners
        .collect_tracked_dirty_service_window(&tracker, |_, _| Ok::<_, Infallible>(summary))
        .expect("local dirty threshold service window stats");
    assert!(tracker.is_empty());
    stats
}

fn expected_threshold_duplicate_marks(total_marks: u64, threshold: u64) -> u64 {
    assert!(threshold != 0);

    let full_flushes = total_marks / threshold;
    let remaining = total_marks % threshold;

    full_flushes
        .saturating_mul(threshold.saturating_sub(1))
        .saturating_add(remaining.saturating_sub(1))
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
        ServiceWindowRunnerMode::DirtyEnqueueMark => {
            let tracker = RemoteFreeServiceRuntimeDirtyOwnerTracker::new();
            assert!(tracker.mark_dirty(missing));
            let error = owners
                .collect_tracked_dirty_service_window(&tracker, |_, _| {
                    Ok::<_, Infallible>(RemoteFreeServiceRetuneSummary::new())
                })
                .expect_err("missing owner");
            assert_eq!(error.owner_id(), missing);
            assert_eq!(tracker.owner_ids(), vec![missing]);
        }
        ServiceWindowRunnerMode::DirtyLocalBuffer
        | ServiceWindowRunnerMode::DirtyLocalReusedBuffer
        | ServiceWindowRunnerMode::DirtyLocalBufferGroup
        | ServiceWindowRunnerMode::DirtyLocalBurstFlush
        | ServiceWindowRunnerMode::DirtyLocalThresholdFlush => {
            assert_local_buffer_missing_owner(owners, missing);
        }
        ServiceWindowRunnerMode::DirtyLocalBufferGroupIntegrated => {
            assert_integrated_missing_owner(owners, missing);
        }
        ServiceWindowRunnerMode::DirtyLocalBufferGroupBounded => {
            assert_bounded_missing_owner(owners, missing);
        }
        ServiceWindowRunnerMode::DirtyLocalBufferGroupValidated => {
            assert_validated_missing_owner(owners, missing);
        }
    }
}

fn assert_local_buffer_missing_owner(
    owners: &mut RemoteFreeServiceRuntimeRetuneOwners<RuntimeTraceBlock>,
    missing: RemoteFreeServiceRuntimeOwnerId,
) {
    let tracker = RemoteFreeServiceRuntimeDirtyOwnerTracker::new();
    let mut buffer = RemoteFreeServiceRuntimeDirtyOwnerLocalBuffer::new();
    assert!(buffer.mark_dirty(missing));
    let flush = buffer.flush_into(&tracker);
    assert_eq!(flush.owner_count, 1);
    assert_eq!(flush.new_tracker_marks, 1);
    let error = owners
        .collect_tracked_dirty_service_window(&tracker, |_, _| {
            Ok::<_, Infallible>(RemoteFreeServiceRetuneSummary::new())
        })
        .expect_err("missing owner");
    assert_eq!(error.owner_id(), missing);
    assert_eq!(tracker.owner_ids(), vec![missing]);
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

impl ServiceWindowDirtyLocalState {
    fn new() -> Self {
        Self {
            tracker: RemoteFreeServiceRuntimeDirtyOwnerTracker::new(),
            buffers: Vec::new(),
            flush_stats: RuntimeLocalDirtyFlushStats::default(),
            buffer_group: RemoteFreeServiceRuntimeDirtyOwnerLocalBuffers::new(),
            buffer_group_flush_stats: RuntimeLocalDirtyFlushStats::default(),
        }
    }

    fn buffer_mut(
        &mut self,
        owner_id: RemoteFreeServiceRuntimeOwnerId,
    ) -> &mut RemoteFreeServiceRuntimeDirtyOwnerLocalBuffer {
        let index = owner_id.index();
        if self.buffers.len() <= index {
            self.buffers.resize_with(
                index.saturating_add(1),
                RemoteFreeServiceRuntimeDirtyOwnerLocalBuffer::new,
            );
        }
        &mut self.buffers[index]
    }

    fn buffer_capacity(&self, owner_id: RemoteFreeServiceRuntimeOwnerId) -> usize {
        self.buffers
            .get(owner_id.index())
            .map_or(0, RemoteFreeServiceRuntimeDirtyOwnerLocalBuffer::capacity)
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
