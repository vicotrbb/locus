use std::collections::BTreeSet;

use super::{
    RemoteFreeOwnerRuntime, RemoteFreeServiceRetuneSummary, RemoteFreeServiceRuntimeOwnerId,
    RemoteFreeServiceRuntimeRetuneOwners, RemoteFreeServiceRuntimeWindowCollectionError,
    RemoteFreeServiceRuntimeWindowStats,
};

/// Ordered set of owner runtimes that should be collected in a service window.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RemoteFreeServiceRuntimeDirtyOwners {
    pending: Vec<RemoteFreeServiceRuntimeOwnerId>,
    seen: BTreeSet<RemoteFreeServiceRuntimeOwnerId>,
}

impl RemoteFreeServiceRuntimeDirtyOwners {
    /// Creates an empty dirty-owner set.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            pending: Vec::new(),
            seen: BTreeSet::new(),
        }
    }

    /// Marks an owner dirty and returns true when this is a new pending mark.
    pub fn mark_dirty(&mut self, owner_id: RemoteFreeServiceRuntimeOwnerId) -> bool {
        if self.seen.insert(owner_id) {
            self.pending.push(owner_id);
            true
        } else {
            false
        }
    }

    /// Returns true when the owner is already marked dirty.
    #[must_use]
    pub fn contains(&self, owner_id: RemoteFreeServiceRuntimeOwnerId) -> bool {
        self.seen.contains(&owner_id)
    }

    /// Returns the number of unique dirty owners.
    #[must_use]
    pub fn len(&self) -> usize {
        self.pending.len()
    }

    /// Returns true when no owners are marked dirty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    /// Returns marked owners in first-marked order.
    #[must_use]
    pub fn owner_ids(&self) -> &[RemoteFreeServiceRuntimeOwnerId] {
        &self.pending
    }

    /// Clears all dirty owner marks.
    pub fn clear(&mut self) {
        self.pending.clear();
        self.seen.clear();
    }
}

