#![allow(missing_docs)]

use locus_alloc::{
    RemoteFreeDrainObservation, RemoteFreeQueueStats, RemoteFreeQueuedByteDrainConfig,
    RemoteFreeQueuedByteDriftReport, RemoteFreeQueuedByteRetuneAction,
    RemoteFreeQueuedByteRetuneHint,
};

#[derive(Debug, Clone, Copy)]
struct ValidatedRetuneActionCase {
    label: &'static str,
    config: RemoteFreeQueuedByteDrainConfig,
    queue_stats: RemoteFreeQueueStats,
    observation: RemoteFreeDrainObservation,
    expected_pending_over_target: u64,
    expected_queued_bytes_over_budget: u64,
    expected_hint: RemoteFreeQueuedByteRetuneHint,
    expected_action: RemoteFreeQueuedByteRetuneAction,
}

fn stats_for_case(
    capacity: usize,
    batch_limit: usize,
    submitted_count: u64,
    pending_count: u64,
    full_count: u64,
    drained_count: u64,
) -> RemoteFreeQueueStats {
    RemoteFreeQueueStats {
        capacity,
        batch_limit,
        submitted_count,
        pending_count,
        full_count,
        disconnected_count: 0,
        drained_count,
    }
}

fn mixed_size_config(queue_capacity: usize) -> RemoteFreeQueuedByteDrainConfig {
    let sizes = [4096_u64, 4096, 8192, 4096, 16_384, 4096, 32_768, 8192];
    let repeated = sizes.iter().copied().cycle().take(64);

    RemoteFreeQueuedByteDrainConfig::from_item_sizes(queue_capacity, 64, repeated)
        .expect("mixed-size queued-byte config")
}

fn validated_uniform_retune_cases() -> [ValidatedRetuneActionCase; 4] {
    [
        ValidatedRetuneActionCase {
            label: "uniform_capacity64_backpressure",
            config: RemoteFreeQueuedByteDrainConfig::from_item_shape(64, 64, 64, 4096)
                .expect("uniform queued-byte config"),
            queue_stats: stats_for_case(64, 64, 253, 64, 3, 189),
            observation: RemoteFreeDrainObservation::new(64, 262_144, 2),
            expected_pending_over_target: 0,
            expected_queued_bytes_over_budget: 0,
            expected_hint: RemoteFreeQueuedByteRetuneHint::IncreaseQueueCapacity,
            expected_action: RemoteFreeQueuedByteRetuneAction::IncreaseQueueCapacity,
        },
        ValidatedRetuneActionCase {
            label: "uniform_capacity128_backpressure_and_drift",
            config: RemoteFreeQueuedByteDrainConfig::from_item_shape(128, 64, 64, 4096)
                .expect("uniform queued-byte config"),
            queue_stats: stats_for_case(128, 64, 254, 128, 2, 126),
            observation: RemoteFreeDrainObservation::new(128, 524_288, 4),
            expected_pending_over_target: 64,
            expected_queued_bytes_over_budget: 262_144,
            expected_hint: RemoteFreeQueuedByteRetuneHint::ReviewMultipleSignals,
            expected_action: RemoteFreeQueuedByteRetuneAction::IncreaseQueueCapacityAndDrainEarlier,
        },
        ValidatedRetuneActionCase {
            label: "uniform_capacity256_end_drain_drift",
            config: RemoteFreeQueuedByteDrainConfig::from_item_shape(256, 64, 64, 4096)
                .expect("uniform queued-byte config"),
            queue_stats: stats_for_case(256, 64, 256, 256, 0, 0),
            observation: RemoteFreeDrainObservation::new(256, 1_048_576, 8),
            expected_pending_over_target: 192,
            expected_queued_bytes_over_budget: 786_432,
            expected_hint: RemoteFreeQueuedByteRetuneHint::ReviewMultipleSignals,
            expected_action: RemoteFreeQueuedByteRetuneAction::DrainEarlier,
        },
        ValidatedRetuneActionCase {
            label: "uniform_policy_capacity256_clean_window",
            config: RemoteFreeQueuedByteDrainConfig::from_item_shape(256, 64, 64, 4096)
                .expect("uniform queued-byte config"),
            queue_stats: stats_for_case(256, 64, 256, 64, 0, 192),
            observation: RemoteFreeDrainObservation::new(64, 262_144, 2),
            expected_pending_over_target: 0,
            expected_queued_bytes_over_budget: 0,
            expected_hint: RemoteFreeQueuedByteRetuneHint::KeepConfig,
            expected_action: RemoteFreeQueuedByteRetuneAction::KeepConfig,
        },
    ]
}

