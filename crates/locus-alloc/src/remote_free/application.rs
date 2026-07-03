use std::fmt;
use std::num::NonZeroUsize;

use super::{
    RemoteFreeQueuedByteDrainConfig, RemoteFreeQueuedByteDrainConfigError,
    RemoteFreeServiceRetuneCandidate, RemoteFreeServiceRetuneGuardDecision,
};

/// Builds validated policy application plans from guarded retune decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteFreeServiceRetunePolicyApplicator {
    queue_capacity_growth_factor: NonZeroUsize,
}

/// Result of translating a guarded retune decision into an application plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteFreeServiceRetunePolicyApplication {
    /// No live queue or drain-policy config should change.
    NoChange {
        /// Guard decision that produced no live config change.
        decision: RemoteFreeServiceRetuneGuardDecision,
    },
    /// Caller may install the validated config for the candidate.
    Apply {
        /// Candidate represented by the config.
        candidate: RemoteFreeServiceRetuneCandidate,
        /// Validated queue sizing and queued-byte drain policy.
        config: RemoteFreeQueuedByteDrainConfig,
    },
}

/// Failure to build or use a remote-free retune policy applicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteFreeServiceRetunePolicyApplicationError {
    /// Queue-capacity growth factor was zero.
    ZeroQueueCapacityGrowthFactor,
    /// Queue-capacity growth factor was one, which cannot increase capacity.
    QueueCapacityGrowthFactorTooSmall,
    /// Candidate cannot be applied to live policy by this applicator.
    NonActionableApplyCandidate {
        /// Candidate carried by the guarded apply decision.
        candidate: RemoteFreeServiceRetuneCandidate,
    },
    /// Queue-capacity growth overflowed.
    QueueCapacityGrowthOverflow {
        /// Current queue capacity.
        queue_capacity: usize,
        /// Requested capacity growth factor.
        growth_factor: usize,
    },
    /// Building the target config failed validation.
    Config(RemoteFreeQueuedByteDrainConfigError),
}

impl RemoteFreeServiceRetunePolicyApplicator {
    /// Creates a policy applicator from a validated capacity growth factor.
    ///
    /// # Errors
    ///
    /// Returns an error when the growth factor is zero or one.
    pub fn try_new(
        queue_capacity_growth_factor: usize,
    ) -> Result<Self, RemoteFreeServiceRetunePolicyApplicationError> {
        let queue_capacity_growth_factor = NonZeroUsize::new(queue_capacity_growth_factor)
            .ok_or(RemoteFreeServiceRetunePolicyApplicationError::ZeroQueueCapacityGrowthFactor)?;

        if queue_capacity_growth_factor.get() == 1 {
            return Err(
                RemoteFreeServiceRetunePolicyApplicationError::QueueCapacityGrowthFactorTooSmall,
            );
        }

        Ok(Self {
            queue_capacity_growth_factor,
        })
    }

    /// Returns the configured queue-capacity growth factor.
    #[must_use]
    pub const fn queue_capacity_growth_factor(self) -> usize {
        self.queue_capacity_growth_factor.get()
    }

    /// Translates a guarded decision into a validated application plan.
    ///
    /// Only `apply` decisions can produce an applied config. Other decisions
    /// are returned as observable no-change outcomes.
    ///
    /// # Errors
    ///
    /// Returns an error when an apply decision carries a non-actionable
    /// candidate, queue-capacity growth overflows, or the target config fails
    /// validation.
    pub fn plan(
        self,
        current_config: RemoteFreeQueuedByteDrainConfig,
        decision: RemoteFreeServiceRetuneGuardDecision,
    ) -> Result<
        RemoteFreeServiceRetunePolicyApplication,
        RemoteFreeServiceRetunePolicyApplicationError,
    > {
        let RemoteFreeServiceRetuneGuardDecision::Apply { candidate } = decision else {
            return Ok(RemoteFreeServiceRetunePolicyApplication::NoChange { decision });
        };

        let config = self.target_config(current_config, candidate)?;
        Ok(RemoteFreeServiceRetunePolicyApplication::Apply { candidate, config })
    }

