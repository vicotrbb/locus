//! Experimental domain allocators for Locus.

use std::alloc::Layout;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TryRecvError};
use std::sync::Arc;

use locus_core::{
    choose_initial_policy, resolve_topology_policy, LifetimeHint, MemoryClass, NodeId,
    PlacementPolicy, PlacementRequest, RequestHome, RequestId, Topology,
};
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

/// Stable status for pinned scratch pool probe events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinnedScratchPoolProbeStatus {
    /// The operation succeeded.
    Ok,
    /// The operation failed.
    Error,
}

impl PinnedScratchPoolProbeStatus {
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

impl fmt::Display for PinnedScratchPoolProbeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Stable pinned scratch pool probe event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinnedScratchPoolProbeEvent {
    /// `pool_checkout=<status>`.
    Checkout,
    /// `checked_out_allocation=<status>`.
    Allocation,
    /// `pool_release=<status>`.
    Release,
    /// `pool_reuse_checkout=<status>`.
    ReuseCheckout,
    /// `pool_reuse_release=<status>`.
    ReuseRelease,
}

impl PinnedScratchPoolProbeEvent {
    /// Returns the stable machine-readable field name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Checkout => "pool_checkout",
            Self::Allocation => "checked_out_allocation",
            Self::Release => "pool_release",
            Self::ReuseCheckout => "pool_reuse_checkout",
            Self::ReuseRelease => "pool_reuse_release",
        }
    }

    /// Parses a stable machine-readable field name.
    #[must_use]
    pub fn from_str_token(value: &str) -> Option<Self> {
        match value {
            "pool_checkout" => Some(Self::Checkout),
            "checked_out_allocation" => Some(Self::Allocation),
            "pool_release" => Some(Self::Release),
            "pool_reuse_checkout" => Some(Self::ReuseCheckout),
            "pool_reuse_release" => Some(Self::ReuseRelease),
            _ => None,
        }
    }

    fn requires_handle(self) -> bool {
        matches!(
            self,
            Self::Checkout | Self::Release | Self::ReuseCheckout | Self::ReuseRelease
        )
    }
}

impl fmt::Display for PinnedScratchPoolProbeEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Stable pinned scratch pool stats phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinnedScratchPoolProbePhase {
    /// Initial pool stats.
    Initial,
    /// Stats after checkout failed.
    CheckoutError,
    /// Stats after the first checkout.
    AfterCheckout,
    /// Stats after the first release.
    AfterRelease,
    /// Stats after reuse checkout.
    AfterReuseCheckout,
    /// Stats after reuse release.
    AfterReuseRelease,
}

impl PinnedScratchPoolProbePhase {
    /// Returns the stable machine-readable phase string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Initial => "initial",
            Self::CheckoutError => "checkout_error",
            Self::AfterCheckout => "after_checkout",
            Self::AfterRelease => "after_release",
            Self::AfterReuseCheckout => "after_reuse_checkout",
            Self::AfterReuseRelease => "after_reuse_release",
        }
    }

    /// Parses a stable machine-readable phase string.
    #[must_use]
    pub fn from_str_token(value: &str) -> Option<Self> {
        match value {
            "initial" => Some(Self::Initial),
            "checkout_error" => Some(Self::CheckoutError),
            "after_checkout" => Some(Self::AfterCheckout),
            "after_release" => Some(Self::AfterRelease),
            "after_reuse_checkout" => Some(Self::AfterReuseCheckout),
            "after_reuse_release" => Some(Self::AfterReuseRelease),
            _ => None,
        }
    }
}

impl fmt::Display for PinnedScratchPoolProbePhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Parsed pinned scratch pool probe event line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PinnedScratchPoolProbeEventLine {
    /// Parsed event field.
    pub event: PinnedScratchPoolProbeEvent,
    /// Parsed event status.
    pub status: PinnedScratchPoolProbeStatus,
    /// Parsed checkout or release handle, when present.
    pub handle: Option<u64>,
    /// Parsed allocation byte count, when present.
    pub bytes: Option<usize>,
}

/// Parsed pinned scratch pool probe stats line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PinnedScratchPoolProbeStatsLine {
    /// Stats phase.
    pub phase: PinnedScratchPoolProbePhase,
    /// Locked bytes owned by the pool.
    pub locked_bytes: usize,
    /// Checked-out arena count.
    pub checked_out: usize,
    /// Idle arena count.
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

/// Parsed pinned scratch pool probe output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PinnedScratchPoolProbeOutput {
    /// Initial pool stats.
    pub initial_stats: PinnedScratchPoolProbeStatsLine,
    /// First checkout event.
    pub checkout: PinnedScratchPoolProbeEventLine,
    /// Stats after checkout failed, when checkout failed.
    pub checkout_error_stats: Option<PinnedScratchPoolProbeStatsLine>,
    /// Allocation event, required when checkout succeeded.
    pub allocation: Option<PinnedScratchPoolProbeEventLine>,
    /// Release event, required when checkout succeeded.
    pub release: Option<PinnedScratchPoolProbeEventLine>,
    /// Reuse checkout event, required when checkout succeeded.
    pub reuse_checkout: Option<PinnedScratchPoolProbeEventLine>,
    /// Reuse release event, required when checkout succeeded.
    pub reuse_release: Option<PinnedScratchPoolProbeEventLine>,
    /// Stats after the first checkout, required when checkout succeeded.
    pub after_checkout_stats: Option<PinnedScratchPoolProbeStatsLine>,
    /// Stats after the first release, required when checkout succeeded.
    pub after_release_stats: Option<PinnedScratchPoolProbeStatsLine>,
    /// Stats after reuse checkout, required when checkout succeeded.
    pub after_reuse_checkout_stats: Option<PinnedScratchPoolProbeStatsLine>,
    /// Stats after reuse release, required when checkout succeeded.
    pub after_reuse_release_stats: Option<PinnedScratchPoolProbeStatsLine>,
}

/// Stable status for near-GPU pinned scratch constructor lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinnedScratchNearGpuProbeStatus {
    /// The pool was constructed from GPU locality.
    Ok,
    /// GPU locality was not available in the discovered topology.
    Unavailable,
    /// Pool construction failed for a reason other than missing locality.
    Error,
}

impl PinnedScratchNearGpuProbeStatus {
    /// Returns a stable machine-readable status string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Unavailable => "unavailable",
            Self::Error => "error",
        }
    }

    /// Parses a stable machine-readable status string.
    #[must_use]
    pub fn from_str_token(value: &str) -> Option<Self> {
        match value {
            "ok" => Some(Self::Ok),
            "unavailable" => Some(Self::Unavailable),
            "error" => Some(Self::Error),
            _ => None,
        }
    }
}

impl fmt::Display for PinnedScratchNearGpuProbeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Parsed near-GPU pinned scratch constructor line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinnedScratchNearGpuPoolLine {
    /// Constructor status.
    pub status: PinnedScratchNearGpuProbeStatus,
    /// Resolved pool home node when construction succeeds.
    pub home_node: Option<NodeId>,
    /// Stable unavailable reason when locality is unavailable.
    pub reason: Option<String>,
}

/// Parsed near-GPU pinned scratch probe output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinnedScratchNearGpuProbeOutput {
    /// Discovered topology node count.
    pub topology_nodes: usize,
    /// Discovered PCI device count.
    pub topology_pci_devices: usize,
    /// Selected GPU BDF, when a candidate was available or supplied.
    pub gpu_bdf: Option<String>,
    /// Probe arena capacity, present after GPU selection.
    pub arena_capacity: Option<usize>,
    /// Probe locked-byte budget, present after GPU selection.
    pub max_locked_bytes: Option<usize>,
    /// Constructor result line.
    pub pool: PinnedScratchNearGpuPoolLine,
    /// Initial pool stats, present when construction succeeds.
    pub initial_stats: Option<PinnedScratchPoolProbeStatsLine>,
    /// Checkout event, present when construction succeeds.
    pub checkout: Option<PinnedScratchPoolProbeEventLine>,
    /// Stats after checkout failed, when checkout failed.
    pub checkout_error_stats: Option<PinnedScratchPoolProbeStatsLine>,
    /// Allocation event, required when checkout succeeds.
    pub allocation: Option<PinnedScratchPoolProbeEventLine>,
    /// Release event, required when checkout succeeds.
    pub release: Option<PinnedScratchPoolProbeEventLine>,
    /// Stats after checkout, required when checkout succeeds.
    pub after_checkout_stats: Option<PinnedScratchPoolProbeStatsLine>,
    /// Stats after release, required when checkout succeeds.
    pub after_release_stats: Option<PinnedScratchPoolProbeStatsLine>,
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

/// Stable run status for mapped scratch THP probe output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappedScratchThpProbeRunStatus {
    /// The Linux probe started.
    Started,
    /// The target platform does not support the probe.
    UnsupportedPlatform,
}

impl MappedScratchThpProbeRunStatus {
    /// Returns a stable machine-readable run status string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Started => "started",
            Self::UnsupportedPlatform => "unsupported-platform",
        }
    }

    /// Parses a stable machine-readable run status string.
    #[must_use]
    pub fn from_str_token(value: &str) -> Option<Self> {
        match value {
            "started" => Some(Self::Started),
            "unsupported-platform" => Some(Self::UnsupportedPlatform),
            _ => None,
        }
    }
}

impl fmt::Display for MappedScratchThpProbeRunStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Stable advice status for mapped scratch THP probe output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappedScratchThpAdviceStatus {
    /// The kernel accepted the advice.
    Ok,
    /// The advice call failed.
    Error,
}

impl MappedScratchThpAdviceStatus {
    /// Returns a stable machine-readable advice status string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Error => "error",
        }
    }

    /// Parses a stable machine-readable advice status string.
    #[must_use]
    pub fn from_str_token(value: &str) -> Option<Self> {
        match value {
            "ok" => Some(Self::Ok),
            "error" => Some(Self::Error),
            _ => None,
        }
    }
}

impl fmt::Display for MappedScratchThpAdviceStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Stable observation status for mapped scratch THP probe output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappedScratchThpObservation {
    /// A page size larger than the base page size was observed.
    Yes,
    /// Only the base page size was observed.
    No,
    /// Observation evidence was not available or was incomplete.
    Unknown,
}

impl MappedScratchThpObservation {
    /// Returns a stable machine-readable observation string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Yes => "yes",
            Self::No => "no",
            Self::Unknown => "unknown",
        }
    }

    /// Parses a stable machine-readable observation string.
    #[must_use]
    pub fn from_str_token(value: &str) -> Option<Self> {
        match value {
            "yes" => Some(Self::Yes),
            "no" => Some(Self::No),
            "unknown" => Some(Self::Unknown),
            _ => None,
        }
    }
}

impl fmt::Display for MappedScratchThpObservation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Parsed mapped scratch THP probe output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MappedScratchThpProbeOutput {
    /// Probe run status.
    pub run_status: MappedScratchThpProbeRunStatus,
    /// Requested advice mode, present when the probe started.
    pub mode: Option<MappedScratchHugePageAdvice>,
    /// Advice call status, present when the probe started.
    pub advice_status: Option<MappedScratchThpAdviceStatus>,
    /// Number of pages touched after successful advice.
    pub touched: Option<usize>,
    /// Observed kernel page size in KiB, when reported as a number.
    pub kernel_page_kb: Option<usize>,
    /// THP observation status, present after successful advice.
    pub observation: Option<MappedScratchThpObservation>,
    /// Stable observation reason, present with `observation`.
    pub observation_reason: Option<String>,
}

/// Stable mapped scratch THP benchmark fault sample mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MappedScratchThpFaultSampleMode {
    /// Default mapped scratch arena with no THP advice.
    Default,
    /// Mapped scratch arena advised with `hugepage`.
    HugePage,
    /// Mapped scratch arena advised with `no_hugepage`.
    NoHugePage,
}

impl MappedScratchThpFaultSampleMode {
    /// Returns a stable machine-readable mode string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::HugePage => "hugepage",
            Self::NoHugePage => "no_hugepage",
        }
    }

    /// Parses a stable machine-readable mode string.
    #[must_use]
    pub fn from_str_token(value: &str) -> Option<Self> {
        match value {
            "default" => Some(Self::Default),
            "hugepage" => Some(Self::HugePage),
            "no_hugepage" => Some(Self::NoHugePage),
            _ => None,
        }
    }
}

impl fmt::Display for MappedScratchThpFaultSampleMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Stable mapped scratch THP benchmark fault sample status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappedScratchThpFaultSampleStatus {
    /// Process fault counters were sampled.
    Available,
    /// Process fault counters were unavailable.
    Unavailable,
}

impl MappedScratchThpFaultSampleStatus {
    /// Returns a stable machine-readable status string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Unavailable => "unavailable",
        }
    }

    /// Parses a stable machine-readable status string.
    #[must_use]
    pub fn from_str_token(value: &str) -> Option<Self> {
        match value {
            "available" => Some(Self::Available),
            "unavailable" => Some(Self::Unavailable),
            _ => None,
        }
    }
}

impl fmt::Display for MappedScratchThpFaultSampleStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Parsed mapped scratch THP benchmark fault sample line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MappedScratchThpFaultSampleLine {
    /// Sample mode.
    pub mode: MappedScratchThpFaultSampleMode,
    /// Fault counter availability.
    pub status: MappedScratchThpFaultSampleStatus,
    /// Number of sample iterations, present when counters are available.
    pub iterations: Option<usize>,
    /// Signed process minor-fault delta.
    pub minor_faults_delta: Option<i128>,
    /// Signed child minor-fault delta.
    pub child_minor_faults_delta: Option<i128>,
    /// Signed process major-fault delta.
    pub major_faults_delta: Option<i128>,
    /// Signed child major-fault delta.
    pub child_major_faults_delta: Option<i128>,
}

/// Parsed mapped scratch THP benchmark fault samples.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MappedScratchThpFaultSamples {
    /// Default mode fault sample.
    pub default: MappedScratchThpFaultSampleLine,
    /// Hugepage advice mode fault sample.
    pub hugepage: MappedScratchThpFaultSampleLine,
    /// No-hugepage advice mode fault sample.
    pub no_hugepage: MappedScratchThpFaultSampleLine,
}

/// Process fault comparison for mapped scratch THP benchmark samples.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MappedScratchThpFaultSampleComparison {
    /// Default mode process minor-fault delta.
    pub default_minor_faults_delta: i128,
    /// Hugepage advice mode process minor-fault delta.
    pub hugepage_minor_faults_delta: i128,
    /// No-hugepage advice mode process minor-fault delta.
    pub no_hugepage_minor_faults_delta: i128,
    /// Hugepage process minor-fault delta minus default process minor-fault delta.
    pub hugepage_vs_default_minor_faults_delta: i128,
    /// Hugepage process minor-fault delta minus no-hugepage process minor-fault delta.
    pub hugepage_vs_no_hugepage_minor_faults_delta: i128,
    /// True when any process or child major-fault delta is nonzero.
    pub major_faults_observed: bool,
}

