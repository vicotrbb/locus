//! Experimental domain allocators for Locus.

use std::alloc::Layout;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TryRecvError};
use std::sync::Arc;

use locus_core::{NodeId, RequestHome, RequestId};
use locus_sys::{
    page_size, MappedRegion, MappedRegionError, PageLockError, PageSizeError, TouchPagesError,
};

const MAX_SUPPORTED_ALIGN: usize = 4096;

/// Safe fixed-size KV block pool tagged with an intended NUMA node.
#[derive(Debug)]
pub struct KvBlockPool {
    home_node: NodeId,
    block_size: usize,
    blocks: Vec<Vec<u8>>,
    free: Vec<usize>,
    allocated: Vec<bool>,
    generations: Vec<u64>,
    allocation_count: u64,
    free_count: u64,
    high_water_mark: usize,
}

/// Opaque handle for a KV block owned by a `KvBlockPool`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KvBlockHandle {
    index: usize,
    generation: u64,
}

/// Logical sequence identifier for KV block tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KvSequenceId(pub u64);

/// Logical KV block table for one sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KvBlockTable {
    sequence_id: KvSequenceId,
    tokens_per_block: u16,
    token_len: u64,
    blocks: Vec<KvBlockHandle>,
}

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

/// Request-scoped scratch arena manager.
#[derive(Debug, Default)]
pub struct RequestScratch {
    arenas: BTreeMap<RequestId, ScratchArena>,
}

/// Request-scoped scratch manager with reusable per-node idle arenas.
#[derive(Debug, Default)]
pub struct RequestScratchPool {
    active: BTreeMap<RequestId, ScratchArena>,
    idle: BTreeMap<(NodeId, usize), Vec<ScratchArena>>,
    created_arenas: u64,
    reused_arenas: u64,
}

/// Owner-drained queue for batching remote frees back to an owning worker.
pub struct RemoteFreeQueue<T> {
    receiver: Receiver<T>,
    sink: RemoteFreeSink<T>,
    capacity: usize,
    batch_limit: usize,
    drained_count: u64,
}

/// Cloneable enqueue handle for remote free work.
pub struct RemoteFreeSink<T> {
    sender: SyncSender<T>,
    submitted_count: Arc<AtomicU64>,
}

/// Reusable request scratch pool accounting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RequestScratchPoolStats {
    /// Number of active request arenas.
    pub active_requests: usize,
    /// Number of idle arenas available for reuse.
    pub idle_arenas: usize,
    /// Number of arenas created by the pool.
    pub created_arenas: u64,
    /// Number of open operations served from an idle arena.
    pub reused_arenas: u64,
}

/// Remote free queue accounting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteFreeQueueStats {
    /// Bounded channel capacity.
    pub capacity: usize,
    /// Maximum items drained per batch.
    pub batch_limit: usize,
    /// Successfully enqueued item count.
    pub submitted_count: u64,
    /// Items drained by the owner.
    pub drained_count: u64,
}

/// Result of one remote free drain operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteFreeDrainStats {
    /// Items drained by this operation.
    pub drained: usize,
    /// Total items drained by the queue after this operation.
    pub total_drained: u64,
}

/// Stable status for mapped scratch page-lock probe lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageLockProbeStatus {
    /// The operation succeeded.
    Ok,
    /// The operation failed.
    Error,
}

impl PageLockProbeStatus {
    /// Returns a stable machine-readable status string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Error => "error",
        }
    }

    /// Parses a stable machine-readable status string.
    #[must_use]
    pub fn from_str_token(value: &str) -> Option<Self> {
        match value {
            "ok" => Some(Self::Ok),
            "error" => Some(Self::Error),
            _ => None,
        }
    }
}

impl fmt::Display for PageLockProbeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Stable page-lock probe field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageLockProbeField {
    /// `page_lock=<status>`.
    Lock,
    /// `page_unlock=<status>`.
    Unlock,
}

impl PageLockProbeField {
    /// Returns the stable machine-readable field name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lock => "page_lock",
            Self::Unlock => "page_unlock",
        }
    }

    /// Parses a stable machine-readable field name.
    #[must_use]
    pub fn from_str_token(value: &str) -> Option<Self> {
        match value {
            "page_lock" => Some(Self::Lock),
            "page_unlock" => Some(Self::Unlock),
            _ => None,
        }
    }
}

impl fmt::Display for PageLockProbeField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Parsed mapped scratch page-lock status line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageLockProbeStatusLine {
    /// Parsed status field.
    pub field: PageLockProbeField,
    /// Parsed status value.
    pub status: PageLockProbeStatus,
}

/// Parsed mapped scratch page-lock probe output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MappedScratchLockProbeOutput {
    /// Page-lock status.
    pub page_lock: PageLockProbeStatus,
    /// Page-unlock status, present when lock succeeded and unlock was attempted.
    pub page_unlock: Option<PageLockProbeStatus>,
}

impl KvBlockPool {
    /// Creates a fixed-size KV block pool.
    ///
    /// # Errors
    ///
    /// Returns an error when block size or capacity is zero.
    pub fn new(
        home_node: NodeId,
        block_size: usize,
        capacity: usize,
    ) -> Result<Self, KvBlockPoolError> {
        if block_size == 0 {
            return Err(KvBlockPoolError::InvalidBlockSize);
        }
        if capacity == 0 {
            return Err(KvBlockPoolError::InvalidCapacity);
        }

        let mut blocks = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            blocks.push(vec![0; block_size]);
        }

        let free = (0..capacity).rev().collect();

