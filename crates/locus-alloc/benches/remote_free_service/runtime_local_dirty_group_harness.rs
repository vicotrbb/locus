#![allow(missing_docs)]

use std::convert::Infallible;

use locus_alloc::{
    RemoteFreeServiceRetuneSummary, RemoteFreeServiceRuntimeDirtyOwnerFlushStats,
    RemoteFreeServiceRuntimeDirtyOwnerLocalBuffers, RemoteFreeServiceRuntimeOwnerId,
    RemoteFreeServiceRuntimeRetuneOwners, RemoteFreeServiceRuntimeWindowStats,
};

use crate::remote_free_service_application_harness::{
    run_runtime_owner_window_with_bounded_local_dirty_group_summary_and_block_bytes,
    run_runtime_owner_window_with_local_dirty_group_summary_and_block_bytes,
    run_runtime_owner_window_with_validated_local_dirty_group_summary_and_block_bytes,
    RuntimeApplicationStats, RuntimeLocalDirtyFlushStats, RuntimeTraceBlock,
};
use crate::remote_free_service_harness::{BURSTS, BURST_BLOCKS};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeLocalDirtyGroupCollectionMode {
    Manual,
    Integrated,
    Bounded,
    Validated,
}

pub(crate) fn collect_runtime_local_dirty_group_window(
    mode: RuntimeLocalDirtyGroupCollectionMode,
    owners: &mut RemoteFreeServiceRuntimeRetuneOwners<RuntimeTraceBlock>,
    owner_id: RemoteFreeServiceRuntimeOwnerId,
    stats: &mut RuntimeApplicationStats,
    block_bytes: u64,
    buffer_group: &mut RemoteFreeServiceRuntimeDirtyOwnerLocalBuffers,
    flush_stats: &mut RuntimeLocalDirtyFlushStats,
) -> RemoteFreeServiceRuntimeWindowStats {
    let previous_capacity = buffer_group.local_buffer_capacity(owner_id);
    let summary = match mode {
        RuntimeLocalDirtyGroupCollectionMode::Manual
        | RuntimeLocalDirtyGroupCollectionMode::Integrated => {
            run_runtime_owner_window_with_local_dirty_group_summary_and_block_bytes(
                owners
                    .owner_mut(owner_id)
                    .expect("owner for local dirty buffer group"),
                stats,
                block_bytes,
                owner_id,
                buffer_group,
            )
        }
        RuntimeLocalDirtyGroupCollectionMode::Bounded => {
            let owner_limit = owners.len();
            run_runtime_owner_window_with_bounded_local_dirty_group_summary_and_block_bytes(
                owners
                    .owner_mut(owner_id)
                    .expect("owner for bounded local dirty buffer group"),
                stats,
                block_bytes,
                owner_id,
                owner_limit,
                buffer_group,
            )
        }
        RuntimeLocalDirtyGroupCollectionMode::Validated => {
            let owner = owners
                .validate_local_dirty_owner(owner_id)
                .expect("validated local dirty owner");
            run_runtime_owner_window_with_validated_local_dirty_group_summary_and_block_bytes(
                owners
                    .owner_mut(owner_id)
                    .expect("owner for validated local dirty buffer group"),
                stats,
                block_bytes,
                owner,
                buffer_group,
            )
        }
    };

    let expected_duplicate_marks = expected_window_duplicate_marks();
    assert_pending_local_buffer(buffer_group, owner_id, expected_duplicate_marks);

    match mode {
        RuntimeLocalDirtyGroupCollectionMode::Manual => {
            let flush = buffer_group.flush_owner(owner_id);
            flush_stats.observe(flush);
            assert_local_group_flush(flush, expected_duplicate_marks);
            assert_capacity_reused(buffer_group, owner_id, previous_capacity);
            assert_eq!(buffer_group.tracker().owner_ids(), vec![owner_id]);

            let stats = owners
                .collect_tracked_dirty_service_window(buffer_group.tracker(), |_, _| {
                    Ok::<_, Infallible>(summary)
                })
                .expect("local dirty buffer group service window stats");
            assert!(buffer_group.tracker().is_empty());
            stats
        }
        RuntimeLocalDirtyGroupCollectionMode::Integrated
        | RuntimeLocalDirtyGroupCollectionMode::Bounded
        | RuntimeLocalDirtyGroupCollectionMode::Validated => {
            let collection = owners
                .collect_local_dirty_service_window(buffer_group, owner_id, |_, _| {
                    Ok::<_, Infallible>(summary)
                })
                .unwrap_or_else(|_| panic!("{}", mode.collection_expect_message()));
            let flush = collection.flush_stats();
            flush_stats.observe(flush);
            assert_local_group_flush(flush, expected_duplicate_marks);
            assert_capacity_reused(buffer_group, owner_id, previous_capacity);
            assert!(buffer_group.tracker().is_empty());
            collection.window_stats()
        }
    }
}