impl MappedScratchThpFaultSamples {
    /// Returns a conservative comparison summary for complete available samples.
    ///
    /// The comparison is supporting evidence for benchmark interpretation only.
    /// It does not prove transparent huge page adoption.
    #[must_use]
    pub fn comparison(&self) -> Option<MappedScratchThpFaultSampleComparison> {
        let default = self.default.available_fault_deltas()?;
        let hugepage = self.hugepage.available_fault_deltas()?;
        let no_hugepage = self.no_hugepage.available_fault_deltas()?;

        Some(MappedScratchThpFaultSampleComparison {
            default_minor_faults_delta: default.minor,
            hugepage_minor_faults_delta: hugepage.minor,
            no_hugepage_minor_faults_delta: no_hugepage.minor,
            hugepage_vs_default_minor_faults_delta: hugepage.minor.checked_sub(default.minor)?,
            hugepage_vs_no_hugepage_minor_faults_delta: hugepage
                .minor
                .checked_sub(no_hugepage.minor)?,
            major_faults_observed: [default, hugepage, no_hugepage]
                .into_iter()
                .any(MappedScratchThpFaultDeltas::major_faults_observed),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MappedScratchThpFaultDeltas {
    minor: i128,
    major: i128,
    child_major: i128,
}

impl MappedScratchThpFaultDeltas {
    fn major_faults_observed(self) -> bool {
        self.major != 0 || self.child_major != 0
    }
}

impl MappedScratchThpFaultSampleLine {
    fn available_fault_deltas(&self) -> Option<MappedScratchThpFaultDeltas> {
        if self.status != MappedScratchThpFaultSampleStatus::Available {
            return None;
        }

        let _iterations = self.iterations?;
        let _child_minor_faults_delta = self.child_minor_faults_delta?;

        Some(MappedScratchThpFaultDeltas {
            minor: self.minor_faults_delta?,
            major: self.major_faults_delta?,
            child_major: self.child_major_faults_delta?,
        })
    }
}

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

    fn prepare_for_reuse(&mut self) {
        self.offset = 0;
        self.high_water_mark = 0;
        self.allocation_count = 0;
        self.reset_count = 0;
    }
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
    /// Linux transparent huge page advice failed.
    #[cfg(target_os = "linux")]
    LinuxTransparentHugePageAdvice(locus_sys::linux::LinuxTransparentHugePageAdviceError),
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

/// Error returned when extracting mapped scratch THP probe output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MappedScratchThpProbeOutputParseError {
    /// A required stable field is missing.
    MissingField(&'static str),
    /// A stable field appears more than once.
    DuplicateField(&'static str),
    /// A line contains a token outside the stable schema.
    InvalidToken(String),
    /// A line contains an unsupported stable field.
    UnknownField(String),
    /// The run status token is not recognized.
    UnknownRunStatus(String),
    /// The advice mode token is not recognized.
    UnknownMode(String),
    /// The advice status token is not recognized.
    UnknownAdviceStatus(String),
    /// The observation status token is not recognized.
    UnknownObservation(String),
    /// A numeric field is malformed.
    InvalidNumber {
        /// Field name.
        field: &'static str,
        /// Rejected value.
        value: String,
    },
    /// The started mode and advice mode differ.
    ModeMismatch {
        /// Mode reported by `mapped_scratch_thp=started`.
        started: MappedScratchHugePageAdvice,
        /// Mode reported by `thp_advice=`.
        advice: MappedScratchHugePageAdvice,
    },
}

/// Error returned when parsing mapped scratch THP benchmark fault sample lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MappedScratchThpFaultSampleLineParseError {
    /// A required field is missing.
    MissingField(&'static str),
    /// A field appears more than once.
    DuplicateField(&'static str),
    /// A token is outside the stable schema.
    InvalidToken(String),
    /// The field is not recognized.
    UnknownField(String),
    /// The sample mode token is not recognized.
    UnknownMode(String),
    /// The sample status token is not recognized.
    UnknownStatus(String),
    /// A numeric field is malformed.
    InvalidNumber {
        /// Field name.
        field: &'static str,
        /// Rejected value.
        value: String,
    },
    /// An unavailable sample included a field that belongs to available samples.
    UnexpectedField {
        /// Parsed sample status.
        status: MappedScratchThpFaultSampleStatus,
        /// Unexpected field name.
        field: &'static str,
    },
}

/// Error returned when extracting mapped scratch THP benchmark fault samples.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MappedScratchThpFaultSamplesParseError {
    /// The output is missing one required mode sample.
    MissingSample(MappedScratchThpFaultSampleMode),
    /// The output contains more than one sample for a mode.
    DuplicateSample(MappedScratchThpFaultSampleMode),
    /// A stable sample line is malformed.
    Line(MappedScratchThpFaultSampleLineParseError),
}

/// Error returned when parsing pinned scratch pool probe lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PinnedScratchPoolProbeLineParseError {
    /// The line does not contain a supported event token.
    MissingEvent,
    /// The line contains a duplicate event token.
    DuplicateEvent,
    /// The line contains a token outside the stable schema.
    InvalidToken(String),
    /// The event field is not recognized.
    UnknownEvent(String),
    /// The status token is not recognized.
    UnknownStatus(String),
    /// The stats phase token is not recognized.
    UnknownPhase(String),
    /// The field is not recognized.
    UnknownField(String),
    /// The field appears more than once.
    DuplicateField(String),
    /// A required field is missing.
    MissingField(String),
    /// A numeric field is malformed.
    InvalidNumber {
        /// Field name.
        field: String,
        /// Rejected value.
        value: String,
    },
}

/// Error returned when extracting pinned scratch pool probe output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PinnedScratchPoolProbeOutputParseError {
    /// The output does not contain initial stats.
    MissingInitialStats,
    /// The output does not contain a first checkout line.
    MissingCheckoutLine,
    /// The output is missing a required successful event line.
    MissingEvent(PinnedScratchPoolProbeEvent),
    /// The output is missing a required stats phase.
    MissingStats(PinnedScratchPoolProbePhase),
    /// The output contains more than one line for an event.
    DuplicateEvent(PinnedScratchPoolProbeEvent),
    /// The output contains more than one line for a stats phase.
    DuplicateStats(PinnedScratchPoolProbePhase),
    /// A stable event line is malformed.
    EventLine(PinnedScratchPoolProbeLineParseError),
    /// A stable stats line is malformed.
    StatsLine(PinnedScratchPoolProbeLineParseError),
}

/// Error returned when parsing near-GPU pinned scratch pool lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PinnedScratchNearGpuProbeLineParseError {
    /// The line does not contain a `near_gpu_pool=` token.
    MissingStatus,
    /// The line contains more than one status token.
    DuplicateStatus,
    /// The line contains a token outside the stable schema.
    InvalidToken(String),
    /// The status token is not recognized.
    UnknownStatus(String),
    /// The field is not recognized.
    UnknownField(String),
    /// The field appears more than once.
    DuplicateField(String),
    /// A required field is missing.
    MissingField(String),
    /// A numeric field is malformed.
    InvalidNumber {
        /// Field name.
        field: String,
        /// Rejected value.
        value: String,
    },
}

/// Error returned when extracting near-GPU pinned scratch probe output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PinnedScratchNearGpuProbeOutputParseError {
    /// A required top-level field is missing.
    MissingField(&'static str),
    /// A top-level field appears more than once.
    DuplicateField(&'static str),
    /// The output is missing a required successful event line.
    MissingEvent(PinnedScratchPoolProbeEvent),
    /// The output is missing a required stats phase.
    MissingStats(PinnedScratchPoolProbePhase),
    /// The output contains more than one line for an event.
    DuplicateEvent(PinnedScratchPoolProbeEvent),
    /// The output contains more than one line for a stats phase.
    DuplicateStats(PinnedScratchPoolProbePhase),
    /// A top-level numeric field is malformed.
    InvalidNumber {
        /// Field name.
        field: &'static str,
        /// Rejected value.
        value: String,
    },
    /// The constructor status line is malformed.
    PoolLine(PinnedScratchNearGpuProbeLineParseError),
    /// A stable event line is malformed.
    EventLine(PinnedScratchPoolProbeLineParseError),
    /// A stable stats line is malformed.
    StatsLine(PinnedScratchPoolProbeLineParseError),
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

impl fmt::Display for MappedScratchThpProbeOutputParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingField(field) => {
                write!(f, "missing mapped scratch THP field: {field}")
            }
            Self::DuplicateField(field) => {
                write!(f, "duplicate mapped scratch THP field: {field}")
            }
            Self::InvalidToken(token) => {
                write!(f, "invalid mapped scratch THP token: {token}")
            }
            Self::UnknownField(field) => {
                write!(f, "unknown mapped scratch THP field: {field}")
            }
            Self::UnknownRunStatus(status) => {
                write!(f, "unknown mapped scratch THP run status: {status}")
            }
            Self::UnknownMode(mode) => {
                write!(f, "unknown mapped scratch THP mode: {mode}")
            }
            Self::UnknownAdviceStatus(status) => {
                write!(f, "unknown mapped scratch THP advice status: {status}")
            }
            Self::UnknownObservation(observation) => {
                write!(f, "unknown mapped scratch THP observation: {observation}")
            }
            Self::InvalidNumber { field, value } => {
                write!(f, "invalid mapped scratch THP number for {field}: {value}")
            }
            Self::ModeMismatch { started, advice } => write!(
                f,
                "mapped scratch THP mode mismatch: started {started}, advice {advice}"
            ),
        }
    }
}

impl std::error::Error for MappedScratchThpProbeOutputParseError {}

impl fmt::Display for MappedScratchThpFaultSampleLineParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingField(field) => {
                write!(f, "missing mapped scratch THP fault sample field: {field}")
            }
            Self::DuplicateField(field) => {
                write!(
                    f,
                    "duplicate mapped scratch THP fault sample field: {field}"
                )
            }
            Self::InvalidToken(token) => {
                write!(f, "invalid mapped scratch THP fault sample token: {token}")
            }
            Self::UnknownField(field) => {
                write!(f, "unknown mapped scratch THP fault sample field: {field}")
            }
            Self::UnknownMode(mode) => {
                write!(f, "unknown mapped scratch THP fault sample mode: {mode}")
            }
            Self::UnknownStatus(status) => {
                write!(
                    f,
                    "unknown mapped scratch THP fault sample status: {status}"
                )
            }
            Self::InvalidNumber { field, value } => write!(
                f,
                "invalid mapped scratch THP fault sample number for {field}: {value}"
            ),
            Self::UnexpectedField { status, field } => write!(
                f,
                "unexpected mapped scratch THP fault sample field for {status}: {field}"
            ),
        }
    }
}

impl std::error::Error for MappedScratchThpFaultSampleLineParseError {}

impl fmt::Display for MappedScratchThpFaultSamplesParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSample(mode) => {
                write!(f, "missing mapped scratch THP fault sample: {mode}")
            }
            Self::DuplicateSample(mode) => {
                write!(f, "duplicate mapped scratch THP fault sample: {mode}")
            }
            Self::Line(source) => {
                write!(f, "invalid mapped scratch THP fault sample line: {source}")
            }
        }
    }
}

impl std::error::Error for MappedScratchThpFaultSamplesParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Line(source) => Some(source),
            Self::MissingSample(_) | Self::DuplicateSample(_) => None,
        }
    }
}

impl fmt::Display for PinnedScratchPoolProbeLineParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingEvent => f.write_str("missing pinned scratch pool probe event"),
            Self::DuplicateEvent => f.write_str("duplicate pinned scratch pool probe event"),
            Self::InvalidToken(token) => {
                write!(f, "invalid pinned scratch pool probe token: {token}")
            }
            Self::UnknownEvent(event) => {
                write!(f, "unknown pinned scratch pool probe event: {event}")
            }
            Self::UnknownStatus(status) => {
                write!(f, "unknown pinned scratch pool probe status: {status}")
            }
            Self::UnknownPhase(phase) => {
                write!(f, "unknown pinned scratch pool probe phase: {phase}")
            }
            Self::UnknownField(field) => {
                write!(f, "unknown pinned scratch pool probe field: {field}")
            }
            Self::DuplicateField(field) => {
                write!(f, "duplicate pinned scratch pool probe field: {field}")
            }
            Self::MissingField(field) => {
                write!(f, "missing pinned scratch pool probe field: {field}")
            }
            Self::InvalidNumber { field, value } => write!(
                f,
                "invalid pinned scratch pool probe number for {field}: {value}"
            ),
        }
    }
}

impl std::error::Error for PinnedScratchPoolProbeLineParseError {}

impl fmt::Display for PinnedScratchPoolProbeOutputParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingInitialStats => f.write_str("missing initial pinned scratch pool stats"),
            Self::MissingCheckoutLine => f.write_str("missing pool_checkout line"),
            Self::MissingEvent(event) => {
                write!(f, "missing pinned scratch pool event line: {event}")
            }
            Self::MissingStats(phase) => {
                write!(f, "missing pinned scratch pool stats phase: {phase}")
            }
            Self::DuplicateEvent(event) => {
                write!(f, "duplicate pinned scratch pool event line: {event}")
            }
            Self::DuplicateStats(phase) => {
                write!(f, "duplicate pinned scratch pool stats phase: {phase}")
            }
            Self::EventLine(source) => {
                write!(f, "invalid pinned scratch pool event line: {source}")
            }
            Self::StatsLine(source) => {
                write!(f, "invalid pinned scratch pool stats line: {source}")
            }
        }
    }
}

impl std::error::Error for PinnedScratchPoolProbeOutputParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::EventLine(source) | Self::StatsLine(source) => Some(source),
            Self::MissingInitialStats
            | Self::MissingCheckoutLine
            | Self::MissingEvent(_)
            | Self::MissingStats(_)
            | Self::DuplicateEvent(_)
            | Self::DuplicateStats(_) => None,
        }
    }
}

impl fmt::Display for PinnedScratchNearGpuProbeLineParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingStatus => f.write_str("missing near-GPU pinned scratch status"),
            Self::DuplicateStatus => f.write_str("duplicate near-GPU pinned scratch status"),
            Self::InvalidToken(token) => {
                write!(f, "invalid near-GPU pinned scratch token: {token}")
            }
            Self::UnknownStatus(status) => {
                write!(f, "unknown near-GPU pinned scratch status: {status}")
            }
            Self::UnknownField(field) => {
                write!(f, "unknown near-GPU pinned scratch field: {field}")
            }
            Self::DuplicateField(field) => {
                write!(f, "duplicate near-GPU pinned scratch field: {field}")
            }
            Self::MissingField(field) => {
                write!(f, "missing near-GPU pinned scratch field: {field}")
            }
            Self::InvalidNumber { field, value } => write!(
                f,
                "invalid near-GPU pinned scratch number for {field}: {value}"
            ),
        }
    }
}

