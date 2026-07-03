use std::alloc::Layout;
use std::fmt;

use locus_core::NodeId;

use crate::MAX_SUPPORTED_ALIGN;

/// Safe bump arena tagged with the NUMA node it is intended to serve.
///
/// This first implementation is intentionally Vec-backed. It validates arena
/// lifetime behavior, alignment rules, accounting, and benchmark structure
/// before introducing Linux NUMA binding or raw mmap allocation.
#[derive(Debug)]
pub struct ScratchArena {
    home_node: NodeId,
    backing: Vec<u8>,
    usable_capacity: usize,
    offset: usize,
    high_water_mark: usize,
    allocation_count: u64,
    reset_count: u64,
}

impl ScratchArena {
    /// Creates a scratch arena with `usable_capacity` bytes available for callers.
    ///
    /// # Errors
    ///
    /// Returns an error when capacity plus alignment slack overflows `usize`.
    pub fn new(home_node: NodeId, usable_capacity: usize) -> Result<Self, ScratchAllocError> {
        let backing_len = usable_capacity
            .checked_add(MAX_SUPPORTED_ALIGN - 1)
            .ok_or(ScratchAllocError::CapacityOverflow)?;

        Ok(Self {
            home_node,
            backing: vec![0; backing_len],
            usable_capacity,
            offset: 0,
            high_water_mark: 0,
            allocation_count: 0,
            reset_count: 0,
        })
    }

    /// Returns the arena home node.
    #[must_use]
    pub fn home_node(&self) -> NodeId {
        self.home_node
    }

    /// Returns usable arena capacity in bytes.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.usable_capacity
    }

    /// Allocates a byte slice with the requested layout.
    ///
    /// # Errors
    ///
    /// Returns an error when alignment is unsupported, allocation math
    /// overflows, or the arena does not have enough remaining capacity.
    pub fn alloc_bytes(&mut self, layout: Layout) -> Result<&mut [u8], ScratchAllocError> {
        if layout.align() > MAX_SUPPORTED_ALIGN {
            return Err(ScratchAllocError::UnsupportedAlignment {
                requested: layout.align(),
                max: MAX_SUPPORTED_ALIGN,
            });
        }

        let base = self.backing.as_ptr() as usize;
        let raw_start = base
            .checked_add(self.offset)
            .ok_or(ScratchAllocError::CapacityOverflow)?;
        let aligned_start = align_up(raw_start, layout.align());
        let start = aligned_start
            .checked_sub(base)
            .ok_or(ScratchAllocError::CapacityOverflow)?;
        let end = start
            .checked_add(layout.size())
            .ok_or(ScratchAllocError::CapacityOverflow)?;

        if end > self.usable_capacity {
            return Err(ScratchAllocError::OutOfMemory {
                requested: layout.size(),
                remaining: self.usable_capacity.saturating_sub(self.offset),
            });
        }

        self.offset = end;
        self.high_water_mark = self.high_water_mark.max(self.offset);
        self.allocation_count = self.allocation_count.saturating_add(1);

        Ok(&mut self.backing[start..end])
    }

    /// Resets the arena and makes previous allocations unavailable to callers.
    pub fn reset(&mut self) {
        self.offset = 0;
        self.reset_count = self.reset_count.saturating_add(1);
    }

    /// Returns current arena accounting.
    #[must_use]
    pub fn stats(&self) -> ScratchArenaStats {
        ScratchArenaStats {
            home_node: self.home_node,
            capacity: self.usable_capacity,
            used: self.offset,
            high_water_mark: self.high_water_mark,
            allocation_count: self.allocation_count,
            reset_count: self.reset_count,
        }
    }

    pub(crate) fn prepare_for_reuse(&mut self) {
        self.offset = 0;
        self.high_water_mark = 0;
        self.allocation_count = 0;
        self.reset_count = 0;
    }
}

/// Scratch arena accounting snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScratchArenaStats {
    /// Arena home node.
    pub home_node: NodeId,
    /// Usable capacity in bytes.
    pub capacity: usize,
    /// Bytes consumed since the last reset.
    pub used: usize,
    /// Maximum consumed bytes observed before reset.
    pub high_water_mark: usize,
    /// Successful allocation count.
    pub allocation_count: u64,
    /// Reset count.
    pub reset_count: u64,
}

/// Scratch allocation failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScratchAllocError {
    /// Requested alignment is larger than the arena currently supports.
    UnsupportedAlignment {
        /// Requested alignment.
        requested: usize,
        /// Maximum supported alignment.
        max: usize,
    },
    /// Arena capacity or allocation math overflowed.
    CapacityOverflow,
    /// Arena did not have enough remaining space.
    OutOfMemory {
        /// Requested allocation size.
        requested: usize,
        /// Remaining unaligned bytes.
        remaining: usize,
    },
}

impl fmt::Display for ScratchAllocError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedAlignment { requested, max } => {
                write!(
                    f,
                    "unsupported scratch alignment {requested}, maximum is {max}"
                )
            }
            Self::CapacityOverflow => f.write_str("scratch arena capacity overflow"),
            Self::OutOfMemory {
                requested,
                remaining,
            } => write!(
                f,
                "scratch arena out of memory: requested {requested}, remaining {remaining}"
            ),
        }
    }
}

impl std::error::Error for ScratchAllocError {}

fn align_up(value: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two());
    (value + align - 1) & !(align - 1)
}

#[cfg(test)]
mod tests {
    use std::alloc::Layout;

    use locus_core::NodeId;

    use super::{ScratchAllocError, ScratchArena};

    #[test]
    fn allocates_aligned_slices() {
        let mut arena = ScratchArena::new(NodeId(0), 256).expect("arena");
        let layout = Layout::from_size_align(32, 64).expect("layout");

        let allocation = arena.alloc_bytes(layout).expect("allocation");

        assert_eq!(allocation.len(), 32);
        assert_eq!(allocation.as_ptr() as usize % 64, 0);
    }

    #[test]
    fn reset_reuses_capacity_and_tracks_stats() {
        let mut arena = ScratchArena::new(NodeId(1), 128).expect("arena");

        let first = arena
            .alloc_bytes(Layout::from_size_align(96, 8).expect("layout"))
            .expect("first allocation");
        assert_eq!(first.len(), 96);
        arena.reset();
        let second = arena
            .alloc_bytes(Layout::from_size_align(32, 8).expect("layout"))
            .expect("second allocation");
        assert_eq!(second.len(), 32);

        let stats = arena.stats();
        assert_eq!(stats.home_node, NodeId(1));
        assert_eq!(stats.used, 32);
        assert!(stats.high_water_mark >= 96);
        assert_eq!(stats.allocation_count, 2);
        assert_eq!(stats.reset_count, 1);
    }

    #[test]
    fn reports_out_of_memory() {
        let mut arena = ScratchArena::new(NodeId(0), 64).expect("arena");

        let error = arena
            .alloc_bytes(Layout::from_size_align(128, 8).expect("layout"))
            .expect_err("allocation should fail");

        assert!(matches!(error, ScratchAllocError::OutOfMemory { .. }));
    }

    #[test]
    fn rejects_alignment_above_supported_page_size() {
        let mut arena = ScratchArena::new(NodeId(0), 8192).expect("arena");

        let error = arena
            .alloc_bytes(Layout::from_size_align(16, 8192).expect("layout"))
            .expect_err("alignment should fail");

        assert_eq!(
            error,
            ScratchAllocError::UnsupportedAlignment {
                requested: 8192,
                max: 4096,
            }
        );
    }
}