        Ok(Self {
            home_node,
            block_size,
            blocks,
            free,
            allocated: vec![false; capacity],
            generations: vec![0; capacity],
            allocation_count: 0,
            free_count: 0,
            high_water_mark: 0,
        })
    }

    /// Allocates one KV block.
    ///
    /// # Errors
    ///
    /// Returns an error when the pool has no free blocks.
    pub fn allocate(&mut self) -> Result<KvBlockHandle, KvBlockPoolError> {
        let index = self.free.pop().ok_or(KvBlockPoolError::OutOfBlocks)?;
        self.allocated[index] = true;
        self.allocation_count = self.allocation_count.saturating_add(1);
        self.high_water_mark = self.high_water_mark.max(self.allocated_count());
        Ok(KvBlockHandle {
            index,
            generation: self.generations[index],
        })
    }

    /// Returns a mutable block slice for a live handle.
    ///
    /// # Errors
    ///
    /// Returns an error when the handle is stale or not allocated.
    pub fn block_mut(&mut self, handle: KvBlockHandle) -> Result<&mut [u8], KvBlockPoolError> {
        self.validate_handle(handle)?;
        Ok(&mut self.blocks[handle.index])
    }

    /// Frees a live KV block handle.
    ///
    /// # Errors
    ///
    /// Returns an error when the handle is stale or not allocated.
    pub fn free(&mut self, handle: KvBlockHandle) -> Result<(), KvBlockPoolError> {
        self.validate_handle(handle)?;
        self.allocated[handle.index] = false;
        self.generations[handle.index] = self.generations[handle.index].saturating_add(1);
        self.free.push(handle.index);
        self.free_count = self.free_count.saturating_add(1);
        Ok(())
    }

    /// Returns pool accounting.
    #[must_use]
    pub fn stats(&self) -> KvBlockPoolStats {
        KvBlockPoolStats {
            home_node: self.home_node,
            block_size: self.block_size,
            capacity: self.blocks.len(),
            allocated: self.allocated_count(),
            free: self.free.len(),
            high_water_mark: self.high_water_mark,
            allocation_count: self.allocation_count,
            free_count: self.free_count,
        }
    }

    fn validate_handle(&self, handle: KvBlockHandle) -> Result<(), KvBlockPoolError> {
        let Some(is_allocated) = self.allocated.get(handle.index) else {
            return Err(KvBlockPoolError::InvalidHandle);
        };
        if !is_allocated || self.generations[handle.index] != handle.generation {
            return Err(KvBlockPoolError::InvalidHandle);
        }
        Ok(())
    }

    fn allocated_count(&self) -> usize {
        self.blocks.len() - self.free.len()
    }
}

impl<T> RemoteFreeQueue<T> {
    /// Creates an owner-drained remote free queue.
    ///
    /// # Errors
    ///
    /// Returns an error when `capacity` or `batch_limit` is zero.
    pub fn new(capacity: usize, batch_limit: usize) -> Result<Self, RemoteFreeQueueError> {
        if capacity == 0 {
            return Err(RemoteFreeQueueError::InvalidCapacity);
        }
        if batch_limit == 0 {
            return Err(RemoteFreeQueueError::InvalidBatchLimit);
        }

        let (sender, receiver) = sync_channel(capacity);
        let submitted_count = Arc::new(AtomicU64::new(0));
        let sink = RemoteFreeSink {
            sender,
            submitted_count,
        };

        Ok(Self {
            receiver,
            sink,
            capacity,
            batch_limit,
            drained_count: 0,
        })
    }

    /// Returns a cloneable sink for remote producers.
    #[must_use]
    pub fn sink(&self) -> RemoteFreeSink<T> {
        self.sink.clone()
    }

    /// Drains up to the configured batch limit and passes each item to `release`.
    #[must_use]
    pub fn drain_batch(&mut self, mut release: impl FnMut(T)) -> RemoteFreeDrainStats {
        let mut drained = 0_usize;

        while drained < self.batch_limit {
            match self.receiver.try_recv() {
                Ok(item) => {
                    release(item);
                    drained += 1;
                }
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            }
        }

        self.drained_count = self.drained_count.saturating_add(drained as u64);

        RemoteFreeDrainStats {
            drained,
            total_drained: self.drained_count,
        }
    }

    /// Returns current queue accounting.
    #[must_use]
    pub fn stats(&self) -> RemoteFreeQueueStats {
        RemoteFreeQueueStats {
            capacity: self.capacity,
            batch_limit: self.batch_limit,
            submitted_count: self.sink.submitted_count(),
            drained_count: self.drained_count,
        }
    }
}

impl<T> RemoteFreeSink<T> {
    /// Enqueues one item for owner-side release.
    ///
    /// # Errors
    ///
    /// Returns the item when the owning queue has been dropped.
    pub fn enqueue(&self, item: T) -> Result<(), RemoteFreeEnqueueError<T>> {
        self.sender
            .send(item)
            .map_err(|source| RemoteFreeEnqueueError { item: source.0 })?;
        self.submitted_count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Returns the number of successfully submitted items.
    #[must_use]
    pub fn submitted_count(&self) -> u64 {
        self.submitted_count.load(Ordering::Relaxed)
    }
}

impl<T> Clone for RemoteFreeSink<T> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            submitted_count: Arc::clone(&self.submitted_count),
        }
    }
}

impl<T> fmt::Debug for RemoteFreeQueue<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RemoteFreeQueue")
            .field("capacity", &self.capacity)
            .field("batch_limit", &self.batch_limit)
            .field("drained_count", &self.drained_count)
            .finish_non_exhaustive()
    }
}

impl<T> fmt::Debug for RemoteFreeSink<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RemoteFreeSink")
            .field("submitted_count", &self.submitted_count())
            .finish_non_exhaustive()
    }
}

