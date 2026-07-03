use std::collections::VecDeque;
use std::fmt;
use std::num::NonZeroU64;

use super::{RemoteFreeQueue, RemoteFreeQueueStats};

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

/// Owner-side controller for remote-free policy and drain accounting.
///
/// The controller owns policy and byte accounting, while domain allocators keep
/// their release logic explicit in the queue drain closure.
///
/// ```
/// use std::num::NonZeroU64;
///
/// use locus_alloc::{RemoteFreeDrainController, RemoteFreeDrainPolicy, RemoteFreeQueue};
///
/// let mut queue = RemoteFreeQueue::new(8, 2).expect("queue");
/// let sink = queue.sink();
/// let mut controller = RemoteFreeDrainController::new(
///     RemoteFreeDrainPolicy::new()
///         .with_max_pending_age(NonZeroU64::new(2).expect("non-zero")),
/// );
///
/// sink.enqueue(vec![0_u8; 4096]).expect("enqueue");
/// controller.record_submit(0, 4096);
///
/// if controller
///     .should_drain_queue(&queue, 2)
///     .expect("controller status")
/// {
///     queue.drain_batch(|buffer| {
///         let released_bytes = u64::try_from(buffer.len()).expect("buffer length fits u64");
///         controller
///             .record_drain(released_bytes)
///             .expect("tracked drain");
///     });
/// }
///
/// assert!(controller.is_empty());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFreeDrainController {
    policy: RemoteFreeDrainPolicy,
    tracker: RemoteFreeDrainTracker,
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

/// Policy status for one owner-side remote-free control point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteFreeDrainControllerStatus {
    /// Queue accounting observed at the control point.
    pub queue_stats: RemoteFreeQueueStats,
    /// Tracker observation used by the policy.
    pub observation: RemoteFreeDrainObservation,
    /// Policy decision for the observation.
    pub decision: RemoteFreeDrainDecision,
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

/// Remote-free drain controller accounting failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteFreeDrainControllerError {
    /// Queue pending count and tracker pending count diverged.
    PendingCountMismatch {
        /// Pending count reported by `RemoteFreeQueue`.
        queue_pending_count: u64,
        /// Pending count reported by `RemoteFreeDrainTracker`.
        tracker_pending_count: u64,
    },
    /// Drain tracker accounting failed.
    Tracker(RemoteFreeDrainTrackerError),
}

impl RemoteFreeDrainController {
    /// Creates a controller with the given drain policy and an empty tracker.
    #[must_use]
    pub fn new(policy: RemoteFreeDrainPolicy) -> Self {
        Self {
            policy,
            tracker: RemoteFreeDrainTracker::new(),
        }
    }

    /// Returns the configured drain policy.
    #[must_use]
    pub const fn policy(&self) -> RemoteFreeDrainPolicy {
        self.policy
    }

    /// Returns the underlying drain tracker.
    #[must_use]
    pub const fn tracker(&self) -> &RemoteFreeDrainTracker {
        &self.tracker
    }

    /// Returns current tracked pending work count.
    #[must_use]
    pub fn pending_count(&self) -> u64 {
        self.tracker.pending_count()
    }

    /// Returns current tracked retained queued bytes.
    #[must_use]
    pub fn queued_bytes(&self) -> u64 {
        self.tracker.queued_bytes()
    }

    /// Returns true when no tracked remote-free work is pending.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tracker.is_empty()
    }

    /// Records one successfully submitted remote-free work item.
    pub fn record_submit(&mut self, submit_turn: u64, queued_bytes: u64) {
        self.tracker.record_submit(submit_turn, queued_bytes);
    }

    /// Records one FIFO owner-side drain.
    ///
    /// # Errors
    ///
    /// Returns an error when tracker accounting is inconsistent.
    pub fn record_drain(
        &mut self,
        released_bytes: u64,
    ) -> Result<RemoteFreeTrackedDrain, RemoteFreeDrainControllerError> {
        self.tracker
            .record_drain(released_bytes)
            .map_err(RemoteFreeDrainControllerError::Tracker)
    }

    /// Builds policy status for the current queue and logical turn.
    ///
    /// # Errors
    ///
    /// Returns an error when queue and tracker pending counts differ.
    pub fn status_for_queue<T>(
        &self,
        queue: &RemoteFreeQueue<T>,
        current_turn: u64,
    ) -> Result<RemoteFreeDrainControllerStatus, RemoteFreeDrainControllerError> {
        let queue_stats = queue.stats();
        let observation = self.tracker.observation(current_turn);

        if queue_stats.pending_count != observation.pending_count {
            return Err(RemoteFreeDrainControllerError::PendingCountMismatch {
                queue_pending_count: queue_stats.pending_count,
                tracker_pending_count: observation.pending_count,
            });
        }

        Ok(RemoteFreeDrainControllerStatus {
            queue_stats,
            observation,
            decision: self.policy.decide(observation),
        })
    }

    /// Returns the policy decision for the current queue and logical turn.
    ///
    /// # Errors
    ///
    /// Returns an error when queue and tracker pending counts differ.
    pub fn decide_for_queue<T>(
        &self,
        queue: &RemoteFreeQueue<T>,
        current_turn: u64,
    ) -> Result<RemoteFreeDrainDecision, RemoteFreeDrainControllerError> {
        self.status_for_queue(queue, current_turn)
            .map(|status| status.decision)
    }

    /// Returns whether the owner should drain for the current queue and turn.
    ///
    /// # Errors
    ///
    /// Returns an error when queue and tracker pending counts differ.
    pub fn should_drain_queue<T>(
        &self,
        queue: &RemoteFreeQueue<T>,
        current_turn: u64,
    ) -> Result<bool, RemoteFreeDrainControllerError> {
        self.decide_for_queue(queue, current_turn)
            .map(RemoteFreeDrainDecision::should_drain)
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

impl fmt::Display for RemoteFreeDrainControllerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PendingCountMismatch {
                queue_pending_count,
                tracker_pending_count,
            } => write!(
                f,
                "remote free queue has {queue_pending_count} pending items but tracker has {tracker_pending_count}"
            ),
            Self::Tracker(source) => write!(f, "{source}"),
        }
    }
}

