use std::fmt;
use std::num::{NonZeroU64, NonZeroUsize};

use super::{
    RemoteFreeDrainPolicy, RemoteFreeQueue, RemoteFreeQueueError, RemoteFreeQueuedByteBudget,
    RemoteFreeQueuedByteBudgetError,
};

/// Validated queue and retained-byte policy sizing for queued remote frees.
///
/// The config ties a retained pending-item window to queue capacity, drain
/// batch size, and queued-byte policy budget. It does not own allocator
/// release behavior, which remains in the `RemoteFreeQueue::drain_batch`
/// closure.
///
/// ```
/// use locus_alloc::{RemoteFreeDrainReason, RemoteFreeDrainObservation};
/// use locus_alloc::{RemoteFreeDrainDecision, RemoteFreeQueuedByteDrainConfig};
///
/// let config = RemoteFreeQueuedByteDrainConfig::from_grouped_item_shape(
///     256,   // queue capacity
///     64,    // drain batch limit
///     4,     // active requests
///     16,    // remotely freed blocks per request
///     4096,  // representative bytes per block
/// )
/// .expect("config");
/// let policy = config.drain_policy();
///
/// assert_eq!(config.target_pending_items(), 64);
/// assert_eq!(config.queued_byte_budget().bytes(), 262_144);
/// assert_eq!(
///     policy.decide(RemoteFreeDrainObservation::new(64, 262_144, 1)),
///     RemoteFreeDrainDecision::Drain(RemoteFreeDrainReason::QueuedBytes)
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteFreeQueuedByteDrainConfig {
    queue_capacity: NonZeroUsize,
    drain_batch_limit: NonZeroUsize,
    target_pending_items: NonZeroU64,
    queued_byte_budget: RemoteFreeQueuedByteBudget,
}

/// Failure to build a queued-byte remote-free drain config.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteFreeQueuedByteDrainConfigError {
    /// Queue capacity was zero.
    ZeroQueueCapacity,
    /// Drain batch limit was zero.
    ZeroDrainBatchLimit,
    /// Target pending item window was zero.
    ZeroTargetPendingItems,
    /// Target pending item window cannot fit in `usize`.
    TargetPendingItemsExceedUsize {
        /// Target pending items.
        target_pending_items: u64,
    },
    /// Queue capacity cannot hold the target pending item window.
    QueueCapacityBelowTarget {
        /// Queue capacity.
        queue_capacity: usize,
        /// Target pending items.
        target_pending_items: u64,
    },
    /// Drain batch size cannot clear the target pending item window.
    DrainBatchLimitBelowTarget {
        /// Drain batch limit.
        drain_batch_limit: usize,
        /// Target pending items.
        target_pending_items: u64,
    },
    /// Retained-byte budget derivation failed.
    Budget(RemoteFreeQueuedByteBudgetError),
}

impl RemoteFreeQueuedByteDrainConfig {
    /// Creates a config from validated queue sizing and retained-byte budget.
    ///
    /// # Errors
    ///
    /// Returns an error when any size is zero, when the pending item target
    /// cannot fit in `usize`, or when queue capacity or drain batch limit are
    /// below the target pending item window.
    pub fn new(
        queue_capacity: usize,
        drain_batch_limit: usize,
        target_pending_items: u64,
        queued_byte_budget: RemoteFreeQueuedByteBudget,
    ) -> Result<Self, RemoteFreeQueuedByteDrainConfigError> {
        let queue_capacity = NonZeroUsize::new(queue_capacity)
            .ok_or(RemoteFreeQueuedByteDrainConfigError::ZeroQueueCapacity)?;
        let drain_batch_limit = NonZeroUsize::new(drain_batch_limit)
            .ok_or(RemoteFreeQueuedByteDrainConfigError::ZeroDrainBatchLimit)?;
        let target_pending_items = NonZeroU64::new(target_pending_items)
            .ok_or(RemoteFreeQueuedByteDrainConfigError::ZeroTargetPendingItems)?;
        let target_pending_items_usize =
            usize::try_from(target_pending_items.get()).map_err(|_| {
                RemoteFreeQueuedByteDrainConfigError::TargetPendingItemsExceedUsize {
                    target_pending_items: target_pending_items.get(),
                }
            })?;

        if queue_capacity.get() < target_pending_items_usize {
            return Err(
                RemoteFreeQueuedByteDrainConfigError::QueueCapacityBelowTarget {
                    queue_capacity: queue_capacity.get(),
                    target_pending_items: target_pending_items.get(),
                },
            );
        }
        if drain_batch_limit.get() < target_pending_items_usize {
            return Err(
                RemoteFreeQueuedByteDrainConfigError::DrainBatchLimitBelowTarget {
                    drain_batch_limit: drain_batch_limit.get(),
                    target_pending_items: target_pending_items.get(),
                },
            );
        }

        Ok(Self {
            queue_capacity,
            drain_batch_limit,
            target_pending_items,
            queued_byte_budget,
        })
    }