impl std::error::Error for PinnedScratchNearGpuProbeLineParseError {}

impl fmt::Display for PinnedScratchNearGpuProbeOutputParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingField(field) => {
                write!(f, "missing near-GPU pinned scratch field: {field}")
            }
            Self::DuplicateField(field) => {
                write!(f, "duplicate near-GPU pinned scratch field: {field}")
            }
            Self::MissingEvent(event) => {
                write!(f, "missing near-GPU pinned scratch event line: {event}")
            }
            Self::MissingStats(phase) => {
                write!(f, "missing near-GPU pinned scratch stats phase: {phase}")
            }
            Self::DuplicateEvent(event) => {
                write!(f, "duplicate near-GPU pinned scratch event line: {event}")
            }
            Self::DuplicateStats(phase) => {
                write!(f, "duplicate near-GPU pinned scratch stats phase: {phase}")
            }
            Self::InvalidNumber { field, value } => write!(
                f,
                "invalid near-GPU pinned scratch number for {field}: {value}"
            ),
            Self::PoolLine(source) => {
                write!(f, "invalid near-GPU pinned scratch pool line: {source}")
            }
            Self::EventLine(source) => {
                write!(f, "invalid near-GPU pinned scratch event line: {source}")
            }
            Self::StatsLine(source) => {
                write!(f, "invalid near-GPU pinned scratch stats line: {source}")
            }
        }
    }
}

impl std::error::Error for PinnedScratchNearGpuProbeOutputParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::PoolLine(source) => Some(source),
            Self::EventLine(source) | Self::StatsLine(source) => Some(source),
            Self::MissingField(_)
            | Self::DuplicateField(_)
            | Self::MissingEvent(_)
            | Self::MissingStats(_)
            | Self::DuplicateEvent(_)
            | Self::DuplicateStats(_)
            | Self::InvalidNumber { .. } => None,
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

/// Extracts mapped scratch THP advice and observation fields from multiline output.
///
/// # Errors
///
/// Returns an error when required stable fields are missing, duplicated,
/// malformed, or internally inconsistent.
pub fn parse_mapped_scratch_thp_probe_output(
    output: &str,
) -> Result<MappedScratchThpProbeOutput, MappedScratchThpProbeOutputParseError> {
    let mut fields = MappedScratchThpProbeOutputFields::default();

    for line in output.lines().map(str::trim) {
        if line.starts_with("mapped_scratch_thp=") {
            parse_mapped_scratch_thp_start_line(
                line,
                &mut fields.run_status,
                &mut fields.started_mode,
            )?;
            continue;
        }

        if line.starts_with("thp_advice=") {
            parse_mapped_scratch_thp_advice_line(
                line,
                &mut fields.advice_status,
                &mut fields.advice_mode,
            )?;
            continue;
        }

        if let Some(value) = line.strip_prefix("touched=") {
            set_mapped_scratch_thp_field(
                &mut fields.touched,
                "touched",
                parse_mapped_scratch_thp_usize("touched", value)?,
            )?;
            continue;
        }

        if let Some(value) = line.strip_prefix("kernel_page_kb=") {
            if fields.kernel_page_kb_seen {
                return Err(MappedScratchThpProbeOutputParseError::DuplicateField(
                    "kernel_page_kb",
                ));
            }
            fields.kernel_page_kb_seen = true;
            if value != "unknown" {
                fields.kernel_page_kb =
                    Some(parse_mapped_scratch_thp_usize("kernel_page_kb", value)?);
            }
            continue;
        }

        if line.starts_with("thp_observed=") {
            parse_mapped_scratch_thp_observation_line(
                line,
                &mut fields.observation,
                &mut fields.observation_reason,
            )?;
        }
    }

    fields.finish()
}

/// Parses one mapped scratch THP benchmark fault sample line.
///
/// # Errors
///
/// Returns an error when required fields are missing, duplicated, malformed, or
/// inconsistent with the sample status.
pub fn parse_mapped_scratch_thp_fault_sample_line(
    line: &str,
) -> Result<MappedScratchThpFaultSampleLine, MappedScratchThpFaultSampleLineParseError> {
    let mut fields = MappedScratchThpFaultSampleLineFields::default();

    for token in line.split_whitespace() {
        fields.parse_token(token)?;
    }

    fields.finish()
}

#[derive(Default)]
struct MappedScratchThpFaultSampleLineFields {
    mode: Option<MappedScratchThpFaultSampleMode>,
    status: Option<MappedScratchThpFaultSampleStatus>,
    iterations: Option<usize>,
    minor_faults_delta: Option<i128>,
    child_minor_faults_delta: Option<i128>,
    major_faults_delta: Option<i128>,
    child_major_faults_delta: Option<i128>,
}

impl MappedScratchThpFaultSampleLineFields {
    fn parse_token(
        &mut self,
        token: &str,
    ) -> Result<(), MappedScratchThpFaultSampleLineParseError> {
        let Some((key, value)) = token.split_once('=') else {
            return Err(MappedScratchThpFaultSampleLineParseError::InvalidToken(
                token.to_owned(),
            ));
        };

        match key {
            "fault_sample" => self.parse_mode(value),
            "status" => self.parse_status(value),
            "iterations" => self.parse_iterations(value),
            "minor_faults_delta" => self.parse_i128_field("minor_faults_delta", value),
            "child_minor_faults_delta" => self.parse_i128_field("child_minor_faults_delta", value),
            "major_faults_delta" => self.parse_i128_field("major_faults_delta", value),
            "child_major_faults_delta" => self.parse_i128_field("child_major_faults_delta", value),
            _ => Err(MappedScratchThpFaultSampleLineParseError::UnknownField(
                key.to_owned(),
            )),
        }
    }

    fn finish(
        self,
    ) -> Result<MappedScratchThpFaultSampleLine, MappedScratchThpFaultSampleLineParseError> {
        let mode = self
            .mode
            .ok_or(MappedScratchThpFaultSampleLineParseError::MissingField(
                "fault_sample",
            ))?;
        let status = self
            .status
            .ok_or(MappedScratchThpFaultSampleLineParseError::MissingField(
                "status",
            ))?;

        match status {
            MappedScratchThpFaultSampleStatus::Available => self.finish_available(mode, status),
            MappedScratchThpFaultSampleStatus::Unavailable => self.finish_unavailable(mode, status),
        }
    }

    fn parse_mode(&mut self, value: &str) -> Result<(), MappedScratchThpFaultSampleLineParseError> {
        let parsed = MappedScratchThpFaultSampleMode::from_str_token(value).ok_or_else(|| {
            MappedScratchThpFaultSampleLineParseError::UnknownMode(value.to_owned())
        })?;
        set_mapped_scratch_thp_fault_sample_field(&mut self.mode, "fault_sample", parsed)
    }

    fn parse_status(
        &mut self,
        value: &str,
    ) -> Result<(), MappedScratchThpFaultSampleLineParseError> {
        let parsed = MappedScratchThpFaultSampleStatus::from_str_token(value).ok_or_else(|| {
            MappedScratchThpFaultSampleLineParseError::UnknownStatus(value.to_owned())
        })?;
        set_mapped_scratch_thp_fault_sample_field(&mut self.status, "status", parsed)
    }

    fn parse_iterations(
        &mut self,
        value: &str,
    ) -> Result<(), MappedScratchThpFaultSampleLineParseError> {
        let parsed = parse_mapped_scratch_thp_fault_sample_usize("iterations", value)?;
        set_mapped_scratch_thp_fault_sample_field(&mut self.iterations, "iterations", parsed)
    }

    fn parse_i128_field(
        &mut self,
        field: &'static str,
        value: &str,
    ) -> Result<(), MappedScratchThpFaultSampleLineParseError> {
        let parsed = parse_mapped_scratch_thp_fault_sample_i128(field, value)?;
        match field {
            "minor_faults_delta" => set_mapped_scratch_thp_fault_sample_field(
                &mut self.minor_faults_delta,
                field,
                parsed,
            ),
            "child_minor_faults_delta" => set_mapped_scratch_thp_fault_sample_field(
                &mut self.child_minor_faults_delta,
                field,
                parsed,
            ),
            "major_faults_delta" => set_mapped_scratch_thp_fault_sample_field(
                &mut self.major_faults_delta,
                field,
                parsed,
            ),
            "child_major_faults_delta" => set_mapped_scratch_thp_fault_sample_field(
                &mut self.child_major_faults_delta,
                field,
                parsed,
            ),
            _ => unreachable!("unsupported fault sample field"),
        }
    }

    fn finish_available(
        self,
        mode: MappedScratchThpFaultSampleMode,
        status: MappedScratchThpFaultSampleStatus,
    ) -> Result<MappedScratchThpFaultSampleLine, MappedScratchThpFaultSampleLineParseError> {
        Ok(MappedScratchThpFaultSampleLine {
            mode,
            status,
            iterations: Some(self.iterations.ok_or(
                MappedScratchThpFaultSampleLineParseError::MissingField("iterations"),
            )?),
            minor_faults_delta: Some(self.minor_faults_delta.ok_or(
                MappedScratchThpFaultSampleLineParseError::MissingField("minor_faults_delta"),
            )?),
            child_minor_faults_delta: Some(self.child_minor_faults_delta.ok_or(
                MappedScratchThpFaultSampleLineParseError::MissingField("child_minor_faults_delta"),
            )?),
            major_faults_delta: Some(self.major_faults_delta.ok_or(
                MappedScratchThpFaultSampleLineParseError::MissingField("major_faults_delta"),
            )?),
            child_major_faults_delta: Some(self.child_major_faults_delta.ok_or(
                MappedScratchThpFaultSampleLineParseError::MissingField("child_major_faults_delta"),
            )?),
        })
    }

    fn finish_unavailable(
        self,
        mode: MappedScratchThpFaultSampleMode,
        status: MappedScratchThpFaultSampleStatus,
    ) -> Result<MappedScratchThpFaultSampleLine, MappedScratchThpFaultSampleLineParseError> {
        for (field, present) in [
            ("iterations", self.iterations.is_some()),
            ("minor_faults_delta", self.minor_faults_delta.is_some()),
            (
                "child_minor_faults_delta",
                self.child_minor_faults_delta.is_some(),
            ),
            ("major_faults_delta", self.major_faults_delta.is_some()),
            (
                "child_major_faults_delta",
                self.child_major_faults_delta.is_some(),
            ),
        ] {
            if present {
                return Err(MappedScratchThpFaultSampleLineParseError::UnexpectedField {
                    status,
                    field,
                });
            }
        }

        Ok(MappedScratchThpFaultSampleLine {
            mode,
            status,
            iterations: None,
            minor_faults_delta: None,
            child_minor_faults_delta: None,
            major_faults_delta: None,
            child_major_faults_delta: None,
        })
    }
}

/// Extracts mapped scratch THP benchmark fault samples from multiline output.
///
/// # Errors
///
/// Returns an error when any sample line is malformed, a mode is missing, or a
/// mode appears more than once.
pub fn parse_mapped_scratch_thp_fault_samples_output(
    output: &str,
) -> Result<MappedScratchThpFaultSamples, MappedScratchThpFaultSamplesParseError> {
    let mut default = None;
    let mut hugepage = None;
    let mut no_hugepage = None;

    for line in output.lines().map(str::trim) {
        if !line.starts_with("fault_sample=") {
            continue;
        }

        let parsed = parse_mapped_scratch_thp_fault_sample_line(line)
            .map_err(MappedScratchThpFaultSamplesParseError::Line)?;
        match parsed.mode {
            MappedScratchThpFaultSampleMode::Default => {
                set_mapped_scratch_thp_fault_sample_output(
                    &mut default,
                    MappedScratchThpFaultSampleMode::Default,
                    parsed,
                )?;
            }
            MappedScratchThpFaultSampleMode::HugePage => {
                set_mapped_scratch_thp_fault_sample_output(
                    &mut hugepage,
                    MappedScratchThpFaultSampleMode::HugePage,
                    parsed,
                )?;
            }
            MappedScratchThpFaultSampleMode::NoHugePage => {
                set_mapped_scratch_thp_fault_sample_output(
                    &mut no_hugepage,
                    MappedScratchThpFaultSampleMode::NoHugePage,
                    parsed,
                )?;
            }
        }
    }

    Ok(MappedScratchThpFaultSamples {
        default: default.ok_or(MappedScratchThpFaultSamplesParseError::MissingSample(
            MappedScratchThpFaultSampleMode::Default,
        ))?,
        hugepage: hugepage.ok_or(MappedScratchThpFaultSamplesParseError::MissingSample(
            MappedScratchThpFaultSampleMode::HugePage,
        ))?,
        no_hugepage: no_hugepage.ok_or(MappedScratchThpFaultSamplesParseError::MissingSample(
            MappedScratchThpFaultSampleMode::NoHugePage,
        ))?,
    })
}

#[derive(Default)]
struct MappedScratchThpProbeOutputFields {
    run_status: Option<MappedScratchThpProbeRunStatus>,
    started_mode: Option<MappedScratchHugePageAdvice>,
    advice_mode: Option<MappedScratchHugePageAdvice>,
    advice_status: Option<MappedScratchThpAdviceStatus>,
    touched: Option<usize>,
    kernel_page_kb: Option<usize>,
    kernel_page_kb_seen: bool,
    observation: Option<MappedScratchThpObservation>,
    observation_reason: Option<String>,
}

impl MappedScratchThpProbeOutputFields {
    fn finish(self) -> Result<MappedScratchThpProbeOutput, MappedScratchThpProbeOutputParseError> {
        let run_status =
            self.run_status
                .ok_or(MappedScratchThpProbeOutputParseError::MissingField(
                    "mapped_scratch_thp",
                ))?;

        if run_status == MappedScratchThpProbeRunStatus::UnsupportedPlatform {
            return Ok(MappedScratchThpProbeOutput {
                run_status,
                mode: None,
                advice_status: None,
                touched: None,
                kernel_page_kb: None,
                observation: None,
                observation_reason: None,
            });
        }

        let started_mode = self
            .started_mode
            .ok_or(MappedScratchThpProbeOutputParseError::MissingField("mode"))?;
        let advice_mode = self
            .advice_mode
            .ok_or(MappedScratchThpProbeOutputParseError::MissingField("mode"))?;
        if started_mode != advice_mode {
            return Err(MappedScratchThpProbeOutputParseError::ModeMismatch {
                started: started_mode,
                advice: advice_mode,
            });
        }

        let advice_status =
            self.advice_status
                .ok_or(MappedScratchThpProbeOutputParseError::MissingField(
                    "thp_advice",
                ))?;

        if advice_status == MappedScratchThpAdviceStatus::Ok {
            if self.touched.is_none() {
                return Err(MappedScratchThpProbeOutputParseError::MissingField(
                    "touched",
                ));
            }
            if self.observation.is_none() {
                return Err(MappedScratchThpProbeOutputParseError::MissingField(
                    "thp_observed",
                ));
            }
            if self.observation_reason.is_none() {
                return Err(MappedScratchThpProbeOutputParseError::MissingField(
                    "reason",
                ));
            }
        }

        Ok(MappedScratchThpProbeOutput {
            run_status,
            mode: Some(started_mode),
            advice_status: Some(advice_status),
            touched: self.touched,
            kernel_page_kb: self.kernel_page_kb,
            observation: self.observation,
            observation_reason: self.observation_reason,
        })
    }
}

