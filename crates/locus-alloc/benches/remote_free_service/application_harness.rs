#![allow(missing_docs)]

use std::thread;

use criterion::{black_box, Criterion};
use locus_alloc::{
    RemoteFreeOwnerRuntime, RemoteFreeOwnerRuntimeApplyOutcome,
    RemoteFreeOwnerRuntimeConfirmOutcome, RemoteFreeOwnerRuntimeRollbackOutcome,
    RemoteFreeQueuedByteDrainConfig, RemoteFreeServiceRetuneCandidate,
    RemoteFreeServiceRetuneGuardDecision, RemoteFreeServiceRetunePolicyApplication,
    RemoteFreeServiceRetunePolicyApplicator, RemoteFreeTryEnqueueErrorKind,
};

use crate::remote_free_service_harness::{
    format_milli, CounterSummary, ServiceTelemetryCase, BATCH_LIMIT, BURSTS, BURST_BLOCKS,
    BYTES_PER_BLOCK, QUEUE_CAPACITY, SAMPLES, TARGET_PENDING_BLOCKS,
};

const QUEUE_CAPACITY_GROWTH_FACTOR: usize = 2;
const RUNTIME_WINDOWS: u64 = 3;
const RUNTIME_INITIAL_QUEUE_CAPACITY: usize = QUEUE_CAPACITY / QUEUE_CAPACITY_GROWTH_FACTOR;

#[derive(Debug)]
struct RuntimeTraceBlock {
    submit_burst: u64,
    allocation: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
struct RuntimeApplicationStats {
    submitted_count: u64,
    drained_count: u64,
    released_bytes: u64,
    policy_drains: u64,
    drain_rounds: u64,
    max_wait_bursts: u64,
    total_wait_bursts: u64,
    install_count: u64,
    confirm_count: u64,
    rollback_count: u64,
    final_queue_capacity: usize,
    final_previous_config_present: bool,
}

pub(crate) fn benchmark_runtime_application(c: &mut Criterion) {
    print_runtime_application_sample();
    print_runtime_application_sample_summary();
    c.bench_function("remote_free_service_runtime_apply_rollback", |b| {
        b.iter(|| {
            let stats = run_runtime_application_sequence();
            assert_runtime_application_stats(stats);
            black_box(stats);
        });
    });

    print_runtime_confirm_sample();
    print_runtime_confirm_sample_summary();
    c.bench_function("remote_free_service_runtime_apply_confirm", |b| {
        b.iter(|| {
            let stats = run_runtime_confirm_sequence();
            assert_runtime_confirm_stats(stats);
            black_box(stats);
        });
    });
}

pub(crate) fn candidate_case(candidate: RemoteFreeServiceRetuneCandidate) -> ServiceTelemetryCase {
    let applicator = RemoteFreeServiceRetunePolicyApplicator::try_new(QUEUE_CAPACITY_GROWTH_FACTOR)
        .expect("policy applicator");
    let current_config = current_config_for_candidate(candidate);
    let application = applicator
        .plan(
            current_config,
            RemoteFreeServiceRetuneGuardDecision::Apply { candidate },
        )
        .expect("policy application plan");

    let RemoteFreeServiceRetunePolicyApplication::Apply { candidate, config } = application else {
        panic!("apply decision produced no policy application");
    };

    service_case_for_applied_config(candidate, config)
}

fn current_config_for_candidate(
    candidate: RemoteFreeServiceRetuneCandidate,
) -> RemoteFreeQueuedByteDrainConfig {
    match candidate {
        RemoteFreeServiceRetuneCandidate::DrainEarlier
        | RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacity => service_config(QUEUE_CAPACITY),
        RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier => {
            service_config(QUEUE_CAPACITY / QUEUE_CAPACITY_GROWTH_FACTOR)
        }
        _ => panic!("candidate cannot be applied by guarded benchmark: {candidate:?}"),
    }
}

fn service_config(queue_capacity: usize) -> RemoteFreeQueuedByteDrainConfig {
    RemoteFreeQueuedByteDrainConfig::from_item_shape(
        queue_capacity,
        BATCH_LIMIT,
        TARGET_PENDING_BLOCKS,
        BYTES_PER_BLOCK,
    )
    .expect("service config")
}

fn service_case_for_applied_config(
    candidate: RemoteFreeServiceRetuneCandidate,
    config: RemoteFreeQueuedByteDrainConfig,
) -> ServiceTelemetryCase {
    assert_service_config(config, expected_capacity_for_candidate(candidate));

    match candidate {
        RemoteFreeServiceRetuneCandidate::DrainEarlier => {
            ServiceTelemetryCase::planner_candidate_drain_earlier()
        }
        RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier => {
            ServiceTelemetryCase::planner_candidate_capacity_and_drain_earlier()
        }
        RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacity => {
            panic!("capacity-only candidate has no guarded service benchmark case")
        }
        _ => panic!("candidate cannot be applied by guarded benchmark: {candidate:?}"),
    }
}

fn expected_capacity_for_candidate(candidate: RemoteFreeServiceRetuneCandidate) -> usize {
    match candidate {
        RemoteFreeServiceRetuneCandidate::DrainEarlier
        | RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier => QUEUE_CAPACITY,
        RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacity => {
            QUEUE_CAPACITY * QUEUE_CAPACITY_GROWTH_FACTOR
        }
        _ => panic!("candidate cannot be applied by guarded benchmark: {candidate:?}"),
    }
}

fn assert_service_config(config: RemoteFreeQueuedByteDrainConfig, queue_capacity: usize) {
    assert_eq!(config.queue_capacity(), queue_capacity);
    assert_eq!(config.drain_batch_limit(), BATCH_LIMIT);
    assert_eq!(config.target_pending_items(), TARGET_PENDING_BLOCKS);
    assert_eq!(config.queued_byte_budget().bytes(), 262_144);
}

impl RuntimeApplicationStats {
    fn new() -> Self {
        Self {
            submitted_count: 0,
            drained_count: 0,
            released_bytes: 0,
            policy_drains: 0,
            drain_rounds: 0,
            max_wait_bursts: 0,
            total_wait_bursts: 0,
            install_count: 0,
            confirm_count: 0,
            rollback_count: 0,
            final_queue_capacity: 0,
            final_previous_config_present: false,
        }
    }

