use super::{RemoteFreeServiceRuntimeDirtyOwnerTracker, RemoteFreeServiceRuntimeOwnerId};

/// Per-worker dirty-owner mark buffer for batching tracker updates.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RemoteFreeServiceRuntimeDirtyOwnerLocalBuffer {
    pending: Vec<RemoteFreeServiceRuntimeOwnerId>,
    duplicate_marks: u64,
}

/// Result of flushing local dirty-owner marks into the shared tracker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RemoteFreeServiceRuntimeDirtyOwnerFlushStats {
    /// Unique owner marks flushed from the local buffer.
    pub owner_count: usize,
    /// Flushed owners that were not already pending in the shared tracker.
    pub new_tracker_marks: u64,
    /// Repeated local marks deduplicated before the flush.
    pub duplicate_local_marks: u64,
}

impl RemoteFreeServiceRuntimeDirtyOwnerLocalBuffer {
    /// Creates an empty local dirty-owner buffer.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            pending: Vec::new(),
            duplicate_marks: 0,
        }
    }

    /// Marks an owner dirty locally and returns true when it was not already pending.
    #[must_use]
    pub fn mark_dirty(&mut self, owner_id: RemoteFreeServiceRuntimeOwnerId) -> bool {
        if self.pending.contains(&owner_id) {
            self.duplicate_marks = self.duplicate_marks.saturating_add(1);
            false
        } else {
            self.pending.push(owner_id);
            true
        }
    }

    /// Returns true when the owner is already marked in the local buffer.
    #[must_use]
    pub fn contains(&self, owner_id: RemoteFreeServiceRuntimeOwnerId) -> bool {
        self.pending.contains(&owner_id)
    }

    /// Returns the number of unique local dirty owners.
    #[must_use]
    pub fn len(&self) -> usize {
        self.pending.len()
    }

    /// Returns true when no local owner marks are pending.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    /// Returns the number of repeated local marks deduplicated before flushing.
    #[must_use]
    pub const fn duplicate_marks(&self) -> u64 {
        self.duplicate_marks
    }

    /// Returns locally marked owners in first-marked order.
    #[must_use]
    pub fn owner_ids(&self) -> &[RemoteFreeServiceRuntimeOwnerId] {
        &self.pending
    }

    /// Flushes unique local owner marks into `tracker` and clears the buffer.
    pub fn flush_into(
        &mut self,
        tracker: &RemoteFreeServiceRuntimeDirtyOwnerTracker,
    ) -> RemoteFreeServiceRuntimeDirtyOwnerFlushStats {
        let mut new_tracker_marks = 0_u64;

        for owner_id in &self.pending {
            if tracker.mark_dirty(*owner_id) {
                new_tracker_marks = new_tracker_marks.saturating_add(1);
            }
        }

        let stats = RemoteFreeServiceRuntimeDirtyOwnerFlushStats {
            owner_count: self.pending.len(),
            new_tracker_marks,
            duplicate_local_marks: self.duplicate_marks,
        };
        self.clear();
        stats
    }

    /// Clears all local dirty-owner marks and duplicate counters.
    pub fn clear(&mut self) {
        self.pending.clear();
        self.duplicate_marks = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::{
        RemoteFreeServiceRuntimeDirtyOwnerLocalBuffer, RemoteFreeServiceRuntimeDirtyOwnerTracker,
        RemoteFreeServiceRuntimeOwnerId,
    };

    #[test]
    fn local_buffer_deduplicates_marks_in_first_marked_order() {
        let first = RemoteFreeServiceRuntimeOwnerId::new(3);
        let second = RemoteFreeServiceRuntimeOwnerId::new(1);
        let mut buffer = RemoteFreeServiceRuntimeDirtyOwnerLocalBuffer::new();

        assert!(buffer.is_empty());
        assert!(buffer.mark_dirty(first));
        assert!(buffer.mark_dirty(second));
        assert!(!buffer.mark_dirty(first));

        assert_eq!(buffer.len(), 2);
        assert!(buffer.contains(first));
        assert_eq!(buffer.duplicate_marks(), 1);
        assert_eq!(buffer.owner_ids(), &[first, second]);

        buffer.clear();
        assert!(buffer.is_empty());
        assert_eq!(buffer.duplicate_marks(), 0);
    }

    #[test]
    fn local_buffer_flushes_unique_marks_to_tracker() {
        let first = RemoteFreeServiceRuntimeOwnerId::new(0);
        let second = RemoteFreeServiceRuntimeOwnerId::new(2);
        let tracker = RemoteFreeServiceRuntimeDirtyOwnerTracker::new();
        let mut buffer = RemoteFreeServiceRuntimeDirtyOwnerLocalBuffer::new();

        assert!(buffer.mark_dirty(first));
        assert!(!buffer.mark_dirty(first));
        assert!(buffer.mark_dirty(second));

        let stats = buffer.flush_into(&tracker);

        assert_eq!(stats.owner_count, 2);
        assert_eq!(stats.new_tracker_marks, 2);
        assert_eq!(stats.duplicate_local_marks, 1);
        assert!(buffer.is_empty());
        assert_eq!(tracker.owner_ids(), vec![first, second]);
    }

    #[test]
    fn local_buffer_flush_preserves_tracker_generation_after_snapshot() {
        let owner_id = RemoteFreeServiceRuntimeOwnerId::new(7);
        let tracker = RemoteFreeServiceRuntimeDirtyOwnerTracker::new();
        let mut buffer = RemoteFreeServiceRuntimeDirtyOwnerLocalBuffer::new();

        assert!(tracker.mark_dirty(owner_id));
        let snapshot = tracker.snapshot();
        assert!(buffer.mark_dirty(owner_id));

        let stats = buffer.flush_into(&tracker);
        tracker.clear_snapshot(&snapshot);

        assert_eq!(stats.owner_count, 1);
        assert_eq!(stats.new_tracker_marks, 0);
        assert_eq!(tracker.owner_ids(), vec![owner_id]);
    }
}
