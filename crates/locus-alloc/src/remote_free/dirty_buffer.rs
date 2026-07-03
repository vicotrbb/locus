use super::{RemoteFreeServiceRuntimeDirtyOwnerTracker, RemoteFreeServiceRuntimeOwnerId};

/// Per-worker dirty-owner mark buffer for batching tracker updates.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RemoteFreeServiceRuntimeDirtyOwnerLocalBuffer {
    pending: Vec<RemoteFreeServiceRuntimeOwnerId>,
    duplicate_marks: u64,
}

/// Reusable local dirty-owner buffers paired with one shared tracker.
#[derive(Debug, Clone, Default)]
pub struct RemoteFreeServiceRuntimeDirtyOwnerLocalBuffers {
    tracker: RemoteFreeServiceRuntimeDirtyOwnerTracker,
    buffers: Vec<RemoteFreeServiceRuntimeDirtyOwnerLocalBuffer>,
}

/// Borrowed local marker for one remote-free runtime owner.
#[derive(Debug)]
pub struct RemoteFreeServiceRuntimeDirtyOwnerLocalMarker<'a> {
    owner_id: RemoteFreeServiceRuntimeOwnerId,
    buffer: &'a mut RemoteFreeServiceRuntimeDirtyOwnerLocalBuffer,
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

    /// Returns the current local owner storage capacity.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.pending.capacity()
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

impl RemoteFreeServiceRuntimeDirtyOwnerLocalBuffers {
    /// Creates an empty local dirty-owner buffer group.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tracker: RemoteFreeServiceRuntimeDirtyOwnerTracker::new(),
            buffers: Vec::new(),
        }
    }

    /// Returns the shared dirty-owner tracker used by this group.
    #[must_use]
    pub const fn tracker(&self) -> &RemoteFreeServiceRuntimeDirtyOwnerTracker {
        &self.tracker
    }

    /// Marks an owner dirty in its local buffer.
    ///
    /// Returns true when the local owner mark was not already pending.
    #[must_use]
    pub fn mark_dirty(&mut self, owner_id: RemoteFreeServiceRuntimeOwnerId) -> bool {
        self.buffer_mut(owner_id).mark_dirty(owner_id)
    }

    /// Borrows a hot-path local marker for one owner.
    #[must_use]
    pub fn local_marker(
        &mut self,
        owner_id: RemoteFreeServiceRuntimeOwnerId,
    ) -> RemoteFreeServiceRuntimeDirtyOwnerLocalMarker<'_> {
        RemoteFreeServiceRuntimeDirtyOwnerLocalMarker {
            owner_id,
            buffer: self.buffer_mut(owner_id),
        }
    }

    /// Flushes one owner's local buffer into the shared dirty-owner tracker.
    pub fn flush_owner(
        &mut self,
        owner_id: RemoteFreeServiceRuntimeOwnerId,
    ) -> RemoteFreeServiceRuntimeDirtyOwnerFlushStats {
        let index = owner_id.index();
        self.ensure_buffer_index(index);
        self.buffers[index].flush_into(&self.tracker)
    }

    /// Returns the local buffer for an owner when it has been allocated.
    #[must_use]
    pub fn local_buffer(
        &self,
        owner_id: RemoteFreeServiceRuntimeOwnerId,
    ) -> Option<&RemoteFreeServiceRuntimeDirtyOwnerLocalBuffer> {
        self.buffers.get(owner_id.index())
    }

    /// Returns the current local storage capacity for one owner buffer.
    #[must_use]
    pub fn local_buffer_capacity(&self, owner_id: RemoteFreeServiceRuntimeOwnerId) -> usize {
        self.local_buffer(owner_id)
            .map_or(0, RemoteFreeServiceRuntimeDirtyOwnerLocalBuffer::capacity)
    }

    fn buffer_mut(
        &mut self,
        owner_id: RemoteFreeServiceRuntimeOwnerId,
    ) -> &mut RemoteFreeServiceRuntimeDirtyOwnerLocalBuffer {
        let index = owner_id.index();
        self.ensure_buffer_index(index);
        &mut self.buffers[index]
    }

    fn ensure_buffer_index(&mut self, index: usize) {
        if self.buffers.len() <= index {
            self.buffers.resize_with(
                index.saturating_add(1),
                RemoteFreeServiceRuntimeDirtyOwnerLocalBuffer::new,
            );
        }
    }
}

