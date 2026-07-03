use std::fmt;

use super::{
    RemoteFreeDrainController, RemoteFreeDrainControllerError, RemoteFreeDrainControllerStatus,
    RemoteFreeDrainStats, RemoteFreeQueue, RemoteFreeQueueError, RemoteFreeQueueStats,
    RemoteFreeQueuedByteDrainConfig, RemoteFreeServiceRetuneCandidate,
    RemoteFreeServiceRetuneGuardDecision, RemoteFreeServiceRetunePolicyApplication,
};

/// Owner-side queue and controller runtime for applying remote-free configs.
#[derive(Debug)]
pub struct RemoteFreeOwnerRuntime<T> {
    config: RemoteFreeQueuedByteDrainConfig,
    previous_config: Option<RemoteFreeQueuedByteDrainConfig>,
    queue: RemoteFreeQueue<T>,
    controller: RemoteFreeDrainController,
}

/// Result of applying a guarded remote-free policy application plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteFreeOwnerRuntimeApplyOutcome {
    /// No queue or controller state changed.
    NoChange {
        /// Guard decision that produced no live config change.
        decision: RemoteFreeServiceRetuneGuardDecision,
    },
    /// Queue and controller were rebuilt with an applied config.
    Installed {
        /// Candidate represented by the installed config.
        candidate: RemoteFreeServiceRetuneCandidate,
        /// Config active before the install.
        previous_config: RemoteFreeQueuedByteDrainConfig,
        /// Config active after the install.
        current_config: RemoteFreeQueuedByteDrainConfig,
    },
}

/// Result of rolling back an owner runtime config.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteFreeOwnerRuntimeRollbackOutcome {
    /// Config replaced by the rollback.
    pub replaced_config: RemoteFreeQueuedByteDrainConfig,
    /// Config restored by the rollback.
    pub restored_config: RemoteFreeQueuedByteDrainConfig,
}

/// Failure while applying or rolling back owner runtime config.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteFreeOwnerRuntimeError {
    /// Queue construction failed.
    Queue(RemoteFreeQueueError),
    /// Controller accounting failed.
    Controller(RemoteFreeDrainControllerError),
    /// Queue or controller still had pending remote-free work.
    NonEmptyBoundary {
        /// Pending item count reported by the queue.
        queue_pending_count: u64,
        /// Pending item count tracked by the controller.
        controller_pending_count: u64,
        /// Retained queued bytes tracked by the controller.
        controller_queued_bytes: u64,
    },
    /// No previous config was available for rollback.
    MissingRollbackConfig,
}

impl<T> RemoteFreeOwnerRuntime<T> {
    /// Creates an owner runtime from a validated config.
    ///
    /// # Errors
    ///
    /// Returns an error if queue construction fails.
    pub fn new(
        config: RemoteFreeQueuedByteDrainConfig,
    ) -> Result<Self, RemoteFreeOwnerRuntimeError> {
        Ok(Self {
            config,
            previous_config: None,
            queue: config.queue().map_err(RemoteFreeOwnerRuntimeError::Queue)?,
            controller: RemoteFreeDrainController::new(config.drain_policy()),
        })
    }

    /// Returns the current validated queue and drain config.
    #[must_use]
    pub const fn config(&self) -> RemoteFreeQueuedByteDrainConfig {
        self.config
    }

    /// Returns the rollback config, if an install has not yet been confirmed.
    #[must_use]
    pub const fn previous_config(&self) -> Option<RemoteFreeQueuedByteDrainConfig> {
        self.previous_config
    }

    /// Returns the current queue accounting.
    #[must_use]
    pub fn queue_stats(&self) -> RemoteFreeQueueStats {
        self.queue.stats()
    }

    /// Returns a fresh sink for the current queue generation.
    #[must_use]
    pub fn sink(&self) -> super::RemoteFreeSink<T> {
        self.queue.sink()
    }

    /// Records one successful enqueue in controller accounting.
    pub fn record_submit(&mut self, submit_turn: u64, queued_bytes: u64) {
        self.controller.record_submit(submit_turn, queued_bytes);
    }

    /// Builds owner-side policy status for the current queue and turn.
    ///
    /// # Errors
    ///
    /// Returns an error if queue and controller pending counts diverge.
    pub fn status(
        &self,
        current_turn: u64,
    ) -> Result<RemoteFreeDrainControllerStatus, RemoteFreeDrainControllerError> {
        self.controller.status_for_queue(&self.queue, current_turn)
    }

