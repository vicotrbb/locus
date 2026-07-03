use std::alloc::Layout;
use std::fmt;

use locus_core::NodeId;
use locus_sys::{
    page_size, MappedRegion, MappedRegionError, PageLockError, PageSizeError, TouchPagesError,
};

use crate::{ScratchArenaStats, MAX_SUPPORTED_ALIGN};

/// Mmap-backed scratch arena tagged with the NUMA node it is intended to serve.
#[derive(Debug)]
pub struct MappedScratchArena {
    home_node: NodeId,
    region: MappedRegion,
    usable_capacity: usize,
    offset: usize,
    high_water_mark: usize,
    allocation_count: u64,
    reset_count: u64,
}

/// Transparent huge page advice for mapped scratch arenas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappedScratchHugePageAdvice {
    /// Ask the kernel to consider the arena mapping for transparent huge pages.
    HugePage,
    /// Ask the kernel to avoid transparent huge pages for the arena mapping.
    NoHugePage,
}

impl MappedScratchHugePageAdvice {
    /// Returns a stable machine-readable advice string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::HugePage => "hugepage",
            Self::NoHugePage => "no_hugepage",
        }
    }

    /// Parses a stable machine-readable advice string.
    #[must_use]
    pub fn from_str_token(value: &str) -> Option<Self> {
        match value {
            "hugepage" => Some(Self::HugePage),
            "no_hugepage" => Some(Self::NoHugePage),
            _ => None,
        }
    }
}

