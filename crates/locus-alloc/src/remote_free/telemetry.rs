use super::{RemoteFreeQueuedByteDriftReport, RemoteFreeQueuedByteRetuneAction};

/// Service-level counts for queued-byte remote-free retune actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RemoteFreeRetuneActionCounts {
    keep_config: u64,
    increase_queue_capacity: u64,
    drain_earlier: u64,
    review_queued_byte_budget: u64,
    increase_queue_capacity_and_drain_earlier: u64,
}

/// Service-level summary of queued-byte remote-free drift telemetry.
///
/// The summary is diagnostic only. It aggregates reports across owner loops and
/// does not mutate queue capacity, drain cadence, or queued-byte budgets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RemoteFreeServiceRetuneSummary {
    observed_reports: u64,
    reports_needing_retune: u64,
    max_pending_items_over_target: u64,
    max_queued_bytes_over_budget: u64,
    queue_backpressure_reports: u64,
    action_counts: RemoteFreeRetuneActionCounts,
}

impl RemoteFreeRetuneActionCounts {
    /// Creates empty action counts.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            keep_config: 0,
            increase_queue_capacity: 0,
            drain_earlier: 0,
            review_queued_byte_budget: 0,
            increase_queue_capacity_and_drain_earlier: 0,
        }
    }

    /// Returns the count for one retune action.
    #[must_use]
    pub const fn count(self, action: RemoteFreeQueuedByteRetuneAction) -> u64 {
        match action {
            RemoteFreeQueuedByteRetuneAction::KeepConfig => self.keep_config,
            RemoteFreeQueuedByteRetuneAction::IncreaseQueueCapacity => self.increase_queue_capacity,
            RemoteFreeQueuedByteRetuneAction::DrainEarlier => self.drain_earlier,
            RemoteFreeQueuedByteRetuneAction::ReviewQueuedByteBudget => {
                self.review_queued_byte_budget
            }
            RemoteFreeQueuedByteRetuneAction::IncreaseQueueCapacityAndDrainEarlier => {
                self.increase_queue_capacity_and_drain_earlier
            }
        }
    }

    fn observe(&mut self, action: RemoteFreeQueuedByteRetuneAction) {
        match action {
            RemoteFreeQueuedByteRetuneAction::KeepConfig => {
                self.keep_config = self.keep_config.saturating_add(1);
            }
            RemoteFreeQueuedByteRetuneAction::IncreaseQueueCapacity => {
                self.increase_queue_capacity = self.increase_queue_capacity.saturating_add(1);
            }
            RemoteFreeQueuedByteRetuneAction::DrainEarlier => {
                self.drain_earlier = self.drain_earlier.saturating_add(1);
            }
            RemoteFreeQueuedByteRetuneAction::ReviewQueuedByteBudget => {
                self.review_queued_byte_budget = self.review_queued_byte_budget.saturating_add(1);
            }
            RemoteFreeQueuedByteRetuneAction::IncreaseQueueCapacityAndDrainEarlier => {
                self.increase_queue_capacity_and_drain_earlier = self
                    .increase_queue_capacity_and_drain_earlier
                    .saturating_add(1);
            }
        }
    }
}

impl RemoteFreeServiceRetuneSummary {
    /// Creates an empty service retune summary.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            observed_reports: 0,
            reports_needing_retune: 0,
            max_pending_items_over_target: 0,
            max_queued_bytes_over_budget: 0,
            queue_backpressure_reports: 0,
            action_counts: RemoteFreeRetuneActionCounts::new(),
        }
    }

    /// Observes one owner-loop queued-byte drift report.
    pub fn observe_report(&mut self, report: RemoteFreeQueuedByteDriftReport) {
        self.observed_reports = self.observed_reports.saturating_add(1);
        if report.needs_retuning() {
            self.reports_needing_retune = self.reports_needing_retune.saturating_add(1);
        }
        if report.has_queue_backpressure() {
            self.queue_backpressure_reports = self.queue_backpressure_reports.saturating_add(1);
        }

        self.max_pending_items_over_target = self
            .max_pending_items_over_target
            .max(report.pending_items_over_target());
        self.max_queued_bytes_over_budget = self
            .max_queued_bytes_over_budget
            .max(report.queued_bytes_over_budget());
        self.action_counts.observe(report.retune_action());
    }

    /// Returns how many owner-loop reports were observed.
    #[must_use]
    pub const fn observed_reports(self) -> u64 {
        self.observed_reports
    }

    /// Returns how many observed reports had any retune signal.
    #[must_use]
    pub const fn reports_needing_retune(self) -> u64 {
        self.reports_needing_retune
    }

    /// Returns the maximum pending-item drift across observed reports.
    #[must_use]
    pub const fn max_pending_items_over_target(self) -> u64 {
        self.max_pending_items_over_target
    }

    /// Returns the maximum queued-byte drift across observed reports.
    #[must_use]
    pub const fn max_queued_bytes_over_budget(self) -> u64 {
        self.max_queued_bytes_over_budget
    }

    /// Returns how many observed reports included queue backpressure.
    #[must_use]
    pub const fn queue_backpressure_reports(self) -> u64 {
        self.queue_backpressure_reports
    }

    /// Returns counts for each observed retune action.
    #[must_use]
    pub const fn action_counts(self) -> RemoteFreeRetuneActionCounts {
        self.action_counts
    }

    /// Returns true when at least one observed report needs retuning.
    #[must_use]
    pub const fn needs_retuning(self) -> bool {
        self.reports_needing_retune > 0
    }
}

