use std::collections::VecDeque;
use std::fmt;
use std::num::NonZeroU64;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TryRecvError, TrySendError};
use std::sync::Arc;

/// Owner-drained queue for batching remote frees back to an owning worker.
pub struct RemoteFreeQueue<T> {
    receiver: Receiver<T>,
    sink: RemoteFreeSink<T>,
    capacity: usize,
    batch_limit: usize,
    drained_count: u64,
}

/// Cloneable enqueue handle for remote free work.
pub struct RemoteFreeSink<T> {
    sender: SyncSender<T>,
    submitted_count: Arc<AtomicU64>,
    full_count: Arc<AtomicU64>,
    disconnected_count: Arc<AtomicU64>,
}

/// Remote free queue accounting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteFreeQueueStats {
    /// Bounded channel capacity.
    pub capacity: usize,
    /// Maximum items drained per batch.
    pub batch_limit: usize,
    /// Successfully enqueued item count.
    pub submitted_count: u64,
    /// Submitted items that have not yet been drained by the owner.
    pub pending_count: u64,
    /// Nonblocking enqueue attempts rejected because the queue was full.
    pub full_count: u64,
    /// Enqueue attempts rejected because the owning queue was dropped.
    pub disconnected_count: u64,
    /// Items drained by the owner.
    pub drained_count: u64,
}

/// Result of one remote free drain operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteFreeDrainStats {
    /// Items drained by this operation.
    pub drained: usize,
    /// Total items drained by the queue after this operation.
    pub total_drained: u64,
}

/// Owner-side signals used to decide whether remote frees should be drained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteFreeDrainObservation {
    /// Submitted items that have not yet been drained by the owner.
    pub pending_count: u64,
    /// Estimated bytes retained by pending remote-free work.
    pub queued_bytes: u64,
    /// Age of the oldest pending item in scheduler-defined logical turns.
    pub oldest_pending_age: u64,
}

/// Pure policy for deciding whether the owner should drain remote frees.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RemoteFreeDrainPolicy {
    pending_count_ceiling: Option<NonZeroU64>,
    queued_byte_budget: Option<NonZeroU64>,
    age_bound: Option<NonZeroU64>,
}

/// Owner-side pending work tracker for remote-free drain policy observations.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RemoteFreeDrainTracker {
    pending_turns: VecDeque<RemoteFreeDrainTurn>,
    pending_count: u64,
    queued_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RemoteFreeDrainTurn {
    submit_turn: u64,
    pending_count: u64,
    queued_bytes: u64,
}

/// Result of applying a remote-free drain policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteFreeDrainDecision {
    /// The owner can defer draining for now.
    Defer,
    /// The owner should drain because a threshold was reached.
    Drain(RemoteFreeDrainReason),
}

/// Threshold that caused a remote-free drain policy to request draining.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteFreeDrainReason {
    /// Pending queued bytes reached the configured maximum.
    QueuedBytes,
    /// Oldest pending item age reached the configured maximum.
    PendingAge,
    /// Pending item count reached the configured maximum.
    PendingCount,
}

/// Metadata for one tracked remote-free drain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteFreeTrackedDrain {
    /// Logical turn recorded for the drained work item.
    pub submit_turn: u64,
    /// Released byte size recorded for the drained work item.
    pub released_bytes: u64,
}

/// Remote-free drain tracker accounting failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteFreeDrainTrackerError {
    /// A drain was recorded with no pending tracked work.
    Empty,
    /// A drain released more bytes than remain in the oldest pending turn.
    ReleasedBytesExceedQueued {
        /// Bytes reported by the drain event.
        released_bytes: u64,
        /// Bytes remaining in the oldest pending turn.
        queued_bytes: u64,
    },
    /// The final item for a pending turn did not release the remaining bytes.
    InconsistentFinalRelease {
        /// Bytes reported by the drain event.
        released_bytes: u64,
        /// Bytes expected for the final item in the oldest pending turn.
        expected_bytes: u64,
    },
}