    /// Drains one owner batch and records release sizes in the controller.
    ///
    /// The release closure must return the number of bytes released by each
    /// drained item.
    ///
    /// # Errors
    ///
    /// Returns an error if controller drain accounting fails.
    pub fn drain_batch(
        &mut self,
        mut release: impl FnMut(T) -> u64,
    ) -> Result<RemoteFreeDrainStats, RemoteFreeOwnerRuntimeError> {
        let controller = &mut self.controller;
        let mut first_error = None;

        let stats = self.queue.drain_batch(|item| {
            let released_bytes = release(item);
            if first_error.is_none() {
                if let Err(error) = controller.record_drain(released_bytes) {
                    first_error = Some(error);
                }
            }
        });

        if let Some(error) = first_error {
            return Err(RemoteFreeOwnerRuntimeError::Controller(error));
        }

        Ok(stats)
    }

    /// Applies a guarded policy application plan at an empty owner boundary.
    ///
    /// # Errors
    ///
    /// Returns an error if the owner has pending work or queue construction
    /// fails.
    pub fn apply(
        &mut self,
        application: RemoteFreeServiceRetunePolicyApplication,
    ) -> Result<RemoteFreeOwnerRuntimeApplyOutcome, RemoteFreeOwnerRuntimeError> {
        match application {
            RemoteFreeServiceRetunePolicyApplication::NoChange { decision } => {
                Ok(RemoteFreeOwnerRuntimeApplyOutcome::NoChange { decision })
            }
            RemoteFreeServiceRetunePolicyApplication::Apply { candidate, config } => {
                self.install_config(candidate, config)
            }
        }
    }

    /// Rolls back to the previous config at an empty owner boundary.
    ///
    /// # Errors
    ///
    /// Returns an error if no previous config exists, the owner has pending
    /// work, or queue construction fails.
    pub fn rollback(
        &mut self,
    ) -> Result<RemoteFreeOwnerRuntimeRollbackOutcome, RemoteFreeOwnerRuntimeError> {
        self.ensure_empty_boundary()?;
        let restored_config = self
            .previous_config
            .take()
            .ok_or(RemoteFreeOwnerRuntimeError::MissingRollbackConfig)?;
        let replaced_config = self.config;

        self.rebuild(restored_config)?;

        Ok(RemoteFreeOwnerRuntimeRollbackOutcome {
            replaced_config,
            restored_config,
        })
    }

    fn install_config(
        &mut self,
        candidate: RemoteFreeServiceRetuneCandidate,
        next_config: RemoteFreeQueuedByteDrainConfig,
    ) -> Result<RemoteFreeOwnerRuntimeApplyOutcome, RemoteFreeOwnerRuntimeError> {
        self.ensure_empty_boundary()?;
        let previous_config = self.config;

        self.rebuild(next_config)?;
        self.previous_config = Some(previous_config);

        Ok(RemoteFreeOwnerRuntimeApplyOutcome::Installed {
            candidate,
            previous_config,
            current_config: next_config,
        })
    }

    fn rebuild(
        &mut self,
        config: RemoteFreeQueuedByteDrainConfig,
    ) -> Result<(), RemoteFreeOwnerRuntimeError> {
        self.queue = config.queue().map_err(RemoteFreeOwnerRuntimeError::Queue)?;
        self.controller = RemoteFreeDrainController::new(config.drain_policy());
        self.config = config;
        Ok(())
    }

    fn ensure_empty_boundary(&self) -> Result<(), RemoteFreeOwnerRuntimeError> {
        let queue_stats = self.queue.stats();
        let controller_pending_count = self.controller.pending_count();
        let controller_queued_bytes = self.controller.queued_bytes();

        if queue_stats.pending_count == 0
            && controller_pending_count == 0
            && controller_queued_bytes == 0
        {
            return Ok(());
        }

        Err(RemoteFreeOwnerRuntimeError::NonEmptyBoundary {
            queue_pending_count: queue_stats.pending_count,
            controller_pending_count,
            controller_queued_bytes,
        })
    }
}

