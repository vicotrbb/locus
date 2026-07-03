use std::fmt;

use super::{
    RemoteFreeOwnerRuntime, RemoteFreeServiceRetuneGuardDecision, RemoteFreeServiceRetuneSummary,
    RemoteFreeServiceRuntimeOwnerId, RemoteFreeServiceRuntimeRetuneOutcome,
    RemoteFreeServiceRuntimeRetuneOwnerError, RemoteFreeServiceRuntimeRetuneOwners,
};

/// Routed owner telemetry for one remote-free service retune observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteFreeServiceRuntimeWindowObservation {
    owner_id: RemoteFreeServiceRuntimeOwnerId,
    summary: RemoteFreeServiceRetuneSummary,
}

/// Aggregated counters returned after processing service-window observations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RemoteFreeServiceRuntimeWindowStats {
    owner_observations: u64,
    observed_reports: u64,
    reports_needing_retune: u64,
    max_pending_items_over_target: u64,
    max_queued_bytes_over_budget: u64,
    queue_backpressure_reports: u64,
    collect_telemetry_decisions: u64,
    hold_decisions: u64,
    apply_decisions: u64,
    confirmed_decisions: u64,
    rollback_decisions: u64,
    mutation_limit_decisions: u64,
    no_change_outcomes: u64,
    applied_outcomes: u64,
    confirmed_outcomes: u64,
    rolled_back_outcomes: u64,
}

/// Failure while processing a routed service-window observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteFreeServiceRuntimeWindowError {
    owner_id: RemoteFreeServiceRuntimeOwnerId,
    source: RemoteFreeServiceRuntimeRetuneOwnerError,
}

/// Failure while collecting and routing service-window owner telemetry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteFreeServiceRuntimeWindowCollectionError<E> {
    /// No owner was registered for the requested ID.
    MissingOwner {
        /// Requested owner ID.
        owner_id: RemoteFreeServiceRuntimeOwnerId,
    },
    /// The caller-provided telemetry collector failed for this owner.
    Collect {
        /// Owner ID being collected.
        owner_id: RemoteFreeServiceRuntimeOwnerId,
        /// Collector error.
        source: E,
    },
    /// Routing the collected summary through the service window failed.
    Window(RemoteFreeServiceRuntimeWindowError),
}

impl RemoteFreeServiceRuntimeWindowObservation {
    /// Creates a routed owner observation from an owner ID and summary.
    #[must_use]
    pub const fn new(
        owner_id: RemoteFreeServiceRuntimeOwnerId,
        summary: RemoteFreeServiceRetuneSummary,
    ) -> Self {
        Self { owner_id, summary }
    }

    /// Returns the registered owner ID that should receive this observation.
    #[must_use]
    pub const fn owner_id(self) -> RemoteFreeServiceRuntimeOwnerId {
        self.owner_id
    }

    /// Returns the service retune summary for this observation.
    #[must_use]
    pub const fn summary(self) -> RemoteFreeServiceRetuneSummary {
        self.summary
    }
}