impl fmt::Display for MappedScratchHugePageAdvice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl MappedScratchArena {
    /// Creates an mmap-backed scratch arena with `usable_capacity` bytes.
    ///
    /// # Errors
    ///
    /// Returns an error when capacity plus alignment slack overflows `usize` or
    /// anonymous mapping creation fails.
    pub fn new(home_node: NodeId, usable_capacity: usize) -> Result<Self, MappedScratchAllocError> {
        let mapping_len = usable_capacity
            .checked_add(MAX_SUPPORTED_ALIGN - 1)
            .ok_or(MappedScratchAllocError::CapacityOverflow)?;
        let region = MappedRegion::anonymous(mapping_len).map_err(MappedScratchAllocError::Map)?;

        Ok(Self {
            home_node,
            region,
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

    /// Returns the start address of the underlying mapping.
    #[must_use]
    pub fn mapping_start_address(&self) -> usize {
        self.region.as_ptr() as usize
    }

    /// Returns the underlying mapping length in bytes.
    #[must_use]
    pub fn mapping_len(&self) -> usize {
        self.region.len()
    }

    /// Allocates a byte slice with the requested layout.
    ///
    /// # Errors
    ///
    /// Returns an error when alignment is unsupported, allocation math
    /// overflows, or the arena does not have enough remaining capacity.
    pub fn alloc_bytes(&mut self, layout: Layout) -> Result<&mut [u8], MappedScratchAllocError> {
        if layout.align() > MAX_SUPPORTED_ALIGN {
            return Err(MappedScratchAllocError::UnsupportedAlignment {
                requested: layout.align(),
                max: MAX_SUPPORTED_ALIGN,
            });
        }

        let base = self.region.as_slice().as_ptr() as usize;
        let raw_start = base
            .checked_add(self.offset)
            .ok_or(MappedScratchAllocError::CapacityOverflow)?;
        let aligned_start = align_up(raw_start, layout.align());
        let start = aligned_start
            .checked_sub(base)
            .ok_or(MappedScratchAllocError::CapacityOverflow)?;
        let end = start
            .checked_add(layout.size())
            .ok_or(MappedScratchAllocError::CapacityOverflow)?;

        if end > self.usable_capacity {
            return Err(MappedScratchAllocError::OutOfMemory {
                requested: layout.size(),
                remaining: self.usable_capacity.saturating_sub(self.offset),
            });
        }

        self.offset = end;
        self.high_water_mark = self.high_water_mark.max(self.offset);
        self.allocation_count = self.allocation_count.saturating_add(1);

        Ok(&mut self.region.as_mut_slice()[start..end])
    }

    /// Resets the arena and makes previous allocations unavailable to callers.
    pub fn reset(&mut self) {
        self.offset = 0;
        self.reset_count = self.reset_count.saturating_add(1);
    }

    /// Write-touches the underlying mapped pages to materialize them.
    ///
    /// # Errors
    ///
    /// Returns an error when page-size discovery fails or page touching fails.
    pub fn write_touch_pages(&mut self) -> Result<usize, MappedScratchAllocError> {
        let size = page_size().map_err(MappedScratchAllocError::PageSize)?;
        self.region
            .write_touch_pages(size)
            .map_err(MappedScratchAllocError::TouchPages)
    }

    /// Locks the arena mapping into RAM.
    ///
    /// # Errors
    ///
    /// Returns an error when the operating system rejects page locking.
    pub fn lock_pages(&self) -> Result<(), MappedScratchAllocError> {
        self.region
            .lock_pages()
            .map_err(MappedScratchAllocError::PageLock)
    }

    /// Unlocks the arena mapping after a successful `lock_pages`.
    ///
    /// # Errors
    ///
    /// Returns an error when the operating system rejects page unlocking.
    pub fn unlock_pages(&self) -> Result<(), MappedScratchAllocError> {
        self.region
            .unlock_pages()
            .map_err(MappedScratchAllocError::PageLock)
    }

    /// Applies Linux `MPOL_BIND` to the mapped arena region.
    ///
    /// # Errors
    ///
    /// Returns an error when the node mask is invalid or the Linux `mbind`
    /// syscall fails.
    #[cfg(target_os = "linux")]
    pub fn bind_to_node(&self, node: NodeId) -> Result<(), MappedScratchAllocError> {
        locus_sys::linux::bind_region_to_node(&self.region, node.0)
            .map_err(MappedScratchAllocError::LinuxNumaPolicy)
    }

    /// Applies Linux transparent huge page advice to the mapped arena region.
    ///
    /// # Errors
    ///
    /// Returns an error when the Linux `madvise` call fails.
    #[cfg(target_os = "linux")]
    pub fn advise_transparent_huge_pages(
        &self,
        advice: MappedScratchHugePageAdvice,
    ) -> Result<(), MappedScratchAllocError> {
        let sys_advice = match advice {
            MappedScratchHugePageAdvice::HugePage => {
                locus_sys::linux::LinuxTransparentHugePageAdvice::HugePage
            }
            MappedScratchHugePageAdvice::NoHugePage => {
                locus_sys::linux::LinuxTransparentHugePageAdvice::NoHugePage
            }
        };

        locus_sys::linux::advise_region_transparent_huge_pages(&self.region, sys_advice)
            .map_err(MappedScratchAllocError::LinuxTransparentHugePageAdvice)
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

/// Mmap-backed scratch allocation failures.
#[derive(Debug)]
pub enum MappedScratchAllocError {
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
    /// System mapping failed.
    Map(MappedRegionError),
    /// Page-size discovery failed.
    PageSize(PageSizeError),
    /// Page touching failed.
    TouchPages(TouchPagesError),
    /// Page locking or unlocking failed.
    PageLock(PageLockError),
    /// Linux NUMA policy application failed.
    #[cfg(target_os = "linux")]
    LinuxNumaPolicy(locus_sys::linux::LinuxNumaPolicyError),
    /// Linux transparent huge page advice failed.
    #[cfg(target_os = "linux")]
    LinuxTransparentHugePageAdvice(locus_sys::linux::LinuxTransparentHugePageAdviceError),
}

impl fmt::Display for MappedScratchAllocError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedAlignment { requested, max } => {
                write!(
                    f,
                    "unsupported mapped scratch alignment {requested}, maximum is {max}"
                )
            }
            Self::CapacityOverflow => f.write_str("mapped scratch arena capacity overflow"),
            Self::OutOfMemory {
                requested,
                remaining,
            } => write!(
                f,
                "mapped scratch arena out of memory: requested {requested}, remaining {remaining}"
            ),
            Self::Map(source) => write!(f, "mapped scratch arena mapping failed: {source}"),
            Self::PageSize(source) => {
                write!(f, "mapped scratch arena page-size lookup failed: {source}")
            }
            Self::TouchPages(source) => {
                write!(f, "mapped scratch arena page touching failed: {source}")
            }
            Self::PageLock(source) => {
                write!(f, "mapped scratch arena page locking failed: {source}")
            }
            #[cfg(target_os = "linux")]
            Self::LinuxNumaPolicy(source) => {
                write!(f, "mapped scratch arena NUMA policy failed: {source}")
            }
            #[cfg(target_os = "linux")]
            Self::LinuxTransparentHugePageAdvice(source) => {
                write!(
                    f,
                    "mapped scratch arena transparent huge page advice failed: {source}"
                )
            }
        }
    }
}

impl std::error::Error for MappedScratchAllocError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Map(source) => Some(source),
            Self::PageSize(source) => Some(source),
            Self::TouchPages(source) => Some(source),
            Self::PageLock(source) => Some(source),
            #[cfg(target_os = "linux")]
            Self::LinuxNumaPolicy(source) => Some(source),
            #[cfg(target_os = "linux")]
            Self::LinuxTransparentHugePageAdvice(source) => Some(source),
            Self::UnsupportedAlignment { .. }
            | Self::CapacityOverflow
            | Self::OutOfMemory { .. } => None,
        }
    }
}

