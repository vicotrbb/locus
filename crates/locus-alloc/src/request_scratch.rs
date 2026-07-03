use std::alloc::Layout;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::fmt;

use locus_core::{NodeId, RequestHome, RequestId};

use crate::{ScratchAllocError, ScratchArena, ScratchArenaStats};

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

#[cfg(test)]
mod tests {
    use std::alloc::Layout;

    use locus_core::{NodeId, RequestHome, RequestId};

    use super::{RequestScratch, RequestScratchError, RequestScratchPool, RequestScratchPoolStats};

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
            RequestScratchPoolStats {
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
}
