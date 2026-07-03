use std::fmt;

use super::{
    RemoteFreeOwnerRuntime, RemoteFreeOwnerRuntimeApplyOutcome,
    RemoteFreeOwnerRuntimeConfirmOutcome, RemoteFreeOwnerRuntimeError,
    RemoteFreeOwnerRuntimeRollbackOutcome, RemoteFreeServiceRetuneGuard,
    RemoteFreeServiceRetuneGuardDecision, RemoteFreeServiceRetunePolicyApplicationError,
    RemoteFreeServiceRetunePolicyApplicator, RemoteFreeServiceRetuneSummary,
};

/// Service-level coordinator for applying guarded retune decisions to owners.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteFreeServiceRuntimeRetuneCoordinator {
    guard: RemoteFreeServiceRetuneGuard,
    applicator: RemoteFreeServiceRetunePolicyApplicator,
}

/// Runtime outcome produced by the service retune coordinator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteFreeServiceRuntimeRetuneOutcome {
    /// The guard decision did not change live runtime config.
    NoChange {
        /// Guard decision observed for this service window.
        decision: RemoteFreeServiceRetuneGuardDecision,
        /// Owner runtime no-change outcome.
        runtime: RemoteFreeOwnerRuntimeApplyOutcome,
    },
    /// The owner runtime installed a guarded candidate config.
    Applied {
        /// Guard decision observed for this service window.
        decision: RemoteFreeServiceRetuneGuardDecision,
        /// Owner runtime install outcome.
        runtime: RemoteFreeOwnerRuntimeApplyOutcome,
    },
    /// The owner runtime confirmed an applied candidate.
    Confirmed {
        /// Guard decision observed for this service window.
        decision: RemoteFreeServiceRetuneGuardDecision,
        /// Owner runtime confirm outcome.
        runtime: RemoteFreeOwnerRuntimeConfirmOutcome,
    },
    /// The owner runtime rolled back a failed candidate.
    RolledBack {
        /// Guard decision observed for this service window.
        decision: RemoteFreeServiceRetuneGuardDecision,
        /// Owner runtime rollback outcome.
        runtime: RemoteFreeOwnerRuntimeRollbackOutcome,
    },
}

/// Failure while coordinating guarded service retune decisions with a runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteFreeServiceRuntimeRetuneError {
    /// Translating a guard decision into a policy application failed.
    Application(RemoteFreeServiceRetunePolicyApplicationError),
    /// Applying, confirming, or rolling back owner runtime state failed.
    Runtime(RemoteFreeOwnerRuntimeError),
}

impl RemoteFreeServiceRuntimeRetuneCoordinator {
    /// Creates a coordinator from an existing service guard and applicator.
    #[must_use]
    pub const fn new(
        guard: RemoteFreeServiceRetuneGuard,
        applicator: RemoteFreeServiceRetunePolicyApplicator,
    ) -> Self {
        Self { guard, applicator }
    }

    /// Returns the service guard state.
    #[must_use]
    pub const fn guard(&self) -> RemoteFreeServiceRetuneGuard {
        self.guard
    }

    /// Returns the policy applicator.
    #[must_use]
    pub const fn applicator(&self) -> RemoteFreeServiceRetunePolicyApplicator {
        self.applicator
    }

    /// Observes one service summary and applies the decision to one owner.
    ///
    /// The caller chooses which owner runtime receives the decision for this
    /// service window. The coordinator keeps the guard state shared across
    /// owners, so mutation limits remain service-wide.
    ///
    /// # Errors
    ///
    /// Returns an error if the applicator cannot translate an apply decision
    /// or if the owner runtime rejects the operation.
    pub fn observe_owner_summary<T>(
        &mut self,
        runtime: &mut RemoteFreeOwnerRuntime<T>,
        summary: RemoteFreeServiceRetuneSummary,
    ) -> Result<RemoteFreeServiceRuntimeRetuneOutcome, RemoteFreeServiceRuntimeRetuneError> {
        let decision = self.guard.observe_summary(summary);
        self.apply_decision(runtime, decision)
    }