/// Parses one pinned scratch pool probe event line.
///
/// # Errors
///
/// Returns an error when the line is missing an event token, contains duplicate
/// fields, contains unsupported tokens, uses an unknown status, or omits a
/// required handle or byte count.
pub fn parse_pinned_scratch_pool_probe_event_line(
    line: &str,
) -> Result<PinnedScratchPoolProbeEventLine, PinnedScratchPoolProbeLineParseError> {
    let mut event = None;
    let mut status_token = None;
    let mut handle = None;
    let mut bytes = None;

    for token in line.split_whitespace() {
        let Some((key, value)) = token.split_once('=') else {
            return Err(PinnedScratchPoolProbeLineParseError::InvalidToken(
                token.to_owned(),
            ));
        };

        if let Some(parsed_event) = PinnedScratchPoolProbeEvent::from_str_token(key) {
            if event.replace(parsed_event).is_some() {
                return Err(PinnedScratchPoolProbeLineParseError::DuplicateEvent);
            }
            if status_token.replace(value).is_some() {
                return Err(PinnedScratchPoolProbeLineParseError::DuplicateEvent);
            }
            continue;
        }

        match key {
            "handle" => {
                if handle.replace(parse_probe_u64_field(key, value)?).is_some() {
                    return Err(PinnedScratchPoolProbeLineParseError::DuplicateField(
                        key.to_owned(),
                    ));
                }
            }
            "bytes" => {
                if bytes
                    .replace(parse_probe_usize_field(key, value)?)
                    .is_some()
                {
                    return Err(PinnedScratchPoolProbeLineParseError::DuplicateField(
                        key.to_owned(),
                    ));
                }
            }
            _ => {
                return Err(PinnedScratchPoolProbeLineParseError::UnknownField(
                    key.to_owned(),
                ));
            }
        }
    }

    let event = event.ok_or(PinnedScratchPoolProbeLineParseError::MissingEvent)?;
    let status_token = status_token.ok_or(PinnedScratchPoolProbeLineParseError::MissingEvent)?;
    let status = PinnedScratchPoolProbeStatus::from_str_token(status_token).ok_or_else(|| {
        PinnedScratchPoolProbeLineParseError::UnknownStatus(status_token.to_owned())
    })?;

    if event == PinnedScratchPoolProbeEvent::Allocation {
        if handle.is_some() {
            return Err(PinnedScratchPoolProbeLineParseError::UnknownField(
                "handle".to_owned(),
            ));
        }
        if status == PinnedScratchPoolProbeStatus::Ok && bytes.is_none() {
            return Err(PinnedScratchPoolProbeLineParseError::MissingField(
                "bytes".to_owned(),
            ));
        }
    } else {
        if bytes.is_some() {
            return Err(PinnedScratchPoolProbeLineParseError::UnknownField(
                "bytes".to_owned(),
            ));
        }
        if status == PinnedScratchPoolProbeStatus::Ok && event.requires_handle() && handle.is_none()
        {
            return Err(PinnedScratchPoolProbeLineParseError::MissingField(
                "handle".to_owned(),
            ));
        }
    }

    Ok(PinnedScratchPoolProbeEventLine {
        event,
        status,
        handle,
        bytes,
    })
}

/// Parses one pinned scratch pool probe stats line.
///
/// # Errors
///
/// Returns an error when the line is not a stats line, contains duplicate or
/// unknown fields, has an unknown phase, or contains malformed numeric values.
pub fn parse_pinned_scratch_pool_probe_stats_line(
    line: &str,
) -> Result<PinnedScratchPoolProbeStatsLine, PinnedScratchPoolProbeLineParseError> {
    let mut tokens = line.split_whitespace();
    match tokens.next() {
        Some("pool_stats") => {}
        Some(token) => {
            return Err(PinnedScratchPoolProbeLineParseError::InvalidToken(
                token.to_owned(),
            ));
        }
        None => {
            return Err(PinnedScratchPoolProbeLineParseError::InvalidToken(
                String::new(),
            ))
        }
    }

    let mut phase = None;
    let mut locked_bytes = None;
    let mut checked_out = None;
    let mut idle = None;
    let mut created_arenas = None;
    let mut reused_arenas = None;
    let mut checkout_count = None;
    let mut release_count = None;

    for token in tokens {
        let Some((key, value)) = token.split_once('=') else {
            return Err(PinnedScratchPoolProbeLineParseError::InvalidToken(
                token.to_owned(),
            ));
        };

        match key {
            "phase" => {
                let parsed =
                    PinnedScratchPoolProbePhase::from_str_token(value).ok_or_else(|| {
                        PinnedScratchPoolProbeLineParseError::UnknownPhase(value.to_owned())
                    })?;
                set_probe_field(&mut phase, key, parsed)?;
            }
            "locked_bytes" => {
                let parsed = parse_probe_usize_field(key, value)?;
                set_probe_field(&mut locked_bytes, key, parsed)?;
            }
            "checked_out" => {
                let parsed = parse_probe_usize_field(key, value)?;
                set_probe_field(&mut checked_out, key, parsed)?;
            }
            "idle" => {
                let parsed = parse_probe_usize_field(key, value)?;
                set_probe_field(&mut idle, key, parsed)?;
            }
            "created_arenas" => {
                let parsed = parse_probe_u64_field(key, value)?;
                set_probe_field(&mut created_arenas, key, parsed)?;
            }
            "reused_arenas" => {
                let parsed = parse_probe_u64_field(key, value)?;
                set_probe_field(&mut reused_arenas, key, parsed)?;
            }
            "checkout_count" => {
                let parsed = parse_probe_u64_field(key, value)?;
                set_probe_field(&mut checkout_count, key, parsed)?;
            }
            "release_count" => {
                let parsed = parse_probe_u64_field(key, value)?;
                set_probe_field(&mut release_count, key, parsed)?;
            }
            _ => {
                return Err(PinnedScratchPoolProbeLineParseError::UnknownField(
                    key.to_owned(),
                ));
            }
        }
    }

    Ok(PinnedScratchPoolProbeStatsLine {
        phase: require_probe_field(phase, "phase")?,
        locked_bytes: require_probe_field(locked_bytes, "locked_bytes")?,
        checked_out: require_probe_field(checked_out, "checked_out")?,
        idle: require_probe_field(idle, "idle")?,
        created_arenas: require_probe_field(created_arenas, "created_arenas")?,
        reused_arenas: require_probe_field(reused_arenas, "reused_arenas")?,
        checkout_count: require_probe_field(checkout_count, "checkout_count")?,
        release_count: require_probe_field(release_count, "release_count")?,
    })
}

/// Extracts pinned scratch pool probe events and stats from multiline output.
///
/// # Errors
///
/// Returns an error when required stable lines are missing, duplicated, or
/// malformed.
pub fn parse_pinned_scratch_pool_probe_output(
    output: &str,
) -> Result<PinnedScratchPoolProbeOutput, PinnedScratchPoolProbeOutputParseError> {
    let mut initial_stats = None;
    let mut checkout_error_stats = None;
    let mut after_checkout_stats = None;
    let mut after_release_stats = None;
    let mut after_reuse_checkout_stats = None;
    let mut after_reuse_release_stats = None;

    let mut checkout = None;
    let mut allocation = None;
    let mut release = None;
    let mut reuse_checkout = None;
    let mut reuse_release = None;

    for line in output.lines().map(str::trim) {
        if is_pinned_scratch_pool_probe_event_line(line) {
            let parsed = parse_pinned_scratch_pool_probe_event_line(line)
                .map_err(PinnedScratchPoolProbeOutputParseError::EventLine)?;
            match parsed.event {
                PinnedScratchPoolProbeEvent::Checkout => {
                    set_probe_output_event(&mut checkout, parsed)?;
                }
                PinnedScratchPoolProbeEvent::Allocation => {
                    set_probe_output_event(&mut allocation, parsed)?;
                }
                PinnedScratchPoolProbeEvent::Release => {
                    set_probe_output_event(&mut release, parsed)?;
                }
                PinnedScratchPoolProbeEvent::ReuseCheckout => {
                    set_probe_output_event(&mut reuse_checkout, parsed)?;
                }
                PinnedScratchPoolProbeEvent::ReuseRelease => {
                    set_probe_output_event(&mut reuse_release, parsed)?;
                }
            }
            continue;
        }

        if is_pinned_scratch_pool_probe_stats_line(line) {
            let parsed = parse_pinned_scratch_pool_probe_stats_line(line)
                .map_err(PinnedScratchPoolProbeOutputParseError::StatsLine)?;
            match parsed.phase {
                PinnedScratchPoolProbePhase::Initial => {
                    set_probe_output_stats(&mut initial_stats, parsed)?;
                }
                PinnedScratchPoolProbePhase::CheckoutError => {
                    set_probe_output_stats(&mut checkout_error_stats, parsed)?;
                }
                PinnedScratchPoolProbePhase::AfterCheckout => {
                    set_probe_output_stats(&mut after_checkout_stats, parsed)?;
                }
                PinnedScratchPoolProbePhase::AfterRelease => {
                    set_probe_output_stats(&mut after_release_stats, parsed)?;
                }
                PinnedScratchPoolProbePhase::AfterReuseCheckout => {
                    set_probe_output_stats(&mut after_reuse_checkout_stats, parsed)?;
                }
                PinnedScratchPoolProbePhase::AfterReuseRelease => {
                    set_probe_output_stats(&mut after_reuse_release_stats, parsed)?;
                }
            }
        }
    }

    let initial_stats =
        initial_stats.ok_or(PinnedScratchPoolProbeOutputParseError::MissingInitialStats)?;
    let checkout = checkout.ok_or(PinnedScratchPoolProbeOutputParseError::MissingCheckoutLine)?;

    if checkout.status == PinnedScratchPoolProbeStatus::Ok {
        require_probe_output_event(allocation, PinnedScratchPoolProbeEvent::Allocation)?;
        require_probe_output_event(release, PinnedScratchPoolProbeEvent::Release)?;
        require_probe_output_event(reuse_checkout, PinnedScratchPoolProbeEvent::ReuseCheckout)?;
        require_probe_output_event(reuse_release, PinnedScratchPoolProbeEvent::ReuseRelease)?;
        require_probe_output_stats(
            after_checkout_stats,
            PinnedScratchPoolProbePhase::AfterCheckout,
        )?;
        require_probe_output_stats(
            after_release_stats,
            PinnedScratchPoolProbePhase::AfterRelease,
        )?;
        require_probe_output_stats(
            after_reuse_checkout_stats,
            PinnedScratchPoolProbePhase::AfterReuseCheckout,
        )?;
        require_probe_output_stats(
            after_reuse_release_stats,
            PinnedScratchPoolProbePhase::AfterReuseRelease,
        )?;
    }

    Ok(PinnedScratchPoolProbeOutput {
        initial_stats,
        checkout,
        checkout_error_stats,
        allocation,
        release,
        reuse_checkout,
        reuse_release,
        after_checkout_stats,
        after_release_stats,
        after_reuse_checkout_stats,
        after_reuse_release_stats,
    })
}

/// Parses one near-GPU pinned scratch constructor line.
///
/// # Errors
///
/// Returns an error when the line is missing the constructor status, has an
/// unknown status, contains duplicate or unknown fields, or omits fields
/// required by the status.
pub fn parse_pinned_scratch_near_gpu_probe_pool_line(
    line: &str,
) -> Result<PinnedScratchNearGpuPoolLine, PinnedScratchNearGpuProbeLineParseError> {
    let mut status_token = None;
    let mut home_node = None;
    let mut reason = None;

    for token in line.split_whitespace() {
        let Some((key, value)) = token.split_once('=') else {
            return Err(PinnedScratchNearGpuProbeLineParseError::InvalidToken(
                token.to_owned(),
            ));
        };

        match key {
            "near_gpu_pool" => {
                if status_token.replace(value).is_some() {
                    return Err(PinnedScratchNearGpuProbeLineParseError::DuplicateStatus);
                }
            }
            "home_node" => {
                if home_node
                    .replace(parse_near_gpu_node_id_field(key, value)?)
                    .is_some()
                {
                    return Err(PinnedScratchNearGpuProbeLineParseError::DuplicateField(
                        key.to_owned(),
                    ));
                }
            }
            "reason" => {
                if reason.replace(value.to_owned()).is_some() {
                    return Err(PinnedScratchNearGpuProbeLineParseError::DuplicateField(
                        key.to_owned(),
                    ));
                }
            }
            _ => {
                return Err(PinnedScratchNearGpuProbeLineParseError::UnknownField(
                    key.to_owned(),
                ));
            }
        }
    }

    let status_token =
        status_token.ok_or(PinnedScratchNearGpuProbeLineParseError::MissingStatus)?;
    let status =
        PinnedScratchNearGpuProbeStatus::from_str_token(status_token).ok_or_else(|| {
            PinnedScratchNearGpuProbeLineParseError::UnknownStatus(status_token.to_owned())
        })?;

    match status {
        PinnedScratchNearGpuProbeStatus::Ok => {
            if reason.is_some() {
                return Err(PinnedScratchNearGpuProbeLineParseError::UnknownField(
                    "reason".to_owned(),
                ));
            }
            require_near_gpu_probe_field(home_node, "home_node")?;
        }
        PinnedScratchNearGpuProbeStatus::Unavailable => {
            if home_node.is_some() {
                return Err(PinnedScratchNearGpuProbeLineParseError::UnknownField(
                    "home_node".to_owned(),
                ));
            }
            require_near_gpu_probe_field(reason.clone(), "reason")?;
        }
        PinnedScratchNearGpuProbeStatus::Error => {
            if home_node.is_some() {
                return Err(PinnedScratchNearGpuProbeLineParseError::UnknownField(
                    "home_node".to_owned(),
                ));
            }
            if reason.is_some() {
                return Err(PinnedScratchNearGpuProbeLineParseError::UnknownField(
                    "reason".to_owned(),
                ));
            }
        }
    }

    Ok(PinnedScratchNearGpuPoolLine {
        status,
        home_node,
        reason,
    })
}

/// Extracts near-GPU pinned scratch probe fields from multiline output.
///
/// # Errors
///
/// Returns an error when required stable lines are missing, duplicated, or
/// malformed.
pub fn parse_pinned_scratch_near_gpu_probe_output(
    output: &str,
) -> Result<PinnedScratchNearGpuProbeOutput, PinnedScratchNearGpuProbeOutputParseError> {
    let mut fields = PinnedScratchNearGpuProbeOutputFields::default();

    for line in output.lines().map(str::trim) {
        if line.is_empty() {
            continue;
        }
        parse_pinned_scratch_near_gpu_probe_output_line(line, &mut fields)?;
    }

    fields.finish()
}

