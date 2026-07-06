use std::collections::BTreeMap;
use std::fmt;

use crate::{
    choose_initial_policy, resolve_topology_policy, LifetimeHint, MemoryClass, NodeId,
    PlacementPolicy, PlacementRequest, Topology,
};

use crate::{MappedScratchAllocError, MappedScratchArena, MAX_SUPPORTED_ALIGN};

/// Budgeted pool of page-locked mapped scratch arenas.
#[derive(Debug)]
pub struct PinnedScratchPool {
    home_node: NodeId,
    arena_capacity: usize,
    arena_mapping_len: usize,
    max_locked_bytes: usize,
    locked_bytes: usize,
    idle: Vec<MappedScratchArena>,
    checked_out: BTreeMap<PinnedScratchHandle, MappedScratchArena>,
    next_handle: u64,
    created_arenas: u64,
    reused_arenas: u64,
    checkout_count: u64,
    release_count: u64,
}

/// Opaque handle for a checked-out pinned scratch arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PinnedScratchHandle(u64);

/// Pinned scratch pool accounting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PinnedScratchPoolStats {
    /// Pool home node.
    pub home_node: NodeId,
    /// Usable capacity of each arena.
    pub arena_capacity: usize,
    /// Locked mapping length of each arena.
    pub arena_mapping_len: usize,
    /// Maximum locked bytes allowed.
    pub max_locked_bytes: usize,
    /// Bytes locked by arenas owned by the pool.
    pub locked_bytes: usize,
    /// Checked-out arena count.
    pub checked_out: usize,
    /// Idle locked arena count.
    pub idle: usize,
    /// Arenas created by the pool.
    pub created_arenas: u64,
    /// Checkouts served from idle arenas.
    pub reused_arenas: u64,
    /// Successful checkout count.
    pub checkout_count: u64,
    /// Successful release count.
    pub release_count: u64,
}

impl PinnedScratchHandle {
    /// Returns the stable handle identifier.
    #[must_use]
    pub fn id(self) -> u64 {
        self.0
    }
}

impl PinnedScratchPool {
    /// Creates a pool of page-locked mapped scratch arenas.
    ///
    /// The pool creates and locks arenas lazily on checkout. Idle arenas remain
    /// locked and are reused until the pool is dropped.
    ///
    /// # Errors
    ///
    /// Returns an error when the arena capacity is zero, capacity math
    /// overflows, or the locked-byte budget cannot fit one arena mapping.
    pub fn new(
        home_node: NodeId,
        arena_capacity: usize,
        max_locked_bytes: usize,
    ) -> Result<Self, PinnedScratchPoolError> {
        if arena_capacity == 0 {
            return Err(PinnedScratchPoolError::InvalidArenaCapacity);
        }

        let arena_mapping_len = arena_capacity.checked_add(MAX_SUPPORTED_ALIGN - 1).ok_or(
            PinnedScratchPoolError::Arena(MappedScratchAllocError::CapacityOverflow),
        )?;
        if arena_mapping_len > max_locked_bytes {
            return Err(PinnedScratchPoolError::InsufficientLockedByteBudget {
                required: arena_mapping_len,
                max_locked_bytes,
            });
        }

        Ok(Self {
            home_node,
            arena_capacity,
            arena_mapping_len,
            max_locked_bytes,
            locked_bytes: 0,
            idle: Vec::new(),
            checked_out: BTreeMap::new(),
            next_handle: 0,
            created_arenas: 0,
            reused_arenas: 0,
            checkout_count: 0,
            release_count: 0,
        })
    }

