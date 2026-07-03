use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use super::{
    RemoteFreeOwnerRuntime, RemoteFreeServiceRetuneSummary, RemoteFreeServiceRuntimeOwnerId,
    RemoteFreeServiceRuntimeRetuneOwners, RemoteFreeServiceRuntimeWindowCollectionError,
    RemoteFreeServiceRuntimeWindowStats, RemoteFreeSink,
};

/// Ordered set of owner runtimes that should be collected in a service window.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RemoteFreeServiceRuntimeDirtyOwners {
    pending: Vec<RemoteFreeServiceRuntimeOwnerId>,
    seen: BTreeSet<RemoteFreeServiceRuntimeOwnerId>,
}

/// Shared dirty-owner tracker for remote enqueue handles.
#[derive(Debug, Clone, Default)]
pub struct RemoteFreeServiceRuntimeDirtyOwnerTracker {
    inner: Arc<Mutex<DirtyOwnerTrackerState>>,
}

/// Snapshot of dirty owners captured for one service collection pass.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RemoteFreeServiceRuntimeDirtyOwnerSnapshot {
    entries: Vec<DirtyOwnerSnapshotEntry>,
}

/// Cloneable remote-free sink that marks its owner dirty after successful enqueue.
#[derive(Debug, Clone)]
pub struct RemoteFreeServiceRuntimeDirtySink<T> {
    owner_id: RemoteFreeServiceRuntimeOwnerId,
    sink: RemoteFreeSink<T>,
    tracker: RemoteFreeServiceRuntimeDirtyOwnerTracker,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DirtyOwnerSnapshotEntry {
    owner_id: RemoteFreeServiceRuntimeOwnerId,
    generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct DirtyOwnerTrackerState {
    pending: Vec<DirtyOwnerTrackerEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DirtyOwnerTrackerEntry {
    owner_id: RemoteFreeServiceRuntimeOwnerId,
    generation: u64,
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

impl RemoteFreeServiceRuntimeDirtyOwnerTracker {
    /// Creates an empty shared dirty-owner tracker.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(DirtyOwnerTrackerState::new())),
        }
    }

    /// Marks an owner dirty and returns true when it was not already pending.
    #[must_use]
    pub fn mark_dirty(&self, owner_id: RemoteFreeServiceRuntimeOwnerId) -> bool {
        self.with_state(|state| state.mark_dirty(owner_id))
    }

    /// Returns the number of unique pending dirty owners.
    #[must_use]
    pub fn len(&self) -> usize {
        self.with_state(|state| state.len())
    }

    /// Returns true when no owners are marked dirty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.with_state(|state| state.is_empty())
    }

    /// Captures the current dirty owners with their generations.
    #[must_use]
    pub fn snapshot(&self) -> RemoteFreeServiceRuntimeDirtyOwnerSnapshot {
        self.with_state(|state| state.snapshot())
    }

    /// Clears only the dirty generations represented by `snapshot`.
    pub fn clear_snapshot(&self, snapshot: &RemoteFreeServiceRuntimeDirtyOwnerSnapshot) {
        self.with_state(|state| state.clear_snapshot(snapshot));
    }

    /// Returns marked owners in first-marked order.
    #[must_use]
    pub fn owner_ids(&self) -> Vec<RemoteFreeServiceRuntimeOwnerId> {
        self.with_state(|state| state.owner_ids())
    }

    /// Wraps a remote-free sink so successful enqueue attempts mark `owner_id`.
    #[must_use]
    pub fn dirty_sink<T>(
        &self,
        owner_id: RemoteFreeServiceRuntimeOwnerId,
        sink: RemoteFreeSink<T>,
    ) -> RemoteFreeServiceRuntimeDirtySink<T> {
        RemoteFreeServiceRuntimeDirtySink {
            owner_id,
            sink,
            tracker: self.clone(),
        }
    }

    fn with_state<R>(&self, update: impl FnOnce(&mut DirtyOwnerTrackerState) -> R) -> R {
        let mut state = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        update(&mut state)
    }
}

impl RemoteFreeServiceRuntimeDirtyOwnerSnapshot {
    /// Returns the dirty owner IDs captured in first-marked order.
    pub fn owner_ids(&self) -> impl Iterator<Item = RemoteFreeServiceRuntimeOwnerId> + '_ {
        self.entries.iter().map(|entry| entry.owner_id)
    }

    /// Returns the number of unique dirty owners in this snapshot.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true when no owners were captured.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl<T> RemoteFreeServiceRuntimeDirtySink<T> {
    /// Returns the owner ID marked by this sink.
    #[must_use]
    pub const fn owner_id(&self) -> RemoteFreeServiceRuntimeOwnerId {
        self.owner_id
    }