impl KvBlockTable {
    /// Creates an empty KV block table.
    ///
    /// # Errors
    ///
    /// Returns an error when `tokens_per_block` is zero.
    pub fn new(
        sequence_id: KvSequenceId,
        tokens_per_block: u16,
    ) -> Result<Self, KvBlockTableError> {
        if tokens_per_block == 0 {
            return Err(KvBlockTableError::InvalidTokensPerBlock);
        }

        Ok(Self {
            sequence_id,
            tokens_per_block,
            token_len: 0,
            blocks: Vec::new(),
        })
    }

    /// Appends tokens and allocates additional blocks as needed.
    ///
    /// # Errors
    ///
    /// Returns an error when the backing pool has insufficient free blocks.
    pub fn append_tokens(
        &mut self,
        pool: &mut KvBlockPool,
        token_count: u64,
    ) -> Result<(), KvBlockTableError> {
        if token_count == 0 {
            return Ok(());
        }

        let new_token_len = self
            .token_len
            .checked_add(token_count)
            .ok_or(KvBlockTableError::TokenCountOverflow)?;
        let needed_blocks = blocks_for_tokens(new_token_len, self.tokens_per_block);

        let additional_blocks = needed_blocks.saturating_sub(self.blocks.len());
        let mut acquired = Vec::with_capacity(additional_blocks);
        for _ in 0..additional_blocks {
            match pool.allocate() {
                Ok(handle) => acquired.push(handle),
                Err(source) => {
                    for handle in acquired {
                        let _ = pool.free(handle);
                    }
                    return Err(KvBlockTableError::Pool(source));
                }
            }
        }

        self.blocks.extend(acquired);
        self.token_len = new_token_len;
        Ok(())
    }

    /// Frees all blocks owned by this table and resets it to empty.
    ///
    /// # Errors
    ///
    /// Returns an error if any stored handle is rejected by the backing pool.
    pub fn release_all(&mut self, pool: &mut KvBlockPool) -> Result<(), KvBlockTableError> {
        for handle in self.blocks.drain(..) {
            pool.free(handle).map_err(KvBlockTableError::Pool)?;
        }
        self.token_len = 0;
        Ok(())
    }

    /// Returns table accounting.
    #[must_use]
    pub fn stats(&self) -> KvBlockTableStats {
        KvBlockTableStats {
            sequence_id: self.sequence_id,
            tokens_per_block: self.tokens_per_block,
            token_len: self.token_len,
            block_count: self.blocks.len(),
        }
    }
}

impl RequestScratch {
    /// Creates an empty request scratch manager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Opens a request-local arena.
    ///
    /// # Errors
    ///
    /// Returns an error when the request has no selected home node, already has
    /// an arena, or arena capacity creation fails.
    pub fn open_request(
        &mut self,
        home: &RequestHome,
        usable_capacity: usize,
    ) -> Result<(), RequestScratchError> {
        let node = home.node.ok_or(RequestScratchError::MissingHomeNode {
            request_id: home.request_id,
        })?;

        match self.arenas.entry(home.request_id) {
            Entry::Occupied(_) => Err(RequestScratchError::AlreadyOpen {
                request_id: home.request_id,
            }),
            Entry::Vacant(entry) => {
                let arena = ScratchArena::new(node, usable_capacity).map_err(|source| {
                    RequestScratchError::Arena {
                        request_id: home.request_id,
                        source,
                    }
                })?;
                entry.insert(arena);
                Ok(())
            }
        }
    }

    /// Allocates from a request-local scratch arena.
    ///
    /// # Errors
    ///
    /// Returns an error when the request is not open or arena allocation fails.
    pub fn alloc_bytes(
        &mut self,
        request_id: RequestId,
        layout: Layout,
    ) -> Result<&mut [u8], RequestScratchError> {
        let arena = self
            .arenas
            .get_mut(&request_id)
            .ok_or(RequestScratchError::NotOpen { request_id })?;
        arena
            .alloc_bytes(layout)
            .map_err(|source| RequestScratchError::Arena { request_id, source })
    }

    /// Resets one request-local scratch arena.
    ///
    /// # Errors
    ///
    /// Returns an error when the request is not open.
    pub fn reset_request(&mut self, request_id: RequestId) -> Result<(), RequestScratchError> {
        let arena = self
            .arenas
            .get_mut(&request_id)
            .ok_or(RequestScratchError::NotOpen { request_id })?;
        arena.reset();
        Ok(())
    }

    /// Closes a request-local scratch arena and returns its final stats.
    ///
    /// # Errors
    ///
    /// Returns an error when the request is not open.
    pub fn close_request(
        &mut self,
        request_id: RequestId,
    ) -> Result<ScratchArenaStats, RequestScratchError> {
        let arena = self
            .arenas
            .remove(&request_id)
            .ok_or(RequestScratchError::NotOpen { request_id })?;
        Ok(arena.stats())
    }

    /// Returns stats for an open request arena.
    #[must_use]
    pub fn stats(&self, request_id: RequestId) -> Option<ScratchArenaStats> {
        self.arenas.get(&request_id).map(ScratchArena::stats)
    }

    /// Returns the number of open request arenas.
    #[must_use]
    pub fn open_request_count(&self) -> usize {
        self.arenas.len()
    }
}