impl<T> RemoteFreeQueue<T> {
    /// Creates an owner-drained remote free queue.
    ///
    /// # Errors
    ///
    /// Returns an error when `capacity` or `batch_limit` is zero.
    pub fn new(capacity: usize, batch_limit: usize) -> Result<Self, RemoteFreeQueueError> {
        if capacity == 0 {
            return Err(RemoteFreeQueueError::InvalidCapacity);
        }
        if batch_limit == 0 {
            return Err(RemoteFreeQueueError::InvalidBatchLimit);
        }

        let (sender, receiver) = sync_channel(capacity);
        let submitted_count = Arc::new(AtomicU64::new(0));
        let full_count = Arc::new(AtomicU64::new(0));
        let disconnected_count = Arc::new(AtomicU64::new(0));
        let sink = RemoteFreeSink {
            sender,
            submitted_count,
            full_count,
            disconnected_count,
        };

        Ok(Self {
            receiver,
            sink,
            capacity,
            batch_limit,
            drained_count: 0,
        })
    }

    /// Returns a cloneable sink for remote producers.
    #[must_use]
    pub fn sink(&self) -> RemoteFreeSink<T> {
        self.sink.clone()
    }

    /// Drains up to the configured batch limit and passes each item to `release`.
    #[must_use]
    pub fn drain_batch(&mut self, mut release: impl FnMut(T)) -> RemoteFreeDrainStats {
        let mut drained = 0_usize;

        while drained < self.batch_limit {
            match self.receiver.try_recv() {
                Ok(item) => {
                    release(item);
                    drained += 1;
                }
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            }
        }

        self.drained_count = self.drained_count.saturating_add(drained as u64);

        RemoteFreeDrainStats {
            drained,
            total_drained: self.drained_count,
        }
    }

    /// Returns current queue accounting.
    #[must_use]
    pub fn stats(&self) -> RemoteFreeQueueStats {
        let submitted_count = self.sink.submitted_count();
        RemoteFreeQueueStats {
            capacity: self.capacity,
            batch_limit: self.batch_limit,
            submitted_count,
            pending_count: submitted_count.saturating_sub(self.drained_count),
            full_count: self.sink.full_count(),
            disconnected_count: self.sink.disconnected_count(),
            drained_count: self.drained_count,
        }
    }
}

impl RemoteFreeDrainTracker {
    /// Creates an empty owner-side drain tracker.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Records one successfully submitted remote-free work item.
    pub fn record_submit(&mut self, submit_turn: u64, queued_bytes: u64) {
        if let Some(back) = self.pending_turns.back_mut() {
            if back.submit_turn == submit_turn {
                back.pending_count = back.pending_count.saturating_add(1);
                back.queued_bytes = back.queued_bytes.saturating_add(queued_bytes);
                self.pending_count = self.pending_count.saturating_add(1);
                self.queued_bytes = self.queued_bytes.saturating_add(queued_bytes);
                return;
            }
        }

        self.pending_turns.push_back(RemoteFreeDrainTurn {
            submit_turn,
            pending_count: 1,
            queued_bytes,
        });
        self.pending_count = self.pending_count.saturating_add(1);
        self.queued_bytes = self.queued_bytes.saturating_add(queued_bytes);
    }