impl RemoteFreeServiceRuntimeWindowStats {
    /// Creates empty service-window stats.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            owner_observations: 0,
            observed_reports: 0,
            reports_needing_retune: 0,
            max_pending_items_over_target: 0,
            max_queued_bytes_over_budget: 0,
            queue_backpressure_reports: 0,
            collect_telemetry_decisions: 0,
            hold_decisions: 0,
            apply_decisions: 0,
            confirmed_decisions: 0,
            rollback_decisions: 0,
            mutation_limit_decisions: 0,
            no_change_outcomes: 0,
            applied_outcomes: 0,
            confirmed_outcomes: 0,
            rolled_back_outcomes: 0,
        }
    }

    /// Merges another service-window stats value into this one.
    pub fn merge(&mut self, other: Self) {
        self.owner_observations = self
            .owner_observations
            .saturating_add(other.owner_observations);
        self.observed_reports = self.observed_reports.saturating_add(other.observed_reports);
        self.reports_needing_retune = self
            .reports_needing_retune
            .saturating_add(other.reports_needing_retune);
        self.max_pending_items_over_target = self
            .max_pending_items_over_target
            .max(other.max_pending_items_over_target);
        self.max_queued_bytes_over_budget = self
            .max_queued_bytes_over_budget
            .max(other.max_queued_bytes_over_budget);
        self.queue_backpressure_reports = self
            .queue_backpressure_reports
            .saturating_add(other.queue_backpressure_reports);
        self.collect_telemetry_decisions = self
            .collect_telemetry_decisions
            .saturating_add(other.collect_telemetry_decisions);
        self.hold_decisions = self.hold_decisions.saturating_add(other.hold_decisions);
        self.apply_decisions = self.apply_decisions.saturating_add(other.apply_decisions);
        self.confirmed_decisions = self
            .confirmed_decisions
            .saturating_add(other.confirmed_decisions);
        self.rollback_decisions = self
            .rollback_decisions
            .saturating_add(other.rollback_decisions);
        self.mutation_limit_decisions = self
            .mutation_limit_decisions
            .saturating_add(other.mutation_limit_decisions);
        self.no_change_outcomes = self
            .no_change_outcomes
            .saturating_add(other.no_change_outcomes);
        self.applied_outcomes = self.applied_outcomes.saturating_add(other.applied_outcomes);
        self.confirmed_outcomes = self
            .confirmed_outcomes
            .saturating_add(other.confirmed_outcomes);
        self.rolled_back_outcomes = self
            .rolled_back_outcomes
            .saturating_add(other.rolled_back_outcomes);
    }

    /// Returns how many routed owner observations were processed.
    #[must_use]
    pub const fn owner_observations(self) -> u64 {
        self.owner_observations
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

    /// Returns the maximum pending-item drift across observed summaries.
    #[must_use]
    pub const fn max_pending_items_over_target(self) -> u64 {
        self.max_pending_items_over_target
    }

    /// Returns the maximum queued-byte drift across observed summaries.
    #[must_use]
    pub const fn max_queued_bytes_over_budget(self) -> u64 {
        self.max_queued_bytes_over_budget
    }

    /// Returns how many reports included queue backpressure.
    #[must_use]
    pub const fn queue_backpressure_reports(self) -> u64 {
        self.queue_backpressure_reports
    }

    /// Returns how many collect-telemetry decisions were observed.
    #[must_use]
    pub const fn collect_telemetry_decisions(self) -> u64 {
        self.collect_telemetry_decisions
    }

    /// Returns how many hold decisions were observed.
    #[must_use]
    pub const fn hold_decisions(self) -> u64 {
        self.hold_decisions
    }

    /// Returns how many apply decisions were observed.
    #[must_use]
    pub const fn apply_decisions(self) -> u64 {
        self.apply_decisions
    }

    /// Returns how many confirmed decisions were observed.
    #[must_use]
    pub const fn confirmed_decisions(self) -> u64 {
        self.confirmed_decisions
    }

    /// Returns how many rollback decisions were observed.
    #[must_use]
    pub const fn rollback_decisions(self) -> u64 {
        self.rollback_decisions
    }

    /// Returns how many mutation-limit decisions were observed.
    #[must_use]
    pub const fn mutation_limit_decisions(self) -> u64 {
        self.mutation_limit_decisions
    }

    /// Returns how many owner-runtime no-change outcomes were observed.
    #[must_use]
    pub const fn no_change_outcomes(self) -> u64 {
        self.no_change_outcomes
    }

    /// Returns how many owner-runtime apply outcomes were observed.
    #[must_use]
    pub const fn applied_outcomes(self) -> u64 {
        self.applied_outcomes
    }

    /// Returns how many owner-runtime confirm outcomes were observed.
    #[must_use]
    pub const fn confirmed_outcomes(self) -> u64 {
        self.confirmed_outcomes
    }

    /// Returns how many owner-runtime rollback outcomes were observed.
    #[must_use]
    pub const fn rolled_back_outcomes(self) -> u64 {
        self.rolled_back_outcomes
    }

    fn observe_summary(&mut self, summary: RemoteFreeServiceRetuneSummary) {
        self.owner_observations = self.owner_observations.saturating_add(1);
        self.observed_reports = self
            .observed_reports
            .saturating_add(summary.observed_reports());
        self.reports_needing_retune = self
            .reports_needing_retune
            .saturating_add(summary.reports_needing_retune());
        self.max_pending_items_over_target = self
            .max_pending_items_over_target
            .max(summary.max_pending_items_over_target());
        self.max_queued_bytes_over_budget = self
            .max_queued_bytes_over_budget
            .max(summary.max_queued_bytes_over_budget());
        self.queue_backpressure_reports = self
            .queue_backpressure_reports
            .saturating_add(summary.queue_backpressure_reports());
    }

    fn observe_outcome(&mut self, outcome: RemoteFreeServiceRuntimeRetuneOutcome) {
        match outcome.decision() {
            RemoteFreeServiceRetuneGuardDecision::CollectTelemetry => {
                self.collect_telemetry_decisions =
                    self.collect_telemetry_decisions.saturating_add(1);
            }
            RemoteFreeServiceRetuneGuardDecision::Hold { .. } => {
                self.hold_decisions = self.hold_decisions.saturating_add(1);
            }
            RemoteFreeServiceRetuneGuardDecision::Apply { .. } => {
                self.apply_decisions = self.apply_decisions.saturating_add(1);
            }
            RemoteFreeServiceRetuneGuardDecision::Confirmed { .. } => {
                self.confirmed_decisions = self.confirmed_decisions.saturating_add(1);
            }
            RemoteFreeServiceRetuneGuardDecision::Rollback { .. } => {
                self.rollback_decisions = self.rollback_decisions.saturating_add(1);
            }
            RemoteFreeServiceRetuneGuardDecision::MutationLimitReached { .. } => {
                self.mutation_limit_decisions = self.mutation_limit_decisions.saturating_add(1);
            }
        }

        match outcome {
            RemoteFreeServiceRuntimeRetuneOutcome::NoChange { .. } => {
                self.no_change_outcomes = self.no_change_outcomes.saturating_add(1);
            }
            RemoteFreeServiceRuntimeRetuneOutcome::Applied { .. } => {
                self.applied_outcomes = self.applied_outcomes.saturating_add(1);
            }
            RemoteFreeServiceRuntimeRetuneOutcome::Confirmed { .. } => {
                self.confirmed_outcomes = self.confirmed_outcomes.saturating_add(1);
            }
            RemoteFreeServiceRuntimeRetuneOutcome::RolledBack { .. } => {
                self.rolled_back_outcomes = self.rolled_back_outcomes.saturating_add(1);
            }
        }
    }
}