#[derive(Default)]
struct PinnedScratchNearGpuProbeOutputFields {
    topology_nodes: Option<usize>,
    topology_pci_devices: Option<usize>,
    gpu_bdf: Option<String>,
    arena_capacity: Option<usize>,
    max_locked_bytes: Option<usize>,
    pool: Option<PinnedScratchNearGpuPoolLine>,
    initial_stats: Option<PinnedScratchPoolProbeStatsLine>,
    checkout_error_stats: Option<PinnedScratchPoolProbeStatsLine>,
    after_checkout_stats: Option<PinnedScratchPoolProbeStatsLine>,
    after_release_stats: Option<PinnedScratchPoolProbeStatsLine>,
    checkout: Option<PinnedScratchPoolProbeEventLine>,
    allocation: Option<PinnedScratchPoolProbeEventLine>,
    release: Option<PinnedScratchPoolProbeEventLine>,
}

impl PinnedScratchNearGpuProbeOutputFields {
    fn finish(
        self,
    ) -> Result<PinnedScratchNearGpuProbeOutput, PinnedScratchNearGpuProbeOutputParseError> {
        let topology_nodes =
            self.topology_nodes
                .ok_or(PinnedScratchNearGpuProbeOutputParseError::MissingField(
                    "topology_nodes",
                ))?;
        let topology_pci_devices = self.topology_pci_devices.ok_or(
            PinnedScratchNearGpuProbeOutputParseError::MissingField("topology_pci_devices"),
        )?;
        let pool = self
            .pool
            .ok_or(PinnedScratchNearGpuProbeOutputParseError::MissingField(
                "near_gpu_pool",
            ))?;

        if pool.status == PinnedScratchNearGpuProbeStatus::Ok {
            require_near_gpu_output_field(self.gpu_bdf.as_ref(), "gpu_bdf")?;
            require_near_gpu_output_field(self.arena_capacity.as_ref(), "arena_capacity")?;
            require_near_gpu_output_field(self.max_locked_bytes.as_ref(), "max_locked_bytes")?;
            require_near_gpu_output_stats(
                self.initial_stats,
                PinnedScratchPoolProbePhase::Initial,
            )?;
            let checkout = require_near_gpu_output_event(
                self.checkout,
                PinnedScratchPoolProbeEvent::Checkout,
            )?;

            if checkout.status == PinnedScratchPoolProbeStatus::Ok {
                require_near_gpu_output_event(
                    self.allocation,
                    PinnedScratchPoolProbeEvent::Allocation,
                )?;
                require_near_gpu_output_event(self.release, PinnedScratchPoolProbeEvent::Release)?;
                require_near_gpu_output_stats(
                    self.after_checkout_stats,
                    PinnedScratchPoolProbePhase::AfterCheckout,
                )?;
                require_near_gpu_output_stats(
                    self.after_release_stats,
                    PinnedScratchPoolProbePhase::AfterRelease,
                )?;
            }
        }

        Ok(PinnedScratchNearGpuProbeOutput {
            topology_nodes,
            topology_pci_devices,
            gpu_bdf: self.gpu_bdf,
            arena_capacity: self.arena_capacity,
            max_locked_bytes: self.max_locked_bytes,
            pool,
            initial_stats: self.initial_stats,
            checkout: self.checkout,
            checkout_error_stats: self.checkout_error_stats,
            allocation: self.allocation,
            release: self.release,
            after_checkout_stats: self.after_checkout_stats,
            after_release_stats: self.after_release_stats,
        })
    }
}

fn parse_pinned_scratch_near_gpu_probe_output_line(
    line: &str,
    fields: &mut PinnedScratchNearGpuProbeOutputFields,
) -> Result<(), PinnedScratchNearGpuProbeOutputParseError> {
    if let Some(value) = line.strip_prefix("topology_nodes=") {
        let parsed = parse_near_gpu_output_usize_field("topology_nodes", value)?;
        return set_near_gpu_output_field(&mut fields.topology_nodes, "topology_nodes", parsed);
    }

    if let Some(value) = line.strip_prefix("topology_pci_devices=") {
        let parsed = parse_near_gpu_output_usize_field("topology_pci_devices", value)?;
        return set_near_gpu_output_field(
            &mut fields.topology_pci_devices,
            "topology_pci_devices",
            parsed,
        );
    }

    if let Some(value) = line.strip_prefix("gpu_bdf=") {
        return set_near_gpu_output_field(&mut fields.gpu_bdf, "gpu_bdf", value.to_owned());
    }

    if let Some(value) = line.strip_prefix("arena_capacity=") {
        let parsed = parse_near_gpu_output_usize_field("arena_capacity", value)?;
        return set_near_gpu_output_field(&mut fields.arena_capacity, "arena_capacity", parsed);
    }

    if let Some(value) = line.strip_prefix("max_locked_bytes=") {
        let parsed = parse_near_gpu_output_usize_field("max_locked_bytes", value)?;
        return set_near_gpu_output_field(&mut fields.max_locked_bytes, "max_locked_bytes", parsed);
    }

    if line.starts_with("near_gpu_pool=") {
        let parsed = parse_pinned_scratch_near_gpu_probe_pool_line(line)
            .map_err(PinnedScratchNearGpuProbeOutputParseError::PoolLine)?;
        if fields.pool.replace(parsed).is_some() {
            return Err(PinnedScratchNearGpuProbeOutputParseError::DuplicateField(
                "near_gpu_pool",
            ));
        }
        return Ok(());
    }

    if is_pinned_scratch_pool_probe_event_line(line) {
        let parsed = parse_pinned_scratch_pool_probe_event_line(line)
            .map_err(PinnedScratchNearGpuProbeOutputParseError::EventLine)?;
        match parsed.event {
            PinnedScratchPoolProbeEvent::Checkout => {
                set_near_gpu_output_event(&mut fields.checkout, parsed)?;
            }
            PinnedScratchPoolProbeEvent::Allocation => {
                set_near_gpu_output_event(&mut fields.allocation, parsed)?;
            }
            PinnedScratchPoolProbeEvent::Release => {
                set_near_gpu_output_event(&mut fields.release, parsed)?;
            }
            PinnedScratchPoolProbeEvent::ReuseCheckout
            | PinnedScratchPoolProbeEvent::ReuseRelease => {}
        }
        return Ok(());
    }

    if is_pinned_scratch_pool_probe_stats_line(line) {
        let parsed = parse_pinned_scratch_pool_probe_stats_line(line)
            .map_err(PinnedScratchNearGpuProbeOutputParseError::StatsLine)?;
        match parsed.phase {
            PinnedScratchPoolProbePhase::Initial => {
                set_near_gpu_output_stats(&mut fields.initial_stats, parsed)?;
            }
            PinnedScratchPoolProbePhase::CheckoutError => {
                set_near_gpu_output_stats(&mut fields.checkout_error_stats, parsed)?;
            }
            PinnedScratchPoolProbePhase::AfterCheckout => {
                set_near_gpu_output_stats(&mut fields.after_checkout_stats, parsed)?;
            }
            PinnedScratchPoolProbePhase::AfterRelease => {
                set_near_gpu_output_stats(&mut fields.after_release_stats, parsed)?;
            }
            PinnedScratchPoolProbePhase::AfterReuseCheckout
            | PinnedScratchPoolProbePhase::AfterReuseRelease => {}
        }
    }

    Ok(())
}

fn parse_mapped_scratch_thp_start_line(
    line: &str,
    run_status: &mut Option<MappedScratchThpProbeRunStatus>,
    mode: &mut Option<MappedScratchHugePageAdvice>,
) -> Result<(), MappedScratchThpProbeOutputParseError> {
    let mut line_status = None;
    let mut line_mode = None;

    for token in line.split_whitespace() {
        let Some((key, value)) = token.split_once('=') else {
            return Err(MappedScratchThpProbeOutputParseError::InvalidToken(
                token.to_owned(),
            ));
        };

        match key {
            "mapped_scratch_thp" => {
                let parsed =
                    MappedScratchThpProbeRunStatus::from_str_token(value).ok_or_else(|| {
                        MappedScratchThpProbeOutputParseError::UnknownRunStatus(value.to_owned())
                    })?;
                set_mapped_scratch_thp_field(&mut line_status, "mapped_scratch_thp", parsed)?;
            }
            "mode" => {
                let parsed =
                    MappedScratchHugePageAdvice::from_str_token(value).ok_or_else(|| {
                        MappedScratchThpProbeOutputParseError::UnknownMode(value.to_owned())
                    })?;
                set_mapped_scratch_thp_field(&mut line_mode, "mode", parsed)?;
            }
            _ => {
                return Err(MappedScratchThpProbeOutputParseError::UnknownField(
                    key.to_owned(),
                ));
            }
        }
    }

    let line_status = line_status.ok_or(MappedScratchThpProbeOutputParseError::MissingField(
        "mapped_scratch_thp",
    ))?;
    set_mapped_scratch_thp_field(run_status, "mapped_scratch_thp", line_status)?;

    if line_status == MappedScratchThpProbeRunStatus::Started {
        let line_mode =
            line_mode.ok_or(MappedScratchThpProbeOutputParseError::MissingField("mode"))?;
        set_mapped_scratch_thp_field(mode, "mode", line_mode)?;
    } else if line_mode.is_some() {
        return Err(MappedScratchThpProbeOutputParseError::UnknownField(
            "mode".to_owned(),
        ));
    }

    Ok(())
}

fn parse_mapped_scratch_thp_advice_line(
    line: &str,
    advice_status: &mut Option<MappedScratchThpAdviceStatus>,
    mode: &mut Option<MappedScratchHugePageAdvice>,
) -> Result<(), MappedScratchThpProbeOutputParseError> {
    let mut line_status = None;
    let mut line_mode = None;

    for token in line.split_whitespace() {
        let Some((key, value)) = token.split_once('=') else {
            return Err(MappedScratchThpProbeOutputParseError::InvalidToken(
                token.to_owned(),
            ));
        };

        match key {
            "thp_advice" => {
                let parsed =
                    MappedScratchThpAdviceStatus::from_str_token(value).ok_or_else(|| {
                        MappedScratchThpProbeOutputParseError::UnknownAdviceStatus(value.to_owned())
                    })?;
                set_mapped_scratch_thp_field(&mut line_status, "thp_advice", parsed)?;
            }
            "mode" => {
                let parsed =
                    MappedScratchHugePageAdvice::from_str_token(value).ok_or_else(|| {
                        MappedScratchThpProbeOutputParseError::UnknownMode(value.to_owned())
                    })?;
                set_mapped_scratch_thp_field(&mut line_mode, "mode", parsed)?;
            }
            _ => {
                return Err(MappedScratchThpProbeOutputParseError::UnknownField(
                    key.to_owned(),
                ));
            }
        }
    }

    let line_status = line_status.ok_or(MappedScratchThpProbeOutputParseError::MissingField(
        "thp_advice",
    ))?;
    let line_mode = line_mode.ok_or(MappedScratchThpProbeOutputParseError::MissingField("mode"))?;

    set_mapped_scratch_thp_field(advice_status, "thp_advice", line_status)?;
    set_mapped_scratch_thp_field(mode, "mode", line_mode)?;

    Ok(())
}

fn parse_mapped_scratch_thp_observation_line(
    line: &str,
    observation: &mut Option<MappedScratchThpObservation>,
    reason: &mut Option<String>,
) -> Result<(), MappedScratchThpProbeOutputParseError> {
    let mut line_observation = None;
    let mut line_reason = None;

    for token in line.split_whitespace() {
        let Some((key, value)) = token.split_once('=') else {
            return Err(MappedScratchThpProbeOutputParseError::InvalidToken(
                token.to_owned(),
            ));
        };

        match key {
            "thp_observed" => {
                let parsed =
                    MappedScratchThpObservation::from_str_token(value).ok_or_else(|| {
                        MappedScratchThpProbeOutputParseError::UnknownObservation(value.to_owned())
                    })?;
                set_mapped_scratch_thp_field(&mut line_observation, "thp_observed", parsed)?;
            }
            "reason" => {
                set_mapped_scratch_thp_field(&mut line_reason, "reason", value.to_owned())?;
            }
            _ => {
                return Err(MappedScratchThpProbeOutputParseError::UnknownField(
                    key.to_owned(),
                ));
            }
        }
    }

    let line_observation = line_observation.ok_or(
        MappedScratchThpProbeOutputParseError::MissingField("thp_observed"),
    )?;
    let line_reason = line_reason.ok_or(MappedScratchThpProbeOutputParseError::MissingField(
        "reason",
    ))?;

    set_mapped_scratch_thp_field(observation, "thp_observed", line_observation)?;
    set_mapped_scratch_thp_field(reason, "reason", line_reason)?;

    Ok(())
}

fn parse_mapped_scratch_thp_usize(
    field: &'static str,
    value: &str,
) -> Result<usize, MappedScratchThpProbeOutputParseError> {
    value
        .parse()
        .map_err(|_| MappedScratchThpProbeOutputParseError::InvalidNumber {
            field,
            value: value.to_owned(),
        })
}

fn set_mapped_scratch_thp_field<T>(
    slot: &mut Option<T>,
    field: &'static str,
    value: T,
) -> Result<(), MappedScratchThpProbeOutputParseError> {
    if slot.replace(value).is_some() {
        return Err(MappedScratchThpProbeOutputParseError::DuplicateField(field));
    }
    Ok(())
}

fn parse_mapped_scratch_thp_fault_sample_usize(
    field: &'static str,
    value: &str,
) -> Result<usize, MappedScratchThpFaultSampleLineParseError> {
    value.parse().map_err(
        |_| MappedScratchThpFaultSampleLineParseError::InvalidNumber {
            field,
            value: value.to_owned(),
        },
    )
}

fn parse_mapped_scratch_thp_fault_sample_i128(
    field: &'static str,
    value: &str,
) -> Result<i128, MappedScratchThpFaultSampleLineParseError> {
    value.parse().map_err(
        |_| MappedScratchThpFaultSampleLineParseError::InvalidNumber {
            field,
            value: value.to_owned(),
        },
    )
}

fn set_mapped_scratch_thp_fault_sample_field<T>(
    slot: &mut Option<T>,
    field: &'static str,
    value: T,
) -> Result<(), MappedScratchThpFaultSampleLineParseError> {
    if slot.replace(value).is_some() {
        return Err(MappedScratchThpFaultSampleLineParseError::DuplicateField(
            field,
        ));
    }
    Ok(())
}

fn set_mapped_scratch_thp_fault_sample_output(
    slot: &mut Option<MappedScratchThpFaultSampleLine>,
    mode: MappedScratchThpFaultSampleMode,
    value: MappedScratchThpFaultSampleLine,
) -> Result<(), MappedScratchThpFaultSamplesParseError> {
    if slot.replace(value).is_some() {
        return Err(MappedScratchThpFaultSamplesParseError::DuplicateSample(
            mode,
        ));
    }
    Ok(())
}