    /// Records one FIFO owner-side drain.
    ///
    /// # Errors
    ///
    /// Returns an error when no work is pending or when the released byte
    /// accounting is inconsistent with the oldest pending turn.
    pub fn record_drain(
        &mut self,
        released_bytes: u64,
    ) -> Result<RemoteFreeTrackedDrain, RemoteFreeDrainTrackerError> {
        let submit_turn = {
            let front = self
                .pending_turns
                .front_mut()
                .ok_or(RemoteFreeDrainTrackerError::Empty)?;

            if released_bytes > front.queued_bytes {
                return Err(RemoteFreeDrainTrackerError::ReleasedBytesExceedQueued {
                    released_bytes,
                    queued_bytes: front.queued_bytes,
                });
            }

            if front.pending_count == 1 && released_bytes != front.queued_bytes {
                return Err(RemoteFreeDrainTrackerError::InconsistentFinalRelease {
                    released_bytes,
                    expected_bytes: front.queued_bytes,
                });
            }

            front.pending_count -= 1;
            front.queued_bytes -= released_bytes;
            front.submit_turn
        };

        self.pending_count -= 1;
        self.queued_bytes -= released_bytes;

        if self
            .pending_turns
            .front()
            .is_some_and(|front| front.pending_count == 0)
        {
            self.pending_turns.pop_front();
        }

        Ok(RemoteFreeTrackedDrain {
            submit_turn,
            released_bytes,
        })
    }

    /// Returns current pending work count.
    #[must_use]
    pub fn pending_count(&self) -> u64 {
        self.pending_count
    }

    /// Returns current retained queued bytes.
    #[must_use]
    pub fn queued_bytes(&self) -> u64 {
        self.queued_bytes
    }

    /// Returns true when no work is pending.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pending_count == 0
    }

    /// Builds a drain-policy observation for the current logical turn.
    #[must_use]
    pub fn observation(&self, current_turn: u64) -> RemoteFreeDrainObservation {
        let oldest_pending_age = self
            .pending_turns
            .front()
            .map_or(0, |front| current_turn.saturating_sub(front.submit_turn));

        RemoteFreeDrainObservation::new(self.pending_count, self.queued_bytes, oldest_pending_age)
    }
}

impl RemoteFreeDrainObservation {
    /// Creates a drain-policy observation.
    #[must_use]
    pub const fn new(pending_count: u64, queued_bytes: u64, oldest_pending_age: u64) -> Self {
        Self {
            pending_count,
            queued_bytes,
            oldest_pending_age,
        }
    }
}

impl RemoteFreeDrainPolicy {
    /// Creates a policy with no thresholds.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            pending_count_ceiling: None,
            queued_byte_budget: None,
            age_bound: None,
        }
    }

    /// Sets the pending item threshold.
    #[must_use]
    pub const fn with_max_pending_count(mut self, max_pending_count: NonZeroU64) -> Self {
        self.pending_count_ceiling = Some(max_pending_count);
        self
    }

    /// Sets the retained byte threshold.
    #[must_use]
    pub const fn with_max_queued_bytes(mut self, max_queued_bytes: NonZeroU64) -> Self {
        self.queued_byte_budget = Some(max_queued_bytes);
        self
    }

    /// Sets the oldest pending item age threshold.
    #[must_use]
    pub const fn with_max_pending_age(mut self, max_pending_age: NonZeroU64) -> Self {
        self.age_bound = Some(max_pending_age);
        self
    }

    /// Returns the policy decision for an owner-side observation.
    #[must_use]
    pub fn decide(self, observation: RemoteFreeDrainObservation) -> RemoteFreeDrainDecision {
        if observation.pending_count == 0 {
            return RemoteFreeDrainDecision::Defer;
        }

        if self
            .queued_byte_budget
            .is_some_and(|threshold| observation.queued_bytes >= threshold.get())
        {
            return RemoteFreeDrainDecision::Drain(RemoteFreeDrainReason::QueuedBytes);
        }

        if self
            .age_bound
            .is_some_and(|threshold| observation.oldest_pending_age >= threshold.get())
        {
            return RemoteFreeDrainDecision::Drain(RemoteFreeDrainReason::PendingAge);
        }

        if self
            .pending_count_ceiling
            .is_some_and(|threshold| observation.pending_count >= threshold.get())
        {
            return RemoteFreeDrainDecision::Drain(RemoteFreeDrainReason::PendingCount);
        }

        RemoteFreeDrainDecision::Defer
    }

    /// Returns whether the owner should drain for an observation.
    #[must_use]
    pub fn should_drain(self, observation: RemoteFreeDrainObservation) -> bool {
        self.decide(observation).should_drain()
    }
}

