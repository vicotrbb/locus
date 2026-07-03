use std::fmt;

use super::{
    RemoteFreeOwnerRuntime, RemoteFreeOwnerRuntimeApplyOutcome,
    RemoteFreeOwnerRuntimeConfirmOutcome, RemoteFreeOwnerRuntimeError,
    RemoteFreeOwnerRuntimeRollbackOutcome, RemoteFreeServiceRetuneGuard,
    RemoteFreeServiceRetuneGuardDecision, RemoteFreeServiceRetunePolicyApplicationError,
    RemoteFreeServiceRetunePolicyApplicator, RemoteFreeServiceRetuneSummary,
};

/// Stable owner index for a registered remote-free runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RemoteFreeServiceRuntimeOwnerId(usize);

/// Service-level coordinator for applying guarded retune decisions to owners.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteFreeServiceRuntimeRetuneCoordinator {
    guard: RemoteFreeServiceRetuneGuard,
    applicator: RemoteFreeServiceRetunePolicyApplicator,
}

/// Registered owner runtimes behind one service-level retune coordinator.
#[derive(Debug)]
pub struct RemoteFreeServiceRuntimeRetuneOwners<T> {
    coordinator: RemoteFreeServiceRuntimeRetuneCoordinator,
    owners: Vec<RemoteFreeOwnerRuntime<T>>,
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

/// Failure while routing a retune decision to a registered owner runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteFreeServiceRuntimeRetuneOwnerError {
    /// No owner was registered for the requested ID.
    MissingOwner {
        /// Requested owner ID.
        owner_id: RemoteFreeServiceRuntimeOwnerId,
    },
    /// The coordinator failed while applying the owner decision.
    Retune(RemoteFreeServiceRuntimeRetuneError),
}

impl RemoteFreeServiceRuntimeOwnerId {
    /// Creates an owner ID from a zero-based index.
    #[must_use]
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the zero-based owner index.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
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

impl<T> RemoteFreeServiceRuntimeRetuneOwners<T> {
    /// Creates an empty owner registry around a service coordinator.
    #[must_use]
    pub const fn new(coordinator: RemoteFreeServiceRuntimeRetuneCoordinator) -> Self {
        Self {
            coordinator,
            owners: Vec::new(),
        }
    }

    /// Creates an owner registry from existing owner runtimes.
    #[must_use]
    pub fn from_owners(
        coordinator: RemoteFreeServiceRuntimeRetuneCoordinator,
        owners: Vec<RemoteFreeOwnerRuntime<T>>,
    ) -> Self {
        Self {
            coordinator,
            owners,
        }
    }

    /// Returns the shared service coordinator.
    #[must_use]
    pub const fn coordinator(&self) -> RemoteFreeServiceRuntimeRetuneCoordinator {
        self.coordinator
    }

    /// Returns the number of registered owner runtimes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.owners.len()
    }