    /// Creates a config from uniform retained item shape inputs.
    ///
    /// # Errors
    ///
    /// Returns an error when queue sizing is invalid or when budget derivation
    /// fails.
    pub fn from_item_shape(
        queue_capacity: usize,
        drain_batch_limit: usize,
        target_pending_items: u64,
        bytes_per_item: u64,
    ) -> Result<Self, RemoteFreeQueuedByteDrainConfigError> {
        let queued_byte_budget =
            RemoteFreeQueuedByteBudget::from_item_shape(target_pending_items, bytes_per_item)
                .map_err(RemoteFreeQueuedByteDrainConfigError::Budget)?;

        Self::new(
            queue_capacity,
            drain_batch_limit,
            target_pending_items,
            queued_byte_budget,
        )
    }

    /// Creates a config from grouped retained item shape inputs.
    ///
    /// # Errors
    ///
    /// Returns an error when queue sizing is invalid or when budget derivation
    /// fails.
    pub fn from_grouped_item_shape(
        queue_capacity: usize,
        drain_batch_limit: usize,
        groups: u64,
        items_per_group: u64,
        bytes_per_item: u64,
    ) -> Result<Self, RemoteFreeQueuedByteDrainConfigError> {
        let target_pending_items = target_pending_items_for_grouped_shape(groups, items_per_group)?;
        let queued_byte_budget = RemoteFreeQueuedByteBudget::from_grouped_item_shape(
            groups,
            items_per_group,
            bytes_per_item,
        )
        .map_err(RemoteFreeQueuedByteDrainConfigError::Budget)?;

        Self::new(
            queue_capacity,
            drain_batch_limit,
            target_pending_items,
            queued_byte_budget,
        )
    }

    /// Returns validated queue capacity.
    #[must_use]
    pub const fn queue_capacity(self) -> usize {
        self.queue_capacity.get()
    }

    /// Returns validated drain batch limit.
    #[must_use]
    pub const fn drain_batch_limit(self) -> usize {
        self.drain_batch_limit.get()
    }

    /// Returns the retained pending item window.
    #[must_use]
    pub const fn target_pending_items(self) -> u64 {
        self.target_pending_items.get()
    }

    /// Returns the queued-byte budget.
    #[must_use]
    pub const fn queued_byte_budget(self) -> RemoteFreeQueuedByteBudget {
        self.queued_byte_budget
    }

    /// Builds the queued-byte drain policy represented by this config.
    #[must_use]
    pub const fn drain_policy(self) -> RemoteFreeDrainPolicy {
        self.queued_byte_budget.into_policy()
    }

    /// Builds a remote-free queue with the validated queue sizing.
    ///
    /// # Errors
    ///
    /// This delegates to `RemoteFreeQueue::new`, which can only fail if the
    /// validated non-zero queue sizing has been corrupted.
    pub fn queue<T>(self) -> Result<RemoteFreeQueue<T>, RemoteFreeQueueError> {
        RemoteFreeQueue::new(self.queue_capacity(), self.drain_batch_limit())
    }
}