    /// Creates a pool whose home node is resolved from GPU PCI locality.
    ///
    /// This resolves the GPU BDF through discovered topology and creates the
    /// pool on the reported NUMA node. It does not select a CUDA device or
    /// register host memory with a GPU runtime.
    ///
    /// # Errors
    ///
    /// Returns an error when topology cannot resolve the GPU to a concrete
    /// NUMA node, or when ordinary pool construction fails.
    pub fn new_near_gpu(
        gpu_bdf: impl Into<String>,
        topology: &Topology,
        arena_capacity: usize,
        max_locked_bytes: usize,
    ) -> Result<Self, PinnedScratchPoolError> {
        let gpu_bdf = gpu_bdf.into();
        let request = PlacementRequest {
            memory_class: MemoryClass::PinnedHost,
            lifetime: LifetimeHint::Process,
            preferred_node: None,
            preferred_gpu: Some(gpu_bdf.clone()),
        };
        let decision = choose_initial_policy(&request);
        let resolved = resolve_topology_policy(&decision, topology);

        let PlacementPolicy::Bind(nodes) = resolved.policy else {
            return Err(PinnedScratchPoolError::GpuLocalityUnavailable {
                gpu: gpu_bdf,
                reason: resolved.reason,
            });
        };

        let mut nodes = nodes.iter();
        let Some(node) = nodes.next() else {
            return Err(PinnedScratchPoolError::GpuLocalityUnavailable {
                gpu: gpu_bdf,
                reason: "GPU locality resolved to an empty node set",
            });
        };
        if nodes.next().is_some() {
            return Err(PinnedScratchPoolError::GpuLocalityUnavailable {
                gpu: gpu_bdf,
                reason: "GPU locality resolved to multiple NUMA nodes",
            });
        }

        Self::new(node, arena_capacity, max_locked_bytes)
    }

    /// Checks out a page-locked arena handle.
    ///
    /// # Errors
    ///
    /// Returns an error when creating or locking a new arena fails, or when the
    /// locked-byte budget is already exhausted.
    pub fn checkout(&mut self) -> Result<PinnedScratchHandle, PinnedScratchPoolError> {
        let arena = if let Some(arena) = self.idle.pop() {
            self.reused_arenas = self.reused_arenas.saturating_add(1);
            arena
        } else {
            self.ensure_can_lock_another_arena()?;
            let arena = MappedScratchArena::new(self.home_node, self.arena_capacity)
                .map_err(PinnedScratchPoolError::Arena)?;
            arena.lock_pages().map_err(PinnedScratchPoolError::Arena)?;
            self.locked_bytes = self.locked_bytes.saturating_add(arena.mapping_len());
            self.created_arenas = self.created_arenas.saturating_add(1);
            arena
        };

        let handle = self.next_handle();
        self.checked_out.insert(handle, arena);
        self.checkout_count = self.checkout_count.saturating_add(1);
        Ok(handle)
    }

    /// Returns a mutable arena for a live checkout handle.
    ///
    /// # Errors
    ///
    /// Returns an error when the handle is not live in this pool.
    pub fn get_mut(
        &mut self,
        handle: PinnedScratchHandle,
    ) -> Result<&mut MappedScratchArena, PinnedScratchPoolError> {
        self.checked_out
            .get_mut(&handle)
            .ok_or(PinnedScratchPoolError::InvalidHandle)
    }

    /// Releases a checked-out arena back into the idle pool.
    ///
    /// # Errors
    ///
    /// Returns an error when the handle is not live in this pool.
    pub fn release(&mut self, handle: PinnedScratchHandle) -> Result<(), PinnedScratchPoolError> {
        let mut arena = self
            .checked_out
            .remove(&handle)
            .ok_or(PinnedScratchPoolError::InvalidHandle)?;
        arena.prepare_for_reuse();
        self.idle.push(arena);
        self.release_count = self.release_count.saturating_add(1);
        Ok(())
    }

    /// Returns current pool accounting.
    #[must_use]
    pub fn stats(&self) -> PinnedScratchPoolStats {
        PinnedScratchPoolStats {
            home_node: self.home_node,
            arena_capacity: self.arena_capacity,
            arena_mapping_len: self.arena_mapping_len,
            max_locked_bytes: self.max_locked_bytes,
            locked_bytes: self.locked_bytes,
            checked_out: self.checked_out.len(),
            idle: self.idle.len(),
            created_arenas: self.created_arenas,
            reused_arenas: self.reused_arenas,
            checkout_count: self.checkout_count,
            release_count: self.release_count,
        }
    }

