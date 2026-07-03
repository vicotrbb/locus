//! Experimental domain allocators for Locus.

use std::fmt;

use locus_core::NodeId;
mod kv_block;
mod mapped_scratch;
mod mapped_scratch_lock_probe;
mod mapped_scratch_thp_fault_sample;
mod mapped_scratch_thp_probe;
mod pinned_scratch;
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

pub use remote_free::{
    RemoteFreeDrainStats, RemoteFreeEnqueueError, RemoteFreeQueue, RemoteFreeQueueError,
    RemoteFreeQueueStats, RemoteFreeSink, RemoteFreeTryEnqueueError, RemoteFreeTryEnqueueErrorKind,
};

pub use request_scratch::{
    RequestScratch, RequestScratchError, RequestScratchPool, RequestScratchPoolStats,
};

pub use scratch_arena::{ScratchAllocError, ScratchArena, ScratchArenaStats};

const MAX_SUPPORTED_ALIGN: usize = 4096;

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

#[cfg(test)]
mod tests {
    use locus_core::NodeId;

    use super::{
        parse_pinned_scratch_near_gpu_probe_output, parse_pinned_scratch_near_gpu_probe_pool_line,
        parse_pinned_scratch_pool_probe_event_line, parse_pinned_scratch_pool_probe_output,
        parse_pinned_scratch_pool_probe_stats_line, PinnedScratchNearGpuPoolLine,
        PinnedScratchNearGpuProbeLineParseError, PinnedScratchNearGpuProbeOutput,
        PinnedScratchNearGpuProbeOutputParseError, PinnedScratchNearGpuProbeStatus,
        PinnedScratchPoolProbeEvent, PinnedScratchPoolProbeEventLine,
        PinnedScratchPoolProbeLineParseError, PinnedScratchPoolProbeOutput,
        PinnedScratchPoolProbeOutputParseError, PinnedScratchPoolProbePhase,
        PinnedScratchPoolProbeStatsLine, PinnedScratchPoolProbeStatus,
    };

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
}