impl<T> RemoteFreeServiceRuntimeRetuneOwners<T> {
    /// Collects and processes all currently marked dirty owners.
    ///
    /// Dirty marks are cleared only after a successful collection window. If an
    /// owner is missing, summary collection fails, or window routing fails, the
    /// dirty set is left intact so callers can retry or inspect it.
    ///
    /// # Errors
    ///
    /// Returns an error with owner ID context if an owner is missing, summary
    /// collection fails, or routing the collected summary fails.
    pub fn collect_dirty_service_window<E>(
        &mut self,
        dirty_owners: &mut RemoteFreeServiceRuntimeDirtyOwners,
        collect: impl FnMut(
            RemoteFreeServiceRuntimeOwnerId,
            &mut RemoteFreeOwnerRuntime<T>,
        ) -> Result<RemoteFreeServiceRetuneSummary, E>,
    ) -> Result<RemoteFreeServiceRuntimeWindowStats, RemoteFreeServiceRuntimeWindowCollectionError<E>>
    {
        let stats =
            self.collect_service_window(dirty_owners.owner_ids().iter().copied(), collect)?;
        dirty_owners.clear();
        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::RemoteFreeServiceRuntimeDirtyOwners;
    use crate::{
        RemoteFreeDrainObservation, RemoteFreeOwnerRuntime, RemoteFreeQueueStats,
        RemoteFreeQueuedByteDrainConfig, RemoteFreeQueuedByteDriftReport,
        RemoteFreeServiceRetuneGuard, RemoteFreeServiceRetunePolicyApplicator,
        RemoteFreeServiceRetuneSummary, RemoteFreeServiceRuntimeOwnerId,
        RemoteFreeServiceRuntimeRetuneCoordinator, RemoteFreeServiceRuntimeRetuneOwners,
        RemoteFreeServiceRuntimeWindowCollectionError,
    };

    fn config(queue_capacity: usize) -> RemoteFreeQueuedByteDrainConfig {
        RemoteFreeQueuedByteDrainConfig::from_item_shape(queue_capacity, 64, 64, 4096)
            .expect("config")
    }

    fn owners(max_mutations: u64) -> RemoteFreeServiceRuntimeRetuneOwners<usize> {
        RemoteFreeServiceRuntimeRetuneOwners::new(RemoteFreeServiceRuntimeRetuneCoordinator::new(
            RemoteFreeServiceRetuneGuard::try_new(2, max_mutations).expect("guard"),
            RemoteFreeServiceRetunePolicyApplicator::try_new(2).expect("applicator"),
        ))
    }

    fn summary(report: RemoteFreeQueuedByteDriftReport) -> RemoteFreeServiceRetuneSummary {
        let mut summary = RemoteFreeServiceRetuneSummary::new();
        summary.observe_report(report);
        summary
    }

    fn report(
        config: RemoteFreeQueuedByteDrainConfig,
        pending_count: u64,
        queued_bytes: u64,
        full_count: u64,
    ) -> RemoteFreeQueuedByteDriftReport {
        RemoteFreeQueuedByteDriftReport::from_observation(
            config,
            RemoteFreeQueueStats {
                capacity: config.queue_capacity(),
                batch_limit: config.drain_batch_limit(),
                submitted_count: pending_count,
                pending_count,
                full_count,
                disconnected_count: 0,
                drained_count: 0,
            },
            RemoteFreeDrainObservation::new(pending_count, queued_bytes, 1),
        )
    }

    #[test]
    fn dirty_owner_set_deduplicates_in_first_marked_order() {
        let first = RemoteFreeServiceRuntimeOwnerId::new(2);
        let second = RemoteFreeServiceRuntimeOwnerId::new(1);
        let mut dirty = RemoteFreeServiceRuntimeDirtyOwners::new();

        assert!(dirty.is_empty());
        assert!(dirty.mark_dirty(first));
        assert!(dirty.mark_dirty(second));
        assert!(!dirty.mark_dirty(first));

        assert_eq!(dirty.len(), 2);
        assert!(dirty.contains(first));
        assert_eq!(dirty.owner_ids(), &[first, second]);

        dirty.clear();
        assert!(dirty.is_empty());
        assert!(!dirty.contains(first));
    }

    #[test]
    fn dirty_collection_clears_marks_after_success() {
        let mut owners = owners(2);
        let owner_id = owners
            .register_owner(RemoteFreeOwnerRuntime::<usize>::new(config(256)).expect("runtime"));
        let mut dirty = RemoteFreeServiceRuntimeDirtyOwners::new();
        dirty.mark_dirty(owner_id);
        dirty.mark_dirty(owner_id);

        let stats = owners
            .collect_dirty_service_window(&mut dirty, |_, runtime| {
                Ok::<_, &'static str>(summary(report(runtime.config(), 96, 524_288, 0)))
            })
            .expect("dirty collection");

        assert_eq!(stats.owner_observations(), 1);
        assert_eq!(stats.hold_decisions(), 1);
        assert!(dirty.is_empty());
    }

    #[test]
    fn dirty_collection_keeps_marks_after_missing_owner() {
        let mut owners = owners(2);
        let missing = RemoteFreeServiceRuntimeOwnerId::new(7);
        let mut dirty = RemoteFreeServiceRuntimeDirtyOwners::new();
        dirty.mark_dirty(missing);

        assert_eq!(
            owners.collect_dirty_service_window(&mut dirty, |_, runtime| {
                Ok::<_, &'static str>(summary(report(runtime.config(), 96, 524_288, 0)))
            }),
            Err(RemoteFreeServiceRuntimeWindowCollectionError::MissingOwner { owner_id: missing })
        );
        assert_eq!(dirty.owner_ids(), &[missing]);
    }

    #[test]
    fn dirty_collection_keeps_marks_after_collector_error() {
        let mut owners = owners(2);
        let owner_id = owners
            .register_owner(RemoteFreeOwnerRuntime::<usize>::new(config(256)).expect("runtime"));
        let mut dirty = RemoteFreeServiceRuntimeDirtyOwners::new();
        dirty.mark_dirty(owner_id);

        assert_eq!(
            owners.collect_dirty_service_window(&mut dirty, |_, _| {
                Err::<RemoteFreeServiceRetuneSummary, _>("collection failed")
            }),
            Err(RemoteFreeServiceRuntimeWindowCollectionError::Collect {
                owner_id,
                source: "collection failed",
            })
        );
        assert_eq!(dirty.owner_ids(), &[owner_id]);
    }
}