    fn ensure_can_lock_another_arena(&self) -> Result<(), PinnedScratchPoolError> {
        let Some(required_total) = self.locked_bytes.checked_add(self.arena_mapping_len) else {
            return Err(PinnedScratchPoolError::LockedByteBudgetExceeded {
                required: self.arena_mapping_len,
                locked_bytes: self.locked_bytes,
                max_locked_bytes: self.max_locked_bytes,
            });
        };

        if required_total > self.max_locked_bytes {
            return Err(PinnedScratchPoolError::LockedByteBudgetExceeded {
                required: self.arena_mapping_len,
                locked_bytes: self.locked_bytes,
                max_locked_bytes: self.max_locked_bytes,
            });
        }

        Ok(())
    }

    fn next_handle(&mut self) -> PinnedScratchHandle {
        let handle = PinnedScratchHandle(self.next_handle);
        self.next_handle = self.next_handle.saturating_add(1);
        handle
    }
}

/// Pinned scratch pool failures.
#[derive(Debug)]
pub enum PinnedScratchPoolError {
    /// Arena capacity must be non-zero.
    InvalidArenaCapacity,
    /// Locked-byte budget must fit at least one arena mapping.
    InsufficientLockedByteBudget {
        /// Required locked bytes for one arena.
        required: usize,
        /// Configured maximum locked bytes.
        max_locked_bytes: usize,
    },
    /// Creating or locking a mapped scratch arena failed.
    Arena(MappedScratchAllocError),
    /// No additional arena can be locked within the budget.
    LockedByteBudgetExceeded {
        /// Required locked bytes for one more arena.
        required: usize,
        /// Currently locked bytes.
        locked_bytes: usize,
        /// Configured maximum locked bytes.
        max_locked_bytes: usize,
    },
    /// GPU locality could not be resolved to one concrete NUMA node.
    GpuLocalityUnavailable {
        /// GPU PCI bus-device-function identifier.
        gpu: String,
        /// Reason locality could not be resolved.
        reason: &'static str,
    },
    /// The checkout handle is not live in this pool.
    InvalidHandle,
}

impl fmt::Display for PinnedScratchPoolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArenaCapacity => {
                f.write_str("pinned scratch pool arena capacity must be non-zero")
            }
            Self::InsufficientLockedByteBudget {
                required,
                max_locked_bytes,
            } => write!(
                f,
                "pinned scratch pool locked-byte budget is too small: required {required}, max {max_locked_bytes}"
            ),
            Self::Arena(source) => write!(f, "pinned scratch pool arena failed: {source}"),
            Self::LockedByteBudgetExceeded {
                required,
                locked_bytes,
                max_locked_bytes,
            } => write!(
                f,
                "pinned scratch pool locked-byte budget exceeded: required {required}, locked {locked_bytes}, max {max_locked_bytes}"
            ),
            Self::GpuLocalityUnavailable { gpu, reason } => write!(
                f,
                "pinned scratch pool GPU locality unavailable for {gpu}: {reason}"
            ),
            Self::InvalidHandle => f.write_str("pinned scratch pool handle is not live"),
        }
    }
}