    fn target_config(
        self,
        current_config: RemoteFreeQueuedByteDrainConfig,
        candidate: RemoteFreeServiceRetuneCandidate,
    ) -> Result<RemoteFreeQueuedByteDrainConfig, RemoteFreeServiceRetunePolicyApplicationError>
    {
        match candidate {
            RemoteFreeServiceRetuneCandidate::DrainEarlier => Ok(current_config),
            RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacity
            | RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier => {
                self.config_with_grown_queue_capacity(current_config)
            }
            RemoteFreeServiceRetuneCandidate::CollectTelemetry
            | RemoteFreeServiceRetuneCandidate::KeepConfig
            | RemoteFreeServiceRetuneCandidate::ReviewQueuedByteBudget => Err(
                RemoteFreeServiceRetunePolicyApplicationError::NonActionableApplyCandidate {
                    candidate,
                },
            ),
        }
    }

    fn config_with_grown_queue_capacity(
        self,
        current_config: RemoteFreeQueuedByteDrainConfig,
    ) -> Result<RemoteFreeQueuedByteDrainConfig, RemoteFreeServiceRetunePolicyApplicationError>
    {
        let growth_factor = self.queue_capacity_growth_factor.get();
        let queue_capacity = current_config
            .queue_capacity()
            .checked_mul(growth_factor)
            .ok_or(
                RemoteFreeServiceRetunePolicyApplicationError::QueueCapacityGrowthOverflow {
                    queue_capacity: current_config.queue_capacity(),
                    growth_factor,
                },
            )?;

        RemoteFreeQueuedByteDrainConfig::new(
            queue_capacity,
            current_config.drain_batch_limit(),
            current_config.target_pending_items(),
            current_config.queued_byte_budget(),
        )
        .map_err(RemoteFreeServiceRetunePolicyApplicationError::Config)
    }
}

impl RemoteFreeServiceRetunePolicyApplication {
    /// Returns the applied candidate, if this plan changes live config.
    #[must_use]
    pub const fn candidate(self) -> Option<RemoteFreeServiceRetuneCandidate> {
        match self {
            Self::Apply { candidate, .. } => Some(candidate),
            Self::NoChange { .. } => None,
        }
    }

    /// Returns the applied config, if this plan changes live config.
    #[must_use]
    pub const fn config(self) -> Option<RemoteFreeQueuedByteDrainConfig> {
        match self {
            Self::Apply { config, .. } => Some(config),
            Self::NoChange { .. } => None,
        }
    }

    /// Returns true when the plan applies a live config.
    #[must_use]
    pub const fn is_apply(self) -> bool {
        matches!(self, Self::Apply { .. })
    }
}

impl fmt::Display for RemoteFreeServiceRetunePolicyApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroQueueCapacityGrowthFactor => {
                f.write_str("remote-free queue-capacity growth factor must be non-zero")
            }
            Self::QueueCapacityGrowthFactorTooSmall => {
                f.write_str("remote-free queue-capacity growth factor must be greater than one")
            }
            Self::NonActionableApplyCandidate { candidate } => {
                write!(
                    f,
                    "remote-free retune candidate {} cannot be applied to live policy",
                    candidate.as_str()
                )
            }
            Self::QueueCapacityGrowthOverflow {
                queue_capacity,
                growth_factor,
            } => write!(
                f,
                "remote-free queue capacity {queue_capacity} overflowed when multiplied by {growth_factor}"
            ),
            Self::Config(source) => write!(f, "{source}"),
        }
    }
}