    fn apply_decision<T>(
        &self,
        runtime: &mut RemoteFreeOwnerRuntime<T>,
        decision: RemoteFreeServiceRetuneGuardDecision,
    ) -> Result<RemoteFreeServiceRuntimeRetuneOutcome, RemoteFreeServiceRuntimeRetuneError> {
        match decision {
            RemoteFreeServiceRetuneGuardDecision::Apply { .. } => {
                let application = self
                    .applicator
                    .plan(runtime.config(), decision)
                    .map_err(RemoteFreeServiceRuntimeRetuneError::Application)?;
                let runtime = runtime
                    .apply(application)
                    .map_err(RemoteFreeServiceRuntimeRetuneError::Runtime)?;

                Ok(RemoteFreeServiceRuntimeRetuneOutcome::Applied { decision, runtime })
            }
            RemoteFreeServiceRetuneGuardDecision::Confirmed { .. } => {
                let runtime = runtime
                    .confirm()
                    .map_err(RemoteFreeServiceRuntimeRetuneError::Runtime)?;

                Ok(RemoteFreeServiceRuntimeRetuneOutcome::Confirmed { decision, runtime })
            }
            RemoteFreeServiceRetuneGuardDecision::Rollback { .. } => {
                let runtime = runtime
                    .rollback()
                    .map_err(RemoteFreeServiceRuntimeRetuneError::Runtime)?;

                Ok(RemoteFreeServiceRuntimeRetuneOutcome::RolledBack { decision, runtime })
            }
            RemoteFreeServiceRetuneGuardDecision::CollectTelemetry
            | RemoteFreeServiceRetuneGuardDecision::Hold { .. }
            | RemoteFreeServiceRetuneGuardDecision::MutationLimitReached { .. } => {
                let application = self
                    .applicator
                    .plan(runtime.config(), decision)
                    .map_err(RemoteFreeServiceRuntimeRetuneError::Application)?;
                let runtime = runtime
                    .apply(application)
                    .map_err(RemoteFreeServiceRuntimeRetuneError::Runtime)?;

                Ok(RemoteFreeServiceRuntimeRetuneOutcome::NoChange { decision, runtime })
            }
        }
    }
}

impl RemoteFreeServiceRuntimeRetuneOutcome {
    /// Returns the guard decision that produced this outcome.
    #[must_use]
    pub const fn decision(self) -> RemoteFreeServiceRetuneGuardDecision {
        match self {
            Self::NoChange { decision, .. }
            | Self::Applied { decision, .. }
            | Self::Confirmed { decision, .. }
            | Self::RolledBack { decision, .. } => decision,
        }
    }
}

impl fmt::Display for RemoteFreeServiceRuntimeRetuneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Application(source) => write!(f, "{source}"),
            Self::Runtime(source) => write!(f, "{source}"),
        }
    }
}