impl RequestScratchPool {
    /// Creates an empty reusable request scratch pool.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Opens a request-local arena, reusing an idle arena when possible.
    ///
    /// # Errors
    ///
    /// Returns an error when the request has no selected home node, already has
    /// an arena, or arena capacity creation fails.
    pub fn open_request(
        &mut self,
        home: &RequestHome,
        usable_capacity: usize,
    ) -> Result<(), RequestScratchError> {
        let node = home.node.ok_or(RequestScratchError::MissingHomeNode {
            request_id: home.request_id,
        })?;

        match self.active.entry(home.request_id) {
            Entry::Occupied(_) => Err(RequestScratchError::AlreadyOpen {
                request_id: home.request_id,
            }),
            Entry::Vacant(entry) => {
                let key = (node, usable_capacity);
                let arena = self
                    .idle
                    .get_mut(&key)
                    .and_then(Vec::pop)
                    .map(|mut arena| {
                        arena.prepare_for_reuse();
                        self.reused_arenas = self.reused_arenas.saturating_add(1);
                        arena
                    })
                    .map_or_else(
                        || {
                            self.created_arenas = self.created_arenas.saturating_add(1);
                            ScratchArena::new(node, usable_capacity)
                        },
                        Ok,
                    )
                    .map_err(|source| RequestScratchError::Arena {
                        request_id: home.request_id,
                        source,
                    })?;
                entry.insert(arena);
                Ok(())
            }
        }
    }

    /// Allocates from a request-local scratch arena.
    ///
    /// # Errors
    ///
    /// Returns an error when the request is not open or arena allocation fails.
    pub fn alloc_bytes(
        &mut self,
        request_id: RequestId,
        layout: Layout,
    ) -> Result<&mut [u8], RequestScratchError> {
        let arena = self
            .active
            .get_mut(&request_id)
            .ok_or(RequestScratchError::NotOpen { request_id })?;
        arena
            .alloc_bytes(layout)
            .map_err(|source| RequestScratchError::Arena { request_id, source })
    }

    /// Resets one request-local scratch arena.
    ///
    /// # Errors
    ///
    /// Returns an error when the request is not open.
    pub fn reset_request(&mut self, request_id: RequestId) -> Result<(), RequestScratchError> {
        let arena = self
            .active
            .get_mut(&request_id)
            .ok_or(RequestScratchError::NotOpen { request_id })?;
        arena.reset();
        Ok(())
    }

    /// Closes a request-local arena, returns its stats, and keeps it for reuse.
    ///
    /// # Errors
    ///
    /// Returns an error when the request is not open.
    pub fn close_request(
        &mut self,
        request_id: RequestId,
    ) -> Result<ScratchArenaStats, RequestScratchError> {
        let mut arena = self
            .active
            .remove(&request_id)
            .ok_or(RequestScratchError::NotOpen { request_id })?;
        let stats = arena.stats();
        arena.reset();
        self.idle
            .entry((arena.home_node(), arena.capacity()))
            .or_default()
            .push(arena);
        Ok(stats)
    }

    /// Returns stats for an open request arena.
    #[must_use]
    pub fn request_stats(&self, request_id: RequestId) -> Option<ScratchArenaStats> {
        self.active.get(&request_id).map(ScratchArena::stats)
    }

    /// Returns pool-level accounting.
    #[must_use]
    pub fn pool_stats(&self) -> RequestScratchPoolStats {
        RequestScratchPoolStats {
            active_requests: self.active.len(),
            idle_arenas: self.idle.values().map(Vec::len).sum(),
            created_arenas: self.created_arenas,
            reused_arenas: self.reused_arenas,
        }
    }
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

    fn prepare_for_reuse(&mut self) {
        self.offset = 0;
        self.high_water_mark = 0;
        self.allocation_count = 0;
        self.reset_count = 0;
    }
}

/// KV block table accounting snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KvBlockTableStats {
    /// Sequence identifier.
    pub sequence_id: KvSequenceId,
    /// Token capacity of one block.
    pub tokens_per_block: u16,
    /// Logical token length.
    pub token_len: u64,
    /// Number of backing blocks.
    pub block_count: usize,
}

/// KV block pool accounting snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KvBlockPoolStats {
    /// Pool home node.
    pub home_node: NodeId,
    /// Size of each fixed block in bytes.
    pub block_size: usize,
    /// Total block capacity.
    pub capacity: usize,
    /// Allocated block count.
    pub allocated: usize,
    /// Free block count.
    pub free: usize,
    /// Maximum allocated blocks observed.
    pub high_water_mark: usize,
    /// Successful allocation count.
    pub allocation_count: u64,
    /// Successful free count.
    pub free_count: u64,
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
}

/// Error returned when parsing a mapped scratch lock status line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageLockProbeStatusLineParseError {
    /// The line does not contain a supported page-lock status token.
    MissingStatus,
    /// The line contains a duplicate page-lock status token.
    DuplicateStatus,
    /// The line contains a token outside the page-lock status schema.
    InvalidToken(String),
    /// The status field is not recognized.
    UnknownField(String),
    /// The status token is not recognized.
    UnknownStatus(String),
}

/// Error returned when extracting mapped scratch lock statuses from multiline output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MappedScratchLockProbeOutputParseError {
    /// The output does not contain a `page_lock=` line.
    MissingLockLine,
    /// The output has `page_lock=ok` but no `page_unlock=` line.
    MissingUnlockLine,
    /// The output contains more than one `page_lock=` line.
    DuplicateLockLine,
    /// The output contains more than one `page_unlock=` line.
    DuplicateUnlockLine,
    /// A discovered status line is malformed.
    Line(PageLockProbeStatusLineParseError),
}

/// KV block pool failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KvBlockPoolError {
    /// Block size must be non-zero.
    InvalidBlockSize,
    /// Capacity must be non-zero.
    InvalidCapacity,
    /// No free blocks are available.
    OutOfBlocks,
    /// The block handle is stale, invalid, or not allocated.
    InvalidHandle,
}