impl RemoteFreeDrainDecision {
    /// Returns true when the decision requests owner-side draining.
    #[must_use]
    pub const fn should_drain(self) -> bool {
        matches!(self, Self::Drain(_))
    }
}

impl<T> RemoteFreeSink<T> {
    /// Enqueues one item for owner-side release.
    ///
    /// # Errors
    ///
    /// Returns the item when the owning queue has been dropped.
    pub fn enqueue(&self, item: T) -> Result<(), RemoteFreeEnqueueError<T>> {
        self.sender.send(item).map_err(|source| {
            self.disconnected_count.fetch_add(1, Ordering::Relaxed);
            RemoteFreeEnqueueError { item: source.0 }
        })?;
        self.submitted_count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Attempts to enqueue one item for owner-side release without blocking.
    ///
    /// # Errors
    ///
    /// Returns the item and a failure kind when the bounded queue is full or the
    /// owning queue has been dropped.
    pub fn try_enqueue(&self, item: T) -> Result<(), RemoteFreeTryEnqueueError<T>> {
        match self.sender.try_send(item) {
            Ok(()) => {
                self.submitted_count.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
            Err(TrySendError::Full(item)) => {
                self.full_count.fetch_add(1, Ordering::Relaxed);
                Err(RemoteFreeTryEnqueueError {
                    item,
                    kind: RemoteFreeTryEnqueueErrorKind::Full,
                })
            }
            Err(TrySendError::Disconnected(item)) => {
                self.disconnected_count.fetch_add(1, Ordering::Relaxed);
                Err(RemoteFreeTryEnqueueError {
                    item,
                    kind: RemoteFreeTryEnqueueErrorKind::Disconnected,
                })
            }
        }
    }

    /// Returns the number of successfully submitted items.
    #[must_use]
    pub fn submitted_count(&self) -> u64 {
        self.submitted_count.load(Ordering::Relaxed)
    }

    /// Returns the number of nonblocking enqueue attempts rejected as full.
    #[must_use]
    pub fn full_count(&self) -> u64 {
        self.full_count.load(Ordering::Relaxed)
    }

    /// Returns the number of enqueue attempts rejected as disconnected.
    #[must_use]
    pub fn disconnected_count(&self) -> u64 {
        self.disconnected_count.load(Ordering::Relaxed)
    }
}

impl<T> Clone for RemoteFreeSink<T> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            submitted_count: Arc::clone(&self.submitted_count),
            full_count: Arc::clone(&self.full_count),
            disconnected_count: Arc::clone(&self.disconnected_count),
        }
    }
}

impl<T> fmt::Debug for RemoteFreeQueue<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RemoteFreeQueue")
            .field("capacity", &self.capacity)
            .field("batch_limit", &self.batch_limit)
            .field("drained_count", &self.drained_count)
            .finish_non_exhaustive()
    }
}

impl<T> fmt::Debug for RemoteFreeSink<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RemoteFreeSink")
            .field("submitted_count", &self.submitted_count())
            .field("full_count", &self.full_count())
            .field("disconnected_count", &self.disconnected_count())
            .finish_non_exhaustive()
    }
}

/// Remote free queue configuration failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteFreeQueueError {
    /// Queue capacity must be non-zero.
    InvalidCapacity,
    /// Drain batch limit must be non-zero.
    InvalidBatchLimit,
}

/// Remote free enqueue failure.
pub struct RemoteFreeEnqueueError<T> {
    item: T,
}

/// Remote free nonblocking enqueue failure.
pub struct RemoteFreeTryEnqueueError<T> {
    item: T,
    kind: RemoteFreeTryEnqueueErrorKind,
}

/// Reason a nonblocking remote free enqueue failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteFreeTryEnqueueErrorKind {
    /// The bounded queue was full.
    Full,
    /// The owning queue was dropped.
    Disconnected,
}