    fn mean_wait_milli(self) -> u64 {
        if self.drained_count == 0 {
            return 0;
        }

        self.total_wait_bursts.saturating_mul(1000) / self.drained_count
    }
}

fn print_runtime_application_sample() {
    let stats = run_runtime_application_sequence();
    assert_runtime_application_stats(stats);

    println!(
        "remote_free_service_runtime_apply_rollback_sample windows={RUNTIME_WINDOWS} initial_queue_capacity={RUNTIME_INITIAL_QUEUE_CAPACITY} installed_queue_capacity={QUEUE_CAPACITY} final_queue_capacity={} submitted_count={} drained_count={} released_bytes={} policy_drains={} drain_rounds={} install_count={} rollback_count={} max_wait_bursts={} mean_wait_bursts={} final_previous_config_present={}",
        stats.final_queue_capacity,
        stats.submitted_count,
        stats.drained_count,
        stats.released_bytes,
        stats.policy_drains,
        stats.drain_rounds,
        stats.install_count,
        stats.rollback_count,
        stats.max_wait_bursts,
        format_milli(stats.mean_wait_milli()),
        stats.final_previous_config_present,
    );
}

fn print_runtime_confirm_sample() {
    let stats = run_runtime_confirm_sequence();
    assert_runtime_confirm_stats(stats);

    println!(
        "remote_free_service_runtime_apply_confirm_sample windows={RUNTIME_WINDOWS} initial_queue_capacity={RUNTIME_INITIAL_QUEUE_CAPACITY} installed_queue_capacity={QUEUE_CAPACITY} final_queue_capacity={} submitted_count={} drained_count={} released_bytes={} policy_drains={} drain_rounds={} install_count={} confirm_count={} rollback_count={} max_wait_bursts={} mean_wait_bursts={} final_previous_config_present={}",
        stats.final_queue_capacity,
        stats.submitted_count,
        stats.drained_count,
        stats.released_bytes,
        stats.policy_drains,
        stats.drain_rounds,
        stats.install_count,
        stats.confirm_count,
        stats.rollback_count,
        stats.max_wait_bursts,
        format_milli(stats.mean_wait_milli()),
        stats.final_previous_config_present,
    );
}

fn print_runtime_application_sample_summary() {
    let mut policy_drains = CounterSummary::new();
    let mut drain_rounds = CounterSummary::new();
    let mut max_wait = CounterSummary::new();
    let mut mean_wait = CounterSummary::new();

    for _ in 0..SAMPLES {
        let stats = run_runtime_application_sequence();
        assert_runtime_application_stats(stats);

        policy_drains.observe(stats.policy_drains);
        drain_rounds.observe(stats.drain_rounds);
        max_wait.observe(stats.max_wait_bursts);
        mean_wait.observe(stats.mean_wait_milli());
    }

    println!(
        "remote_free_service_runtime_apply_rollback_sample_summary windows={RUNTIME_WINDOWS} samples={SAMPLES} policy_drains_min={} policy_drains_max={} policy_drains_mean={} drain_rounds_min={} drain_rounds_max={} drain_rounds_mean={} max_wait_min={} max_wait_max={} max_wait_mean={} mean_wait_min={} mean_wait_max={} mean_wait_mean={}",
        policy_drains.min,
        policy_drains.max,
        format_milli(policy_drains.mean_milli(SAMPLES)),
        drain_rounds.min,
        drain_rounds.max,
        format_milli(drain_rounds.mean_milli(SAMPLES)),
        max_wait.min,
        max_wait.max,
        format_milli(max_wait.mean_milli(SAMPLES)),
        format_milli(mean_wait.min),
        format_milli(mean_wait.max),
        format_milli(mean_wait.mean_milli(SAMPLES) / 1000),
    );
}

fn print_runtime_confirm_sample_summary() {
    let mut policy_drains = CounterSummary::new();
    let mut drain_rounds = CounterSummary::new();
    let mut max_wait = CounterSummary::new();
    let mut mean_wait = CounterSummary::new();

    for _ in 0..SAMPLES {
        let stats = run_runtime_confirm_sequence();
        assert_runtime_confirm_stats(stats);

        policy_drains.observe(stats.policy_drains);
        drain_rounds.observe(stats.drain_rounds);
        max_wait.observe(stats.max_wait_bursts);
        mean_wait.observe(stats.mean_wait_milli());
    }

    println!(
        "remote_free_service_runtime_apply_confirm_sample_summary windows={RUNTIME_WINDOWS} samples={SAMPLES} policy_drains_min={} policy_drains_max={} policy_drains_mean={} drain_rounds_min={} drain_rounds_max={} drain_rounds_mean={} max_wait_min={} max_wait_max={} max_wait_mean={} mean_wait_min={} mean_wait_max={} mean_wait_mean={}",
        policy_drains.min,
        policy_drains.max,
        format_milli(policy_drains.mean_milli(SAMPLES)),
        drain_rounds.min,
        drain_rounds.max,
        format_milli(drain_rounds.mean_milli(SAMPLES)),
        max_wait.min,
        max_wait.max,
        format_milli(max_wait.mean_milli(SAMPLES)),
        format_milli(mean_wait.min),
        format_milli(mean_wait.max),
        format_milli(mean_wait.mean_milli(SAMPLES) / 1000),
    );
}

fn run_runtime_application_sequence() -> RuntimeApplicationStats {
    let mut runtime = RemoteFreeOwnerRuntime::new(service_config(RUNTIME_INITIAL_QUEUE_CAPACITY))
        .expect("owner runtime");
    let mut stats = RuntimeApplicationStats::new();

    run_runtime_owner_window(&mut runtime, &mut stats);

    let applicator = RemoteFreeServiceRetunePolicyApplicator::try_new(QUEUE_CAPACITY_GROWTH_FACTOR)
        .expect("policy applicator");
    let application = applicator
        .plan(
            runtime.config(),
            RemoteFreeServiceRetuneGuardDecision::Apply {
                candidate: RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier,
            },
        )
        .expect("application plan");
    assert_eq!(
        runtime.apply(application),
        Ok(RemoteFreeOwnerRuntimeApplyOutcome::Installed {
            candidate: RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier,
            previous_config: service_config(RUNTIME_INITIAL_QUEUE_CAPACITY),
            current_config: service_config(QUEUE_CAPACITY),
        })
    );
    stats.install_count = stats.install_count.saturating_add(1);

    run_runtime_owner_window(&mut runtime, &mut stats);

    assert_eq!(
        runtime.rollback(),
        Ok(RemoteFreeOwnerRuntimeRollbackOutcome {
            replaced_config: service_config(QUEUE_CAPACITY),
            restored_config: service_config(RUNTIME_INITIAL_QUEUE_CAPACITY),
        })
    );
    stats.rollback_count = stats.rollback_count.saturating_add(1);

    run_runtime_owner_window(&mut runtime, &mut stats);

    stats.final_queue_capacity = runtime.config().queue_capacity();
    stats.final_previous_config_present = runtime.previous_config().is_some();
    stats
}

fn run_runtime_confirm_sequence() -> RuntimeApplicationStats {
    let mut runtime = RemoteFreeOwnerRuntime::new(service_config(RUNTIME_INITIAL_QUEUE_CAPACITY))
        .expect("owner runtime");
    let mut stats = RuntimeApplicationStats::new();

    run_runtime_owner_window(&mut runtime, &mut stats);

    let applicator = RemoteFreeServiceRetunePolicyApplicator::try_new(QUEUE_CAPACITY_GROWTH_FACTOR)
        .expect("policy applicator");
    let application = applicator
        .plan(
            runtime.config(),
            RemoteFreeServiceRetuneGuardDecision::Apply {
                candidate: RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier,
            },
        )
        .expect("application plan");
    assert_eq!(
        runtime.apply(application),
        Ok(RemoteFreeOwnerRuntimeApplyOutcome::Installed {
            candidate: RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier,
            previous_config: service_config(RUNTIME_INITIAL_QUEUE_CAPACITY),
            current_config: service_config(QUEUE_CAPACITY),
        })
    );
    stats.install_count = stats.install_count.saturating_add(1);

    run_runtime_owner_window(&mut runtime, &mut stats);

    assert_eq!(
        runtime.confirm(),
        Ok(RemoteFreeOwnerRuntimeConfirmOutcome {
            confirmed_config: service_config(QUEUE_CAPACITY),
            cleared_previous_config: Some(service_config(RUNTIME_INITIAL_QUEUE_CAPACITY)),
        })
    );
    stats.confirm_count = stats.confirm_count.saturating_add(1);

    run_runtime_owner_window(&mut runtime, &mut stats);

    stats.final_queue_capacity = runtime.config().queue_capacity();
    stats.final_previous_config_present = runtime.previous_config().is_some();
    stats
}

fn run_runtime_owner_window(
    runtime: &mut RemoteFreeOwnerRuntime<RuntimeTraceBlock>,
    stats: &mut RuntimeApplicationStats,
) {
    let sink = runtime.sink();

    for burst in 0..BURSTS {
        for _ in 0..BURST_BLOCKS {
            let mut block = RuntimeTraceBlock {
                submit_burst: burst,
                allocation: vec![0_u8; usize::try_from(BYTES_PER_BLOCK).expect("block size")],
            };

            loop {
                match sink.try_enqueue(block) {
                    Ok(()) => {
                        runtime.record_submit(burst, BYTES_PER_BLOCK);
                        stats.submitted_count = stats.submitted_count.saturating_add(1);
                        break;
                    }
                    Err(error) if error.kind() == RemoteFreeTryEnqueueErrorKind::Full => {
                        block = error.into_item();
                        if drain_runtime_batch(runtime, burst, stats) == 0 {
                            thread::yield_now();
                        }
                    }
                    Err(error) => panic!("runtime remote-free enqueue failed: {error}"),
                }
            }
        }

        let completed_bursts = burst.saturating_add(1);
        let runtime_status = runtime.status(completed_bursts).expect("runtime status");
        if runtime_status.decision.should_drain() {
            let drained = drain_runtime_batch(runtime, completed_bursts, stats);
            if drained > 0 {
                stats.policy_drains = stats.policy_drains.saturating_add(1);
            }
        }
    }

    while runtime.queue_stats().pending_count != 0 {
        if drain_runtime_batch(runtime, BURSTS, stats) == 0 {
            thread::yield_now();
        }
    }
}

fn drain_runtime_batch(
    runtime: &mut RemoteFreeOwnerRuntime<RuntimeTraceBlock>,
    current_burst: u64,
    stats: &mut RuntimeApplicationStats,
) -> usize {
    let drained = runtime
        .drain_batch(|block| {
            let released_bytes =
                u64::try_from(block.allocation.len()).expect("allocation length fits u64");
            let wait_bursts = current_burst.saturating_sub(block.submit_burst);

            stats.released_bytes = stats.released_bytes.saturating_add(released_bytes);
            stats.drained_count = stats.drained_count.saturating_add(1);
            stats.total_wait_bursts = stats.total_wait_bursts.saturating_add(wait_bursts);
            stats.max_wait_bursts = stats.max_wait_bursts.max(wait_bursts);

            released_bytes
        })
        .expect("runtime drain");

    if drained.drained > 0 {
        stats.drain_rounds = stats.drain_rounds.saturating_add(1);
    }

    drained.drained
}

fn assert_runtime_application_stats(stats: RuntimeApplicationStats) {
    assert_eq!(
        stats.submitted_count,
        RUNTIME_WINDOWS * BURSTS * BURST_BLOCKS
    );
    assert_eq!(stats.drained_count, stats.submitted_count);
    assert_eq!(
        stats.released_bytes,
        stats.submitted_count.saturating_mul(BYTES_PER_BLOCK)
    );
    assert_eq!(stats.policy_drains, RUNTIME_WINDOWS * 4);
    assert_eq!(stats.drain_rounds, RUNTIME_WINDOWS * 4);
    assert_eq!(stats.install_count, 1);
    assert_eq!(stats.confirm_count, 0);
    assert_eq!(stats.rollback_count, 1);
    assert_eq!(stats.max_wait_bursts, 2);
    assert_eq!(stats.mean_wait_milli(), 1_500);
    assert_eq!(stats.final_queue_capacity, RUNTIME_INITIAL_QUEUE_CAPACITY);
    assert!(!stats.final_previous_config_present);
}

fn assert_runtime_confirm_stats(stats: RuntimeApplicationStats) {
    assert_eq!(
        stats.submitted_count,
        RUNTIME_WINDOWS * BURSTS * BURST_BLOCKS
    );
    assert_eq!(stats.drained_count, stats.submitted_count);
    assert_eq!(
        stats.released_bytes,
        stats.submitted_count.saturating_mul(BYTES_PER_BLOCK)
    );
    assert_eq!(stats.policy_drains, RUNTIME_WINDOWS * 4);
    assert_eq!(stats.drain_rounds, RUNTIME_WINDOWS * 4);
    assert_eq!(stats.install_count, 1);
    assert_eq!(stats.confirm_count, 1);
    assert_eq!(stats.rollback_count, 0);
    assert_eq!(stats.max_wait_bursts, 2);
    assert_eq!(stats.mean_wait_milli(), 1_500);
    assert_eq!(stats.final_queue_capacity, QUEUE_CAPACITY);
    assert!(!stats.final_previous_config_present);
}