/// KV block table failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KvBlockTableError {
    /// Token count per block must be non-zero.
    InvalidTokensPerBlock,
    /// Token count overflowed.
    TokenCountOverflow,
    /// Backing pool operation failed.
    Pool(KvBlockPoolError),
}

/// Remote free queue configuration failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteFreeQueueError {
    /// Queue capacity must be non-zero.
    InvalidCapacity,
    /// Drain batch limit must be non-zero.
    InvalidBatchLimit,
}

/// Remote free enqueue failure.
pub struct RemoteFreeEnqueueError<T> {
    item: T,
}

impl<T> RemoteFreeEnqueueError<T> {
    /// Returns the item that could not be enqueued.
    #[must_use]
    pub fn into_item(self) -> T {
        self.item
    }
}

impl fmt::Display for KvBlockTableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTokensPerBlock => f.write_str("KV tokens per block must be non-zero"),
            Self::TokenCountOverflow => f.write_str("KV block table token count overflow"),
            Self::Pool(source) => write!(f, "KV block table pool operation failed: {source}"),
        }
    }
}

impl std::error::Error for KvBlockTableError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Pool(source) => Some(source),
            Self::InvalidTokensPerBlock | Self::TokenCountOverflow => None,
        }
    }
}

impl fmt::Display for KvBlockPoolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBlockSize => f.write_str("KV block size must be non-zero"),
            Self::InvalidCapacity => f.write_str("KV block pool capacity must be non-zero"),
            Self::OutOfBlocks => f.write_str("KV block pool is out of blocks"),
            Self::InvalidHandle => f.write_str("KV block handle is invalid or stale"),
        }
    }
}

impl std::error::Error for KvBlockPoolError {}

impl fmt::Display for RemoteFreeQueueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCapacity => f.write_str("remote free queue capacity must be non-zero"),
            Self::InvalidBatchLimit => {
                f.write_str("remote free queue batch limit must be non-zero")
            }
        }
    }
}

impl std::error::Error for RemoteFreeQueueError {}

impl<T> fmt::Debug for RemoteFreeEnqueueError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RemoteFreeEnqueueError")
            .finish_non_exhaustive()
    }
}

impl<T> fmt::Display for RemoteFreeEnqueueError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("remote free queue receiver is closed")
    }
}

impl<T> std::error::Error for RemoteFreeEnqueueError<T> {}

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
            Self::UnsupportedAlignment { .. }
            | Self::CapacityOverflow
            | Self::OutOfMemory { .. } => None,
        }
    }
}

impl fmt::Display for PageLockProbeStatusLineParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingStatus => f.write_str("missing page lock status token"),
            Self::DuplicateStatus => f.write_str("duplicate page lock status token"),
            Self::InvalidToken(token) => write!(f, "invalid page lock status token: {token}"),
            Self::UnknownField(field) => write!(f, "unknown page lock status field: {field}"),
            Self::UnknownStatus(status) => write!(f, "unknown page lock status: {status}"),
        }
    }
}

impl std::error::Error for PageLockProbeStatusLineParseError {}

impl fmt::Display for MappedScratchLockProbeOutputParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingLockLine => f.write_str("missing page_lock line"),
            Self::MissingUnlockLine => f.write_str("missing page_unlock line after page_lock=ok"),
            Self::DuplicateLockLine => f.write_str("duplicate page_lock line"),
            Self::DuplicateUnlockLine => f.write_str("duplicate page_unlock line"),
            Self::Line(source) => write!(f, "invalid page lock status line: {source}"),
        }
    }
}

impl std::error::Error for MappedScratchLockProbeOutputParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Line(source) => Some(source),
            Self::MissingLockLine
            | Self::MissingUnlockLine
            | Self::DuplicateLockLine
            | Self::DuplicateUnlockLine => None,
        }
    }
}

/// Request-scoped scratch allocation failures.
#[derive(Debug, PartialEq, Eq)]
pub enum RequestScratchError {
    /// Request had no home node.
    MissingHomeNode {
        /// Request identifier.
        request_id: RequestId,
    },
    /// Request already has an open arena.
    AlreadyOpen {
        /// Request identifier.
        request_id: RequestId,
    },
    /// Request does not have an open arena.
    NotOpen {
        /// Request identifier.
        request_id: RequestId,
    },
    /// Underlying arena operation failed.
    Arena {
        /// Request identifier.
        request_id: RequestId,
        /// Source allocation error.
        source: ScratchAllocError,
    },
}

impl fmt::Display for RequestScratchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingHomeNode { request_id } => {
                write!(f, "request {} does not have a home node", request_id.0)
            }
            Self::AlreadyOpen { request_id } => {
                write!(f, "request {} scratch arena is already open", request_id.0)
            }
            Self::NotOpen { request_id } => {
                write!(f, "request {} scratch arena is not open", request_id.0)
            }
            Self::Arena { request_id, source } => {
                write!(
                    f,
                    "request {} scratch allocation failed: {source}",
                    request_id.0
                )
            }
        }
    }
}

impl std::error::Error for RequestScratchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Arena { source, .. } => Some(source),
            Self::MissingHomeNode { .. } | Self::AlreadyOpen { .. } | Self::NotOpen { .. } => None,
        }
    }
}