impl RemoteFreeServiceRuntimeWindowError {
    /// Returns the owner ID for the failed observation.
    #[must_use]
    pub const fn owner_id(&self) -> RemoteFreeServiceRuntimeOwnerId {
        self.owner_id
    }

    /// Returns the underlying owner-routing or retune error.
    #[must_use]
    pub const fn source_error(&self) -> &RemoteFreeServiceRuntimeRetuneOwnerError {
        &self.source
    }
}

impl<E> RemoteFreeServiceRuntimeWindowCollectionError<E> {
    /// Returns the owner ID for the failed collection or routing step.
    #[must_use]
    pub const fn owner_id(&self) -> RemoteFreeServiceRuntimeOwnerId {
        match self {
            Self::MissingOwner { owner_id } | Self::Collect { owner_id, .. } => *owner_id,
            Self::Window(source) => source.owner_id(),
        }
    }
}

impl<T> RemoteFreeServiceRuntimeRetuneOwners<T> {
    /// Processes routed owner observations for one service retune window.
    ///
    /// Observations are processed in iterator order. A later error stops the
    /// window after any earlier observations have already been applied.
    ///
    /// # Errors
    ///
    /// Returns an error with owner ID context if routing or retuning fails.
    pub fn observe_service_window(
        &mut self,
        observations: impl IntoIterator<Item = RemoteFreeServiceRuntimeWindowObservation>,
    ) -> Result<RemoteFreeServiceRuntimeWindowStats, RemoteFreeServiceRuntimeWindowError> {
        let mut stats = RemoteFreeServiceRuntimeWindowStats::new();

        for observation in observations {
            stats.observe_summary(observation.summary());
            let outcome = self
                .observe_owner_summary(observation.owner_id(), observation.summary())
                .map_err(|source| RemoteFreeServiceRuntimeWindowError {
                    owner_id: observation.owner_id(),
                    source,
                })?;
            stats.observe_outcome(outcome);
        }

        Ok(stats)
    }