impl<T> RemoteFreeEnqueueError<T> {
    /// Returns the item that could not be enqueued.
    #[must_use]
    pub fn into_item(self) -> T {
        self.item
    }
}

impl<T> RemoteFreeTryEnqueueError<T> {
    /// Returns why the item could not be enqueued.
    #[must_use]
    pub fn kind(&self) -> RemoteFreeTryEnqueueErrorKind {
        self.kind
    }

    /// Returns the item that could not be enqueued.
    #[must_use]
    pub fn into_item(self) -> T {
        self.item
    }
}

impl fmt::Display for RemoteFreeQueueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCapacity => f.write_str("remote free queue capacity must be non-zero"),
            Self::InvalidBatchLimit => {
                f.write_str("remote free queue batch limit must be non-zero")
            }
        }
    }
}

impl std::error::Error for RemoteFreeQueueError {}

impl<T> fmt::Debug for RemoteFreeEnqueueError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RemoteFreeEnqueueError")
            .finish_non_exhaustive()
    }
}

impl<T> fmt::Debug for RemoteFreeTryEnqueueError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RemoteFreeTryEnqueueError")
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}

impl<T> fmt::Display for RemoteFreeEnqueueError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("remote free queue receiver is closed")
    }
}

impl<T> fmt::Display for RemoteFreeTryEnqueueError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            RemoteFreeTryEnqueueErrorKind::Full => f.write_str("remote free queue is full"),
            RemoteFreeTryEnqueueErrorKind::Disconnected => {
                f.write_str("remote free queue receiver is closed")
            }
        }
    }
}

impl fmt::Display for RemoteFreeDrainTrackerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("remote free drain tracker is empty"),
            Self::ReleasedBytesExceedQueued {
                released_bytes,
                queued_bytes,
            } => write!(
                f,
                "remote free drain released {released_bytes} bytes with only {queued_bytes} queued"
            ),
            Self::InconsistentFinalRelease {
                released_bytes,
                expected_bytes,
            } => write!(
                f,
                "remote free final drain released {released_bytes} bytes but expected {expected_bytes}"
            ),
        }
    }
}

impl<T> std::error::Error for RemoteFreeEnqueueError<T> {}

impl<T> std::error::Error for RemoteFreeTryEnqueueError<T> {}