fn parse_probe_usize_field(
    field: &str,
    value: &str,
) -> Result<usize, PinnedScratchPoolProbeLineParseError> {
    value
        .parse()
        .map_err(|_| PinnedScratchPoolProbeLineParseError::InvalidNumber {
            field: field.to_owned(),
            value: value.to_owned(),
        })
}

fn parse_probe_u64_field(
    field: &str,
    value: &str,
) -> Result<u64, PinnedScratchPoolProbeLineParseError> {
    value
        .parse()
        .map_err(|_| PinnedScratchPoolProbeLineParseError::InvalidNumber {
            field: field.to_owned(),
            value: value.to_owned(),
        })
}

fn parse_near_gpu_node_id_field(
    field: &str,
    value: &str,
) -> Result<NodeId, PinnedScratchNearGpuProbeLineParseError> {
    value
        .parse()
        .map(NodeId)
        .map_err(|_| PinnedScratchNearGpuProbeLineParseError::InvalidNumber {
            field: field.to_owned(),
            value: value.to_owned(),
        })
}

fn parse_near_gpu_output_usize_field(
    field: &'static str,
    value: &str,
) -> Result<usize, PinnedScratchNearGpuProbeOutputParseError> {
    value.parse().map_err(
        |_| PinnedScratchNearGpuProbeOutputParseError::InvalidNumber {
            field,
            value: value.to_owned(),
        },
    )
}

fn set_probe_field<T>(
    slot: &mut Option<T>,
    field: &str,
    value: T,
) -> Result<(), PinnedScratchPoolProbeLineParseError> {
    if slot.replace(value).is_some() {
        return Err(PinnedScratchPoolProbeLineParseError::DuplicateField(
            field.to_owned(),
        ));
    }
    Ok(())
}

fn require_probe_field<T>(
    value: Option<T>,
    field: &str,
) -> Result<T, PinnedScratchPoolProbeLineParseError> {
    value.ok_or_else(|| PinnedScratchPoolProbeLineParseError::MissingField(field.to_owned()))
}

fn require_near_gpu_probe_field<T>(
    value: Option<T>,
    field: &str,
) -> Result<T, PinnedScratchNearGpuProbeLineParseError> {
    value.ok_or_else(|| PinnedScratchNearGpuProbeLineParseError::MissingField(field.to_owned()))
}

fn set_near_gpu_output_field<T>(
    slot: &mut Option<T>,
    field: &'static str,
    value: T,
) -> Result<(), PinnedScratchNearGpuProbeOutputParseError> {
    if slot.replace(value).is_some() {
        return Err(PinnedScratchNearGpuProbeOutputParseError::DuplicateField(
            field,
        ));
    }
    Ok(())
}

fn require_near_gpu_output_field<T>(
    value: Option<&T>,
    field: &'static str,
) -> Result<(), PinnedScratchNearGpuProbeOutputParseError> {
    value
        .map(|_| ())
        .ok_or(PinnedScratchNearGpuProbeOutputParseError::MissingField(
            field,
        ))
}

fn is_pinned_scratch_pool_probe_event_line(line: &str) -> bool {
    [
        "pool_checkout=",
        "checked_out_allocation=",
        "pool_release=",
        "pool_reuse_checkout=",
        "pool_reuse_release=",
    ]
    .iter()
    .any(|prefix| line.starts_with(prefix))
}

fn is_pinned_scratch_pool_probe_stats_line(line: &str) -> bool {
    line.split_whitespace().next() == Some("pool_stats")
}

fn set_near_gpu_output_event(
    slot: &mut Option<PinnedScratchPoolProbeEventLine>,
    value: PinnedScratchPoolProbeEventLine,
) -> Result<(), PinnedScratchNearGpuProbeOutputParseError> {
    if slot.replace(value).is_some() {
        return Err(PinnedScratchNearGpuProbeOutputParseError::DuplicateEvent(
            value.event,
        ));
    }
    Ok(())
}

fn set_near_gpu_output_stats(
    slot: &mut Option<PinnedScratchPoolProbeStatsLine>,
    value: PinnedScratchPoolProbeStatsLine,
) -> Result<(), PinnedScratchNearGpuProbeOutputParseError> {
    if slot.replace(value).is_some() {
        return Err(PinnedScratchNearGpuProbeOutputParseError::DuplicateStats(
            value.phase,
        ));
    }
    Ok(())
}

fn require_near_gpu_output_event(
    value: Option<PinnedScratchPoolProbeEventLine>,
    event: PinnedScratchPoolProbeEvent,
) -> Result<PinnedScratchPoolProbeEventLine, PinnedScratchNearGpuProbeOutputParseError> {
    value.ok_or(PinnedScratchNearGpuProbeOutputParseError::MissingEvent(
        event,
    ))
}

fn require_near_gpu_output_stats(
    value: Option<PinnedScratchPoolProbeStatsLine>,
    phase: PinnedScratchPoolProbePhase,
) -> Result<PinnedScratchPoolProbeStatsLine, PinnedScratchNearGpuProbeOutputParseError> {
    value.ok_or(PinnedScratchNearGpuProbeOutputParseError::MissingStats(
        phase,
    ))
}

fn set_probe_output_event(
    slot: &mut Option<PinnedScratchPoolProbeEventLine>,
    value: PinnedScratchPoolProbeEventLine,
) -> Result<(), PinnedScratchPoolProbeOutputParseError> {
    if slot.replace(value).is_some() {
        return Err(PinnedScratchPoolProbeOutputParseError::DuplicateEvent(
            value.event,
        ));
    }
    Ok(())
}

fn set_probe_output_stats(
    slot: &mut Option<PinnedScratchPoolProbeStatsLine>,
    value: PinnedScratchPoolProbeStatsLine,
) -> Result<(), PinnedScratchPoolProbeOutputParseError> {
    if slot.replace(value).is_some() {
        return Err(PinnedScratchPoolProbeOutputParseError::DuplicateStats(
            value.phase,
        ));
    }
    Ok(())
}

fn require_probe_output_event(
    value: Option<PinnedScratchPoolProbeEventLine>,
    event: PinnedScratchPoolProbeEvent,
) -> Result<PinnedScratchPoolProbeEventLine, PinnedScratchPoolProbeOutputParseError> {
    value.ok_or(PinnedScratchPoolProbeOutputParseError::MissingEvent(event))
}