    /// Collects and processes owner summaries for one service retune window.
    ///
    /// Each owner is mutably borrowed only while `collect` creates its summary.
    /// The borrow ends before the summary is routed through the service
    /// coordinator, so apply, confirm, rollback, or no-change operations happen
    /// through the registry after collection.
    ///
    /// # Errors
    ///
    /// Returns an error with owner ID context if an owner is missing, summary
    /// collection fails, or routing the collected summary fails.
    pub fn collect_service_window<E>(
        &mut self,
        owner_ids: impl IntoIterator<Item = RemoteFreeServiceRuntimeOwnerId>,
        mut collect: impl FnMut(
            RemoteFreeServiceRuntimeOwnerId,
            &mut RemoteFreeOwnerRuntime<T>,
        ) -> Result<RemoteFreeServiceRetuneSummary, E>,
    ) -> Result<RemoteFreeServiceRuntimeWindowStats, RemoteFreeServiceRuntimeWindowCollectionError<E>>
    {
        let mut stats = RemoteFreeServiceRuntimeWindowStats::new();

        for owner_id in owner_ids {
            let summary = {
                let runtime = self.owner_mut(owner_id).ok_or(
                    RemoteFreeServiceRuntimeWindowCollectionError::MissingOwner { owner_id },
                )?;
                collect(owner_id, runtime).map_err(|source| {
                    RemoteFreeServiceRuntimeWindowCollectionError::Collect { owner_id, source }
                })?
            };

            let window_stats = self
                .observe_service_window([RemoteFreeServiceRuntimeWindowObservation::new(
                    owner_id, summary,
                )])
                .map_err(RemoteFreeServiceRuntimeWindowCollectionError::Window)?;
            stats.merge(window_stats);
        }

        Ok(stats)
    }
}

impl fmt::Display for RemoteFreeServiceRuntimeWindowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "remote-free service window failed for owner {}: {}",
            self.owner_id.index(),
            self.source
        )
    }
}

impl std::error::Error for RemoteFreeServiceRuntimeWindowError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

impl<E> fmt::Display for RemoteFreeServiceRuntimeWindowCollectionError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingOwner { owner_id } => write!(
                f,
                "remote-free owner runtime {} is not registered",
                owner_id.index()
            ),
            Self::Collect { owner_id, source } => write!(
                f,
                "remote-free service window collection failed for owner {}: {}",
                owner_id.index(),
                source
            ),
            Self::Window(source) => write!(f, "{source}"),
        }
    }
}

