//! Experimental domain allocators for Locus.

mod kv_block;
mod mapped_scratch;
mod mapped_scratch_lock_probe;
mod mapped_scratch_thp_fault_sample;
mod mapped_scratch_thp_probe;
mod pinned_scratch;
mod pinned_scratch_near_gpu_probe;
mod pinned_scratch_pool_probe;
mod remote_free;
mod request_scratch;
mod scratch_arena;

pub use kv_block::{
    KvBlockHandle, KvBlockPool, KvBlockPoolError, KvBlockPoolStats, KvBlockTable,
    KvBlockTableError, KvBlockTableStats, KvSequenceId,
};

pub use mapped_scratch::{
    MappedScratchAllocError, MappedScratchArena, MappedScratchHugePageAdvice,
};

pub use mapped_scratch_lock_probe::{
    parse_mapped_scratch_lock_probe_output, parse_page_lock_probe_status_line,
    MappedScratchLockProbeOutput, MappedScratchLockProbeOutputParseError, PageLockProbeField,
    PageLockProbeStatus, PageLockProbeStatusLine, PageLockProbeStatusLineParseError,
};

pub use mapped_scratch_thp_probe::{
    parse_mapped_scratch_thp_probe_output, MappedScratchThpAdviceStatus,
    MappedScratchThpObservation, MappedScratchThpProbeOutput,
    MappedScratchThpProbeOutputParseError, MappedScratchThpProbeRunStatus,
};

pub use mapped_scratch_thp_fault_sample::{
    parse_mapped_scratch_thp_fault_sample_line, parse_mapped_scratch_thp_fault_samples_output,
    MappedScratchThpFaultSampleComparison, MappedScratchThpFaultSampleLine,
    MappedScratchThpFaultSampleLineParseError, MappedScratchThpFaultSampleMode,
    MappedScratchThpFaultSampleStatus, MappedScratchThpFaultSamples,
    MappedScratchThpFaultSamplesParseError,
};

pub use pinned_scratch::{
    PinnedScratchHandle, PinnedScratchPool, PinnedScratchPoolError, PinnedScratchPoolStats,
};

pub use pinned_scratch_near_gpu_probe::{
    parse_pinned_scratch_near_gpu_probe_output, parse_pinned_scratch_near_gpu_probe_pool_line,
    PinnedScratchNearGpuPoolLine, PinnedScratchNearGpuProbeLineParseError,
    PinnedScratchNearGpuProbeOutput, PinnedScratchNearGpuProbeOutputParseError,
    PinnedScratchNearGpuProbeStatus,
};

pub use pinned_scratch_pool_probe::{
    parse_pinned_scratch_pool_probe_event_line, parse_pinned_scratch_pool_probe_output,
    parse_pinned_scratch_pool_probe_stats_line, PinnedScratchPoolProbeEvent,
    PinnedScratchPoolProbeEventLine, PinnedScratchPoolProbeLineParseError,
    PinnedScratchPoolProbeOutput, PinnedScratchPoolProbeOutputParseError,
    PinnedScratchPoolProbePhase, PinnedScratchPoolProbeStatsLine, PinnedScratchPoolProbeStatus,
};

pub use remote_free::{
    RemoteFreeDrainStats, RemoteFreeEnqueueError, RemoteFreeQueue, RemoteFreeQueueError,
    RemoteFreeQueueStats, RemoteFreeSink, RemoteFreeTryEnqueueError, RemoteFreeTryEnqueueErrorKind,
};

pub use request_scratch::{
    RequestScratch, RequestScratchError, RequestScratchPool, RequestScratchPoolStats,
};

pub use scratch_arena::{ScratchAllocError, ScratchArena, ScratchArenaStats};

const MAX_SUPPORTED_ALIGN: usize = 4096;