#[cfg(test)]
mod tests {
    use super::{RemoteFreeRetuneActionCounts, RemoteFreeServiceRetuneSummary};
    use crate::{
        RemoteFreeDrainObservation, RemoteFreeQueueStats, RemoteFreeQueuedByteDrainConfig,
        RemoteFreeQueuedByteDriftReport, RemoteFreeQueuedByteRetuneAction,
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

    fn report(
        pending_count: u64,
        queued_bytes: u64,
        full_count: u64,
    ) -> RemoteFreeQueuedByteDriftReport {
        RemoteFreeQueuedByteDriftReport::from_observation(
            config(),
            queue_stats(full_count),
            RemoteFreeDrainObservation::new(pending_count, queued_bytes, 1),
        )
    }

    #[test]
    fn retune_action_counts_start_empty() {
        let counts = RemoteFreeRetuneActionCounts::new();

        assert_eq!(
            counts.count(RemoteFreeQueuedByteRetuneAction::KeepConfig),
            0
        );
        assert_eq!(
            counts.count(RemoteFreeQueuedByteRetuneAction::IncreaseQueueCapacity),
            0
        );
        assert_eq!(
            counts.count(RemoteFreeQueuedByteRetuneAction::DrainEarlier),
            0
        );
        assert_eq!(
            counts.count(RemoteFreeQueuedByteRetuneAction::ReviewQueuedByteBudget),
            0
        );
        assert_eq!(
            counts.count(RemoteFreeQueuedByteRetuneAction::IncreaseQueueCapacityAndDrainEarlier),
            0
        );
    }

    #[test]
    fn service_retune_summary_counts_actions_and_maxima() {
        let mut summary = RemoteFreeServiceRetuneSummary::new();

        summary.observe_report(report(64, 262_144, 0));
        summary.observe_report(report(96, 524_288, 0));
        summary.observe_report(report(96, 524_288, 2));

        let counts = summary.action_counts();
        assert_eq!(summary.observed_reports(), 3);
        assert_eq!(summary.reports_needing_retune(), 2);
        assert!(summary.needs_retuning());
        assert_eq!(summary.max_pending_items_over_target(), 32);
        assert_eq!(summary.max_queued_bytes_over_budget(), 262_144);
        assert_eq!(summary.queue_backpressure_reports(), 1);
        assert_eq!(
            counts.count(RemoteFreeQueuedByteRetuneAction::KeepConfig),
            1
        );
        assert_eq!(
            counts.count(RemoteFreeQueuedByteRetuneAction::DrainEarlier),
            1
        );
        assert_eq!(
            counts.count(RemoteFreeQueuedByteRetuneAction::IncreaseQueueCapacityAndDrainEarlier),
            1
        );
    }

    #[test]
    fn service_retune_summary_reports_clean_service() {
        let mut summary = RemoteFreeServiceRetuneSummary::new();

        summary.observe_report(report(32, 131_072, 0));
        summary.observe_report(report(64, 262_144, 0));

        assert_eq!(summary.observed_reports(), 2);
        assert_eq!(summary.reports_needing_retune(), 0);
        assert!(!summary.needs_retuning());
        assert_eq!(
            summary
                .action_counts()
                .count(RemoteFreeQueuedByteRetuneAction::KeepConfig),
            2
        );
    }
}
