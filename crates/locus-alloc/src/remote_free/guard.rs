use std::fmt;
use std::num::NonZeroU64;

use super::{
    RemoteFreeServiceRetuneCandidate, RemoteFreeServiceRetuneDryRunPlanner,
    RemoteFreeServiceRetuneSummary,
};

/// Guarded service retune decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteFreeServiceRetuneGuardDecision {
    /// No telemetry has been observed yet.
    CollectTelemetry,
    /// Keep observing without applying a candidate.
    Hold {
        /// Current candidate selected by service telemetry.
        candidate: RemoteFreeServiceRetuneCandidate,
        /// Current consecutive actionable-candidate streak.
        consecutive_candidate_windows: u64,
    },
    /// Caller may apply the candidate and validate the next service window.
    Apply {
        /// Candidate to apply.
        candidate: RemoteFreeServiceRetuneCandidate,
    },
    /// Pending candidate was validated by a clean follow-up window.
    Confirmed {
        /// Candidate that was confirmed.
        candidate: RemoteFreeServiceRetuneCandidate,
    },
    /// Pending candidate failed validation and should be rolled back.
    Rollback {
        /// Candidate that failed validation.
        candidate: RemoteFreeServiceRetuneCandidate,
        /// Candidate observed during the failed validation window.
        observed_candidate: RemoteFreeServiceRetuneCandidate,
    },
    /// Candidate was stable but the guard has exhausted its mutation budget.
    MutationLimitReached {
        /// Candidate that would otherwise have been applied.
        candidate: RemoteFreeServiceRetuneCandidate,
    },
}

/// Guarded non-mutating bridge from service telemetry to explicit retune plans.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteFreeServiceRetuneGuard {
    dry_run: RemoteFreeServiceRetuneDryRunPlanner,
    max_mutations: NonZeroU64,
    applied_mutations: u64,
    confirmed_mutations: u64,
    rollbacks: u64,
    pending_validation: Option<RemoteFreeServiceRetuneCandidate>,
}

/// Failure to build a guarded service retune planner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteFreeServiceRetuneGuardError {
    /// Stable window count was zero.
    ZeroRequiredStableWindows,
    /// Mutation cap was zero.
    ZeroMaxMutations,
}

impl RemoteFreeServiceRetuneGuard {
    /// Creates a guarded service retune planner.
    #[must_use]
    pub const fn new(required_stable_windows: NonZeroU64, max_mutations: NonZeroU64) -> Self {
        Self {
            dry_run: RemoteFreeServiceRetuneDryRunPlanner::new(required_stable_windows),
            max_mutations,
            applied_mutations: 0,
            confirmed_mutations: 0,
            rollbacks: 0,
            pending_validation: None,
        }
    }

    /// Creates a guarded service retune planner from raw counts.
    ///
    /// # Errors
    ///
    /// Returns an error when either count is zero.
    pub fn try_new(
        required_stable_windows: u64,
        max_mutations: u64,
    ) -> Result<Self, RemoteFreeServiceRetuneGuardError> {
        let required_stable_windows = NonZeroU64::new(required_stable_windows)
            .ok_or(RemoteFreeServiceRetuneGuardError::ZeroRequiredStableWindows)?;
        let max_mutations = NonZeroU64::new(max_mutations)
            .ok_or(RemoteFreeServiceRetuneGuardError::ZeroMaxMutations)?;

        Ok(Self::new(required_stable_windows, max_mutations))
    }

    /// Observes one service window and returns the guarded retune decision.
    pub fn observe_summary(
        &mut self,
        summary: RemoteFreeServiceRetuneSummary,
    ) -> RemoteFreeServiceRetuneGuardDecision {
        let candidate = self.dry_run.observe_summary(summary);

        if let Some(pending_candidate) = self.pending_validation.take() {
            if summary.needs_retuning() {
                self.rollbacks = self.rollbacks.saturating_add(1);
                self.dry_run.reset();
                return RemoteFreeServiceRetuneGuardDecision::Rollback {
                    candidate: pending_candidate,
                    observed_candidate: candidate,
                };
            }

            self.confirmed_mutations = self.confirmed_mutations.saturating_add(1);
            return RemoteFreeServiceRetuneGuardDecision::Confirmed {
                candidate: pending_candidate,
            };
        }

        match self.dry_run.would_apply_candidate() {
            Some(candidate) if self.applied_mutations < self.max_mutations.get() => {
                self.applied_mutations = self.applied_mutations.saturating_add(1);
                self.pending_validation = Some(candidate);
                self.dry_run.reset();
                RemoteFreeServiceRetuneGuardDecision::Apply { candidate }
            }
            Some(candidate) => {
                RemoteFreeServiceRetuneGuardDecision::MutationLimitReached { candidate }
            }
            None if candidate == RemoteFreeServiceRetuneCandidate::CollectTelemetry => {
                RemoteFreeServiceRetuneGuardDecision::CollectTelemetry
            }
            None => RemoteFreeServiceRetuneGuardDecision::Hold {
                candidate,
                consecutive_candidate_windows: self.dry_run.consecutive_candidate_windows(),
            },
        }
    }

    /// Returns how many apply decisions have been emitted.
    #[must_use]
    pub const fn applied_mutations(self) -> u64 {
        self.applied_mutations
    }

    /// Returns how many applied candidates were confirmed.
    #[must_use]
    pub const fn confirmed_mutations(self) -> u64 {
        self.confirmed_mutations
    }

    /// Returns how many applied candidates were rolled back.
    #[must_use]
    pub const fn rollbacks(self) -> u64 {
        self.rollbacks
    }

