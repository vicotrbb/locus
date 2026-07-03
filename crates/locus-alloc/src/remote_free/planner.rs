use super::{RemoteFreeQueuedByteRetuneAction, RemoteFreeServiceRetuneSummary};

/// Non-mutating service-level remote-free candidate to benchmark next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteFreeServiceRetuneCandidate {
    /// No service telemetry has been observed yet.
    CollectTelemetry,
    /// Keep the current config.
    KeepConfig,
    /// Benchmark a larger queue capacity while preserving the retained window.
    IncreaseQueueCapacity,
    /// Benchmark earlier owner-side drains using the current retained window.
    DrainEarlier,
    /// Recheck workload size shape or byte budget before changing cadence.
    ReviewQueuedByteBudget,
    /// Benchmark a larger queue capacity paired with earlier owner-side drains.
    IncreaseQueueCapacityAndDrainEarlier,
}

impl RemoteFreeServiceRetuneCandidate {
    /// Builds a non-mutating benchmark candidate from service telemetry.
    #[must_use]
    pub const fn from_summary(summary: RemoteFreeServiceRetuneSummary) -> Self {
        if summary.observed_reports() == 0 {
            return Self::CollectTelemetry;
        }
        if !summary.needs_retuning() {
            return Self::KeepConfig;
        }

        let counts = summary.action_counts();
        if counts.count(RemoteFreeQueuedByteRetuneAction::IncreaseQueueCapacityAndDrainEarlier) > 0
        {
            return Self::IncreaseQueueCapacityAndDrainEarlier;
        }
        if counts.count(RemoteFreeQueuedByteRetuneAction::DrainEarlier) > 0 {
            return Self::DrainEarlier;
        }
        if counts.count(RemoteFreeQueuedByteRetuneAction::ReviewQueuedByteBudget) > 0 {
            return Self::ReviewQueuedByteBudget;
        }
        if counts.count(RemoteFreeQueuedByteRetuneAction::IncreaseQueueCapacity) > 0 {
            return Self::IncreaseQueueCapacity;
        }

        Self::KeepConfig
    }

    /// Returns a stable label for logs and benchmark output.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CollectTelemetry => "collect_telemetry",
            Self::KeepConfig => "keep_config",
            Self::IncreaseQueueCapacity => "increase_queue_capacity",
            Self::DrainEarlier => "drain_earlier",
            Self::ReviewQueuedByteBudget => "review_queued_byte_budget",
            Self::IncreaseQueueCapacityAndDrainEarlier => {
                "increase_queue_capacity_and_drain_earlier"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RemoteFreeServiceRetuneCandidate;
    use crate::{
        RemoteFreeDrainObservation, RemoteFreeQueueStats, RemoteFreeQueuedByteDrainConfig,
        RemoteFreeQueuedByteDriftReport, RemoteFreeServiceRetuneSummary,
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

    fn summary_from_reports(
        reports: impl IntoIterator<Item = RemoteFreeQueuedByteDriftReport>,
    ) -> RemoteFreeServiceRetuneSummary {
        let mut summary = RemoteFreeServiceRetuneSummary::new();
        for report in reports {
            summary.observe_report(report);
        }
        summary
    }

    #[test]
    fn planner_distinguishes_empty_and_clean_telemetry() {
        assert_eq!(
            RemoteFreeServiceRetuneCandidate::from_summary(RemoteFreeServiceRetuneSummary::new()),
            RemoteFreeServiceRetuneCandidate::CollectTelemetry
        );

        let clean = summary_from_reports([report(64, 262_144, 0)]);
        assert_eq!(
            RemoteFreeServiceRetuneCandidate::from_summary(clean),
            RemoteFreeServiceRetuneCandidate::KeepConfig
        );
    }

    #[test]
    fn planner_selects_capacity_for_backpressure_only() {
        let summary = summary_from_reports([report(64, 262_144, 3)]);

        assert_eq!(
            RemoteFreeServiceRetuneCandidate::from_summary(summary),
            RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacity
        );
    }

    #[test]
    fn planner_selects_earlier_drains_for_retained_window_drift() {
        let summary = summary_from_reports([report(96, 524_288, 0)]);

        assert_eq!(
            RemoteFreeServiceRetuneCandidate::from_summary(summary),
            RemoteFreeServiceRetuneCandidate::DrainEarlier
        );
    }

    #[test]
    fn planner_selects_budget_review_for_byte_shape_drift() {
        let summary = summary_from_reports([report(32, 524_288, 0)]);

        assert_eq!(
            RemoteFreeServiceRetuneCandidate::from_summary(summary),
            RemoteFreeServiceRetuneCandidate::ReviewQueuedByteBudget
        );
    }

    #[test]
    fn planner_prioritizes_combined_capacity_and_drain_candidate() {
        let summary = summary_from_reports([report(96, 524_288, 3), report(96, 524_288, 0)]);

        assert_eq!(
            RemoteFreeServiceRetuneCandidate::from_summary(summary),
            RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier
        );
    }

    #[test]
    fn planner_exposes_stable_labels() {
        assert_eq!(
            RemoteFreeServiceRetuneCandidate::CollectTelemetry.as_str(),
            "collect_telemetry"
        );
        assert_eq!(
            RemoteFreeServiceRetuneCandidate::KeepConfig.as_str(),
            "keep_config"
        );
        assert_eq!(
            RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacity.as_str(),
            "increase_queue_capacity"
        );
        assert_eq!(
            RemoteFreeServiceRetuneCandidate::DrainEarlier.as_str(),
            "drain_earlier"
        );
        assert_eq!(
            RemoteFreeServiceRetuneCandidate::ReviewQueuedByteBudget.as_str(),
            "review_queued_byte_budget"
        );
        assert_eq!(
            RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier.as_str(),
            "increase_queue_capacity_and_drain_earlier"
        );
    }
}
