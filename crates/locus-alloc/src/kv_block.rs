use std::fmt;

use locus_core::NodeId;

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

fn blocks_for_tokens(token_count: u64, tokens_per_block: u16) -> usize {
    let tokens_per_block = u64::from(tokens_per_block);
    let blocks = token_count.saturating_add(tokens_per_block - 1) / tokens_per_block;
    usize::try_from(blocks).unwrap_or(usize::MAX)
}

#[cfg(test)]
mod tests {
    use locus_core::NodeId;

    use super::{KvBlockPool, KvBlockPoolError, KvBlockTable, KvBlockTableError, KvSequenceId};

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
}