    /// Returns the underlying remote-free sink.
    #[must_use]
    pub const fn sink(&self) -> &RemoteFreeSink<T> {
        &self.sink
    }

    /// Enqueues one item and marks the owner dirty after success.
    ///
    /// # Errors
    ///
    /// Returns the item when the owning queue has been dropped.
    pub fn enqueue(&self, item: T) -> Result<(), super::RemoteFreeEnqueueError<T>> {
        self.sink.enqueue(item)?;
        let _ = self.tracker.mark_dirty(self.owner_id);
        Ok(())
    }

    /// Attempts to enqueue one item and marks the owner dirty after success.
    ///
    /// # Errors
    ///
    /// Returns the item and a failure kind when the bounded queue is full or the
    /// owning queue has been dropped.
    pub fn try_enqueue(&self, item: T) -> Result<(), super::RemoteFreeTryEnqueueError<T>> {
        self.sink.try_enqueue(item)?;
        let _ = self.tracker.mark_dirty(self.owner_id);
        Ok(())
    }
}

impl DirtyOwnerTrackerState {
    fn new() -> Self {
        Self {
            pending: Vec::new(),
        }
    }

    fn mark_dirty(&mut self, owner_id: RemoteFreeServiceRuntimeOwnerId) -> bool {
        if let Some(entry) = self
            .pending
            .iter_mut()
            .find(|entry| entry.owner_id == owner_id)
        {
            entry.generation = entry.generation.saturating_add(1);
            false
        } else {
            self.pending.push(DirtyOwnerTrackerEntry {
                owner_id,
                generation: 1,
            });
            true
        }
    }

    fn len(&self) -> usize {
        self.pending.len()
    }

    fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    fn owner_ids(&self) -> Vec<RemoteFreeServiceRuntimeOwnerId> {
        self.pending.iter().map(|entry| entry.owner_id).collect()
    }

    fn snapshot(&self) -> RemoteFreeServiceRuntimeDirtyOwnerSnapshot {
        RemoteFreeServiceRuntimeDirtyOwnerSnapshot {
            entries: self
                .pending
                .iter()
                .map(|entry| DirtyOwnerSnapshotEntry {
                    owner_id: entry.owner_id,
                    generation: entry.generation,
                })
                .collect(),
        }
    }

