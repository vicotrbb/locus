use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TryRecvError, TrySendError};
use std::sync::Arc;

mod application;
mod budget;
mod config;
mod controller;
mod coordinator;
mod drift;
mod guard;
mod planner;
mod runtime;
mod service_window;
mod telemetry;

pub use application::{
    RemoteFreeServiceRetunePolicyApplication, RemoteFreeServiceRetunePolicyApplicationError,
    RemoteFreeServiceRetunePolicyApplicator,
};
pub use budget::{RemoteFreeQueuedByteBudget, RemoteFreeQueuedByteBudgetError};
pub use config::{RemoteFreeQueuedByteDrainConfig, RemoteFreeQueuedByteDrainConfigError};
pub use controller::{
    RemoteFreeDrainController, RemoteFreeDrainControllerError, RemoteFreeDrainControllerStatus,
    RemoteFreeDrainDecision, RemoteFreeDrainObservation, RemoteFreeDrainPolicy,
    RemoteFreeDrainReason, RemoteFreeDrainTracker, RemoteFreeDrainTrackerError,
    RemoteFreeTrackedDrain,
};
pub use coordinator::{
    RemoteFreeServiceRuntimeOwnerId, RemoteFreeServiceRuntimeRetuneCoordinator,
    RemoteFreeServiceRuntimeRetuneError, RemoteFreeServiceRuntimeRetuneOutcome,
    RemoteFreeServiceRuntimeRetuneOwnerError, RemoteFreeServiceRuntimeRetuneOwners,
};
pub use drift::{
    RemoteFreeQueuedByteDriftReport, RemoteFreeQueuedByteRetuneAction,
    RemoteFreeQueuedByteRetuneHint,
};
pub use guard::{
    RemoteFreeServiceRetuneGuard, RemoteFreeServiceRetuneGuardDecision,
    RemoteFreeServiceRetuneGuardError,
};
pub use planner::{
    RemoteFreeServiceRetuneCandidate, RemoteFreeServiceRetuneDryRunPlanner,
    RemoteFreeServiceRetuneDryRunPlannerError,
};
pub use runtime::{
    RemoteFreeOwnerRuntime, RemoteFreeOwnerRuntimeApplyOutcome,
    RemoteFreeOwnerRuntimeConfirmOutcome, RemoteFreeOwnerRuntimeError,
    RemoteFreeOwnerRuntimeRollbackOutcome,
};
pub use service_window::{
    RemoteFreeServiceRuntimeWindowCollectionError, RemoteFreeServiceRuntimeWindowError,
    RemoteFreeServiceRuntimeWindowObservation, RemoteFreeServiceRuntimeWindowStats,
};
pub use telemetry::{RemoteFreeRetuneActionCounts, RemoteFreeServiceRetuneSummary};

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

impl<T> std::error::Error for RemoteFreeEnqueueError<T> {}

impl<T> std::error::Error for RemoteFreeTryEnqueueError<T> {}

#[cfg(test)]
mod tests {
    use super::{RemoteFreeQueue, RemoteFreeQueueError, RemoteFreeTryEnqueueErrorKind};

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
}