impl std::error::Error for PinnedScratchPoolError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Arena(source) => Some(source),
            Self::InvalidArenaCapacity
            | Self::InsufficientLockedByteBudget { .. }
            | Self::LockedByteBudgetExceeded { .. }
            | Self::GpuLocalityUnavailable { .. }
            | Self::InvalidHandle => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::alloc::Layout;

    use crate::sys::PageLockError;
    use crate::{NodeId, PciDevice, Topology};

    use super::{PinnedScratchHandle, PinnedScratchPool, PinnedScratchPoolError};
    use crate::{MappedScratchAllocError, MAX_SUPPORTED_ALIGN};

    #[test]
    fn pinned_scratch_pool_checks_out_and_reuses_locked_arenas() {
        let arena_capacity = 4096;
        let arena_mapping_len = arena_capacity + MAX_SUPPORTED_ALIGN - 1;
        let mut pool =
            PinnedScratchPool::new(NodeId(0), arena_capacity, arena_mapping_len * 2).expect("pool");

        let first = match pool.checkout() {
            Ok(handle) => handle,
            Err(error) if pinned_scratch_lock_unavailable(&error) => return,
            Err(error) => panic!("unexpected pinned scratch checkout error: {error}"),
        };
        assert_eq!(first.id(), 0);

        let allocation = pool
            .get_mut(first)
            .expect("checked-out arena")
            .alloc_bytes(Layout::from_size_align(128, 64).expect("layout"))
            .expect("allocation");
        allocation[0] = 7;
        assert_eq!(allocation[0], 7);

        let stats = pool.stats();
        assert_eq!(stats.home_node, NodeId(0));
        assert_eq!(stats.arena_capacity, arena_capacity);
        assert_eq!(stats.arena_mapping_len, arena_mapping_len);
        assert_eq!(stats.max_locked_bytes, arena_mapping_len * 2);
        assert_eq!(stats.locked_bytes, arena_mapping_len);
        assert_eq!(stats.checked_out, 1);
        assert_eq!(stats.idle, 0);
        assert_eq!(stats.created_arenas, 1);
        assert_eq!(stats.reused_arenas, 0);
        assert_eq!(stats.checkout_count, 1);
        assert_eq!(stats.release_count, 0);

        pool.release(first).expect("release first handle");
        assert!(matches!(
            pool.get_mut(first),
            Err(PinnedScratchPoolError::InvalidHandle)
        ));

        let stats = pool.stats();
        assert_eq!(stats.locked_bytes, arena_mapping_len);
        assert_eq!(stats.checked_out, 0);
        assert_eq!(stats.idle, 1);
        assert_eq!(stats.release_count, 1);

        let second = pool.checkout().expect("reuse idle arena");
        assert_eq!(second.id(), 1);
        let reused_arena_stats = pool.get_mut(second).expect("reused arena").stats();
        assert_eq!(reused_arena_stats.used, 0);
        assert_eq!(reused_arena_stats.high_water_mark, 0);
        assert_eq!(reused_arena_stats.allocation_count, 0);
        assert_eq!(reused_arena_stats.reset_count, 0);

        let stats = pool.stats();
        assert_eq!(stats.locked_bytes, arena_mapping_len);
        assert_eq!(stats.checked_out, 1);
        assert_eq!(stats.idle, 0);
        assert_eq!(stats.created_arenas, 1);
        assert_eq!(stats.reused_arenas, 1);
        assert_eq!(stats.checkout_count, 2);

        pool.release(second).expect("release second handle");
    }

    #[test]
    fn pinned_scratch_pool_enforces_locked_byte_budget() {
        let arena_capacity = 4096;
        let arena_mapping_len = arena_capacity + MAX_SUPPORTED_ALIGN - 1;
        let mut pool =
            PinnedScratchPool::new(NodeId(0), arena_capacity, arena_mapping_len).expect("pool");

        let first = match pool.checkout() {
            Ok(handle) => handle,
            Err(error) if pinned_scratch_lock_unavailable(&error) => return,
            Err(error) => panic!("unexpected pinned scratch checkout error: {error}"),
        };

        let error = pool.checkout().expect_err("budget should be exhausted");
        assert!(matches!(
            error,
            PinnedScratchPoolError::LockedByteBudgetExceeded {
                required,
                locked_bytes,
                max_locked_bytes,
            } if required == arena_mapping_len
                && locked_bytes == arena_mapping_len
                && max_locked_bytes == arena_mapping_len
        ));

        pool.release(first).expect("release first handle");
    }

    #[test]
    fn pinned_scratch_pool_rejects_invalid_configuration() {
        assert!(matches!(
            PinnedScratchPool::new(NodeId(0), 0, 4096),
            Err(PinnedScratchPoolError::InvalidArenaCapacity)
        ));

        let arena_capacity = 4096;
        let arena_mapping_len = arena_capacity + MAX_SUPPORTED_ALIGN - 1;
        let error = PinnedScratchPool::new(NodeId(0), arena_capacity, arena_mapping_len - 1)
            .expect_err("budget should be too small");
        assert!(matches!(
            error,
            PinnedScratchPoolError::InsufficientLockedByteBudget {
                required,
                max_locked_bytes,
            } if required == arena_mapping_len && max_locked_bytes == arena_mapping_len - 1
        ));
    }

    #[test]
    fn pinned_scratch_pool_rejects_invalid_handles() {
        let arena_capacity = 4096;
        let arena_mapping_len = arena_capacity + MAX_SUPPORTED_ALIGN - 1;
        let mut pool =
            PinnedScratchPool::new(NodeId(0), arena_capacity, arena_mapping_len).expect("pool");
        let invalid = PinnedScratchHandle(u64::MAX);

        assert!(matches!(
            pool.get_mut(invalid),
            Err(PinnedScratchPoolError::InvalidHandle)
        ));
        assert!(matches!(
            pool.release(invalid),
            Err(PinnedScratchPoolError::InvalidHandle)
        ));
    }

    #[test]
    fn pinned_scratch_pool_resolves_home_node_from_gpu_topology() {
        let arena_capacity = 4096;
        let arena_mapping_len = arena_capacity + MAX_SUPPORTED_ALIGN - 1;
        let topology = Topology {
            pci_devices: vec![PciDevice {
                bdf: "0000:81:00.0".to_owned(),
                numa_node: Some(NodeId(2)),
                local_cpus: None,
            }],
            ..Topology::default()
        };

        let pool = PinnedScratchPool::new_near_gpu(
            "0000:81:00.0",
            &topology,
            arena_capacity,
            arena_mapping_len,
        )
        .expect("near GPU pool");

        let stats = pool.stats();
        assert_eq!(stats.home_node, NodeId(2));
        assert_eq!(stats.locked_bytes, 0);
    }

    #[test]
    fn pinned_scratch_pool_rejects_missing_gpu_topology() {
        let error =
            PinnedScratchPool::new_near_gpu("0000:81:00.0", &Topology::default(), 4096, 8191)
                .expect_err("missing GPU should fail");

        assert!(matches!(
            error,
            PinnedScratchPoolError::GpuLocalityUnavailable {
                gpu,
                reason,
            } if gpu == "0000:81:00.0"
                && reason == "GPU PCI device was not discovered, using local first-touch behavior"
        ));
    }

    #[test]
    fn pinned_scratch_pool_rejects_gpu_without_numa_node() {
        let topology = Topology {
            pci_devices: vec![PciDevice {
                bdf: "0000:81:00.0".to_owned(),
                numa_node: None,
                local_cpus: None,
            }],
            ..Topology::default()
        };

        let error = PinnedScratchPool::new_near_gpu("0000:81:00.0", &topology, 4096, 8191)
            .expect_err("unknown GPU node should fail");

        assert!(matches!(
            error,
            PinnedScratchPoolError::GpuLocalityUnavailable {
                gpu,
                reason,
            } if gpu == "0000:81:00.0"
                && reason
                    == "GPU PCI device has no reported NUMA node, using local first-touch behavior"
        ));
    }

    fn pinned_scratch_lock_unavailable(error: &PinnedScratchPoolError) -> bool {
        matches!(
            error,
            PinnedScratchPoolError::Arena(MappedScratchAllocError::PageLock(
                PageLockError::LockFailed(source)
            )) if matches!(
                source.kind(),
                std::io::ErrorKind::OutOfMemory
                    | std::io::ErrorKind::PermissionDenied
                    | std::io::ErrorKind::WouldBlock
            )
        )
    }
}