/// Parses one mapped scratch page-lock status line.
///
/// The expected format is `page_lock=<status>` or `page_unlock=<status>`.
///
/// # Errors
///
/// Returns an error when the line is missing a status token, contains duplicate
/// status tokens, contains unsupported tokens, or uses an unknown field or status.
pub fn parse_page_lock_probe_status_line(
    line: &str,
) -> Result<PageLockProbeStatusLine, PageLockProbeStatusLineParseError> {
    let mut field = None;
    let mut status_token = None;

    for token in line.split_whitespace() {
        let Some((key, value)) = token.split_once('=') else {
            return Err(PageLockProbeStatusLineParseError::InvalidToken(
                token.to_owned(),
            ));
        };

        let parsed_field = PageLockProbeField::from_str_token(key)
            .ok_or_else(|| PageLockProbeStatusLineParseError::UnknownField(key.to_owned()))?;
        if field.replace(parsed_field).is_some() {
            return Err(PageLockProbeStatusLineParseError::DuplicateStatus);
        }
        if status_token.replace(value).is_some() {
            return Err(PageLockProbeStatusLineParseError::DuplicateStatus);
        }
    }

    let field = field.ok_or(PageLockProbeStatusLineParseError::MissingStatus)?;
    let status_token = status_token.ok_or(PageLockProbeStatusLineParseError::MissingStatus)?;
    let status = PageLockProbeStatus::from_str_token(status_token)
        .ok_or_else(|| PageLockProbeStatusLineParseError::UnknownStatus(status_token.to_owned()))?;

    Ok(PageLockProbeStatusLine { field, status })
}

/// Extracts mapped scratch page-lock statuses from multiline probe output.
///
/// # Errors
///
/// Returns an error when the output has no `page_lock=` line, has duplicate
/// status lines, has `page_lock=ok` without a `page_unlock=` line, or contains a
/// malformed page-lock status line.
pub fn parse_mapped_scratch_lock_probe_output(
    output: &str,
) -> Result<MappedScratchLockProbeOutput, MappedScratchLockProbeOutputParseError> {
    let mut page_lock = None;
    let mut page_unlock = None;

    for line in output.lines().map(str::trim) {
        if !(line.starts_with("page_lock=") || line.starts_with("page_unlock=")) {
            continue;
        }

        let parsed = parse_page_lock_probe_status_line(line)
            .map_err(MappedScratchLockProbeOutputParseError::Line)?;
        match parsed.field {
            PageLockProbeField::Lock => {
                if page_lock.replace(parsed.status).is_some() {
                    return Err(MappedScratchLockProbeOutputParseError::DuplicateLockLine);
                }
            }
            PageLockProbeField::Unlock => {
                if page_unlock.replace(parsed.status).is_some() {
                    return Err(MappedScratchLockProbeOutputParseError::DuplicateUnlockLine);
                }
            }
        }
    }

    let page_lock = page_lock.ok_or(MappedScratchLockProbeOutputParseError::MissingLockLine)?;
    if page_lock == PageLockProbeStatus::Ok && page_unlock.is_none() {
        return Err(MappedScratchLockProbeOutputParseError::MissingUnlockLine);
    }

    Ok(MappedScratchLockProbeOutput {
        page_lock,
        page_unlock,
    })
}

fn align_up(value: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two());
    (value + align - 1) & !(align - 1)
}

fn blocks_for_tokens(token_count: u64, tokens_per_block: u16) -> usize {
    let tokens_per_block = u64::from(tokens_per_block);
    let blocks = token_count.saturating_add(tokens_per_block - 1) / tokens_per_block;
    usize::try_from(blocks).unwrap_or(usize::MAX)
}

#[cfg(test)]
mod tests {
    use std::alloc::Layout;

    use locus_core::{NodeId, RequestHome, RequestId};

    use super::{
        parse_mapped_scratch_lock_probe_output, parse_page_lock_probe_status_line, KvBlockPool,
        KvBlockPoolError, KvBlockTable, KvBlockTableError, KvSequenceId, MappedScratchAllocError,
        MappedScratchArena, MappedScratchLockProbeOutput, MappedScratchLockProbeOutputParseError,
        PageLockError, PageLockProbeField, PageLockProbeStatus, PageLockProbeStatusLine,
        PageLockProbeStatusLineParseError, RemoteFreeQueue, RemoteFreeQueueError, RequestScratch,
        RequestScratchError, RequestScratchPool, ScratchAllocError, ScratchArena,
    };

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

    #[test]
    fn parses_mapped_scratch_lock_status_lines() {
        assert_eq!(
            parse_page_lock_probe_status_line("page_lock=ok").expect("lock ok"),
            PageLockProbeStatusLine {
                field: PageLockProbeField::Lock,
                status: PageLockProbeStatus::Ok,
            }
        );
        assert_eq!(
            parse_page_lock_probe_status_line("page_unlock=error").expect("unlock error"),
            PageLockProbeStatusLine {
                field: PageLockProbeField::Unlock,
                status: PageLockProbeStatus::Error,
            }
        );
        assert_eq!(PageLockProbeStatus::Ok.to_string(), "ok");
        assert_eq!(PageLockProbeField::Unlock.to_string(), "page_unlock");
    }

    #[test]
    fn rejects_invalid_mapped_scratch_lock_status_lines() {
        assert_eq!(
            parse_page_lock_probe_status_line("page_lock").expect_err("invalid token"),
            PageLockProbeStatusLineParseError::InvalidToken("page_lock".to_owned())
        );
        assert_eq!(
            parse_page_lock_probe_status_line("page_lock=maybe").expect_err("unknown status"),
            PageLockProbeStatusLineParseError::UnknownStatus("maybe".to_owned())
        );
        assert_eq!(
            parse_page_lock_probe_status_line("page_lock_error=mlock failed")
                .expect_err("unknown field"),
            PageLockProbeStatusLineParseError::UnknownField("page_lock_error".to_owned())
        );
        assert_eq!(
            parse_page_lock_probe_status_line("page_lock=ok page_unlock=ok")
                .expect_err("duplicate status"),
            PageLockProbeStatusLineParseError::DuplicateStatus
        );
    }

