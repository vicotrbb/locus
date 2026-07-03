use super::{
    RemoteFreeDrainControllerStatus, RemoteFreeDrainObservation, RemoteFreeQueueStats,
    RemoteFreeQueuedByteDrainConfig,
};

/// Drift between a queued-byte drain config and live remote-free observations.
///
/// The report is diagnostic only. It does not mutate drain policy and it treats
/// queue `full_count` as a cumulative backpressure signal from nonblocking
/// enqueue attempts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteFreeQueuedByteDriftReport {
    target_pending_items: u64,
    queued_byte_budget: u64,
    observed_pending_count: u64,
    observed_queued_bytes: u64,
    observed_full_count: u64,
    pending_items_over_target: u64,
    queued_bytes_over_budget: u64,
}

impl RemoteFreeQueuedByteDriftReport {
    /// Builds a report from a controller status snapshot.
    #[must_use]
    pub fn from_status(
        config: RemoteFreeQueuedByteDrainConfig,
        status: RemoteFreeDrainControllerStatus,
    ) -> Self {
        Self::from_observation(config, status.queue_stats, status.observation)
    }

    /// Builds a report from queue stats and a policy observation.
    #[must_use]
    pub fn from_observation(
        config: RemoteFreeQueuedByteDrainConfig,
        queue_stats: RemoteFreeQueueStats,
        observation: RemoteFreeDrainObservation,
    ) -> Self {
        let target_pending_items = config.target_pending_items();
        let queued_byte_budget = config.queued_byte_budget().bytes();
        let pending_items_over_target = observation
            .pending_count
            .saturating_sub(target_pending_items);
        let queued_bytes_over_budget = observation.queued_bytes.saturating_sub(queued_byte_budget);

        Self {
            target_pending_items,
            queued_byte_budget,
            observed_pending_count: observation.pending_count,
            observed_queued_bytes: observation.queued_bytes,
            observed_full_count: queue_stats.full_count,
            pending_items_over_target,
            queued_bytes_over_budget,
        }
    }

    /// Returns the configured target pending item window.
    #[must_use]
    pub const fn target_pending_items(self) -> u64 {
        self.target_pending_items
    }

    /// Returns the configured queued-byte budget.
    #[must_use]
    pub const fn queued_byte_budget(self) -> u64 {
        self.queued_byte_budget
    }

    /// Returns the observed pending remote-free item count.
    #[must_use]
    pub const fn observed_pending_count(self) -> u64 {
        self.observed_pending_count
    }

    /// Returns the observed retained queued bytes.
    #[must_use]
    pub const fn observed_queued_bytes(self) -> u64 {
        self.observed_queued_bytes
    }

    /// Returns the observed cumulative queue-full count.
    #[must_use]
    pub const fn observed_full_count(self) -> u64 {
        self.observed_full_count
    }

    /// Returns how many observed pending items exceed the configured target.
    #[must_use]
    pub const fn pending_items_over_target(self) -> u64 {
        self.pending_items_over_target
    }

    /// Returns how many observed queued bytes exceed the configured budget.
    #[must_use]
    pub const fn queued_bytes_over_budget(self) -> u64 {
        self.queued_bytes_over_budget
    }

    /// Returns true when observed pending items exceed the configured target.
    #[must_use]
    pub const fn has_pending_item_drift(self) -> bool {
        self.pending_items_over_target > 0
    }

    /// Returns true when observed queued bytes exceed the configured budget.
    #[must_use]
    pub const fn has_queued_byte_drift(self) -> bool {
        self.queued_bytes_over_budget > 0
    }

    /// Returns true when the queue has reported nonblocking enqueue backpressure.
    #[must_use]
    pub const fn has_queue_backpressure(self) -> bool {
        self.observed_full_count > 0
    }

    /// Returns true when any observed signal suggests the config needs review.
    #[must_use]
    pub const fn needs_retuning(self) -> bool {
        self.has_pending_item_drift()
            || self.has_queued_byte_drift()
            || self.has_queue_backpressure()
    }
}