fn target_pending_items_for_grouped_shape(
    groups: u64,
    items_per_group: u64,
) -> Result<u64, RemoteFreeQueuedByteDrainConfigError> {
    if groups == 0 {
        return Err(RemoteFreeQueuedByteDrainConfigError::Budget(
            RemoteFreeQueuedByteBudgetError::ZeroGroups,
        ));
    }
    if items_per_group == 0 {
        return Err(RemoteFreeQueuedByteDrainConfigError::Budget(
            RemoteFreeQueuedByteBudgetError::ZeroItemsPerGroup,
        ));
    }

    groups
        .checked_mul(items_per_group)
        .ok_or(RemoteFreeQueuedByteDrainConfigError::Budget(
            RemoteFreeQueuedByteBudgetError::PendingItemsOverflow {
                groups,
                items_per_group,
            },
        ))
}

impl fmt::Display for RemoteFreeQueuedByteDrainConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroQueueCapacity => f.write_str("remote-free queue capacity must be non-zero"),
            Self::ZeroDrainBatchLimit => {
                f.write_str("remote-free drain batch limit must be non-zero")
            }
            Self::ZeroTargetPendingItems => {
                f.write_str("remote-free target pending item window must be non-zero")
            }
            Self::TargetPendingItemsExceedUsize {
                target_pending_items,
            } => write!(
                f,
                "remote-free target pending item window {target_pending_items} does not fit usize"
            ),
            Self::QueueCapacityBelowTarget {
                queue_capacity,
                target_pending_items,
            } => write!(
                f,
                "remote-free queue capacity {queue_capacity} is below target pending item window {target_pending_items}"
            ),
            Self::DrainBatchLimitBelowTarget {
                drain_batch_limit,
                target_pending_items,
            } => write!(
                f,
                "remote-free drain batch limit {drain_batch_limit} is below target pending item window {target_pending_items}"
            ),
            Self::Budget(source) => write!(f, "{source}"),
        }
    }
}