impl std::error::Error for RemoteFreeServiceRuntimeRetuneError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Application(source) => Some(source),
            Self::Runtime(source) => Some(source),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{RemoteFreeServiceRuntimeRetuneCoordinator, RemoteFreeServiceRuntimeRetuneOutcome};
    use crate::{
        RemoteFreeDrainObservation, RemoteFreeOwnerRuntime, RemoteFreeOwnerRuntimeApplyOutcome,
        RemoteFreeOwnerRuntimeRollbackOutcome, RemoteFreeQueueStats,
        RemoteFreeQueuedByteDrainConfig, RemoteFreeQueuedByteDriftReport,
        RemoteFreeServiceRetuneCandidate, RemoteFreeServiceRetuneGuard,
        RemoteFreeServiceRetuneGuardDecision, RemoteFreeServiceRetunePolicyApplicator,
        RemoteFreeServiceRetuneSummary,
    };

    fn config(queue_capacity: usize) -> RemoteFreeQueuedByteDrainConfig {
        RemoteFreeQueuedByteDrainConfig::from_item_shape(queue_capacity, 64, 64, 4096)
            .expect("config")
    }

    fn coordinator(max_mutations: u64) -> RemoteFreeServiceRuntimeRetuneCoordinator {
        RemoteFreeServiceRuntimeRetuneCoordinator::new(
            RemoteFreeServiceRetuneGuard::try_new(2, max_mutations).expect("guard"),
            RemoteFreeServiceRetunePolicyApplicator::try_new(2).expect("applicator"),
        )
    }

    fn summary(report: RemoteFreeQueuedByteDriftReport) -> RemoteFreeServiceRetuneSummary {
        let mut summary = RemoteFreeServiceRetuneSummary::new();
        summary.observe_report(report);
        summary
    }

    fn report(
        config: RemoteFreeQueuedByteDrainConfig,
        pending_count: u64,
        queued_bytes: u64,
        full_count: u64,
    ) -> RemoteFreeQueuedByteDriftReport {
        RemoteFreeQueuedByteDriftReport::from_observation(
            config,
            RemoteFreeQueueStats {
                capacity: config.queue_capacity(),
                batch_limit: config.drain_batch_limit(),
                submitted_count: pending_count,
                pending_count,
                full_count,
                disconnected_count: 0,
                drained_count: 0,
            },
            RemoteFreeDrainObservation::new(pending_count, queued_bytes, 1),
        )
    }

    #[test]
    fn coordinator_applies_and_confirms_owner_runtime() {
        let mut coordinator = coordinator(2);
        let mut runtime = RemoteFreeOwnerRuntime::<usize>::new(config(256)).expect("runtime");

        let drift = summary(report(runtime.config(), 96, 524_288, 0));
        let clean = summary(report(runtime.config(), 64, 262_144, 0));

        assert!(matches!(
            coordinator
                .observe_owner_summary(&mut runtime, drift)
                .expect("hold"),
            RemoteFreeServiceRuntimeRetuneOutcome::NoChange {
                decision: RemoteFreeServiceRetuneGuardDecision::Hold {
                    candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
                    ..
                },
                ..
            }
        ));

        assert!(matches!(
            coordinator
                .observe_owner_summary(&mut runtime, drift)
                .expect("apply"),
            RemoteFreeServiceRuntimeRetuneOutcome::Applied {
                decision: RemoteFreeServiceRetuneGuardDecision::Apply {
                    candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
                },
                runtime: RemoteFreeOwnerRuntimeApplyOutcome::Installed { .. },
            }
        ));
        assert!(runtime.previous_config().is_some());

        assert!(matches!(
            coordinator
                .observe_owner_summary(&mut runtime, clean)
                .expect("confirm"),
            RemoteFreeServiceRuntimeRetuneOutcome::Confirmed {
                decision: RemoteFreeServiceRetuneGuardDecision::Confirmed {
                    candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
                },
                ..
            }
        ));
        assert_eq!(runtime.previous_config(), None);
        assert_eq!(coordinator.guard().confirmed_mutations(), 1);
    }

    #[test]
    fn coordinator_rolls_back_failed_owner_validation() {
        let mut coordinator = coordinator(2);
        let mut runtime = RemoteFreeOwnerRuntime::<usize>::new(config(128)).expect("runtime");
        let initial_config = runtime.config();

        let combined = summary(report(runtime.config(), 96, 524_288, 1));
        let byte_drift = summary(report(config(256), 64, 524_352, 0));

        coordinator
            .observe_owner_summary(&mut runtime, combined)
            .expect("hold");
        coordinator
            .observe_owner_summary(&mut runtime, combined)
            .expect("apply");

        assert_eq!(runtime.config().queue_capacity(), 256);
        let outcome = coordinator
            .observe_owner_summary(&mut runtime, byte_drift)
            .expect("rollback");

        assert_eq!(
            outcome,
            RemoteFreeServiceRuntimeRetuneOutcome::RolledBack {
                decision: RemoteFreeServiceRetuneGuardDecision::Rollback {
                    candidate:
                        RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier,
                    observed_candidate: RemoteFreeServiceRetuneCandidate::ReviewQueuedByteBudget,
                },
                runtime: RemoteFreeOwnerRuntimeRollbackOutcome {
                    replaced_config: config(256),
                    restored_config: initial_config,
                },
            }
        );
        assert_eq!(runtime.config(), initial_config);
        assert_eq!(runtime.previous_config(), None);
        assert_eq!(coordinator.guard().rollbacks(), 1);
    }
}