#[cfg(test)]
mod tests {
    use super::RemoteFreeQueuedByteDriftReport;
    use crate::{
        RemoteFreeDrainControllerStatus, RemoteFreeDrainDecision, RemoteFreeDrainObservation,
        RemoteFreeQueueStats, RemoteFreeQueuedByteDrainConfig,
    };

    fn config() -> RemoteFreeQueuedByteDrainConfig {
        RemoteFreeQueuedByteDrainConfig::from_item_shape(256, 64, 64, 4096)
            .expect("queued-byte config")
    }

    fn queue_stats(full_count: u64) -> RemoteFreeQueueStats {
        RemoteFreeQueueStats {
            capacity: 256,
            batch_limit: 64,
            submitted_count: 64,
            pending_count: 64,
            full_count,
            disconnected_count: 0,
            drained_count: 0,
        }
    }

    #[test]
    fn queued_byte_drift_report_accepts_matching_config_window() {
        let report = RemoteFreeQueuedByteDriftReport::from_observation(
            config(),
            queue_stats(0),
            RemoteFreeDrainObservation::new(64, 262_144, 1),
        );

        assert_eq!(report.target_pending_items(), 64);
        assert_eq!(report.queued_byte_budget(), 262_144);
        assert_eq!(report.observed_pending_count(), 64);
        assert_eq!(report.observed_queued_bytes(), 262_144);
        assert_eq!(report.observed_full_count(), 0);
        assert_eq!(report.pending_items_over_target(), 0);
        assert_eq!(report.queued_bytes_over_budget(), 0);
        assert!(!report.has_pending_item_drift());
        assert!(!report.has_queued_byte_drift());
        assert!(!report.has_queue_backpressure());
        assert!(!report.needs_retuning());
    }

    #[test]
    fn queued_byte_drift_report_separates_pending_and_byte_drift() {
        let pending_only = RemoteFreeQueuedByteDriftReport::from_observation(
            config(),
            queue_stats(0),
            RemoteFreeDrainObservation::new(80, 262_144, 1),
        );
        let bytes_only = RemoteFreeQueuedByteDriftReport::from_observation(
            config(),
            queue_stats(0),
            RemoteFreeDrainObservation::new(32, 300_000, 1),
        );

        assert_eq!(pending_only.pending_items_over_target(), 16);
        assert_eq!(pending_only.queued_bytes_over_budget(), 0);
        assert!(pending_only.has_pending_item_drift());
        assert!(!pending_only.has_queued_byte_drift());

        assert_eq!(bytes_only.pending_items_over_target(), 0);
        assert_eq!(bytes_only.queued_bytes_over_budget(), 37_856);
        assert!(!bytes_only.has_pending_item_drift());
        assert!(bytes_only.has_queued_byte_drift());
    }

    #[test]
    fn queued_byte_drift_report_marks_queue_backpressure() {
        let report = RemoteFreeQueuedByteDriftReport::from_observation(
            config(),
            queue_stats(3),
            RemoteFreeDrainObservation::new(64, 262_144, 1),
        );

        assert_eq!(report.observed_full_count(), 3);
        assert!(report.has_queue_backpressure());
        assert!(report.needs_retuning());
    }

    #[test]
    fn queued_byte_drift_report_builds_from_controller_status() {
        let status = RemoteFreeDrainControllerStatus {
            queue_stats: queue_stats(2),
            observation: RemoteFreeDrainObservation::new(96, 300_000, 2),
            decision: RemoteFreeDrainDecision::Defer,
        };

        let report = RemoteFreeQueuedByteDriftReport::from_status(config(), status);

        assert_eq!(report.pending_items_over_target(), 32);
        assert_eq!(report.queued_bytes_over_budget(), 37_856);
        assert_eq!(report.observed_full_count(), 2);
        assert!(report.needs_retuning());
    }
}
