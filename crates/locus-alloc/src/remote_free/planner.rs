use std::fmt;
use std::num::NonZeroU64;

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

    const fn is_dry_run_actionable(self) -> bool {
        matches!(
            self,
            Self::IncreaseQueueCapacity
                | Self::DrainEarlier
                | Self::IncreaseQueueCapacityAndDrainEarlier
        )
    }
}

/// Non-mutating dry-run planner over consecutive service telemetry windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteFreeServiceRetuneDryRunPlanner {
    required_stable_windows: NonZeroU64,
    observed_windows: u64,
    current_candidate: RemoteFreeServiceRetuneCandidate,
    consecutive_candidate_windows: u64,
    would_apply_candidate: Option<RemoteFreeServiceRetuneCandidate>,
}

/// Failure to build a dry-run service retune planner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteFreeServiceRetuneDryRunPlannerError {
    /// Stable window count was zero.
    ZeroRequiredStableWindows,
}

impl RemoteFreeServiceRetuneDryRunPlanner {
    /// Creates a dry-run service planner from a non-zero stability window.
    #[must_use]
    pub const fn new(required_stable_windows: NonZeroU64) -> Self {
        Self {
            required_stable_windows,
            observed_windows: 0,
            current_candidate: RemoteFreeServiceRetuneCandidate::CollectTelemetry,
            consecutive_candidate_windows: 0,
            would_apply_candidate: None,
        }
    }

    /// Creates a dry-run service planner from a raw stability window count.
    ///
    /// # Errors
    ///
    /// Returns an error when `required_stable_windows` is zero.
    pub fn try_new(
        required_stable_windows: u64,
    ) -> Result<Self, RemoteFreeServiceRetuneDryRunPlannerError> {
        let required_stable_windows = NonZeroU64::new(required_stable_windows)
            .ok_or(RemoteFreeServiceRetuneDryRunPlannerError::ZeroRequiredStableWindows)?;

        Ok(Self::new(required_stable_windows))
    }

    /// Observes one service window and returns the selected candidate.
    pub fn observe_summary(
        &mut self,
        summary: RemoteFreeServiceRetuneSummary,
    ) -> RemoteFreeServiceRetuneCandidate {
        let candidate = RemoteFreeServiceRetuneCandidate::from_summary(summary);
        self.observe_candidate(candidate);
        candidate
    }

    /// Returns how many consecutive matching windows are required.
    #[must_use]
    pub const fn required_stable_windows(self) -> u64 {
        self.required_stable_windows.get()
    }

    /// Returns how many service windows have been observed.
    #[must_use]
    pub const fn observed_windows(self) -> u64 {
        self.observed_windows
    }

    /// Returns the candidate from the latest observed service window.
    #[must_use]
    pub const fn current_candidate(self) -> RemoteFreeServiceRetuneCandidate {
        self.current_candidate
    }

    /// Returns the current consecutive actionable-candidate streak.
    #[must_use]
    pub const fn consecutive_candidate_windows(self) -> u64 {
        self.consecutive_candidate_windows
    }

    /// Returns the candidate that would be applied by a future adaptive policy.
    ///
    /// This planner never mutates policy. A candidate is returned only when the
    /// same actionable candidate has appeared for the configured stability
    /// window.
    #[must_use]
    pub const fn would_apply_candidate(self) -> Option<RemoteFreeServiceRetuneCandidate> {
        self.would_apply_candidate
    }

    fn observe_candidate(&mut self, candidate: RemoteFreeServiceRetuneCandidate) {
        self.observed_windows = self.observed_windows.saturating_add(1);

        if !candidate.is_dry_run_actionable() {
            self.current_candidate = candidate;
            self.consecutive_candidate_windows = 0;
            self.would_apply_candidate = None;
            return;
        }

        if self.current_candidate == candidate {
            self.consecutive_candidate_windows =
                self.consecutive_candidate_windows.saturating_add(1);
        } else {
            self.current_candidate = candidate;
            self.consecutive_candidate_windows = 1;
        }

        self.would_apply_candidate =
            if self.consecutive_candidate_windows >= self.required_stable_windows.get() {
                Some(candidate)
            } else {
                None
            };
    }
}