fn validated_mixed_size_retune_cases() -> [ValidatedRetuneActionCase; 3] {
    [
        ValidatedRetuneActionCase {
            label: "mixed_capacity128_backpressure_and_drift",
            config: mixed_size_config(128),
            queue_stats: stats_for_case(128, 64, 254, 128, 2, 126),
            observation: RemoteFreeDrainObservation::new(128, 1_310_720, 4),
            expected_pending_over_target: 64,
            expected_queued_bytes_over_budget: 655_360,
            expected_hint: RemoteFreeQueuedByteRetuneHint::ReviewMultipleSignals,
            expected_action: RemoteFreeQueuedByteRetuneAction::IncreaseQueueCapacityAndDrainEarlier,
        },
        ValidatedRetuneActionCase {
            label: "mixed_capacity256_end_drain_drift",
            config: mixed_size_config(256),
            queue_stats: stats_for_case(256, 64, 256, 256, 0, 0),
            observation: RemoteFreeDrainObservation::new(256, 2_621_440, 8),
            expected_pending_over_target: 192,
            expected_queued_bytes_over_budget: 1_966_080,
            expected_hint: RemoteFreeQueuedByteRetuneHint::ReviewMultipleSignals,
            expected_action: RemoteFreeQueuedByteRetuneAction::DrainEarlier,
        },
        ValidatedRetuneActionCase {
            label: "mixed_policy_capacity256_clean_window",
            config: mixed_size_config(256),
            queue_stats: stats_for_case(256, 64, 256, 64, 0, 192),
            observation: RemoteFreeDrainObservation::new(64, 655_360, 2),
            expected_pending_over_target: 0,
            expected_queued_bytes_over_budget: 0,
            expected_hint: RemoteFreeQueuedByteRetuneHint::KeepConfig,
            expected_action: RemoteFreeQueuedByteRetuneAction::KeepConfig,
        },
    ]
}

fn validated_domain_retune_cases() -> [ValidatedRetuneActionCase; 5] {
    [
        ValidatedRetuneActionCase {
            label: "owner_loop_clean_window",
            config: RemoteFreeQueuedByteDrainConfig::from_grouped_item_shape(
                128, 64, 4, 16, 10_240,
            )
            .expect("owner-loop queued-byte config"),
            queue_stats: stats_for_case(128, 64, 256, 64, 0, 192),
            observation: RemoteFreeDrainObservation::new(64, 655_360, 2),
            expected_pending_over_target: 0,
            expected_queued_bytes_over_budget: 0,
            expected_hint: RemoteFreeQueuedByteRetuneHint::KeepConfig,
            expected_action: RemoteFreeQueuedByteRetuneAction::KeepConfig,
        },
        ValidatedRetuneActionCase {
            label: "kv_end_drain_drift",
            config: RemoteFreeQueuedByteDrainConfig::from_item_shape(256, 64, 64, 4096)
                .expect("kv queued-byte config"),
            queue_stats: stats_for_case(256, 64, 256, 256, 0, 0),
            observation: RemoteFreeDrainObservation::new(256, 1_048_576, 8),
            expected_pending_over_target: 192,
            expected_queued_bytes_over_budget: 786_432,
            expected_hint: RemoteFreeQueuedByteRetuneHint::ReviewMultipleSignals,
            expected_action: RemoteFreeQueuedByteRetuneAction::DrainEarlier,
        },
        ValidatedRetuneActionCase {
            label: "kv_policy_clean_window",
            config: RemoteFreeQueuedByteDrainConfig::from_item_shape(256, 64, 64, 4096)
                .expect("kv queued-byte config"),
            queue_stats: stats_for_case(256, 64, 256, 64, 0, 192),
            observation: RemoteFreeDrainObservation::new(64, 262_144, 2),
            expected_pending_over_target: 0,
            expected_queued_bytes_over_budget: 0,
            expected_hint: RemoteFreeQueuedByteRetuneHint::KeepConfig,
            expected_action: RemoteFreeQueuedByteRetuneAction::KeepConfig,
        },
        ValidatedRetuneActionCase {
            label: "request_end_drain_drift",
            config: RemoteFreeQueuedByteDrainConfig::from_item_shape(16, 8, 8, 32_768)
                .expect("request queued-byte config"),
            queue_stats: stats_for_case(16, 8, 16, 16, 0, 0),
            observation: RemoteFreeDrainObservation::new(16, 524_288, 4),
            expected_pending_over_target: 8,
            expected_queued_bytes_over_budget: 262_144,
            expected_hint: RemoteFreeQueuedByteRetuneHint::ReviewMultipleSignals,
            expected_action: RemoteFreeQueuedByteRetuneAction::DrainEarlier,
        },
        ValidatedRetuneActionCase {
            label: "request_policy_clean_window",
            config: RemoteFreeQueuedByteDrainConfig::from_item_shape(16, 8, 8, 32_768)
                .expect("request queued-byte config"),
            queue_stats: stats_for_case(16, 8, 16, 8, 0, 8),
            observation: RemoteFreeDrainObservation::new(8, 262_144, 2),
            expected_pending_over_target: 0,
            expected_queued_bytes_over_budget: 0,
            expected_hint: RemoteFreeQueuedByteRetuneHint::KeepConfig,
            expected_action: RemoteFreeQueuedByteRetuneAction::KeepConfig,
        },
    ]
}

fn assert_validated_retune_action(case: ValidatedRetuneActionCase) {
    let report = RemoteFreeQueuedByteDriftReport::from_observation(
        case.config,
        case.queue_stats,
        case.observation,
    );

    assert_eq!(
        report.pending_items_over_target(),
        case.expected_pending_over_target,
        "{} pending drift",
        case.label
    );
    assert_eq!(
        report.queued_bytes_over_budget(),
        case.expected_queued_bytes_over_budget,
        "{} queued-byte drift",
        case.label
    );
    assert_eq!(
        report.retune_hint(),
        case.expected_hint,
        "{} retune hint",
        case.label
    );
    assert_eq!(
        report.retune_action(),
        case.expected_action,
        "{} retune action",
        case.label
    );
}

#[test]
fn queued_byte_retune_action_matches_validated_surface_matrix() {
    for case in validated_uniform_retune_cases()
        .into_iter()
        .chain(validated_mixed_size_retune_cases())
        .chain(validated_domain_retune_cases())
    {
        assert_validated_retune_action(case);
    }
}