impl std::error::Error for RemoteFreeDrainTrackerError {}

impl std::error::Error for RemoteFreeDrainControllerError {}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    use super::{
        RemoteFreeDrainController, RemoteFreeDrainControllerError, RemoteFreeDrainDecision,
        RemoteFreeDrainObservation, RemoteFreeDrainPolicy, RemoteFreeDrainReason,
        RemoteFreeDrainTracker, RemoteFreeDrainTrackerError,
    };
    use crate::RemoteFreeQueue;

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
    fn remote_free_drain_controller_reports_queue_policy_status() {
        let mut controller = RemoteFreeDrainController::new(
            RemoteFreeDrainPolicy::new()
                .with_max_pending_age(NonZeroU64::new(1).expect("non-zero")),
        );
        let queue = RemoteFreeQueue::new(4, 2).expect("queue");
        let sink = queue.sink();

        sink.enqueue(1).expect("enqueue first");
        sink.enqueue(2).expect("enqueue second");
        controller.record_submit(0, 4096);
        controller.record_submit(0, 8192);

        let status = controller
            .status_for_queue(&queue, 1)
            .expect("controller status");

        assert_eq!(status.queue_stats.pending_count, 2);
        assert_eq!(controller.pending_count(), 2);
        assert_eq!(controller.queued_bytes(), 12_288);
        assert!(!controller.is_empty());
        assert_eq!(
            status.observation,
            RemoteFreeDrainObservation::new(2, 12_288, 1)
        );
        assert_eq!(
            status.decision,
            RemoteFreeDrainDecision::Drain(RemoteFreeDrainReason::PendingAge)
        );
        assert!(controller
            .should_drain_queue(&queue, 1)
            .expect("should drain"));
    }

    #[test]
    fn remote_free_drain_controller_rejects_pending_count_drift() {
        let controller = RemoteFreeDrainController::new(RemoteFreeDrainPolicy::new());
        let queue = RemoteFreeQueue::new(4, 2).expect("queue");
        let sink = queue.sink();

        sink.enqueue(1).expect("enqueue first");

        assert_eq!(
            controller
                .status_for_queue(&queue, 0)
                .expect_err("pending counts should differ"),
            RemoteFreeDrainControllerError::PendingCountMismatch {
                queue_pending_count: 1,
                tracker_pending_count: 0,
            }
        );
    }

    #[test]
    fn remote_free_drain_controller_records_queue_drains() {
        let mut controller = RemoteFreeDrainController::new(RemoteFreeDrainPolicy::new());
        let mut queue = RemoteFreeQueue::new(4, 2).expect("queue");
        let sink = queue.sink();

        sink.enqueue(1).expect("enqueue first");
        sink.enqueue(2).expect("enqueue second");
        controller.record_submit(0, 4096);
        controller.record_submit(0, 8192);

        let mut released = Vec::new();
        let drain_stats = queue.drain_batch(|item| {
            let tracked = controller.record_drain(4096 * item).expect("tracked drain");
            released.push((item, tracked.submit_turn, tracked.released_bytes));
        });

        assert_eq!(drain_stats.drained, 2);
        assert_eq!(released, vec![(1, 0, 4096), (2, 0, 8192)]);
        assert!(controller.is_empty());
        assert_eq!(
            controller
                .decide_for_queue(&queue, 1)
                .expect("empty decision"),
            RemoteFreeDrainDecision::Defer
        );
    }

    #[test]
    fn remote_free_drain_controller_owner_loop_releases_allocated_buffers() {
        let mut controller = RemoteFreeDrainController::new(
            RemoteFreeDrainPolicy::new()
                .with_max_pending_age(NonZeroU64::new(2).expect("non-zero")),
        );
        let mut queue = RemoteFreeQueue::new(4, 2).expect("queue");
        let sink = queue.sink();

        sink.enqueue(vec![0_u8; 4096]).expect("enqueue first");
        controller.record_submit(0, 4096);
        sink.enqueue(vec![0_u8; 8192]).expect("enqueue second");
        controller.record_submit(1, 8192);

        let policy_status = controller
            .status_for_queue(&queue, 2)
            .expect("controller status");
        assert_eq!(
            policy_status.decision,
            RemoteFreeDrainDecision::Drain(RemoteFreeDrainReason::PendingAge)
        );

        let mut released_bytes = 0_u64;
        let drain_stats = queue.drain_batch(|buffer| {
            let buffer_len = u64::try_from(buffer.len()).expect("buffer length fits u64");
            let tracked = controller.record_drain(buffer_len).expect("tracked drain");
            assert_eq!(tracked.released_bytes, buffer_len);
            released_bytes = released_bytes.saturating_add(buffer_len);
        });

        assert_eq!(drain_stats.drained, 2);
        assert_eq!(released_bytes, 12_288);
        assert_eq!(
            controller
                .status_for_queue(&queue, 3)
                .expect("empty status")
                .decision,
            RemoteFreeDrainDecision::Defer
        );
        assert!(controller.is_empty());
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