impl std::error::Error for RemoteFreeQueuedByteDrainConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Budget(source) => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{RemoteFreeQueuedByteDrainConfig, RemoteFreeQueuedByteDrainConfigError};
    use crate::{
        RemoteFreeDrainDecision, RemoteFreeDrainObservation, RemoteFreeDrainReason,
        RemoteFreeQueuedByteBudget, RemoteFreeQueuedByteBudgetError,
    };

    #[test]
    fn remote_free_queued_byte_drain_config_accepts_valid_sizing() {
        let budget =
            RemoteFreeQueuedByteBudget::from_item_shape(64, 4096).expect("queued-byte budget");
        let config =
            RemoteFreeQueuedByteDrainConfig::new(256, 64, 64, budget).expect("drain config");

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
    fn remote_free_queued_byte_drain_config_derives_grouped_shape() {
        let config =
            RemoteFreeQueuedByteDrainConfig::from_grouped_item_shape(256, 64, 4, 16, 10 * 1024)
                .expect("drain config");

        assert_eq!(config.queue_capacity(), 256);
        assert_eq!(config.drain_batch_limit(), 64);
        assert_eq!(config.target_pending_items(), 64);
        assert_eq!(config.queued_byte_budget().bytes(), 655_360);
    }

    #[test]
    fn remote_free_queued_byte_drain_config_derives_uniform_item_shape() {
        let config = RemoteFreeQueuedByteDrainConfig::from_item_shape(256, 64, 64, 4096)
            .expect("drain config");

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
    fn remote_free_queued_byte_drain_config_builds_queue() {
        let config = RemoteFreeQueuedByteDrainConfig::from_grouped_item_shape(256, 64, 4, 16, 4096)
            .expect("drain config");
        let queue = config.queue::<Vec<u8>>().expect("queue");

        assert_eq!(queue.stats().capacity, 256);
        assert_eq!(queue.stats().batch_limit, 64);
    }

    #[test]
    fn remote_free_queued_byte_drain_config_rejects_zero_sizing() {
        let budget =
            RemoteFreeQueuedByteBudget::from_item_shape(64, 4096).expect("queued-byte budget");

        assert_eq!(
            RemoteFreeQueuedByteDrainConfig::new(0, 64, 64, budget),
            Err(RemoteFreeQueuedByteDrainConfigError::ZeroQueueCapacity)
        );
        assert_eq!(
            RemoteFreeQueuedByteDrainConfig::new(256, 0, 64, budget),
            Err(RemoteFreeQueuedByteDrainConfigError::ZeroDrainBatchLimit)
        );
        assert_eq!(
            RemoteFreeQueuedByteDrainConfig::new(256, 64, 0, budget),
            Err(RemoteFreeQueuedByteDrainConfigError::ZeroTargetPendingItems)
        );
    }

    #[test]
    fn remote_free_queued_byte_drain_config_rejects_queue_below_target() {
        let budget =
            RemoteFreeQueuedByteBudget::from_item_shape(64, 4096).expect("queued-byte budget");

        assert_eq!(
            RemoteFreeQueuedByteDrainConfig::new(63, 64, 64, budget),
            Err(
                RemoteFreeQueuedByteDrainConfigError::QueueCapacityBelowTarget {
                    queue_capacity: 63,
                    target_pending_items: 64,
                }
            )
        );
    }

    #[test]
    fn remote_free_queued_byte_drain_config_rejects_batch_below_target() {
        let budget =
            RemoteFreeQueuedByteBudget::from_item_shape(64, 4096).expect("queued-byte budget");

        assert_eq!(
            RemoteFreeQueuedByteDrainConfig::new(256, 63, 64, budget),
            Err(
                RemoteFreeQueuedByteDrainConfigError::DrainBatchLimitBelowTarget {
                    drain_batch_limit: 63,
                    target_pending_items: 64,
                }
            )
        );
    }

    #[test]
    fn remote_free_queued_byte_drain_config_rejects_pending_window_that_exceeds_usize() {
        if usize::BITS >= u64::BITS {
            return;
        }

        let budget =
            RemoteFreeQueuedByteBudget::from_item_shape(64, 4096).expect("queued-byte budget");

        assert_eq!(
            RemoteFreeQueuedByteDrainConfig::new(usize::MAX, usize::MAX, u64::MAX, budget),
            Err(
                RemoteFreeQueuedByteDrainConfigError::TargetPendingItemsExceedUsize {
                    target_pending_items: u64::MAX,
                }
            )
        );
    }

    #[test]
    fn remote_free_queued_byte_drain_config_propagates_grouped_budget_errors() {
        assert_eq!(
            RemoteFreeQueuedByteDrainConfig::from_grouped_item_shape(256, 64, 0, 16, 4096),
            Err(RemoteFreeQueuedByteDrainConfigError::Budget(
                RemoteFreeQueuedByteBudgetError::ZeroGroups,
            ))
        );
        assert_eq!(
            RemoteFreeQueuedByteDrainConfig::from_grouped_item_shape(256, 64, 4, 0, 4096),
            Err(RemoteFreeQueuedByteDrainConfigError::Budget(
                RemoteFreeQueuedByteBudgetError::ZeroItemsPerGroup,
            ))
        );
        assert_eq!(
            RemoteFreeQueuedByteDrainConfig::from_grouped_item_shape(256, 64, 4, 16, 0),
            Err(RemoteFreeQueuedByteDrainConfigError::Budget(
                RemoteFreeQueuedByteBudgetError::ZeroBytesPerItem,
            ))
        );
    }

    #[test]
    fn remote_free_queued_byte_drain_config_propagates_uniform_budget_errors() {
        assert_eq!(
            RemoteFreeQueuedByteDrainConfig::from_item_shape(256, 64, 0, 4096),
            Err(RemoteFreeQueuedByteDrainConfigError::Budget(
                RemoteFreeQueuedByteBudgetError::ZeroPendingItems,
            ))
        );
        assert_eq!(
            RemoteFreeQueuedByteDrainConfig::from_item_shape(256, 64, 64, 0),
            Err(RemoteFreeQueuedByteDrainConfigError::Budget(
                RemoteFreeQueuedByteBudgetError::ZeroBytesPerItem,
            ))
        );
        assert_eq!(
            RemoteFreeQueuedByteDrainConfig::from_item_shape(256, 64, u64::MAX, 2),
            Err(RemoteFreeQueuedByteDrainConfigError::Budget(
                RemoteFreeQueuedByteBudgetError::RetainedBytesOverflow {
                    pending_items: u64::MAX,
                    bytes_per_item: 2,
                },
            ))
        );
    }
}