    /// Returns the candidate waiting for validation, if any.
    #[must_use]
    pub const fn pending_validation(self) -> Option<RemoteFreeServiceRetuneCandidate> {
        self.pending_validation
    }

    /// Returns the configured mutation limit.
    #[must_use]
    pub const fn max_mutations(self) -> u64 {
        self.max_mutations.get()
    }
}

impl RemoteFreeServiceRetuneGuardDecision {
    /// Returns a stable label for logs and benchmark output.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CollectTelemetry => "collect_telemetry",
            Self::Hold { .. } => "hold",
            Self::Apply { .. } => "apply",
            Self::Confirmed { .. } => "confirmed",
            Self::Rollback { .. } => "rollback",
            Self::MutationLimitReached { .. } => "mutation_limit_reached",
        }
    }
}

impl fmt::Display for RemoteFreeServiceRetuneGuardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroRequiredStableWindows => {
                f.write_str("remote free guard stable window count must be non-zero")
            }
            Self::ZeroMaxMutations => {
                f.write_str("remote free guard mutation limit must be non-zero")
            }
        }
    }
}

impl std::error::Error for RemoteFreeServiceRetuneGuardError {}

#[cfg(test)]
mod tests {
    use super::{RemoteFreeServiceRetuneGuard, RemoteFreeServiceRetuneGuardDecision};
    use crate::{
        RemoteFreeDrainObservation, RemoteFreeQueueStats, RemoteFreeQueuedByteDrainConfig,
        RemoteFreeQueuedByteDriftReport, RemoteFreeServiceRetuneCandidate,
        RemoteFreeServiceRetuneSummary,
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
    fn guarded_planner_rejects_zero_config_values() {
        assert!(RemoteFreeServiceRetuneGuard::try_new(0, 1).is_err());
        assert!(RemoteFreeServiceRetuneGuard::try_new(2, 0).is_err());
    }

    #[test]
    fn guarded_planner_applies_after_stable_candidate_and_confirms_clean_window() {
        let mut guard = RemoteFreeServiceRetuneGuard::try_new(2, 1).expect("guard");
        let drain_earlier = summary_from_reports([report(96, 524_288, 0)]);

        assert_eq!(
            guard.observe_summary(drain_earlier),
            RemoteFreeServiceRetuneGuardDecision::Hold {
                candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
                consecutive_candidate_windows: 1,
            }
        );
        assert_eq!(
            guard.observe_summary(drain_earlier),
            RemoteFreeServiceRetuneGuardDecision::Apply {
                candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
            }
        );
        assert_eq!(guard.applied_mutations(), 1);
        assert_eq!(
            guard.pending_validation(),
            Some(RemoteFreeServiceRetuneCandidate::DrainEarlier)
        );

        assert_eq!(
            guard.observe_summary(summary_from_reports([report(64, 262_144, 0)])),
            RemoteFreeServiceRetuneGuardDecision::Confirmed {
                candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
            }
        );
        assert_eq!(guard.confirmed_mutations(), 1);
        assert_eq!(guard.rollbacks(), 0);
        assert_eq!(guard.pending_validation(), None);
    }

    #[test]
    fn guarded_planner_rolls_back_failed_validation_window() {
        let mut guard = RemoteFreeServiceRetuneGuard::try_new(2, 1).expect("guard");
        let drain_earlier = summary_from_reports([report(96, 524_288, 0)]);

        guard.observe_summary(drain_earlier);
        guard.observe_summary(drain_earlier);

        assert_eq!(
            guard.observe_summary(summary_from_reports([report(96, 524_288, 3)])),
            RemoteFreeServiceRetuneGuardDecision::Rollback {
                candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
                observed_candidate:
                    RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier,
            }
        );
        assert_eq!(guard.applied_mutations(), 1);
        assert_eq!(guard.confirmed_mutations(), 0);
        assert_eq!(guard.rollbacks(), 1);
        assert_eq!(guard.pending_validation(), None);
    }

    #[test]
    fn guarded_planner_enforces_mutation_limit() {
        let mut guard = RemoteFreeServiceRetuneGuard::try_new(2, 1).expect("guard");
        let drain_earlier = summary_from_reports([report(96, 524_288, 0)]);

        guard.observe_summary(drain_earlier);
        guard.observe_summary(drain_earlier);
        guard.observe_summary(summary_from_reports([report(64, 262_144, 0)]));

        guard.observe_summary(drain_earlier);
        assert_eq!(
            guard.observe_summary(drain_earlier),
            RemoteFreeServiceRetuneGuardDecision::MutationLimitReached {
                candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
            }
        );
        assert_eq!(guard.applied_mutations(), 1);
    }

    #[test]
    fn guarded_decisions_expose_stable_labels() {
        assert_eq!(
            RemoteFreeServiceRetuneGuardDecision::CollectTelemetry.as_str(),
            "collect_telemetry"
        );
        assert_eq!(
            RemoteFreeServiceRetuneGuardDecision::Hold {
                candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
                consecutive_candidate_windows: 1,
            }
            .as_str(),
            "hold"
        );
        assert_eq!(
            RemoteFreeServiceRetuneGuardDecision::Apply {
                candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
            }
            .as_str(),
            "apply"
        );
        assert_eq!(
            RemoteFreeServiceRetuneGuardDecision::Confirmed {
                candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
            }
            .as_str(),
            "confirmed"
        );
        assert_eq!(
            RemoteFreeServiceRetuneGuardDecision::Rollback {
                candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
                observed_candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
            }
            .as_str(),
            "rollback"
        );
        assert_eq!(
            RemoteFreeServiceRetuneGuardDecision::MutationLimitReached {
                candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
            }
            .as_str(),
            "mutation_limit_reached"
        );
    }
}