impl RemoteFreeServiceRuntimeDirtyOwnerLocalMarker<'_> {
    /// Marks the marker's owner dirty in the borrowed local buffer.
    ///
    /// Returns true when the local owner mark was not already pending.
    #[must_use]
    pub fn mark_dirty(&mut self) -> bool {
        self.buffer.mark_dirty(self.owner_id)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        RemoteFreeServiceRuntimeDirtyOwnerLocalBuffer,
        RemoteFreeServiceRuntimeDirtyOwnerLocalBuffers, RemoteFreeServiceRuntimeDirtyOwnerTracker,
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
        assert!(buffer.capacity() >= 2);
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

    #[test]
    fn local_buffer_group_flushes_one_owner_at_a_time() {
        let first = RemoteFreeServiceRuntimeOwnerId::new(0);
        let second = RemoteFreeServiceRuntimeOwnerId::new(2);
        let mut buffers = RemoteFreeServiceRuntimeDirtyOwnerLocalBuffers::new();

        assert!(buffers.mark_dirty(first));
        assert!(!buffers.mark_dirty(first));
        assert!(buffers.mark_dirty(second));

        let first_stats = buffers.flush_owner(first);

        assert_eq!(first_stats.owner_count, 1);
        assert_eq!(first_stats.new_tracker_marks, 1);
        assert_eq!(first_stats.duplicate_local_marks, 1);
        assert!(buffers
            .local_buffer(first)
            .expect("first buffer")
            .is_empty());
        assert_eq!(
            buffers
                .local_buffer(second)
                .expect("second buffer")
                .owner_ids(),
            &[second]
        );
        assert_eq!(buffers.tracker().owner_ids(), vec![first]);

        let second_stats = buffers.flush_owner(second);

        assert_eq!(second_stats.owner_count, 1);
        assert_eq!(second_stats.new_tracker_marks, 1);
        assert_eq!(second_stats.duplicate_local_marks, 0);
        assert_eq!(buffers.tracker().owner_ids(), vec![first, second]);
    }

    #[test]
    fn local_buffer_group_reuses_capacity_after_flush() {
        let owner_id = RemoteFreeServiceRuntimeOwnerId::new(1);
        let mut buffers = RemoteFreeServiceRuntimeDirtyOwnerLocalBuffers::new();

        assert!(buffers.mark_dirty(owner_id));
        let before_flush_capacity = buffers.local_buffer_capacity(owner_id);
        let stats = buffers.flush_owner(owner_id);
        let after_flush_capacity = buffers.local_buffer_capacity(owner_id);

        assert_eq!(stats.owner_count, 1);
        assert!(before_flush_capacity >= 1);
        assert!(after_flush_capacity >= before_flush_capacity);
        assert!(buffers
            .local_buffer(owner_id)
            .expect("owner buffer")
            .is_empty());
    }

    #[test]
    fn local_buffer_group_marker_borrows_hot_owner_buffer() {
        let owner_id = RemoteFreeServiceRuntimeOwnerId::new(4);
        let mut buffers = RemoteFreeServiceRuntimeDirtyOwnerLocalBuffers::new();

        {
            let mut marker = buffers.local_marker(owner_id);
            assert!(marker.mark_dirty());
            assert!(!marker.mark_dirty());
        }

        let buffer = buffers.local_buffer(owner_id).expect("owner buffer");
        assert_eq!(buffer.owner_ids(), &[owner_id]);
        assert_eq!(buffer.duplicate_marks(), 1);
    }

    #[test]
    fn local_buffer_group_preserves_tracker_generation_after_snapshot() {
        let owner_id = RemoteFreeServiceRuntimeOwnerId::new(7);
        let mut buffers = RemoteFreeServiceRuntimeDirtyOwnerLocalBuffers::new();

        assert!(buffers.tracker().mark_dirty(owner_id));
        let snapshot = buffers.tracker().snapshot();
        assert!(buffers.mark_dirty(owner_id));

        let stats = buffers.flush_owner(owner_id);
        buffers.tracker().clear_snapshot(&snapshot);

        assert_eq!(stats.owner_count, 1);
        assert_eq!(stats.new_tracker_marks, 0);
        assert_eq!(buffers.tracker().owner_ids(), vec![owner_id]);
    }
}