    fn clear_snapshot(&mut self, snapshot: &RemoteFreeServiceRuntimeDirtyOwnerSnapshot) {
        self.pending.retain(|entry| {
            let Some(snapshot_entry) = snapshot
                .entries
                .iter()
                .find(|snapshot_entry| snapshot_entry.owner_id == entry.owner_id)
            else {
                return true;
            };

            entry.generation > snapshot_entry.generation
        });
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

    /// Collects and processes owners captured from a shared dirty-owner tracker.
    ///
    /// The tracker is snapshotted before collection. A successful collection
    /// clears only the generations included in that snapshot, preserving newer
    /// marks that arrive while collection is running. Failed collection leaves
    /// the tracker unchanged.
    ///
    /// # Errors
    ///
    /// Returns an error with owner ID context if an owner is missing, summary
    /// collection fails, or routing the collected summary fails.
    pub fn collect_tracked_dirty_service_window<E>(
        &mut self,
        tracker: &RemoteFreeServiceRuntimeDirtyOwnerTracker,
        collect: impl FnMut(
            RemoteFreeServiceRuntimeOwnerId,
            &mut RemoteFreeOwnerRuntime<T>,
        ) -> Result<RemoteFreeServiceRetuneSummary, E>,
    ) -> Result<RemoteFreeServiceRuntimeWindowStats, RemoteFreeServiceRuntimeWindowCollectionError<E>>
    {
        let snapshot = tracker.snapshot();
        let stats = self.collect_service_window(snapshot.owner_ids(), collect)?;
        tracker.clear_snapshot(&snapshot);
        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DirtyOwnerTrackerState, RemoteFreeServiceRuntimeDirtyOwnerTracker,
        RemoteFreeServiceRuntimeDirtyOwners,
    };
    use crate::{
        RemoteFreeDrainObservation, RemoteFreeOwnerRuntime, RemoteFreeQueue, RemoteFreeQueueStats,
        RemoteFreeQueuedByteDrainConfig, RemoteFreeQueuedByteDriftReport,
        RemoteFreeServiceRetuneCandidate, RemoteFreeServiceRetuneGuard,
        RemoteFreeServiceRetunePolicyApplicator, RemoteFreeServiceRetuneSummary,
        RemoteFreeServiceRuntimeOwnerId, RemoteFreeServiceRuntimeRetuneCoordinator,
        RemoteFreeServiceRuntimeRetuneOwners, RemoteFreeServiceRuntimeWindowCollectionError,
        RemoteFreeTryEnqueueErrorKind,
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

    #[test]
    fn dirty_tracker_clears_only_snapshot_generations() {
        let first = RemoteFreeServiceRuntimeOwnerId::new(1);
        let second = RemoteFreeServiceRuntimeOwnerId::new(2);
        let mut tracker = DirtyOwnerTrackerState::new();

        assert!(tracker.mark_dirty(first));
        assert!(tracker.mark_dirty(second));
        let snapshot = tracker.snapshot();

        assert!(!tracker.mark_dirty(first));
        tracker.clear_snapshot(&snapshot);

        assert_eq!(tracker.owner_ids(), vec![first]);
    }

    #[test]
    fn dirty_sink_marks_owner_only_after_successful_enqueue() {
        let owner_id = RemoteFreeServiceRuntimeOwnerId::new(3);
        let tracker = RemoteFreeServiceRuntimeDirtyOwnerTracker::new();
        let mut queue = RemoteFreeQueue::new(1, 1).expect("queue");
        let sink = tracker.dirty_sink(owner_id, queue.sink());

        sink.try_enqueue(1).expect("first enqueue");
        let full = sink.try_enqueue(2).expect_err("queue full");

        assert_eq!(full.kind(), RemoteFreeTryEnqueueErrorKind::Full);
        assert_eq!(tracker.owner_ids(), vec![owner_id]);

        let mut drained = Vec::new();
        let _ = queue.drain_batch(|item| drained.push(item));
        sink.try_enqueue(3).expect("second enqueue");

        assert_eq!(drained, vec![1]);
        assert_eq!(tracker.owner_ids(), vec![owner_id]);
        let snapshot = tracker.snapshot();
        assert_eq!(snapshot.len(), 1);
        tracker.clear_snapshot(&snapshot);
        assert!(tracker.is_empty());
    }

    #[test]
    fn tracked_dirty_collection_preserves_new_marks_after_snapshot() {
        let mut owners = owners(2);
        let owner_id = owners
            .register_owner(RemoteFreeOwnerRuntime::<usize>::new(config(256)).expect("runtime"));
        let tracker = RemoteFreeServiceRuntimeDirtyOwnerTracker::new();
        let _ = tracker.mark_dirty(owner_id);

        let stats = owners
            .collect_tracked_dirty_service_window(&tracker, |_, runtime| {
                let _ = tracker.mark_dirty(owner_id);
                Ok::<_, &'static str>(summary(report(runtime.config(), 96, 524_288, 0)))
            })
            .expect("tracked dirty collection");

        assert_eq!(stats.owner_observations(), 1);
        assert_eq!(stats.hold_decisions(), 1);
        assert_eq!(tracker.owner_ids(), vec![owner_id]);
    }

    #[test]
    fn tracked_dirty_collection_keeps_marks_after_error() {
        let mut owners = owners(2);
        let owner_id = owners
            .register_owner(RemoteFreeOwnerRuntime::<usize>::new(config(256)).expect("runtime"));
        let tracker = RemoteFreeServiceRuntimeDirtyOwnerTracker::new();
        let _ = tracker.mark_dirty(owner_id);

        assert_eq!(
            owners.collect_tracked_dirty_service_window(&tracker, |_, _| {
                Err::<RemoteFreeServiceRetuneSummary, _>("collection failed")
            }),
            Err(RemoteFreeServiceRuntimeWindowCollectionError::Collect {
                owner_id,
                source: "collection failed",
            })
        );
        assert_eq!(tracker.owner_ids(), vec![owner_id]);
    }

    #[test]
    fn tracked_dirty_collection_can_apply_with_shared_guard() {
        let mut owners = owners(2);
        let owner_id = owners
            .register_owner(RemoteFreeOwnerRuntime::<usize>::new(config(256)).expect("runtime"));
        let tracker = RemoteFreeServiceRuntimeDirtyOwnerTracker::new();

        let _ = tracker.mark_dirty(owner_id);
        owners
            .collect_tracked_dirty_service_window(&tracker, |_, runtime| {
                Ok::<_, &'static str>(summary(report(runtime.config(), 96, 524_288, 0)))
            })
            .expect("hold");

        let _ = tracker.mark_dirty(owner_id);
        let stats = owners
            .collect_tracked_dirty_service_window(&tracker, |_, runtime| {
                Ok::<_, &'static str>(summary(report(runtime.config(), 96, 524_288, 0)))
            })
            .expect("apply");

        assert_eq!(stats.apply_decisions(), 1);
        assert_eq!(owners.coordinator().guard().applied_mutations(), 1);
        assert_eq!(
            owners.coordinator().guard().pending_validation(),
            Some(RemoteFreeServiceRetuneCandidate::DrainEarlier)
        );
    }
}