fn require_probe_output_stats(
    value: Option<PinnedScratchPoolProbeStatsLine>,
    phase: PinnedScratchPoolProbePhase,
) -> Result<PinnedScratchPoolProbeStatsLine, PinnedScratchPoolProbeOutputParseError> {
    value.ok_or(PinnedScratchPoolProbeOutputParseError::MissingStats(phase))
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

    use locus_core::{NodeId, PciDevice, RequestHome, RequestId, Topology};

    use super::{
        parse_mapped_scratch_lock_probe_output, parse_mapped_scratch_thp_fault_sample_line,
        parse_mapped_scratch_thp_fault_samples_output, parse_mapped_scratch_thp_probe_output,
        parse_page_lock_probe_status_line, parse_pinned_scratch_near_gpu_probe_output,
        parse_pinned_scratch_near_gpu_probe_pool_line, parse_pinned_scratch_pool_probe_event_line,
        parse_pinned_scratch_pool_probe_output, parse_pinned_scratch_pool_probe_stats_line,
        KvBlockPool, KvBlockPoolError, KvBlockTable, KvBlockTableError, KvSequenceId,
        MappedScratchAllocError, MappedScratchArena, MappedScratchHugePageAdvice,
        MappedScratchLockProbeOutput, MappedScratchLockProbeOutputParseError,
        MappedScratchThpAdviceStatus, MappedScratchThpFaultSampleComparison,
        MappedScratchThpFaultSampleLine, MappedScratchThpFaultSampleLineParseError,
        MappedScratchThpFaultSampleMode, MappedScratchThpFaultSampleStatus,
        MappedScratchThpFaultSamples, MappedScratchThpFaultSamplesParseError,
        MappedScratchThpObservation, MappedScratchThpProbeOutput,
        MappedScratchThpProbeOutputParseError, MappedScratchThpProbeRunStatus, PageLockError,
        PageLockProbeField, PageLockProbeStatus, PageLockProbeStatusLine,
        PageLockProbeStatusLineParseError, PinnedScratchHandle, PinnedScratchNearGpuPoolLine,
        PinnedScratchNearGpuProbeLineParseError, PinnedScratchNearGpuProbeOutput,
        PinnedScratchNearGpuProbeOutputParseError, PinnedScratchNearGpuProbeStatus,
        PinnedScratchPool, PinnedScratchPoolError, PinnedScratchPoolProbeEvent,
        PinnedScratchPoolProbeEventLine, PinnedScratchPoolProbeLineParseError,
        PinnedScratchPoolProbeOutput, PinnedScratchPoolProbeOutputParseError,
        PinnedScratchPoolProbePhase, PinnedScratchPoolProbeStatsLine, PinnedScratchPoolProbeStatus,
        RemoteFreeQueue, RemoteFreeQueueError, RequestScratch, RequestScratchError,
        RequestScratchPool, ScratchAllocError, ScratchArena, MAX_SUPPORTED_ALIGN,
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

    #[cfg(target_os = "linux")]
    #[test]
    fn mapped_scratch_arena_applies_transparent_huge_page_advice() {
        let arena = MappedScratchArena::new(NodeId(0), 4096).expect("arena");

        arena
            .advise_transparent_huge_pages(MappedScratchHugePageAdvice::HugePage)
            .expect("huge page advice");
        arena
            .advise_transparent_huge_pages(MappedScratchHugePageAdvice::NoHugePage)
            .expect("no huge page advice");
    }

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
    fn parses_mapped_scratch_thp_probe_output() {
        let hugepage_output = "\
mapped_scratch_thp=started mode=hugepage
mapping_start=0xffff8753f000
mapping_len=4198399
base_page_kb=4
thp_advice=ok mode=hugepage
touched=1025
numa_maps=unavailable
thp_observed=unknown reason=numa_maps_unavailable
";

        assert_eq!(
            parse_mapped_scratch_thp_probe_output(hugepage_output).expect("hugepage output"),
            MappedScratchThpProbeOutput {
                run_status: MappedScratchThpProbeRunStatus::Started,
                mode: Some(MappedScratchHugePageAdvice::HugePage),
                advice_status: Some(MappedScratchThpAdviceStatus::Ok),
                touched: Some(1025),
                kernel_page_kb: None,
                observation: Some(MappedScratchThpObservation::Unknown),
                observation_reason: Some("numa_maps_unavailable".to_owned()),
            }
        );

        let no_hugepage_output = "\
mapped_scratch_thp=started mode=no_hugepage
thp_advice=ok mode=no_hugepage
touched=1025
kernel_page_kb=4
thp_observed=no reason=base_page_size
";

        assert_eq!(
            parse_mapped_scratch_thp_probe_output(no_hugepage_output).expect("no hugepage output"),
            MappedScratchThpProbeOutput {
                run_status: MappedScratchThpProbeRunStatus::Started,
                mode: Some(MappedScratchHugePageAdvice::NoHugePage),
                advice_status: Some(MappedScratchThpAdviceStatus::Ok),
                touched: Some(1025),
                kernel_page_kb: Some(4),
                observation: Some(MappedScratchThpObservation::No),
                observation_reason: Some("base_page_size".to_owned()),
            }
        );

        assert_eq!(
            MappedScratchHugePageAdvice::HugePage.to_string(),
            "hugepage"
        );
        assert_eq!(MappedScratchThpAdviceStatus::Ok.to_string(), "ok");
        assert_eq!(MappedScratchThpObservation::Yes.to_string(), "yes");
    }

    #[test]
    fn parses_mapped_scratch_thp_unsupported_output() {
        assert_eq!(
            parse_mapped_scratch_thp_probe_output("mapped_scratch_thp=unsupported-platform\n")
                .expect("unsupported output"),
            MappedScratchThpProbeOutput {
                run_status: MappedScratchThpProbeRunStatus::UnsupportedPlatform,
                mode: None,
                advice_status: None,
                touched: None,
                kernel_page_kb: None,
                observation: None,
                observation_reason: None,
            }
        );
    }

    #[test]
    fn parses_mapped_scratch_thp_advice_error_output() {
        let output = "\
mapped_scratch_thp=started mode=hugepage
thp_advice=error mode=hugepage
thp_advice_error=madvise failed: Invalid argument
";

        assert_eq!(
            parse_mapped_scratch_thp_probe_output(output).expect("advice error output"),
            MappedScratchThpProbeOutput {
                run_status: MappedScratchThpProbeRunStatus::Started,
                mode: Some(MappedScratchHugePageAdvice::HugePage),
                advice_status: Some(MappedScratchThpAdviceStatus::Error),
                touched: None,
                kernel_page_kb: None,
                observation: None,
                observation_reason: None,
            }
        );
    }

    #[test]
    fn rejects_invalid_mapped_scratch_thp_probe_output() {
        assert_eq!(
            parse_mapped_scratch_thp_probe_output("thp_advice=ok mode=hugepage\n")
                .expect_err("missing start"),
            MappedScratchThpProbeOutputParseError::MissingField("mapped_scratch_thp")
        );
        assert_eq!(
            parse_mapped_scratch_thp_probe_output(
                "mapped_scratch_thp=started mode=hugepage\nthp_advice=ok mode=no_hugepage\n"
            )
            .expect_err("mode mismatch"),
            MappedScratchThpProbeOutputParseError::ModeMismatch {
                started: MappedScratchHugePageAdvice::HugePage,
                advice: MappedScratchHugePageAdvice::NoHugePage,
            }
        );
        assert_eq!(
            parse_mapped_scratch_thp_probe_output(
                "mapped_scratch_thp=started mode=hugepage\nthp_advice=ok mode=hugepage\n"
            )
            .expect_err("missing touched"),
            MappedScratchThpProbeOutputParseError::MissingField("touched")
        );
        assert_eq!(
            parse_mapped_scratch_thp_probe_output(
                "mapped_scratch_thp=started mode=hugepage\nmapped_scratch_thp=started mode=hugepage\n"
            )
            .expect_err("duplicate start"),
            MappedScratchThpProbeOutputParseError::DuplicateField("mapped_scratch_thp")
        );
        assert_eq!(
            parse_mapped_scratch_thp_probe_output(
                "mapped_scratch_thp=started mode=hugepage\nthp_advice=ok mode=hugepage\ntouched=abc\n"
            )
            .expect_err("bad touched"),
            MappedScratchThpProbeOutputParseError::InvalidNumber {
                field: "touched",
                value: "abc".to_owned(),
            }
        );
    }

    #[test]
    fn parses_mapped_scratch_thp_fault_sample_lines() {
        assert_eq!(
            parse_mapped_scratch_thp_fault_sample_line(
                "fault_sample=hugepage status=available iterations=8 minor_faults_delta=8224 child_minor_faults_delta=-2 major_faults_delta=0 child_major_faults_delta=1"
            )
            .expect("available sample"),
            MappedScratchThpFaultSampleLine {
                mode: MappedScratchThpFaultSampleMode::HugePage,
                status: MappedScratchThpFaultSampleStatus::Available,
                iterations: Some(8),
                minor_faults_delta: Some(8224),
                child_minor_faults_delta: Some(-2),
                major_faults_delta: Some(0),
                child_major_faults_delta: Some(1),
            }
        );

        assert_eq!(
            parse_mapped_scratch_thp_fault_sample_line("fault_sample=default status=unavailable")
                .expect("unavailable sample"),
            MappedScratchThpFaultSampleLine {
                mode: MappedScratchThpFaultSampleMode::Default,
                status: MappedScratchThpFaultSampleStatus::Unavailable,
                iterations: None,
                minor_faults_delta: None,
                child_minor_faults_delta: None,
                major_faults_delta: None,
                child_major_faults_delta: None,
            }
        );

        assert_eq!(
            MappedScratchThpFaultSampleMode::NoHugePage.to_string(),
            "no_hugepage"
        );
        assert_eq!(
            MappedScratchThpFaultSampleStatus::Available.to_string(),
            "available"
        );
    }

    #[test]
    fn parses_mapped_scratch_thp_fault_samples_from_benchmark_output() {
        let output = "\
Gnuplot not found, using plotters backend
fault_sample=default status=available iterations=8 minor_faults_delta=16400 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
fault_sample=hugepage status=available iterations=8 minor_faults_delta=8224 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
fault_sample=no_hugepage status=available iterations=8 minor_faults_delta=16400 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
Benchmarking mapped_scratch_write_touch_4mib_default
";

        assert_eq!(
            parse_mapped_scratch_thp_fault_samples_output(output).expect("fault samples"),
            MappedScratchThpFaultSamples {
                default: MappedScratchThpFaultSampleLine {
                    mode: MappedScratchThpFaultSampleMode::Default,
                    status: MappedScratchThpFaultSampleStatus::Available,
                    iterations: Some(8),
                    minor_faults_delta: Some(16400),
                    child_minor_faults_delta: Some(0),
                    major_faults_delta: Some(0),
                    child_major_faults_delta: Some(0),
                },
                hugepage: MappedScratchThpFaultSampleLine {
                    mode: MappedScratchThpFaultSampleMode::HugePage,
                    status: MappedScratchThpFaultSampleStatus::Available,
                    iterations: Some(8),
                    minor_faults_delta: Some(8224),
                    child_minor_faults_delta: Some(0),
                    major_faults_delta: Some(0),
                    child_major_faults_delta: Some(0),
                },
                no_hugepage: MappedScratchThpFaultSampleLine {
                    mode: MappedScratchThpFaultSampleMode::NoHugePage,
                    status: MappedScratchThpFaultSampleStatus::Available,
                    iterations: Some(8),
                    minor_faults_delta: Some(16400),
                    child_minor_faults_delta: Some(0),
                    major_faults_delta: Some(0),
                    child_major_faults_delta: Some(0),
                },
            }
        );
    }

    #[test]
    fn compares_mapped_scratch_thp_fault_samples() {
        let output = "\
fault_sample=default status=available iterations=8 minor_faults_delta=16400 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
fault_sample=hugepage status=available iterations=8 minor_faults_delta=8224 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
fault_sample=no_hugepage status=available iterations=8 minor_faults_delta=16400 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
";
        let samples = parse_mapped_scratch_thp_fault_samples_output(output).expect("samples");

        assert_eq!(
            samples.comparison(),
            Some(MappedScratchThpFaultSampleComparison {
                default_minor_faults_delta: 16400,
                hugepage_minor_faults_delta: 8224,
                no_hugepage_minor_faults_delta: 16400,
                hugepage_vs_default_minor_faults_delta: -8176,
                hugepage_vs_no_hugepage_minor_faults_delta: -8176,
                major_faults_observed: false,
            })
        );
    }

    #[test]
    fn skips_incomplete_mapped_scratch_thp_fault_sample_comparison() {
        let samples = MappedScratchThpFaultSamples {
            default: MappedScratchThpFaultSampleLine {
                mode: MappedScratchThpFaultSampleMode::Default,
                status: MappedScratchThpFaultSampleStatus::Available,
                iterations: Some(8),
                minor_faults_delta: Some(16400),
                child_minor_faults_delta: Some(0),
                major_faults_delta: Some(0),
                child_major_faults_delta: Some(0),
            },
            hugepage: MappedScratchThpFaultSampleLine {
                mode: MappedScratchThpFaultSampleMode::HugePage,
                status: MappedScratchThpFaultSampleStatus::Unavailable,
                iterations: None,
                minor_faults_delta: None,
                child_minor_faults_delta: None,
                major_faults_delta: None,
                child_major_faults_delta: None,
            },
            no_hugepage: MappedScratchThpFaultSampleLine {
                mode: MappedScratchThpFaultSampleMode::NoHugePage,
                status: MappedScratchThpFaultSampleStatus::Available,
                iterations: Some(8),
                minor_faults_delta: Some(16400),
                child_minor_faults_delta: Some(0),
                major_faults_delta: Some(0),
                child_major_faults_delta: Some(0),
            },
        };

        assert_eq!(samples.comparison(), None);
    }

    #[test]
    fn flags_major_faults_in_mapped_scratch_thp_fault_sample_comparison() {
        let mut samples = MappedScratchThpFaultSamples {
            default: MappedScratchThpFaultSampleLine {
                mode: MappedScratchThpFaultSampleMode::Default,
                status: MappedScratchThpFaultSampleStatus::Available,
                iterations: Some(8),
                minor_faults_delta: Some(16400),
                child_minor_faults_delta: Some(0),
                major_faults_delta: Some(0),
                child_major_faults_delta: Some(0),
            },
            hugepage: MappedScratchThpFaultSampleLine {
                mode: MappedScratchThpFaultSampleMode::HugePage,
                status: MappedScratchThpFaultSampleStatus::Available,
                iterations: Some(8),
                minor_faults_delta: Some(8224),
                child_minor_faults_delta: Some(0),
                major_faults_delta: Some(1),
                child_major_faults_delta: Some(0),
            },
            no_hugepage: MappedScratchThpFaultSampleLine {
                mode: MappedScratchThpFaultSampleMode::NoHugePage,
                status: MappedScratchThpFaultSampleStatus::Available,
                iterations: Some(8),
                minor_faults_delta: Some(16400),
                child_minor_faults_delta: Some(0),
                major_faults_delta: Some(0),
                child_major_faults_delta: Some(0),
            },
        };

        assert!(
            samples
                .comparison()
                .expect("comparison")
                .major_faults_observed
        );

        samples.hugepage.major_faults_delta = Some(0);
        samples.no_hugepage.child_major_faults_delta = Some(-1);

        assert!(
            samples
                .comparison()
                .expect("comparison")
                .major_faults_observed
        );
    }

    #[test]
    fn rejects_invalid_mapped_scratch_thp_fault_sample_lines() {
        assert_eq!(
            parse_mapped_scratch_thp_fault_sample_line("status=available iterations=8")
                .expect_err("missing mode"),
            MappedScratchThpFaultSampleLineParseError::MissingField("fault_sample")
        );
        assert_eq!(
            parse_mapped_scratch_thp_fault_sample_line("fault_sample=maybe status=available")
                .expect_err("unknown mode"),
            MappedScratchThpFaultSampleLineParseError::UnknownMode("maybe".to_owned())
        );
        assert_eq!(
            parse_mapped_scratch_thp_fault_sample_line("fault_sample=default status=maybe")
                .expect_err("unknown status"),
            MappedScratchThpFaultSampleLineParseError::UnknownStatus("maybe".to_owned())
        );
        assert_eq!(
            parse_mapped_scratch_thp_fault_sample_line(
                "fault_sample=default status=available iterations=abc minor_faults_delta=1 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0"
            )
            .expect_err("bad iterations"),
            MappedScratchThpFaultSampleLineParseError::InvalidNumber {
                field: "iterations",
                value: "abc".to_owned(),
            }
        );
        assert_eq!(
            parse_mapped_scratch_thp_fault_sample_line(
                "fault_sample=default status=available iterations=8 minor_faults_delta=1 child_minor_faults_delta=0 major_faults_delta=0"
            )
            .expect_err("missing child major faults"),
            MappedScratchThpFaultSampleLineParseError::MissingField("child_major_faults_delta")
        );
        assert_eq!(
            parse_mapped_scratch_thp_fault_sample_line(
                "fault_sample=default status=unavailable iterations=8"
            )
            .expect_err("unexpected unavailable field"),
            MappedScratchThpFaultSampleLineParseError::UnexpectedField {
                status: MappedScratchThpFaultSampleStatus::Unavailable,
                field: "iterations",
            }
        );
        assert_eq!(
            parse_mapped_scratch_thp_fault_sample_line(
                "fault_sample=default status=unavailable status=available"
            )
            .expect_err("duplicate status"),
            MappedScratchThpFaultSampleLineParseError::DuplicateField("status")
        );
        assert_eq!(
            parse_mapped_scratch_thp_fault_sample_line(
                "fault_sample=default status=unavailable extra=1"
            )
            .expect_err("unknown field"),
            MappedScratchThpFaultSampleLineParseError::UnknownField("extra".to_owned())
        );
    }

    #[test]
    fn rejects_invalid_mapped_scratch_thp_fault_sample_output() {
        assert_eq!(
            parse_mapped_scratch_thp_fault_samples_output(
                "fault_sample=default status=unavailable\n"
            )
            .expect_err("missing hugepage"),
            MappedScratchThpFaultSamplesParseError::MissingSample(
                MappedScratchThpFaultSampleMode::HugePage,
            )
        );
        assert_eq!(
            parse_mapped_scratch_thp_fault_samples_output(
                "fault_sample=default status=unavailable\nfault_sample=default status=unavailable\nfault_sample=hugepage status=unavailable\nfault_sample=no_hugepage status=unavailable\n"
            )
            .expect_err("duplicate default"),
            MappedScratchThpFaultSamplesParseError::DuplicateSample(
                MappedScratchThpFaultSampleMode::Default,
            )
        );
        assert_eq!(
            parse_mapped_scratch_thp_fault_samples_output(
                "fault_sample=default status=maybe\nfault_sample=hugepage status=unavailable\nfault_sample=no_hugepage status=unavailable\n"
            )
            .expect_err("bad line"),
            MappedScratchThpFaultSamplesParseError::Line(
                MappedScratchThpFaultSampleLineParseError::UnknownStatus("maybe".to_owned())
            )
        );
    }

    #[test]
    fn parses_pinned_scratch_pool_probe_event_lines() {
        assert_eq!(
            parse_pinned_scratch_pool_probe_event_line("pool_checkout=ok handle=0")
                .expect("checkout"),
            PinnedScratchPoolProbeEventLine {
                event: PinnedScratchPoolProbeEvent::Checkout,
                status: PinnedScratchPoolProbeStatus::Ok,
                handle: Some(0),
                bytes: None,
            }
        );
        assert_eq!(
            parse_pinned_scratch_pool_probe_event_line("checked_out_allocation=ok bytes=256")
                .expect("allocation"),
            PinnedScratchPoolProbeEventLine {
                event: PinnedScratchPoolProbeEvent::Allocation,
                status: PinnedScratchPoolProbeStatus::Ok,
                handle: None,
                bytes: Some(256),
            }
        );
        assert_eq!(PinnedScratchPoolProbeStatus::Ok.to_string(), "ok");
        assert_eq!(
            PinnedScratchPoolProbeEvent::ReuseCheckout.to_string(),
            "pool_reuse_checkout"
        );
    }

    #[test]
    fn parses_pinned_scratch_pool_probe_stats_lines() {
        assert_eq!(
            parse_pinned_scratch_pool_probe_stats_line(
                "pool_stats phase=after_release locked_bytes=20479 checked_out=0 idle=1 created_arenas=1 reused_arenas=0 checkout_count=1 release_count=1"
            )
            .expect("stats"),
            PinnedScratchPoolProbeStatsLine {
                phase: PinnedScratchPoolProbePhase::AfterRelease,
                locked_bytes: 20479,
                checked_out: 0,
                idle: 1,
                created_arenas: 1,
                reused_arenas: 0,
                checkout_count: 1,
                release_count: 1,
            }
        );
        assert_eq!(
            PinnedScratchPoolProbePhase::AfterReuseRelease.to_string(),
            "after_reuse_release"
        );
    }

    #[test]
    fn rejects_invalid_pinned_scratch_pool_probe_lines() {
        assert_eq!(
            parse_pinned_scratch_pool_probe_event_line("pool_checkout").expect_err("invalid token"),
            PinnedScratchPoolProbeLineParseError::InvalidToken("pool_checkout".to_owned())
        );
        assert_eq!(
            parse_pinned_scratch_pool_probe_event_line("pool_checkout=maybe handle=0")
                .expect_err("unknown status"),
            PinnedScratchPoolProbeLineParseError::UnknownStatus("maybe".to_owned())
        );
        assert_eq!(
            parse_pinned_scratch_pool_probe_event_line("pool_checkout=ok")
                .expect_err("missing handle"),
            PinnedScratchPoolProbeLineParseError::MissingField("handle".to_owned())
        );
        assert_eq!(
            parse_pinned_scratch_pool_probe_event_line("checked_out_allocation=ok")
                .expect_err("missing bytes"),
            PinnedScratchPoolProbeLineParseError::MissingField("bytes".to_owned())
        );
        assert_eq!(
            parse_pinned_scratch_pool_probe_event_line(
                "checked_out_allocation=ok handle=0 bytes=256"
            )
            .expect_err("unexpected handle"),
            PinnedScratchPoolProbeLineParseError::UnknownField("handle".to_owned())
        );
        assert_eq!(
            parse_pinned_scratch_pool_probe_stats_line(
                "pool_stats phase=bad locked_bytes=0 checked_out=0 idle=0 created_arenas=0 reused_arenas=0 checkout_count=0 release_count=0"
            )
            .expect_err("unknown phase"),
            PinnedScratchPoolProbeLineParseError::UnknownPhase("bad".to_owned())
        );
        assert_eq!(
            parse_pinned_scratch_pool_probe_stats_line(
                "pool_stats phase=initial locked_bytes=abc checked_out=0 idle=0 created_arenas=0 reused_arenas=0 checkout_count=0 release_count=0"
            )
            .expect_err("invalid number"),
            PinnedScratchPoolProbeLineParseError::InvalidNumber {
                field: "locked_bytes".to_owned(),
                value: "abc".to_owned(),
            }
        );
    }

    #[test]
    fn parses_pinned_scratch_pool_probe_output() {
        let output = "\
arena_capacity=16384
max_locked_bytes=40958
pool_stats phase=initial locked_bytes=0 checked_out=0 idle=0 created_arenas=0 reused_arenas=0 checkout_count=0 release_count=0
pool_checkout=ok handle=0
checked_out_mapping_start=0xffffbc46e000
checked_out_mapping_len=20479
checked_out_allocation=ok bytes=256
pool_stats phase=after_checkout locked_bytes=20479 checked_out=1 idle=0 created_arenas=1 reused_arenas=0 checkout_count=1 release_count=0
pool_release=ok handle=0
pool_stats phase=after_release locked_bytes=20479 checked_out=0 idle=1 created_arenas=1 reused_arenas=0 checkout_count=1 release_count=1
pool_reuse_checkout=ok handle=1
pool_stats phase=after_reuse_checkout locked_bytes=20479 checked_out=1 idle=0 created_arenas=1 reused_arenas=1 checkout_count=2 release_count=1
pool_reuse_release=ok handle=1
pool_stats phase=after_reuse_release locked_bytes=20479 checked_out=0 idle=1 created_arenas=1 reused_arenas=1 checkout_count=2 release_count=2
";

        let parsed = parse_pinned_scratch_pool_probe_output(output).expect("probe output");
        assert_eq!(parsed.initial_stats.locked_bytes, 0);
        assert_eq!(parsed.checkout.handle, Some(0));
        assert_eq!(parsed.allocation.expect("allocation").bytes, Some(256));
        assert_eq!(parsed.release.expect("release").handle, Some(0));
        assert_eq!(
            parsed.reuse_checkout.expect("reuse checkout").handle,
            Some(1)
        );
        assert_eq!(
            parsed
                .after_reuse_release_stats
                .expect("after reuse release")
                .reused_arenas,
            1
        );
    }

    #[test]
    fn parses_pinned_scratch_pool_checkout_error_output() {
        let output = "\
arena_capacity=16384
max_locked_bytes=40958
pool_stats phase=initial locked_bytes=0 checked_out=0 idle=0 created_arenas=0 reused_arenas=0 checkout_count=0 release_count=0
pool_checkout=error
pool_checkout_error=pinned scratch pool arena failed
pool_stats phase=checkout_error locked_bytes=0 checked_out=0 idle=0 created_arenas=0 reused_arenas=0 checkout_count=0 release_count=0
";

        assert_eq!(
            parse_pinned_scratch_pool_probe_output(output).expect("checkout error output"),
            PinnedScratchPoolProbeOutput {
                initial_stats: PinnedScratchPoolProbeStatsLine {
                    phase: PinnedScratchPoolProbePhase::Initial,
                    locked_bytes: 0,
                    checked_out: 0,
                    idle: 0,
                    created_arenas: 0,
                    reused_arenas: 0,
                    checkout_count: 0,
                    release_count: 0,
                },
                checkout: PinnedScratchPoolProbeEventLine {
                    event: PinnedScratchPoolProbeEvent::Checkout,
                    status: PinnedScratchPoolProbeStatus::Error,
                    handle: None,
                    bytes: None,
                },
                checkout_error_stats: Some(PinnedScratchPoolProbeStatsLine {
                    phase: PinnedScratchPoolProbePhase::CheckoutError,
                    locked_bytes: 0,
                    checked_out: 0,
                    idle: 0,
                    created_arenas: 0,
                    reused_arenas: 0,
                    checkout_count: 0,
                    release_count: 0,
                }),
                allocation: None,
                release: None,
                reuse_checkout: None,
                reuse_release: None,
                after_checkout_stats: None,
                after_release_stats: None,
                after_reuse_checkout_stats: None,
                after_reuse_release_stats: None,
            }
        );
    }

    #[test]
    fn rejects_invalid_pinned_scratch_pool_probe_output() {
        assert_eq!(
            parse_pinned_scratch_pool_probe_output(
                "pool_stats phase=initial locked_bytes=0 checked_out=0 idle=0 created_arenas=0 reused_arenas=0 checkout_count=0 release_count=0\n"
            )
            .expect_err("missing checkout"),
            PinnedScratchPoolProbeOutputParseError::MissingCheckoutLine
        );
        assert_eq!(
            parse_pinned_scratch_pool_probe_output("pool_checkout=ok handle=0\n")
                .expect_err("missing initial stats"),
            PinnedScratchPoolProbeOutputParseError::MissingInitialStats
        );
        assert_eq!(
            parse_pinned_scratch_pool_probe_output(
                "pool_stats phase=initial locked_bytes=0 checked_out=0 idle=0 created_arenas=0 reused_arenas=0 checkout_count=0 release_count=0
pool_checkout=ok handle=0
pool_release=ok handle=0
pool_reuse_checkout=ok handle=1
pool_reuse_release=ok handle=1
pool_stats phase=after_checkout locked_bytes=1 checked_out=1 idle=0 created_arenas=1 reused_arenas=0 checkout_count=1 release_count=0
pool_stats phase=after_release locked_bytes=1 checked_out=0 idle=1 created_arenas=1 reused_arenas=0 checkout_count=1 release_count=1
pool_stats phase=after_reuse_checkout locked_bytes=1 checked_out=1 idle=0 created_arenas=1 reused_arenas=1 checkout_count=2 release_count=1
pool_stats phase=after_reuse_release locked_bytes=1 checked_out=0 idle=1 created_arenas=1 reused_arenas=1 checkout_count=2 release_count=2
"
            )
            .expect_err("missing allocation"),
            PinnedScratchPoolProbeOutputParseError::MissingEvent(
                PinnedScratchPoolProbeEvent::Allocation
            )
        );
        assert_eq!(
            parse_pinned_scratch_pool_probe_output(
                "pool_stats phase=initial locked_bytes=0 checked_out=0 idle=0 created_arenas=0 reused_arenas=0 checkout_count=0 release_count=0
pool_stats phase=initial locked_bytes=0 checked_out=0 idle=0 created_arenas=0 reused_arenas=0 checkout_count=0 release_count=0
pool_checkout=error
"
            )
            .expect_err("duplicate initial stats"),
            PinnedScratchPoolProbeOutputParseError::DuplicateStats(
                PinnedScratchPoolProbePhase::Initial
            )
        );
        assert_eq!(
            parse_pinned_scratch_pool_probe_output(
                "pool_stats phase=initial locked_bytes=0 checked_out=0 idle=0 created_arenas=0 reused_arenas=0 checkout_count=0 release_count=0
pool_checkout=error
pool_checkout=error
"
            )
            .expect_err("duplicate checkout"),
            PinnedScratchPoolProbeOutputParseError::DuplicateEvent(
                PinnedScratchPoolProbeEvent::Checkout
            )
        );
    }

    #[test]
    fn parses_pinned_scratch_near_gpu_pool_lines() {
        assert_eq!(
            parse_pinned_scratch_near_gpu_probe_pool_line("near_gpu_pool=ok home_node=1")
                .expect("ok line"),
            PinnedScratchNearGpuPoolLine {
                status: PinnedScratchNearGpuProbeStatus::Ok,
                home_node: Some(NodeId(1)),
                reason: None,
            }
        );
        assert_eq!(
            parse_pinned_scratch_near_gpu_probe_pool_line(
                "near_gpu_pool=unavailable reason=no_gpu_with_numa_node"
            )
            .expect("unavailable line"),
            PinnedScratchNearGpuPoolLine {
                status: PinnedScratchNearGpuProbeStatus::Unavailable,
                home_node: None,
                reason: Some("no_gpu_with_numa_node".to_owned()),
            }
        );
        assert_eq!(
            parse_pinned_scratch_near_gpu_probe_pool_line("near_gpu_pool=error")
                .expect("error line"),
            PinnedScratchNearGpuPoolLine {
                status: PinnedScratchNearGpuProbeStatus::Error,
                home_node: None,
                reason: None,
            }
        );
        assert_eq!(
            PinnedScratchNearGpuProbeStatus::Unavailable.to_string(),
            "unavailable"
        );
    }

    #[test]
    fn rejects_invalid_pinned_scratch_near_gpu_pool_lines() {
        assert_eq!(
            parse_pinned_scratch_near_gpu_probe_pool_line("near_gpu_pool=maybe")
                .expect_err("unknown status"),
            PinnedScratchNearGpuProbeLineParseError::UnknownStatus("maybe".to_owned())
        );
        assert_eq!(
            parse_pinned_scratch_near_gpu_probe_pool_line("near_gpu_pool=ok")
                .expect_err("missing home node"),
            PinnedScratchNearGpuProbeLineParseError::MissingField("home_node".to_owned())
        );
        assert_eq!(
            parse_pinned_scratch_near_gpu_probe_pool_line("near_gpu_pool=unavailable")
                .expect_err("missing reason"),
            PinnedScratchNearGpuProbeLineParseError::MissingField("reason".to_owned())
        );
        assert_eq!(
            parse_pinned_scratch_near_gpu_probe_pool_line("near_gpu_pool=ok home_node=abc")
                .expect_err("bad home node"),
            PinnedScratchNearGpuProbeLineParseError::InvalidNumber {
                field: "home_node".to_owned(),
                value: "abc".to_owned(),
            }
        );
    }

    #[test]
    fn parses_pinned_scratch_near_gpu_unavailable_output() {
        let output = "\
topology_nodes=0
topology_pci_devices=0
near_gpu_pool=unavailable reason=no_gpu_with_numa_node
";

        assert_eq!(
            parse_pinned_scratch_near_gpu_probe_output(output).expect("unavailable output"),
            PinnedScratchNearGpuProbeOutput {
                topology_nodes: 0,
                topology_pci_devices: 0,
                gpu_bdf: None,
                arena_capacity: None,
                max_locked_bytes: None,
                pool: PinnedScratchNearGpuPoolLine {
                    status: PinnedScratchNearGpuProbeStatus::Unavailable,
                    home_node: None,
                    reason: Some("no_gpu_with_numa_node".to_owned()),
                },
                initial_stats: None,
                checkout: None,
                checkout_error_stats: None,
                allocation: None,
                release: None,
                after_checkout_stats: None,
                after_release_stats: None,
            }
        );
    }

    #[test]
    fn parses_pinned_scratch_near_gpu_success_output() {
        let output = "\
topology_nodes=2
topology_pci_devices=4
gpu_bdf=0000:65:00.0
arena_capacity=16384
max_locked_bytes=40958
near_gpu_pool=ok home_node=1
pool_stats phase=initial locked_bytes=0 checked_out=0 idle=0 created_arenas=0 reused_arenas=0 checkout_count=0 release_count=0
pool_checkout=ok handle=0
checked_out_mapping_start=0xffffbc46e000
checked_out_mapping_len=20479
checked_out_allocation=ok bytes=256
pool_stats phase=after_checkout locked_bytes=20479 checked_out=1 idle=0 created_arenas=1 reused_arenas=0 checkout_count=1 release_count=0
pool_release=ok handle=0
pool_stats phase=after_release locked_bytes=20479 checked_out=0 idle=1 created_arenas=1 reused_arenas=0 checkout_count=1 release_count=1
";

        let parsed = parse_pinned_scratch_near_gpu_probe_output(output).expect("success output");
        assert_eq!(parsed.topology_nodes, 2);
        assert_eq!(parsed.topology_pci_devices, 4);
        assert_eq!(parsed.gpu_bdf.as_deref(), Some("0000:65:00.0"));
        assert_eq!(parsed.arena_capacity, Some(16384));
        assert_eq!(parsed.max_locked_bytes, Some(40958));
        assert_eq!(parsed.pool.home_node, Some(NodeId(1)));
        assert_eq!(parsed.initial_stats.expect("initial").locked_bytes, 0);
        assert_eq!(parsed.checkout.expect("checkout").handle, Some(0));
        assert_eq!(parsed.allocation.expect("allocation").bytes, Some(256));
        assert_eq!(
            parsed
                .after_release_stats
                .expect("after release")
                .release_count,
            1
        );
    }

    #[test]
    fn parses_pinned_scratch_near_gpu_error_output() {
        let output = "\
topology_nodes=2
topology_pci_devices=4
gpu_bdf=0000:65:00.0
arena_capacity=16384
max_locked_bytes=40958
near_gpu_pool=error
near_gpu_pool_error=pinned scratch pool arena failed
";

        let parsed = parse_pinned_scratch_near_gpu_probe_output(output).expect("error output");
        assert_eq!(parsed.pool.status, PinnedScratchNearGpuProbeStatus::Error);
        assert_eq!(parsed.gpu_bdf.as_deref(), Some("0000:65:00.0"));
        assert!(parsed.checkout.is_none());
    }

    #[test]
    fn rejects_invalid_pinned_scratch_near_gpu_probe_output() {
        assert_eq!(
            parse_pinned_scratch_near_gpu_probe_output(
                "topology_pci_devices=0\nnear_gpu_pool=unavailable reason=no_gpu_with_numa_node\n"
            )
            .expect_err("missing topology nodes"),
            PinnedScratchNearGpuProbeOutputParseError::MissingField("topology_nodes")
        );
        assert_eq!(
            parse_pinned_scratch_near_gpu_probe_output(
                "topology_nodes=0\ntopology_nodes=1\ntopology_pci_devices=0\nnear_gpu_pool=unavailable reason=no_gpu_with_numa_node\n"
            )
            .expect_err("duplicate topology nodes"),
            PinnedScratchNearGpuProbeOutputParseError::DuplicateField("topology_nodes")
        );
        assert_eq!(
            parse_pinned_scratch_near_gpu_probe_output(
                "topology_nodes=abc\ntopology_pci_devices=0\nnear_gpu_pool=unavailable reason=no_gpu_with_numa_node\n"
            )
            .expect_err("invalid number"),
            PinnedScratchNearGpuProbeOutputParseError::InvalidNumber {
                field: "topology_nodes",
                value: "abc".to_owned(),
            }
        );
        assert_eq!(
            parse_pinned_scratch_near_gpu_probe_output(
                "topology_nodes=1
topology_pci_devices=1
gpu_bdf=0000:65:00.0
arena_capacity=16384
max_locked_bytes=40958
near_gpu_pool=ok home_node=0
pool_stats phase=initial locked_bytes=0 checked_out=0 idle=0 created_arenas=0 reused_arenas=0 checkout_count=0 release_count=0
pool_checkout=ok handle=0
pool_release=ok handle=0
pool_stats phase=after_checkout locked_bytes=1 checked_out=1 idle=0 created_arenas=1 reused_arenas=0 checkout_count=1 release_count=0
pool_stats phase=after_release locked_bytes=1 checked_out=0 idle=1 created_arenas=1 reused_arenas=0 checkout_count=1 release_count=1
"
            )
            .expect_err("missing allocation"),
            PinnedScratchNearGpuProbeOutputParseError::MissingEvent(
                PinnedScratchPoolProbeEvent::Allocation
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