    /// Returns true when no owner runtimes are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.owners.is_empty()
    }

    /// Registers an owner runtime and returns its stable owner ID.
    pub fn register_owner(
        &mut self,
        runtime: RemoteFreeOwnerRuntime<T>,
    ) -> RemoteFreeServiceRuntimeOwnerId {
        let owner_id = RemoteFreeServiceRuntimeOwnerId::new(self.owners.len());
        self.owners.push(runtime);
        owner_id
    }

    /// Returns a registered owner runtime by ID.
    #[must_use]
    pub fn owner(
        &self,
        owner_id: RemoteFreeServiceRuntimeOwnerId,
    ) -> Option<&RemoteFreeOwnerRuntime<T>> {
        self.owners.get(owner_id.index())
    }

    /// Returns a mutable registered owner runtime by ID.
    #[must_use]
    pub fn owner_mut(
        &mut self,
        owner_id: RemoteFreeServiceRuntimeOwnerId,
    ) -> Option<&mut RemoteFreeOwnerRuntime<T>> {
        self.owners.get_mut(owner_id.index())
    }

    /// Observes a summary for a registered owner and applies the decision.
    ///
    /// # Errors
    ///
    /// Returns an error if the owner ID is not registered or the coordinator
    /// fails while applying the owner decision.
    pub fn observe_owner_summary(
        &mut self,
        owner_id: RemoteFreeServiceRuntimeOwnerId,
        summary: RemoteFreeServiceRetuneSummary,
    ) -> Result<RemoteFreeServiceRuntimeRetuneOutcome, RemoteFreeServiceRuntimeRetuneOwnerError>
    {
        let runtime = self
            .owners
            .get_mut(owner_id.index())
            .ok_or(RemoteFreeServiceRuntimeRetuneOwnerError::MissingOwner { owner_id })?;

        self.coordinator
            .observe_owner_summary(runtime, summary)
            .map_err(RemoteFreeServiceRuntimeRetuneOwnerError::Retune)
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

impl fmt::Display for RemoteFreeServiceRuntimeRetuneOwnerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingOwner { owner_id } => write!(
                f,
                "remote-free owner runtime {} is not registered",
                owner_id.index()
            ),
            Self::Retune(source) => write!(f, "{source}"),
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

impl std::error::Error for RemoteFreeServiceRuntimeRetuneOwnerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::MissingOwner { .. } => None,
            Self::Retune(source) => Some(source),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        RemoteFreeServiceRuntimeOwnerId, RemoteFreeServiceRuntimeRetuneCoordinator,
        RemoteFreeServiceRuntimeRetuneOutcome, RemoteFreeServiceRuntimeRetuneOwnerError,
        RemoteFreeServiceRuntimeRetuneOwners,
    };
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

    #[test]
    fn owner_registry_routes_summaries_by_owner_id() {
        let mut owners = RemoteFreeServiceRuntimeRetuneOwners::new(coordinator(2));
        let first = owners
            .register_owner(RemoteFreeOwnerRuntime::<usize>::new(config(256)).expect("first"));
        let second = owners
            .register_owner(RemoteFreeOwnerRuntime::<usize>::new(config(128)).expect("second"));

        assert_eq!(first.index(), 0);
        assert_eq!(second.index(), 1);
        assert_eq!(owners.len(), 2);
        assert!(!owners.is_empty());

        let first_drift = summary(report(config(256), 96, 524_288, 0));
        let second_combined = summary(report(config(128), 96, 524_288, 1));
        let second_byte_drift = summary(report(config(256), 64, 524_352, 0));

        owners
            .observe_owner_summary(first, first_drift)
            .expect("first hold");
        owners
            .observe_owner_summary(first, first_drift)
            .expect("first apply");
        owners
            .observe_owner_summary(first, summary(report(config(256), 64, 262_144, 0)))
            .expect("first confirm");

        owners
            .observe_owner_summary(second, second_combined)
            .expect("second hold");
        owners
            .observe_owner_summary(second, second_combined)
            .expect("second apply");
        let outcome = owners
            .observe_owner_summary(second, second_byte_drift)
            .expect("second rollback");

        assert!(matches!(
            outcome,
            RemoteFreeServiceRuntimeRetuneOutcome::RolledBack { .. }
        ));
        assert_eq!(
            owners.owner(first).expect("first owner").previous_config(),
            None
        );
        assert_eq!(
            owners
                .owner(second)
                .expect("second owner")
                .config()
                .queue_capacity(),
            128
        );
        assert_eq!(owners.coordinator().guard().applied_mutations(), 2);
        assert_eq!(owners.coordinator().guard().confirmed_mutations(), 1);
        assert_eq!(owners.coordinator().guard().rollbacks(), 1);
    }

    #[test]
    fn owner_registry_reports_missing_owner_id() {
        let mut owners = RemoteFreeServiceRuntimeRetuneOwners::<usize>::new(coordinator(2));
        let missing = RemoteFreeServiceRuntimeOwnerId::new(7);

        assert_eq!(
            owners.observe_owner_summary(missing, summary(report(config(256), 96, 524_288, 0))),
            Err(RemoteFreeServiceRuntimeRetuneOwnerError::MissingOwner { owner_id: missing })
        );
    }
}
