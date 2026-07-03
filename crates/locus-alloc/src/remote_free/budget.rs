use std::fmt;
use std::num::NonZeroU64;

use super::RemoteFreeDrainPolicy;

/// Non-zero retained-byte budget for queued remote-free work.
///
/// The budget keeps domain sizing arithmetic explicit while centralizing zero
/// validation and checked multiplication.
///
/// ```
/// use locus_alloc::{RemoteFreeDrainReason, RemoteFreeDrainObservation};
/// use locus_alloc::{RemoteFreeDrainDecision, RemoteFreeQueuedByteBudget};
///
/// let budget = RemoteFreeQueuedByteBudget::from_grouped_item_shape(
///     4,     // active requests
///     16,    // remotely freed blocks per request
///     4096,  // representative bytes per block
/// )
/// .expect("budget");
/// let policy = budget.into_policy();
///
/// assert_eq!(budget.bytes(), 262_144);
/// assert_eq!(
///     policy.decide(RemoteFreeDrainObservation::new(64, 262_144, 1)),
///     RemoteFreeDrainDecision::Drain(RemoteFreeDrainReason::QueuedBytes)
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RemoteFreeQueuedByteBudget {
    bytes: NonZeroU64,
}

/// Failure to derive a remote-free queued-byte budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteFreeQueuedByteBudgetError {
    /// The final queued-byte budget was zero.
    ZeroQueuedBytes,
    /// Pending item count was zero.
    ZeroPendingItems,
    /// Group count was zero.
    ZeroGroups,
    /// Items per group was zero.
    ZeroItemsPerGroup,
    /// Bytes per pending item was zero.
    ZeroBytesPerItem,
    /// Multiplying groups by items per group overflowed.
    PendingItemsOverflow {
        /// Number of groups.
        groups: u64,
        /// Number of items per group.
        items_per_group: u64,
    },
    /// Multiplying pending items by bytes per item overflowed.
    RetainedBytesOverflow {
        /// Number of pending items.
        pending_items: u64,
        /// Representative bytes retained by each pending item.
        bytes_per_item: u64,
    },
}

impl RemoteFreeQueuedByteBudget {
    /// Creates a budget from already validated non-zero bytes.
    #[must_use]
    pub const fn new(bytes: NonZeroU64) -> Self {
        Self { bytes }
    }

    /// Creates a budget from raw bytes.
    ///
    /// # Errors
    ///
    /// Returns an error when `bytes` is zero.
    pub fn try_new(bytes: u64) -> Result<Self, RemoteFreeQueuedByteBudgetError> {
        NonZeroU64::new(bytes)
            .map(Self::new)
            .ok_or(RemoteFreeQueuedByteBudgetError::ZeroQueuedBytes)
    }

    /// Derives a budget from pending item count and representative item size.
    ///
    /// # Errors
    ///
    /// Returns an error when either input is zero or when multiplication
    /// overflows `u64`.
    pub fn from_item_shape(
        pending_items: u64,
        bytes_per_item: u64,
    ) -> Result<Self, RemoteFreeQueuedByteBudgetError> {
        if pending_items == 0 {
            return Err(RemoteFreeQueuedByteBudgetError::ZeroPendingItems);
        }
        if bytes_per_item == 0 {
            return Err(RemoteFreeQueuedByteBudgetError::ZeroBytesPerItem);
        }

        let retained_bytes = pending_items.checked_mul(bytes_per_item).ok_or(
            RemoteFreeQueuedByteBudgetError::RetainedBytesOverflow {
                pending_items,
                bytes_per_item,
            },
        )?;

        Self::try_new(retained_bytes)
    }

    /// Derives a budget from grouped pending items.
    ///
    /// This matches shapes such as request concurrency, remote-free blocks per
    /// request, and representative bytes per block.
    ///
    /// # Errors
    ///
    /// Returns an error when any input is zero or when multiplication overflows
    /// `u64`.
    pub fn from_grouped_item_shape(
        groups: u64,
        items_per_group: u64,
        bytes_per_item: u64,
    ) -> Result<Self, RemoteFreeQueuedByteBudgetError> {
        if groups == 0 {
            return Err(RemoteFreeQueuedByteBudgetError::ZeroGroups);
        }
        if items_per_group == 0 {
            return Err(RemoteFreeQueuedByteBudgetError::ZeroItemsPerGroup);
        }

        let pending_items = groups.checked_mul(items_per_group).ok_or(
            RemoteFreeQueuedByteBudgetError::PendingItemsOverflow {
                groups,
                items_per_group,
            },
        )?;

        Self::from_item_shape(pending_items, bytes_per_item)
    }

    /// Returns the queued-byte budget.
    #[must_use]
    pub const fn bytes(self) -> u64 {
        self.bytes.get()
    }

    /// Returns the queued-byte budget as `NonZeroU64`.
    #[must_use]
    pub const fn as_non_zero_u64(self) -> NonZeroU64 {
        self.bytes
    }

    /// Builds a queued-byte-only remote-free drain policy.
    #[must_use]
    pub const fn into_policy(self) -> RemoteFreeDrainPolicy {
        RemoteFreeDrainPolicy::new().with_max_queued_bytes(self.bytes)
    }
}