impl<E> std::error::Error for RemoteFreeServiceRuntimeWindowCollectionError<E>
where
    E: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::MissingOwner { .. } => None,
            Self::Collect { source, .. } => Some(source),
            Self::Window(source) => Some(source),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        RemoteFreeServiceRuntimeWindowCollectionError, RemoteFreeServiceRuntimeWindowError,
        RemoteFreeServiceRuntimeWindowObservation, RemoteFreeServiceRuntimeWindowStats,
    };
    use crate::{
        RemoteFreeDrainObservation, RemoteFreeOwnerRuntime, RemoteFreeQueueStats,
        RemoteFreeQueuedByteDrainConfig, RemoteFreeQueuedByteDriftReport,
        RemoteFreeServiceRetuneGuard, RemoteFreeServiceRetunePolicyApplicator,
        RemoteFreeServiceRetuneSummary, RemoteFreeServiceRuntimeOwnerId,
        RemoteFreeServiceRuntimeRetuneCoordinator, RemoteFreeServiceRuntimeRetuneOwnerError,
        RemoteFreeServiceRuntimeRetuneOwners,
    };

    fn config(queue_capacity: usize) -> RemoteFreeQueuedByteDrainConfig {
        RemoteFreeQueuedByteDrainConfig::from_item_shape(queue_capacity, 64, 64, 4096)
            .expect("config")
    }

    fn owners(max_mutations: u64) -> RemoteFreeServiceRuntimeRetuneOwners<usize> {
        RemoteFreeServiceRuntimeRetuneOwners::new(RemoteFreeServiceRuntimeRetuneCoordinator::new(
            RemoteFreeServiceRetuneGuard::try_new(2, max_mutations).expect("guard"),
            RemoteFreeServiceRetunePolicyApplicator::try_new(2).expect("applicator"),
        ))
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
    fn stats_merge_sums_counts_and_keeps_maxima() {
        let mut left = RemoteFreeServiceRuntimeWindowStats::new();
        left.observe_summary(summary(report(config(128), 96, 524_288, 1)));

        let mut right = RemoteFreeServiceRuntimeWindowStats::new();
        right.observe_summary(summary(report(config(256), 128, 786_432, 0)));

        left.merge(right);

        assert_eq!(left.owner_observations(), 2);
        assert_eq!(left.observed_reports(), 2);
        assert_eq!(left.reports_needing_retune(), 2);
        assert_eq!(left.max_pending_items_over_target(), 64);
        assert_eq!(left.max_queued_bytes_over_budget(), 524_288);
        assert_eq!(left.queue_backpressure_reports(), 1);
    }

    #[test]
    fn service_window_runner_routes_registered_observations() {
        let mut owners = owners(2);
        let owner_id = owners
            .register_owner(RemoteFreeOwnerRuntime::<usize>::new(config(256)).expect("runtime"));
        let drift = summary(report(config(256), 96, 524_288, 0));
        let clean = summary(report(config(256), 64, 262_144, 0));

        let first = owners
            .observe_service_window([RemoteFreeServiceRuntimeWindowObservation::new(
                owner_id, drift,
            )])
            .expect("hold");
        let second = owners
            .observe_service_window([RemoteFreeServiceRuntimeWindowObservation::new(
                owner_id, drift,
            )])
            .expect("apply");
        let third = owners
            .observe_service_window([RemoteFreeServiceRuntimeWindowObservation::new(
                owner_id, clean,
            )])
            .expect("confirm");

        let mut total = RemoteFreeServiceRuntimeWindowStats::new();
        total.merge(first);
        total.merge(second);
        total.merge(third);

        assert_eq!(total.owner_observations(), 3);
        assert_eq!(total.observed_reports(), 3);
        assert_eq!(total.reports_needing_retune(), 2);
        assert_eq!(total.hold_decisions(), 1);
        assert_eq!(total.apply_decisions(), 1);
        assert_eq!(total.confirmed_decisions(), 1);
        assert_eq!(total.no_change_outcomes(), 1);
        assert_eq!(total.applied_outcomes(), 1);
        assert_eq!(total.confirmed_outcomes(), 1);
        assert_eq!(
            owners.owner(owner_id).expect("owner").previous_config(),
            None
        );
    }

    #[test]
    fn service_window_runner_reports_missing_owner_context() {
        let mut owners = owners(2);
        let missing = RemoteFreeServiceRuntimeOwnerId::new(9);
        let drift = summary(report(config(256), 96, 524_288, 0));

        assert_eq!(
            owners.observe_service_window([RemoteFreeServiceRuntimeWindowObservation::new(
                missing, drift,
            )]),
            Err(RemoteFreeServiceRuntimeWindowError {
                owner_id: missing,
                source: RemoteFreeServiceRuntimeRetuneOwnerError::MissingOwner {
                    owner_id: missing,
                },
            })
        );
    }

    #[test]
    fn service_window_collector_routes_borrowed_owner_summaries() {
        let mut owners = owners(2);
        let owner_id = owners
            .register_owner(RemoteFreeOwnerRuntime::<usize>::new(config(256)).expect("runtime"));

        let first = owners
            .collect_service_window([owner_id], |_, runtime| {
                Ok::<_, &'static str>(summary(report(runtime.config(), 96, 524_288, 0)))
            })
            .expect("hold");
        let second = owners
            .collect_service_window([owner_id], |_, runtime| {
                Ok::<_, &'static str>(summary(report(runtime.config(), 96, 524_288, 0)))
            })
            .expect("apply");
        let third = owners
            .collect_service_window([owner_id], |_, runtime| {
                Ok::<_, &'static str>(summary(report(runtime.config(), 64, 262_144, 0)))
            })
            .expect("confirm");

        let mut total = RemoteFreeServiceRuntimeWindowStats::new();
        total.merge(first);
        total.merge(second);
        total.merge(third);

        assert_eq!(total.owner_observations(), 3);
        assert_eq!(total.reports_needing_retune(), 2);
        assert_eq!(total.hold_decisions(), 1);
        assert_eq!(total.apply_decisions(), 1);
        assert_eq!(total.confirmed_decisions(), 1);
        assert_eq!(
            owners.owner(owner_id).expect("owner").previous_config(),
            None
        );
    }

    #[test]
    fn service_window_collector_reports_missing_owner_context() {
        let mut owners = owners(2);
        let missing = RemoteFreeServiceRuntimeOwnerId::new(11);

        assert_eq!(
            owners.collect_service_window([missing], |_, runtime| {
                Ok::<_, &'static str>(summary(report(runtime.config(), 96, 524_288, 0)))
            }),
            Err(RemoteFreeServiceRuntimeWindowCollectionError::MissingOwner { owner_id: missing })
        );
    }

    #[test]
    fn service_window_collector_reports_collection_error_context() {
        let mut owners = owners(2);
        let owner_id = owners
            .register_owner(RemoteFreeOwnerRuntime::<usize>::new(config(256)).expect("runtime"));

        assert_eq!(
            owners.collect_service_window([owner_id], |_, _| {
                Err::<RemoteFreeServiceRetuneSummary, _>("collection failed")
            }),
            Err(RemoteFreeServiceRuntimeWindowCollectionError::Collect {
                owner_id,
                source: "collection failed",
            })
        );
    }
}