fn align_up(value: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two());
    (value + align - 1) & !(align - 1)
}

#[cfg(test)]
mod tests {
    use std::alloc::Layout;

    use locus_core::NodeId;
    use locus_sys::PageLockError;

    use super::{MappedScratchAllocError, MappedScratchArena};

    #[test]
    fn mapped_scratch_arena_allocates_aligned_slices() {
        let mut arena = MappedScratchArena::new(NodeId(0), 256).expect("arena");
        let allocation = arena
            .alloc_bytes(Layout::from_size_align(32, 64).expect("layout"))
            .expect("allocation");

        assert_eq!(allocation.len(), 32);
        assert_eq!(allocation.as_ptr() as usize % 64, 0);
        allocation[0] = 3;
        assert_eq!(allocation[0], 3);
    }

    #[test]
    fn mapped_scratch_arena_resets_and_tracks_stats() {
        let mut arena = MappedScratchArena::new(NodeId(1), 128).expect("arena");
        arena
            .alloc_bytes(Layout::from_size_align(96, 8).expect("layout"))
            .expect("allocation");
        arena.reset();
        arena
            .alloc_bytes(Layout::from_size_align(16, 8).expect("layout"))
            .expect("allocation");

        let stats = arena.stats();
        assert_eq!(stats.home_node, NodeId(1));
        assert_eq!(stats.used, 16);
        assert_eq!(stats.allocation_count, 2);
        assert_eq!(stats.reset_count, 1);
    }

    #[test]
    fn mapped_scratch_arena_reports_out_of_memory() {
        let mut arena = MappedScratchArena::new(NodeId(0), 64).expect("arena");

        let error = arena
            .alloc_bytes(Layout::from_size_align(128, 8).expect("layout"))
            .expect_err("allocation should fail");

        assert!(matches!(error, MappedScratchAllocError::OutOfMemory { .. }));
    }

    #[test]
    fn mapped_scratch_arena_write_touches_pages() {
        let mut arena = MappedScratchArena::new(NodeId(0), 8192).expect("arena");

        let touched = arena.write_touch_pages().expect("touch pages");

        assert!(touched >= 1);
    }

    #[test]
    fn mapped_scratch_arena_locks_and_unlocks_pages() {
        let arena = MappedScratchArena::new(NodeId(0), 4096).expect("arena");

        match arena.lock_pages() {
            Ok(()) => arena.unlock_pages().expect("unlock pages"),
            Err(MappedScratchAllocError::PageLock(PageLockError::LockFailed(source)))
                if matches!(
                    source.kind(),
                    std::io::ErrorKind::OutOfMemory
                        | std::io::ErrorKind::PermissionDenied
                        | std::io::ErrorKind::WouldBlock
                ) => {}
            Err(error) => panic!("unexpected mapped scratch page lock error: {error}"),
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn mapped_scratch_arena_applies_transparent_huge_page_advice() {
        let arena = MappedScratchArena::new(NodeId(0), 4096).expect("arena");

        arena
            .advise_transparent_huge_pages(super::MappedScratchHugePageAdvice::HugePage)
            .expect("huge page advice");
        arena
            .advise_transparent_huge_pages(super::MappedScratchHugePageAdvice::NoHugePage)
            .expect("no huge page advice");
    }

    #[test]
    fn mapped_scratch_arena_reports_mapping_identity() {
        let arena = MappedScratchArena::new(NodeId(0), 8192).expect("arena");

        assert_ne!(arena.mapping_start_address(), 0);
        assert!(arena.mapping_len() >= arena.capacity());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn mapped_scratch_arena_rejects_invalid_bind_node() {
        let arena = MappedScratchArena::new(NodeId(0), 4096).expect("arena");
        let error = arena
            .bind_to_node(NodeId(4096))
            .expect_err("invalid node should fail before syscall");

        assert!(matches!(
            error,
            MappedScratchAllocError::LinuxNumaPolicy(
                locus_sys::linux::LinuxNumaPolicyError::InvalidNode(4096)
            )
        ));
    }
}
