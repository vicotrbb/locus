//! KV-block pooling for inference serving.
//!
//! [`KvBlockPool`] is a fixed-size block pool with generation-validated
//! handles, LIFO reuse order (validated for cache warmth by experiment
//! 0358, `documentation/experiments/0358-reuse-order-cache-warmth.md`),
//! and optional mapped-region backing (experiment 0359,
//! `documentation/experiments/0359-mapped-backed-kv-pool.md`). Combined
//! with per-worker [`crate::remote_free::ChunkMailbox`]es it forms the
//! design validated by LOCUS-EVAL v1
//! (`documentation/evaluations/0001-locus-eval-v1.md`).

mod block;

pub use block::{
    KvBlockHandle, KvBlockPool, KvBlockPoolError, KvBlockPoolStats, KvBlockTable,
    KvBlockTableError, KvBlockTableStats, KvReuseOrder, KvSequenceId,
};