pub(crate) fn assert_integrated_missing_owner(
    owners: &mut RemoteFreeServiceRuntimeRetuneOwners<RuntimeTraceBlock>,
    missing: RemoteFreeServiceRuntimeOwnerId,
) {
    let mut buffer_group = RemoteFreeServiceRuntimeDirtyOwnerLocalBuffers::new();
    let error = owners
        .collect_local_dirty_service_window(&mut buffer_group, missing, |_, _| {
            Ok::<_, Infallible>(RemoteFreeServiceRetuneSummary::new())
        })
        .expect_err("missing owner");
    assert_eq!(error.owner_id(), missing);
    assert!(buffer_group.tracker().is_empty());
    assert!(buffer_group.local_buffer(missing).is_none());
}

pub(crate) fn assert_bounded_missing_owner(
    owners: &mut RemoteFreeServiceRuntimeRetuneOwners<RuntimeTraceBlock>,
    missing: RemoteFreeServiceRuntimeOwnerId,
) {
    let mut buffer_group = RemoteFreeServiceRuntimeDirtyOwnerLocalBuffers::new();
    let owner_limit = owners.len();
    let mark_error = buffer_group
        .try_mark_dirty(missing, owner_limit)
        .expect_err("bounded mark rejects missing owner");
    assert_eq!(mark_error.owner_id(), missing);
    assert_eq!(mark_error.owner_limit(), owner_limit);
    assert!(buffer_group.local_buffer(missing).is_none());

    let error = owners
        .collect_local_dirty_service_window(&mut buffer_group, missing, |_, _| {
            Ok::<_, Infallible>(RemoteFreeServiceRetuneSummary::new())
        })
        .expect_err("missing owner");
    assert_eq!(error.owner_id(), missing);
    assert!(buffer_group.tracker().is_empty());
    assert!(buffer_group.local_buffer(missing).is_none());
}

pub(crate) fn assert_validated_missing_owner(
    owners: &mut RemoteFreeServiceRuntimeRetuneOwners<RuntimeTraceBlock>,
    missing: RemoteFreeServiceRuntimeOwnerId,
) {
    let buffer_group = RemoteFreeServiceRuntimeDirtyOwnerLocalBuffers::new();
    let error = owners
        .validate_local_dirty_owner(missing)
        .expect_err("validated owner rejects missing owner");
    assert_eq!(error.owner_id(), missing);
    assert_eq!(error.owner_limit(), owners.len());
    assert!(buffer_group.local_buffer(missing).is_none());
}

fn expected_window_duplicate_marks() -> u64 {
    BURSTS.saturating_mul(BURST_BLOCKS).saturating_sub(1)
}

impl RuntimeLocalDirtyGroupCollectionMode {
    fn collection_expect_message(self) -> &'static str {
        match self {
            Self::Manual => "manual local dirty buffer group service window stats",
            Self::Integrated => "integrated local dirty buffer group service window stats",
            Self::Bounded => "bounded local dirty buffer group service window stats",
            Self::Validated => "validated local dirty buffer group service window stats",
        }
    }
}

fn assert_pending_local_buffer(
    buffer_group: &RemoteFreeServiceRuntimeDirtyOwnerLocalBuffers,
    owner_id: RemoteFreeServiceRuntimeOwnerId,
    expected_duplicate_marks: u64,
) {
    let buffer = buffer_group
        .local_buffer(owner_id)
        .expect("local dirty group buffer");
    assert_eq!(buffer.owner_ids(), &[owner_id]);
    assert_eq!(buffer.duplicate_marks(), expected_duplicate_marks);
}

fn assert_local_group_flush(
    flush: RemoteFreeServiceRuntimeDirtyOwnerFlushStats,
    expected_duplicate_marks: u64,
) {
    assert_eq!(flush.owner_count, 1);
    assert_eq!(flush.new_tracker_marks, 1);
    assert_eq!(flush.duplicate_local_marks, expected_duplicate_marks);
}

fn assert_capacity_reused(
    buffer_group: &RemoteFreeServiceRuntimeDirtyOwnerLocalBuffers,
    owner_id: RemoteFreeServiceRuntimeOwnerId,
    previous_capacity: usize,
) {
    assert!(buffer_group.local_buffer_capacity(owner_id) >= 1);
    assert!(buffer_group.local_buffer_capacity(owner_id) >= previous_capacity);
}