impl std::error::Error for RemoteFreeServiceRetunePolicyApplicationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Config(source) => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        RemoteFreeServiceRetunePolicyApplication, RemoteFreeServiceRetunePolicyApplicationError,
        RemoteFreeServiceRetunePolicyApplicator,
    };
    use crate::{
        RemoteFreeDrainDecision, RemoteFreeDrainObservation, RemoteFreeDrainReason,
        RemoteFreeQueuedByteBudget, RemoteFreeQueuedByteDrainConfig,
        RemoteFreeServiceRetuneCandidate, RemoteFreeServiceRetuneGuardDecision,
    };

    fn config(queue_capacity: usize) -> RemoteFreeQueuedByteDrainConfig {
        RemoteFreeQueuedByteDrainConfig::from_item_shape(queue_capacity, 64, 64, 4096)
            .expect("config")
    }

    fn applicator() -> RemoteFreeServiceRetunePolicyApplicator {
        RemoteFreeServiceRetunePolicyApplicator::try_new(2).expect("applicator")
    }

    #[test]
    fn policy_applicator_rejects_invalid_growth_factors() {
        assert_eq!(
            RemoteFreeServiceRetunePolicyApplicator::try_new(0),
            Err(RemoteFreeServiceRetunePolicyApplicationError::ZeroQueueCapacityGrowthFactor)
        );
        assert_eq!(
            RemoteFreeServiceRetunePolicyApplicator::try_new(1),
            Err(RemoteFreeServiceRetunePolicyApplicationError::QueueCapacityGrowthFactorTooSmall)
        );
    }

    #[test]
    fn drain_earlier_apply_uses_current_validated_config() {
        let current_config = config(256);
        let plan = applicator()
            .plan(
                current_config,
                RemoteFreeServiceRetuneGuardDecision::Apply {
                    candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
                },
            )
            .expect("plan");

        assert_eq!(
            plan,
            RemoteFreeServiceRetunePolicyApplication::Apply {
                candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
                config: current_config,
            }
        );
        assert_eq!(
            plan.candidate(),
            Some(RemoteFreeServiceRetuneCandidate::DrainEarlier)
        );
        assert_eq!(plan.config(), Some(current_config));
        assert!(plan.is_apply());
    }

    #[test]
    fn capacity_apply_grows_capacity_and_preserves_window() {
        let plan = applicator()
            .plan(
                config(128),
                RemoteFreeServiceRetuneGuardDecision::Apply {
                    candidate:
                        RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier,
                },
            )
            .expect("plan");
        let config = plan.config().expect("applied config");

        assert_eq!(config.queue_capacity(), 256);
        assert_eq!(config.drain_batch_limit(), 64);
        assert_eq!(config.target_pending_items(), 64);
        assert_eq!(config.queued_byte_budget().bytes(), 262_144);
        assert_eq!(
            config
                .drain_policy()
                .decide(RemoteFreeDrainObservation::new(64, 262_144, 1)),
            RemoteFreeDrainDecision::Drain(RemoteFreeDrainReason::QueuedBytes)
        );
    }

    #[test]
    fn capacity_only_apply_grows_capacity_and_preserves_budget() {
        let plan = applicator()
            .plan(
                config(256),
                RemoteFreeServiceRetuneGuardDecision::Apply {
                    candidate: RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacity,
                },
            )
            .expect("plan");
        let config = plan.config().expect("applied config");

        assert_eq!(config.queue_capacity(), 512);
        assert_eq!(config.queued_byte_budget().bytes(), 262_144);
    }

    #[test]
    fn non_apply_decisions_return_no_change() {
        let decisions = [
            RemoteFreeServiceRetuneGuardDecision::CollectTelemetry,
            RemoteFreeServiceRetuneGuardDecision::Hold {
                candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
                consecutive_candidate_windows: 1,
            },
            RemoteFreeServiceRetuneGuardDecision::Confirmed {
                candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
            },
            RemoteFreeServiceRetuneGuardDecision::Rollback {
                candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
                observed_candidate:
                    RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier,
            },
            RemoteFreeServiceRetuneGuardDecision::MutationLimitReached {
                candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
            },
        ];

        for decision in decisions {
            let plan = applicator().plan(config(256), decision).expect("plan");
            assert_eq!(
                plan,
                RemoteFreeServiceRetunePolicyApplication::NoChange { decision }
            );
            assert_eq!(plan.candidate(), None);
            assert_eq!(plan.config(), None);
            assert!(!plan.is_apply());
        }
    }

    #[test]
    fn policy_applicator_rejects_non_actionable_apply_candidates() {
        for candidate in [
            RemoteFreeServiceRetuneCandidate::CollectTelemetry,
            RemoteFreeServiceRetuneCandidate::KeepConfig,
            RemoteFreeServiceRetuneCandidate::ReviewQueuedByteBudget,
        ] {
            assert_eq!(
                applicator().plan(
                    config(256),
                    RemoteFreeServiceRetuneGuardDecision::Apply { candidate },
                ),
                Err(
                    RemoteFreeServiceRetunePolicyApplicationError::NonActionableApplyCandidate {
                        candidate,
                    },
                )
            );
        }
    }

    #[test]
    fn policy_applicator_rejects_capacity_overflow() {
        let budget =
            RemoteFreeQueuedByteBudget::from_item_shape(64, 4096).expect("queued-byte budget");
        let current_config =
            RemoteFreeQueuedByteDrainConfig::new(usize::MAX, 64, 64, budget).expect("config");

        assert_eq!(
            applicator().plan(
                current_config,
                RemoteFreeServiceRetuneGuardDecision::Apply {
                    candidate: RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacity,
                },
            ),
            Err(
                RemoteFreeServiceRetunePolicyApplicationError::QueueCapacityGrowthOverflow {
                    queue_capacity: usize::MAX,
                    growth_factor: 2,
                },
            )
        );
    }
}