impl fmt::Display for RemoteFreeServiceRetuneDryRunPlannerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroRequiredStableWindows => {
                f.write_str("remote free dry-run stable window count must be non-zero")
            }
        }
    }
}

impl std::error::Error for RemoteFreeServiceRetuneDryRunPlannerError {}

#[cfg(test)]
mod tests {
    use super::{RemoteFreeServiceRetuneCandidate, RemoteFreeServiceRetuneDryRunPlanner};
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

    #[test]
    fn dry_run_planner_rejects_zero_stability_window() {
        assert!(RemoteFreeServiceRetuneDryRunPlanner::try_new(0).is_err());
    }

    #[test]
    fn dry_run_planner_requires_consecutive_actionable_windows() {
        let mut planner = RemoteFreeServiceRetuneDryRunPlanner::try_new(2).expect("planner");

        assert_eq!(planner.required_stable_windows(), 2);
        assert_eq!(
            planner.observe_summary(summary_from_reports([report(64, 262_144, 0)])),
            RemoteFreeServiceRetuneCandidate::KeepConfig
        );
        assert_eq!(planner.observed_windows(), 1);
        assert_eq!(planner.consecutive_candidate_windows(), 0);
        assert_eq!(planner.would_apply_candidate(), None);

        let drain_earlier = summary_from_reports([report(96, 524_288, 0)]);
        assert_eq!(
            planner.observe_summary(drain_earlier),
            RemoteFreeServiceRetuneCandidate::DrainEarlier
        );
        assert_eq!(planner.consecutive_candidate_windows(), 1);
        assert_eq!(planner.would_apply_candidate(), None);

        assert_eq!(
            planner.observe_summary(drain_earlier),
            RemoteFreeServiceRetuneCandidate::DrainEarlier
        );
        assert_eq!(planner.consecutive_candidate_windows(), 2);
        assert_eq!(
            planner.would_apply_candidate(),
            Some(RemoteFreeServiceRetuneCandidate::DrainEarlier)
        );
    }

    #[test]
    fn dry_run_planner_resets_streak_on_candidate_change() {
        let mut planner = RemoteFreeServiceRetuneDryRunPlanner::try_new(2).expect("planner");

        planner.observe_summary(summary_from_reports([report(96, 524_288, 0)]));
        assert_eq!(planner.consecutive_candidate_windows(), 1);

        planner.observe_summary(summary_from_reports([report(96, 524_288, 3)]));
        assert_eq!(
            planner.current_candidate(),
            RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier
        );
        assert_eq!(planner.consecutive_candidate_windows(), 1);
        assert_eq!(planner.would_apply_candidate(), None);
    }

    #[test]
    fn dry_run_planner_resets_on_clean_telemetry() {
        let mut planner = RemoteFreeServiceRetuneDryRunPlanner::try_new(2).expect("planner");
        let drain_earlier = summary_from_reports([report(96, 524_288, 0)]);

        planner.observe_summary(drain_earlier);
        planner.observe_summary(drain_earlier);
        assert_eq!(
            planner.would_apply_candidate(),
            Some(RemoteFreeServiceRetuneCandidate::DrainEarlier)
        );

        planner.observe_summary(summary_from_reports([report(64, 262_144, 0)]));
        assert_eq!(
            planner.current_candidate(),
            RemoteFreeServiceRetuneCandidate::KeepConfig
        );
        assert_eq!(planner.consecutive_candidate_windows(), 0);
        assert_eq!(planner.would_apply_candidate(), None);
    }

    #[test]
    fn dry_run_planner_resets_on_budget_review_candidate() {
        let mut planner = RemoteFreeServiceRetuneDryRunPlanner::try_new(2).expect("planner");

        assert_eq!(
            planner.observe_summary(summary_from_reports([report(32, 524_288, 0)])),
            RemoteFreeServiceRetuneCandidate::ReviewQueuedByteBudget
        );
        assert_eq!(planner.consecutive_candidate_windows(), 0);
        assert_eq!(planner.would_apply_candidate(), None);
    }
}