impl std::error::Error for RemoteFreeDrainTrackerError {}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    use super::{
        RemoteFreeDrainDecision, RemoteFreeDrainObservation, RemoteFreeDrainPolicy,
        RemoteFreeDrainReason, RemoteFreeDrainTracker, RemoteFreeDrainTrackerError,
        RemoteFreeQueue, RemoteFreeQueueError, RemoteFreeTryEnqueueErrorKind,
    };

    #[test]
    fn remote_free_queue_drains_in_batches() {
        let mut queue = RemoteFreeQueue::new(8, 2).expect("queue");
        let sink = queue.sink();

        sink.enqueue(1).expect("enqueue first");
        sink.enqueue(2).expect("enqueue second");
        sink.enqueue(3).expect("enqueue third");

        let mut released = Vec::new();
        let first = queue.drain_batch(|item| released.push(item));

        assert_eq!(first.drained, 2);
        assert_eq!(first.total_drained, 2);
        assert_eq!(released, vec![1, 2]);
        assert_eq!(
            queue.stats(),
            super::RemoteFreeQueueStats {
                capacity: 8,
                batch_limit: 2,
                submitted_count: 3,
                pending_count: 1,
                full_count: 0,
                disconnected_count: 0,
                drained_count: 2,
            }
        );

        let second = queue.drain_batch(|item| released.push(item));

        assert_eq!(second.drained, 1);
        assert_eq!(second.total_drained, 3);
        assert_eq!(released, vec![1, 2, 3]);
    }

    #[test]
    fn remote_free_try_enqueue_reports_backpressure() {
        let mut queue = RemoteFreeQueue::new(1, 1).expect("queue");
        let sink = queue.sink();

        sink.try_enqueue(1).expect("enqueue first");
        let error = sink.try_enqueue(2).expect_err("queue should be full");

        assert_eq!(error.kind(), RemoteFreeTryEnqueueErrorKind::Full);
        assert_eq!(error.into_item(), 2);
        assert_eq!(sink.submitted_count(), 1);
        assert_eq!(sink.full_count(), 1);
        assert_eq!(sink.disconnected_count(), 0);
        assert_eq!(
            queue.stats(),
            super::RemoteFreeQueueStats {
                capacity: 1,
                batch_limit: 1,
                submitted_count: 1,
                pending_count: 1,
                full_count: 1,
                disconnected_count: 0,
                drained_count: 0,
            }
        );

        let mut released = Vec::new();
        let drain = queue.drain_batch(|item| released.push(item));

        assert_eq!(drain.drained, 1);
        assert_eq!(released, vec![1]);
        sink.try_enqueue(3).expect("enqueue after drain");
        assert_eq!(sink.submitted_count(), 2);
        assert_eq!(sink.full_count(), 1);
    }

    #[test]
    fn remote_free_queue_rejects_invalid_configuration() {
        assert_eq!(
            RemoteFreeQueue::<u8>::new(0, 1).expect_err("zero capacity"),
            RemoteFreeQueueError::InvalidCapacity
        );
        assert_eq!(
            RemoteFreeQueue::<u8>::new(1, 0).expect_err("zero batch limit"),
            RemoteFreeQueueError::InvalidBatchLimit
        );
    }

    #[test]
    fn remote_free_sink_returns_item_when_owner_is_dropped() {
        let queue = RemoteFreeQueue::new(1, 1).expect("queue");
        let sink = queue.sink();
        drop(queue);

        let error = sink.enqueue(7).expect_err("receiver is closed");

        assert_eq!(error.into_item(), 7);
        assert_eq!(sink.submitted_count(), 0);
        assert_eq!(sink.disconnected_count(), 1);

        let error = sink.try_enqueue(8).expect_err("receiver is closed");

        assert_eq!(error.kind(), RemoteFreeTryEnqueueErrorKind::Disconnected);
        assert_eq!(error.into_item(), 8);
        assert_eq!(sink.submitted_count(), 0);
        assert_eq!(sink.full_count(), 0);
        assert_eq!(sink.disconnected_count(), 2);
    }

    #[test]
    fn remote_free_drain_policy_defers_without_thresholds() {
        let policy = RemoteFreeDrainPolicy::new();
        let observation = RemoteFreeDrainObservation::new(256, 2_621_440, 8);

        assert_eq!(policy.decide(observation), RemoteFreeDrainDecision::Defer);
        assert!(!policy.should_drain(observation));
    }

    #[test]
    fn remote_free_drain_policy_ignores_empty_observations() {
        let policy = RemoteFreeDrainPolicy::new()
            .with_max_pending_count(NonZeroU64::new(1).expect("non-zero"))
            .with_max_queued_bytes(NonZeroU64::new(1).expect("non-zero"))
            .with_max_pending_age(NonZeroU64::new(1).expect("non-zero"));
        let observation = RemoteFreeDrainObservation::new(0, 1_048_576, 16);

        assert_eq!(policy.decide(observation), RemoteFreeDrainDecision::Defer);
    }

    #[test]
    fn remote_free_drain_policy_prefers_queued_bytes_reason() {
        let policy = RemoteFreeDrainPolicy::new()
            .with_max_pending_count(NonZeroU64::new(64).expect("non-zero"))
            .with_max_queued_bytes(NonZeroU64::new(655_360).expect("non-zero"))
            .with_max_pending_age(NonZeroU64::new(2).expect("non-zero"));
        let observation = RemoteFreeDrainObservation::new(64, 655_360, 2);

        assert_eq!(
            policy.decide(observation),
            RemoteFreeDrainDecision::Drain(RemoteFreeDrainReason::QueuedBytes)
        );
        assert!(policy.should_drain(observation));
    }

    #[test]
    fn remote_free_drain_policy_triggers_by_pending_age() {
        let policy = RemoteFreeDrainPolicy::new()
            .with_max_pending_age(NonZeroU64::new(2).expect("non-zero"));
        let observation = RemoteFreeDrainObservation::new(32, 327_680, 2);

        assert_eq!(
            policy.decide(observation),
            RemoteFreeDrainDecision::Drain(RemoteFreeDrainReason::PendingAge)
        );
    }

    #[test]
    fn remote_free_drain_policy_triggers_by_pending_count() {
        let policy = RemoteFreeDrainPolicy::new()
            .with_max_pending_count(NonZeroU64::new(64).expect("non-zero"));
        let observation = RemoteFreeDrainObservation::new(64, 327_680, 1);

        assert_eq!(
            policy.decide(observation),
            RemoteFreeDrainDecision::Drain(RemoteFreeDrainReason::PendingCount)
        );
    }

    #[test]
    fn remote_free_drain_tracker_reports_policy_observations() {
        let mut tracker = RemoteFreeDrainTracker::new();

        tracker.record_submit(0, 4096);
        tracker.record_submit(0, 8192);
        tracker.record_submit(1, 16_384);

        assert_eq!(tracker.pending_count(), 3);
        assert_eq!(tracker.queued_bytes(), 28_672);
        assert_eq!(
            tracker.observation(2),
            RemoteFreeDrainObservation::new(3, 28_672, 2)
        );

        let first = tracker.record_drain(4096).expect("first drain");
        assert_eq!(first.submit_turn, 0);
        assert_eq!(first.released_bytes, 4096);
        assert_eq!(
            tracker.observation(2),
            RemoteFreeDrainObservation::new(2, 24_576, 2)
        );

        let second = tracker.record_drain(8192).expect("second drain");
        assert_eq!(second.submit_turn, 0);
        assert_eq!(
            tracker.observation(2),
            RemoteFreeDrainObservation::new(1, 16_384, 1)
        );

        let third = tracker.record_drain(16_384).expect("third drain");
        assert_eq!(third.submit_turn, 1);
        assert!(tracker.is_empty());
        assert_eq!(
            tracker.observation(3),
            RemoteFreeDrainObservation::new(0, 0, 0)
        );
    }

    #[test]
    fn remote_free_drain_tracker_rejects_empty_drain() {
        let mut tracker = RemoteFreeDrainTracker::new();

        assert_eq!(
            tracker.record_drain(1).expect_err("empty tracker"),
            RemoteFreeDrainTrackerError::Empty
        );
    }

    #[test]
    fn remote_free_drain_tracker_rejects_over_release() {
        let mut tracker = RemoteFreeDrainTracker::new();
        tracker.record_submit(0, 4096);

        assert_eq!(
            tracker.record_drain(8192).expect_err("over release"),
            RemoteFreeDrainTrackerError::ReleasedBytesExceedQueued {
                released_bytes: 8192,
                queued_bytes: 4096,
            }
        );
        assert_eq!(
            tracker.observation(1),
            RemoteFreeDrainObservation::new(1, 4096, 1)
        );
    }

    #[test]
    fn remote_free_drain_tracker_rejects_inconsistent_final_release() {
        let mut tracker = RemoteFreeDrainTracker::new();
        tracker.record_submit(0, 4096);
        tracker.record_submit(0, 4096);

        tracker.record_drain(2048).expect("first drain");
        assert_eq!(
            tracker.record_drain(2048).expect_err("final mismatch"),
            RemoteFreeDrainTrackerError::InconsistentFinalRelease {
                released_bytes: 2048,
                expected_bytes: 6144,
            }
        );
    }
}
