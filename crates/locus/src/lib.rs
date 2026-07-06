//! NUMA-aware memory pooling primitives for CPU LLM inference serving.
//!
//! Locus provides a KV-block pool with owner-drained remote frees: workers
//! free whole request chunks into per-worker mailboxes and the pool owner
//! drains them, keeping the free path off the allocation hot path. The
//! design and its performance claims are backed by the experiment record in
//! `documentation/` of the repository; every documented public item cites
//! the experiment or evaluation that validated it.
//!
//! Validated headline result (LOCUS-EVAL v1 with touch-parity audit,
//! `documentation/evaluations/0001-locus-eval-v1.md`): on deterministic
//! serving-shaped KV traces at touch parity, the mailbox design is 1.6x to
//! 2.7x faster than mimalloc; when write bandwidth dominates, the margin
//! compresses to about 1.15x over system malloc.
//!
//! Unsafe code policy: the crate denies `unsafe_code` everywhere except the
//! [`sys`] module, the single audited boundary for `mmap`/`mbind` and raw
//! memory handles (ADR 0002).

#![deny(unsafe_code)]

#[allow(unsafe_code)]
pub mod sys;

pub mod kv;
pub mod remote_free;
pub mod topology;

#[doc(hidden)]
pub mod cpuset;
#[doc(hidden)]
pub mod policy;
#[doc(hidden)]
pub mod request;

#[doc(hidden)]
pub mod mapped_scratch;
#[doc(hidden)]
pub mod mapped_scratch_lock_probe;
#[doc(hidden)]
pub mod mapped_scratch_thp_fault_sample;
#[doc(hidden)]
pub mod mapped_scratch_thp_page_sample;
#[doc(hidden)]
pub mod mapped_scratch_thp_probe;
#[doc(hidden)]
pub mod pinned_scratch;
#[doc(hidden)]
pub mod pinned_scratch_near_gpu_probe;
#[doc(hidden)]
pub mod pinned_scratch_pool_probe;
#[doc(hidden)]
pub mod request_scratch;
#[doc(hidden)]
pub mod scratch_arena;

pub use kv::{
    KvBlockHandle, KvBlockPool, KvBlockPoolError, KvBlockPoolStats, KvBlockTable,
    KvBlockTableError, KvBlockTableStats, KvReuseOrder, KvSequenceId,
};
pub use remote_free::{ChunkMailbox, ChunkMailboxSender};
pub use topology::NodeId;
#[doc(hidden)]
pub use topology::{NumaNode, PciDevice, Topology};

#[doc(hidden)]
pub use cpuset::{CpuSet, CpuSetParseError};
#[doc(hidden)]
pub use mapped_scratch::{
    MappedScratchAllocError, MappedScratchArena, MappedScratchHugePageAdvice,
};
#[doc(hidden)]
pub use mapped_scratch_lock_probe::{
    parse_mapped_scratch_lock_probe_output, parse_page_lock_probe_status_line,
    MappedScratchLockProbeOutput, MappedScratchLockProbeOutputParseError, PageLockProbeField,
    PageLockProbeStatus, PageLockProbeStatusLine, PageLockProbeStatusLineParseError,
};
#[doc(hidden)]
pub use mapped_scratch_thp_fault_sample::{
    parse_mapped_scratch_thp_fault_sample_line, parse_mapped_scratch_thp_fault_samples_output,
    MappedScratchThpFaultSampleComparison, MappedScratchThpFaultSampleLine,
    MappedScratchThpFaultSampleLineParseError, MappedScratchThpFaultSampleMode,
    MappedScratchThpFaultSampleStatus, MappedScratchThpFaultSamples,
    MappedScratchThpFaultSamplesParseError,
};
#[doc(hidden)]
pub use mapped_scratch_thp_page_sample::{
    parse_mapped_scratch_thp_page_sample_line, parse_mapped_scratch_thp_page_samples_output,
    MappedScratchThpPageSampleLine, MappedScratchThpPageSampleLineParseError,
    MappedScratchThpPageSampleSource, MappedScratchThpPageSampleStatus,
    MappedScratchThpPageSamples, MappedScratchThpPageSamplesParseError,
};
#[doc(hidden)]
pub use mapped_scratch_thp_probe::{
    parse_mapped_scratch_thp_probe_output, MappedScratchThpAdviceStatus,
    MappedScratchThpObservation, MappedScratchThpProbeOutput,
    MappedScratchThpProbeOutputParseError, MappedScratchThpProbeRunStatus,
};
#[doc(hidden)]
pub use pinned_scratch::{
    PinnedScratchHandle, PinnedScratchPool, PinnedScratchPoolError, PinnedScratchPoolStats,
};
#[doc(hidden)]
pub use pinned_scratch_near_gpu_probe::{
    parse_pinned_scratch_near_gpu_probe_output, parse_pinned_scratch_near_gpu_probe_pool_line,
    PinnedScratchNearGpuPoolLine, PinnedScratchNearGpuProbeLineParseError,
    PinnedScratchNearGpuProbeOutput, PinnedScratchNearGpuProbeOutputParseError,
    PinnedScratchNearGpuProbeStatus,
};
#[doc(hidden)]
pub use pinned_scratch_pool_probe::{
    parse_pinned_scratch_pool_probe_event_line, parse_pinned_scratch_pool_probe_output,
    parse_pinned_scratch_pool_probe_stats_line, PinnedScratchPoolProbeEvent,
    PinnedScratchPoolProbeEventLine, PinnedScratchPoolProbeLineParseError,
    PinnedScratchPoolProbeOutput, PinnedScratchPoolProbeOutputParseError,
    PinnedScratchPoolProbePhase, PinnedScratchPoolProbeStatsLine, PinnedScratchPoolProbeStatus,
};
#[doc(hidden)]
pub use policy::{
    choose_initial_policy, resolve_topology_policy, LifetimeHint, LocalityDecision, MemoryClass,
    NodeSet, PlacementPolicy, PlacementRequest,
};
#[doc(hidden)]
pub use remote_free::{
    RemoteFreeDrainStats, RemoteFreeEnqueueError, RemoteFreeQueue, RemoteFreeQueueError,
    RemoteFreeQueueStats, RemoteFreeSink, RemoteFreeTryEnqueueError, RemoteFreeTryEnqueueErrorKind,
};
#[doc(hidden)]
pub use request::{choose_request_home, GpuId, RequestAffinity, RequestHome, RequestId};
#[doc(hidden)]
pub use request_scratch::{
    RequestScratch, RequestScratchError, RequestScratchPool, RequestScratchPoolStats,
};
#[doc(hidden)]
pub use scratch_arena::{ScratchAllocError, ScratchArena, ScratchArenaStats};

const MAX_SUPPORTED_ALIGN: usize = 4096;