    #[test]
    fn parses_mapped_scratch_lock_probe_output() {
        let output = "\
mapping_start=0xffff8367a000
mapping_len=20479
touched=5
page_lock=ok
page_unlock=ok
";

        assert_eq!(
            parse_mapped_scratch_lock_probe_output(output).expect("probe output"),
            MappedScratchLockProbeOutput {
                page_lock: PageLockProbeStatus::Ok,
                page_unlock: Some(PageLockProbeStatus::Ok),
            }
        );

        let lock_error_output = "\
mapping_start=0xffff8367a000
page_lock=error
page_lock_error=mlock failed: Cannot allocate memory
";

        assert_eq!(
            parse_mapped_scratch_lock_probe_output(lock_error_output).expect("lock error output"),
            MappedScratchLockProbeOutput {
                page_lock: PageLockProbeStatus::Error,
                page_unlock: None,
            }
        );
    }

    #[test]
    fn rejects_invalid_mapped_scratch_lock_probe_output() {
        assert_eq!(
            parse_mapped_scratch_lock_probe_output("page_unlock=ok\n").expect_err("missing lock"),
            MappedScratchLockProbeOutputParseError::MissingLockLine
        );
        assert_eq!(
            parse_mapped_scratch_lock_probe_output("page_lock=ok\n").expect_err("missing unlock"),
            MappedScratchLockProbeOutputParseError::MissingUnlockLine
        );
        assert_eq!(
            parse_mapped_scratch_lock_probe_output("page_lock=error\npage_lock=ok\n")
                .expect_err("duplicate lock"),
            MappedScratchLockProbeOutputParseError::DuplicateLockLine
        );
        assert_eq!(
            parse_mapped_scratch_lock_probe_output(
                "page_lock=ok\npage_unlock=ok\npage_unlock=error\n"
            )
            .expect_err("duplicate unlock"),
            MappedScratchLockProbeOutputParseError::DuplicateUnlockLine
        );
        assert_eq!(
            parse_mapped_scratch_lock_probe_output("page_lock=maybe\n").expect_err("bad lock line"),
            MappedScratchLockProbeOutputParseError::Line(
                PageLockProbeStatusLineParseError::UnknownStatus("maybe".to_owned())
            )
        );
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

    #[test]
    fn manages_request_scoped_arenas() {
        let mut scratch = RequestScratch::new();
        let request_id = RequestId(11);
        let home = RequestHome {
            request_id,
            node: Some(NodeId(1)),
            reason: "test",
        };

        scratch.open_request(&home, 256).expect("open request");
        let allocation = scratch
            .alloc_bytes(request_id, Layout::from_size_align(64, 32).expect("layout"))
            .expect("allocation");
        assert_eq!(allocation.len(), 64);

        scratch.reset_request(request_id).expect("reset request");
        let stats = scratch.close_request(request_id).expect("close request");

        assert_eq!(stats.home_node, NodeId(1));
        assert_eq!(stats.reset_count, 1);
        assert_eq!(scratch.open_request_count(), 0);
    }

    #[test]
    fn rejects_request_without_home_node() {
        let mut scratch = RequestScratch::new();
        let error = scratch
            .open_request(
                &RequestHome {
                    request_id: RequestId(9),
                    node: None,
                    reason: "test",
                },
                128,
            )
            .expect_err("missing home node should fail");

        assert_eq!(
            error,
            RequestScratchError::MissingHomeNode {
                request_id: RequestId(9),
            }
        );
    }

    #[test]
    fn rejects_alloc_for_closed_request() {
        let mut scratch = RequestScratch::new();
        let error = scratch
            .alloc_bytes(RequestId(7), Layout::from_size_align(8, 8).expect("layout"))
            .expect_err("request is not open");

        assert_eq!(
            error,
            RequestScratchError::NotOpen {
                request_id: RequestId(7),
            }
        );
    }

    #[test]
    fn reuses_closed_request_arenas_by_node_and_capacity() {
        let mut pool = RequestScratchPool::new();
        let first = RequestHome {
            request_id: RequestId(1),
            node: Some(NodeId(0)),
            reason: "test",
        };
        let second = RequestHome {
            request_id: RequestId(2),
            node: Some(NodeId(0)),
            reason: "test",
        };

        pool.open_request(&first, 256).expect("open first");
        pool.alloc_bytes(
            first.request_id,
            Layout::from_size_align(64, 16).expect("layout"),
        )
        .expect("first allocation");
        let first_stats = pool
            .close_request(first.request_id)
            .expect("close first request");
        pool.open_request(&second, 256).expect("open second");
        let second_stats = pool
            .request_stats(second.request_id)
            .expect("second request stats");

        assert_eq!(first_stats.allocation_count, 1);
        assert_eq!(second_stats.allocation_count, 0);
        assert_eq!(second_stats.high_water_mark, 0);
        assert_eq!(
            pool.pool_stats(),
            super::RequestScratchPoolStats {
                active_requests: 1,
                idle_arenas: 0,
                created_arenas: 1,
                reused_arenas: 1,
            }
        );
    }

    #[test]
    fn keeps_different_capacity_arenas_in_separate_idle_classes() {
        let mut pool = RequestScratchPool::new();
        let first = RequestHome {
            request_id: RequestId(1),
            node: Some(NodeId(0)),
            reason: "test",
        };
        let second = RequestHome {
            request_id: RequestId(2),
            node: Some(NodeId(0)),
            reason: "test",
        };

        pool.open_request(&first, 256).expect("open first");
        pool.close_request(first.request_id).expect("close first");
        pool.open_request(&second, 512).expect("open second");

        let stats = pool.pool_stats();
        assert_eq!(stats.created_arenas, 2);
        assert_eq!(stats.reused_arenas, 0);
        assert_eq!(stats.idle_arenas, 1);
    }

    #[test]
    fn allocates_and_reuses_kv_blocks() {
        let mut pool = KvBlockPool::new(NodeId(0), 4096, 2).expect("pool");

        let first = pool.allocate().expect("first block");
        let second = pool.allocate().expect("second block");
        assert_eq!(pool.allocate(), Err(KvBlockPoolError::OutOfBlocks));

        pool.block_mut(first).expect("first block")[0] = 7;
        pool.free(first).expect("free first");
        let third = pool.allocate().expect("reused block");

        assert_ne!(first, third);
        assert_eq!(pool.block_mut(third).expect("third block")[0], 7);

        let stats = pool.stats();
        assert_eq!(stats.home_node, NodeId(0));
        assert_eq!(stats.capacity, 2);
        assert_eq!(stats.allocated, 2);
        assert_eq!(stats.high_water_mark, 2);

        pool.free(second).expect("free second");
        pool.free(third).expect("free third");
        assert_eq!(pool.stats().free, 2);
    }

    #[test]
    fn rejects_stale_kv_block_handles() {
        let mut pool = KvBlockPool::new(NodeId(1), 1024, 1).expect("pool");
        let handle = pool.allocate().expect("block");

        pool.free(handle).expect("free block");

        assert_eq!(pool.free(handle), Err(KvBlockPoolError::InvalidHandle));
        assert_eq!(
            pool.block_mut(handle).expect_err("stale handle"),
            KvBlockPoolError::InvalidHandle
        );
    }

    #[test]
    fn rejects_invalid_kv_pool_configuration() {
        assert_eq!(
            KvBlockPool::new(NodeId(0), 0, 1).expect_err("zero block size"),
            KvBlockPoolError::InvalidBlockSize
        );
        assert_eq!(
            KvBlockPool::new(NodeId(0), 4096, 0).expect_err("zero capacity"),
            KvBlockPoolError::InvalidCapacity
        );
    }

    #[test]
    fn grows_and_releases_kv_block_table() {
        let mut pool = KvBlockPool::new(NodeId(0), 4096, 8).expect("pool");
        let mut table = KvBlockTable::new(KvSequenceId(99), 16).expect("table");

        table.append_tokens(&mut pool, 1).expect("append first");
        assert_eq!(table.stats().block_count, 1);
        table
            .append_tokens(&mut pool, 15)
            .expect("fill first block");
        assert_eq!(table.stats().block_count, 1);
        table
            .append_tokens(&mut pool, 1)
            .expect("open second block");
        assert_eq!(table.stats().block_count, 2);
        assert_eq!(pool.stats().allocated, 2);

        table.release_all(&mut pool).expect("release table");
        assert_eq!(table.stats().token_len, 0);
        assert_eq!(table.stats().block_count, 0);
        assert_eq!(pool.stats().free, 8);
    }

    #[test]
    fn reports_pool_exhaustion_from_kv_block_table() {
        let mut pool = KvBlockPool::new(NodeId(0), 4096, 1).expect("pool");
        let mut table = KvBlockTable::new(KvSequenceId(1), 1).expect("table");

        let error = table
            .append_tokens(&mut pool, 2)
            .expect_err("pool should run out of blocks");

        assert_eq!(
            error,
            KvBlockTableError::Pool(KvBlockPoolError::OutOfBlocks)
        );
        assert_eq!(pool.stats().allocated, 0);
        assert_eq!(table.stats().block_count, 0);
    }

    #[test]
    fn rejects_zero_tokens_per_kv_block() {
        assert_eq!(
            KvBlockTable::new(KvSequenceId(1), 0).expect_err("invalid table"),
            KvBlockTableError::InvalidTokensPerBlock
        );
    }

    #[test]
    fn remote_free_queue_drains_in_batches() {
        let mut queue = RemoteFreeQueue::new(8, 2).expect("queue");
        let sink = queue.sink();

        sink.enqueue(1).expect("enqueue first");
        sink.enqueue(2).expect("enqueue second");
        sink.enqueue(3).expect("enqueue third");

        let mut released = Vec::new();
        let first = queue.drain_batch(|item| released.push(item));

        assert_eq!(first.drained, 2);
        assert_eq!(first.total_drained, 2);
        assert_eq!(released, vec![1, 2]);
        assert_eq!(
            queue.stats(),
            super::RemoteFreeQueueStats {
                capacity: 8,
                batch_limit: 2,
                submitted_count: 3,
                drained_count: 2,
            }
        );

        let second = queue.drain_batch(|item| released.push(item));

        assert_eq!(second.drained, 1);
        assert_eq!(second.total_drained, 3);
        assert_eq!(released, vec![1, 2, 3]);
    }

    #[test]
    fn remote_free_queue_rejects_invalid_configuration() {
        assert_eq!(
            RemoteFreeQueue::<u8>::new(0, 1).expect_err("zero capacity"),
            RemoteFreeQueueError::InvalidCapacity
        );
        assert_eq!(
            RemoteFreeQueue::<u8>::new(1, 0).expect_err("zero batch limit"),
            RemoteFreeQueueError::InvalidBatchLimit
        );
    }

    #[test]
    fn remote_free_sink_returns_item_when_owner_is_dropped() {
        let queue = RemoteFreeQueue::new(1, 1).expect("queue");
        let sink = queue.sink();
        drop(queue);

        let error = sink.enqueue(7).expect_err("receiver is closed");

        assert_eq!(error.into_item(), 7);
        assert_eq!(sink.submitted_count(), 0);
    }
}