impl fmt::Display for RemoteFreeOwnerRuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Queue(source) => write!(f, "{source}"),
            Self::Controller(source) => write!(f, "{source}"),
            Self::NonEmptyBoundary {
                queue_pending_count,
                controller_pending_count,
                controller_queued_bytes,
            } => write!(
                f,
                "remote-free owner boundary is not empty: queue_pending_count={queue_pending_count} controller_pending_count={controller_pending_count} controller_queued_bytes={controller_queued_bytes}"
            ),
            Self::MissingRollbackConfig => {
                f.write_str("remote-free owner runtime has no config to roll back to")
            }
        }
    }
}

impl std::error::Error for RemoteFreeOwnerRuntimeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Queue(source) => Some(source),
            Self::Controller(source) => Some(source),
            Self::NonEmptyBoundary { .. } | Self::MissingRollbackConfig => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        RemoteFreeOwnerRuntime, RemoteFreeOwnerRuntimeApplyOutcome, RemoteFreeOwnerRuntimeError,
        RemoteFreeOwnerRuntimeRollbackOutcome,
    };
    use crate::{
        RemoteFreeDrainDecision, RemoteFreeDrainReason, RemoteFreeQueuedByteDrainConfig,
        RemoteFreeServiceRetuneCandidate, RemoteFreeServiceRetuneGuardDecision,
        RemoteFreeServiceRetunePolicyApplication, RemoteFreeServiceRetunePolicyApplicator,
        RemoteFreeTryEnqueueErrorKind,
    };

    fn config(queue_capacity: usize) -> RemoteFreeQueuedByteDrainConfig {
        RemoteFreeQueuedByteDrainConfig::from_item_shape(queue_capacity, 64, 64, 4096)
            .expect("config")
    }

    fn application(
        current_config: RemoteFreeQueuedByteDrainConfig,
        candidate: RemoteFreeServiceRetuneCandidate,
    ) -> RemoteFreeServiceRetunePolicyApplication {
        RemoteFreeServiceRetunePolicyApplicator::try_new(2)
            .expect("applicator")
            .plan(
                current_config,
                RemoteFreeServiceRetuneGuardDecision::Apply { candidate },
            )
            .expect("application")
    }

    #[test]
    fn owner_runtime_builds_queue_and_controller_from_config() {
        let runtime = RemoteFreeOwnerRuntime::<usize>::new(config(128)).expect("runtime");

        assert_eq!(runtime.config().queue_capacity(), 128);
        assert_eq!(runtime.previous_config(), None);
        assert_eq!(runtime.queue_stats().capacity, 128);
        assert_eq!(runtime.queue_stats().batch_limit, 64);
        assert_eq!(
            runtime.status(1).expect("status").decision,
            RemoteFreeDrainDecision::Defer
        );
    }

    #[test]
    fn owner_runtime_installs_apply_plan_at_empty_boundary() {
        let mut runtime = RemoteFreeOwnerRuntime::<usize>::new(config(128)).expect("runtime");
        let application = application(
            runtime.config(),
            RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacity,
        );

        assert_eq!(
            runtime.apply(application),
            Ok(RemoteFreeOwnerRuntimeApplyOutcome::Installed {
                candidate: RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacity,
                previous_config: config(128),
                current_config: config(256),
            })
        );
        assert_eq!(runtime.config(), config(256));
        assert_eq!(runtime.previous_config(), Some(config(128)));
        assert_eq!(runtime.queue_stats().capacity, 256);
    }

    #[test]
    fn owner_runtime_no_change_plan_does_not_store_rollback_config() {
        let mut runtime = RemoteFreeOwnerRuntime::<usize>::new(config(128)).expect("runtime");
        let decision = RemoteFreeServiceRetuneGuardDecision::Hold {
            candidate: RemoteFreeServiceRetuneCandidate::DrainEarlier,
            consecutive_candidate_windows: 1,
        };

        assert_eq!(
            runtime.apply(RemoteFreeServiceRetunePolicyApplication::NoChange { decision }),
            Ok(RemoteFreeOwnerRuntimeApplyOutcome::NoChange { decision })
        );
        assert_eq!(runtime.config(), config(128));
        assert_eq!(runtime.previous_config(), None);
    }

    #[test]
    fn owner_runtime_rolls_back_to_previous_config_at_empty_boundary() {
        let mut runtime = RemoteFreeOwnerRuntime::<usize>::new(config(128)).expect("runtime");
        let application = application(
            runtime.config(),
            RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacity,
        );
        runtime.apply(application).expect("install");

        assert_eq!(
            runtime.rollback(),
            Ok(RemoteFreeOwnerRuntimeRollbackOutcome {
                replaced_config: config(256),
                restored_config: config(128),
            })
        );
        assert_eq!(runtime.config(), config(128));
        assert_eq!(runtime.previous_config(), None);
        assert_eq!(runtime.queue_stats().capacity, 128);
    }

    #[test]
    fn owner_runtime_rejects_missing_rollback_config() {
        let mut runtime = RemoteFreeOwnerRuntime::<usize>::new(config(128)).expect("runtime");

        assert_eq!(
            runtime.rollback(),
            Err(RemoteFreeOwnerRuntimeError::MissingRollbackConfig)
        );
    }

    #[test]
    fn owner_runtime_rejects_install_with_pending_work() {
        let mut runtime = RemoteFreeOwnerRuntime::new(config(128)).expect("runtime");
        let sink = runtime.sink();
        sink.enqueue(vec![0_u8; 4096]).expect("enqueue");
        runtime.record_submit(0, 4096);
        let application = application(
            runtime.config(),
            RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacity,
        );

        assert_eq!(
            runtime.apply(application),
            Err(RemoteFreeOwnerRuntimeError::NonEmptyBoundary {
                queue_pending_count: 1,
                controller_pending_count: 1,
                controller_queued_bytes: 4096,
            })
        );
    }

    #[test]
    fn owner_runtime_rejects_rollback_with_pending_work() {
        let mut runtime = RemoteFreeOwnerRuntime::new(config(128)).expect("runtime");
        let application = application(
            runtime.config(),
            RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacity,
        );
        runtime.apply(application).expect("install");

        let sink = runtime.sink();
        sink.enqueue(vec![0_u8; 4096]).expect("enqueue");
        runtime.record_submit(0, 4096);

        assert_eq!(
            runtime.rollback(),
            Err(RemoteFreeOwnerRuntimeError::NonEmptyBoundary {
                queue_pending_count: 1,
                controller_pending_count: 1,
                controller_queued_bytes: 4096,
            })
        );
    }

    #[test]
    fn owner_runtime_drain_batch_records_released_bytes() {
        let mut runtime = RemoteFreeOwnerRuntime::new(config(128)).expect("runtime");
        let sink = runtime.sink();
        sink.enqueue(vec![0_u8; 4096]).expect("enqueue");
        runtime.record_submit(0, 4096);

        let stats = runtime
            .drain_batch(|allocation| u64::try_from(allocation.len()).expect("len fits u64"))
            .expect("drain");

        assert_eq!(stats.drained, 1);
        assert_eq!(runtime.queue_stats().pending_count, 0);
        assert_eq!(
            runtime.status(1).expect("status").decision,
            RemoteFreeDrainDecision::Defer
        );
    }

    #[test]
    fn owner_runtime_rebuild_disconnects_old_sinks() {
        let mut runtime = RemoteFreeOwnerRuntime::<usize>::new(config(128)).expect("runtime");
        let old_sink = runtime.sink();
        let application = application(
            runtime.config(),
            RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacity,
        );
        runtime.apply(application).expect("install");

        let error = old_sink.try_enqueue(1).expect_err("old sink disconnected");
        assert_eq!(error.kind(), RemoteFreeTryEnqueueErrorKind::Disconnected);

        let new_sink = runtime.sink();
        new_sink.try_enqueue(2).expect("new sink live");
        runtime.record_submit(0, 4096);
        assert_eq!(
            runtime.status(1).expect("status").decision,
            RemoteFreeDrainDecision::Defer
        );
    }

    #[test]
    fn owner_runtime_status_uses_installed_policy() {
        let mut runtime = RemoteFreeOwnerRuntime::new(config(128)).expect("runtime");
        let sink = runtime.sink();

        for _ in 0..64 {
            sink.enqueue(vec![0_u8; 4096]).expect("enqueue");
            runtime.record_submit(0, 4096);
        }

        assert_eq!(
            runtime.status(1).expect("status").decision,
            RemoteFreeDrainDecision::Drain(RemoteFreeDrainReason::QueuedBytes)
        );
    }
}