impl fmt::Display for RemoteFreeQueuedByteBudgetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroQueuedBytes => {
                f.write_str("remote-free queued-byte budget must be non-zero")
            }
            Self::ZeroPendingItems => f.write_str("remote-free pending item count must be non-zero"),
            Self::ZeroGroups => f.write_str("remote-free grouped budget count must be non-zero"),
            Self::ZeroItemsPerGroup => {
                f.write_str("remote-free grouped budget items per group must be non-zero")
            }
            Self::ZeroBytesPerItem => {
                f.write_str("remote-free queued-byte item size must be non-zero")
            }
            Self::PendingItemsOverflow {
                groups,
                items_per_group,
            } => write!(
                f,
                "remote-free pending item count overflowed for {groups} groups and {items_per_group} items per group"
            ),
            Self::RetainedBytesOverflow {
                pending_items,
                bytes_per_item,
            } => write!(
                f,
                "remote-free queued-byte budget overflowed for {pending_items} items and {bytes_per_item} bytes per item"
            ),
        }
    }
}

impl std::error::Error for RemoteFreeQueuedByteBudgetError {}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    use super::{RemoteFreeQueuedByteBudget, RemoteFreeQueuedByteBudgetError};
    use crate::{RemoteFreeDrainDecision, RemoteFreeDrainObservation, RemoteFreeDrainReason};

    #[test]
    fn remote_free_queued_byte_budget_accepts_non_zero_bytes() {
        let budget = RemoteFreeQueuedByteBudget::new(NonZeroU64::new(4096).expect("non-zero"));

        assert_eq!(budget.bytes(), 4096);
        assert_eq!(budget.as_non_zero_u64().get(), 4096);
    }

    #[test]
    fn remote_free_queued_byte_budget_rejects_zero_bytes() {
        assert_eq!(
            RemoteFreeQueuedByteBudget::try_new(0),
            Err(RemoteFreeQueuedByteBudgetError::ZeroQueuedBytes)
        );
    }

    #[test]
    fn remote_free_queued_byte_budget_derives_item_shape() {
        let budget =
            RemoteFreeQueuedByteBudget::from_item_shape(64, 4096).expect("budget from shape");

        assert_eq!(budget.bytes(), 262_144);
    }

    #[test]
    fn remote_free_queued_byte_budget_rejects_zero_item_shape_inputs() {
        assert_eq!(
            RemoteFreeQueuedByteBudget::from_item_shape(0, 4096),
            Err(RemoteFreeQueuedByteBudgetError::ZeroPendingItems)
        );
        assert_eq!(
            RemoteFreeQueuedByteBudget::from_item_shape(64, 0),
            Err(RemoteFreeQueuedByteBudgetError::ZeroBytesPerItem)
        );
    }

    #[test]
    fn remote_free_queued_byte_budget_rejects_item_shape_overflow() {
        assert_eq!(
            RemoteFreeQueuedByteBudget::from_item_shape(u64::MAX, 2),
            Err(RemoteFreeQueuedByteBudgetError::RetainedBytesOverflow {
                pending_items: u64::MAX,
                bytes_per_item: 2,
            })
        );
    }

    #[test]
    fn remote_free_queued_byte_budget_derives_grouped_item_shape() {
        let budget = RemoteFreeQueuedByteBudget::from_grouped_item_shape(4, 16, 10 * 1024)
            .expect("budget from grouped shape");

        assert_eq!(budget.bytes(), 655_360);
    }

    #[test]
    fn remote_free_queued_byte_budget_rejects_zero_grouped_item_shape_inputs() {
        assert_eq!(
            RemoteFreeQueuedByteBudget::from_grouped_item_shape(0, 16, 4096),
            Err(RemoteFreeQueuedByteBudgetError::ZeroGroups)
        );
        assert_eq!(
            RemoteFreeQueuedByteBudget::from_grouped_item_shape(4, 0, 4096),
            Err(RemoteFreeQueuedByteBudgetError::ZeroItemsPerGroup)
        );
        assert_eq!(
            RemoteFreeQueuedByteBudget::from_grouped_item_shape(4, 16, 0),
            Err(RemoteFreeQueuedByteBudgetError::ZeroBytesPerItem)
        );
    }

    #[test]
    fn remote_free_queued_byte_budget_rejects_grouped_item_shape_overflow() {
        assert_eq!(
            RemoteFreeQueuedByteBudget::from_grouped_item_shape(u64::MAX, 2, 4096),
            Err(RemoteFreeQueuedByteBudgetError::PendingItemsOverflow {
                groups: u64::MAX,
                items_per_group: 2,
            })
        );
        assert_eq!(
            RemoteFreeQueuedByteBudget::from_grouped_item_shape(2, u64::MAX / 2, 3),
            Err(RemoteFreeQueuedByteBudgetError::RetainedBytesOverflow {
                pending_items: u64::MAX - 1,
                bytes_per_item: 3,
            })
        );
    }

    #[test]
    fn remote_free_queued_byte_budget_builds_queued_byte_policy() {
        let budget =
            RemoteFreeQueuedByteBudget::from_item_shape(64, 4096).expect("budget from shape");
        let policy = budget.into_policy();

        assert_eq!(
            policy.decide(RemoteFreeDrainObservation::new(63, 258_048, 8)),
            RemoteFreeDrainDecision::Defer
        );
        assert_eq!(
            policy.decide(RemoteFreeDrainObservation::new(64, 262_144, 1)),
            RemoteFreeDrainDecision::Drain(RemoteFreeDrainReason::QueuedBytes)
        );
    }
}
