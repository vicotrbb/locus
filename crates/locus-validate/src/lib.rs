//! Validation helpers that combine Locus probe verdicts.

use std::fmt;

use locus_alloc::{
    parse_mapped_scratch_thp_fault_samples_output, parse_mapped_scratch_thp_probe_output,
    parse_pinned_scratch_near_gpu_probe_output, parse_pinned_scratch_pool_probe_output,
    MappedScratchHugePageAdvice, MappedScratchThpAdviceStatus,
    MappedScratchThpFaultSampleComparison, MappedScratchThpFaultSampleStatus,
    MappedScratchThpFaultSamples, MappedScratchThpFaultSamplesParseError,
    MappedScratchThpObservation, MappedScratchThpProbeOutput,
    MappedScratchThpProbeOutputParseError, PinnedScratchNearGpuProbeOutput,
    PinnedScratchNearGpuProbeOutputParseError, PinnedScratchNearGpuProbeStatus,
    PinnedScratchPoolProbeOutput, PinnedScratchPoolProbeOutputParseError,
    PinnedScratchPoolProbeStatus,
};

/// Host page-locked scratch validation gate status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinnedScratchValidationGateStatus {
    /// Host page-locked scratch reuse was proven by the probe output.
    Ready,
    /// Host page-locked scratch reuse was not proven by the probe output.
    NotReady,
}

impl PinnedScratchValidationGateStatus {
    /// Returns a stable machine-readable status string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::NotReady => "not_ready",
        }
    }

    /// Parses a stable machine-readable status string.
    #[must_use]
    pub fn from_str_token(value: &str) -> Option<Self> {
        match value {
            "ready" => Some(Self::Ready),
            "not_ready" => Some(Self::NotReady),
            _ => None,
        }
    }
}

impl fmt::Display for PinnedScratchValidationGateStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Reason for the host page-locked scratch validation gate status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinnedScratchValidationGateReason {
    /// Host page-locked scratch reuse was proven by the probe output.
    Ready,
    /// First checkout failed.
    CheckoutFailed,
    /// Allocation through the checked-out arena failed or was missing.
    AllocationFailed,
    /// First release failed or was missing.
    ReleaseFailed,
    /// Reuse checkout failed or was missing.
    ReuseCheckoutFailed,
    /// Reuse release failed or was missing.
    ReuseReleaseFailed,
    /// First release did not leave an idle arena in pool stats.
    ReleaseDidNotLeaveIdleArena,
    /// Reuse checkout did not increase reused arena accounting.
    ReuseNotObserved,
    /// Final stats did not show locked bytes remaining in the pool.
    LockedBytesMissing,
}

impl PinnedScratchValidationGateReason {
    /// Returns a stable machine-readable reason string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::CheckoutFailed => "checkout_failed",
            Self::AllocationFailed => "allocation_failed",
            Self::ReleaseFailed => "release_failed",
            Self::ReuseCheckoutFailed => "reuse_checkout_failed",
            Self::ReuseReleaseFailed => "reuse_release_failed",
            Self::ReleaseDidNotLeaveIdleArena => "release_did_not_leave_idle_arena",
            Self::ReuseNotObserved => "reuse_not_observed",
            Self::LockedBytesMissing => "locked_bytes_missing",
        }
    }

    /// Parses a stable machine-readable reason string.
    #[must_use]
    pub fn from_str_token(value: &str) -> Option<Self> {
        match value {
            "ready" => Some(Self::Ready),
            "checkout_failed" => Some(Self::CheckoutFailed),
            "allocation_failed" => Some(Self::AllocationFailed),
            "release_failed" => Some(Self::ReleaseFailed),
            "reuse_checkout_failed" => Some(Self::ReuseCheckoutFailed),
            "reuse_release_failed" => Some(Self::ReuseReleaseFailed),
            "release_did_not_leave_idle_arena" => Some(Self::ReleaseDidNotLeaveIdleArena),
            "reuse_not_observed" => Some(Self::ReuseNotObserved),
            "locked_bytes_missing" => Some(Self::LockedBytesMissing),
            _ => None,
        }
    }
}

impl fmt::Display for PinnedScratchValidationGateReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Parsed host page-locked scratch validation gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PinnedScratchValidationGate {
    /// Final gate status.
    pub status: PinnedScratchValidationGateStatus,
    /// Reason for the status.
    pub reason: PinnedScratchValidationGateReason,
    /// Parsed pinned scratch pool probe output.
    pub probe: PinnedScratchPoolProbeOutput,
}

/// Status and reason parsed from a pinned scratch validation gate line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PinnedScratchValidationGateVerdict {
    /// Parsed gate status.
    pub status: PinnedScratchValidationGateStatus,
    /// Parsed gate reason.
    pub reason: PinnedScratchValidationGateReason,
}

/// Near-GPU pinned scratch validation gate status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinnedScratchNearGpuValidationGateStatus {
    /// GPU-local pinned scratch checkout was proven by the probe output.
    Ready,
    /// GPU PCI NUMA locality was not visible in the probe environment.
    Unavailable,
    /// GPU-local pinned scratch checkout was not proven by the probe output.
    NotReady,
}

impl PinnedScratchNearGpuValidationGateStatus {
    /// Returns a stable machine-readable status string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Unavailable => "unavailable",
            Self::NotReady => "not_ready",
        }
    }

    /// Parses a stable machine-readable status string.
    #[must_use]
    pub fn from_str_token(value: &str) -> Option<Self> {
        match value {
            "ready" => Some(Self::Ready),
            "unavailable" => Some(Self::Unavailable),
            "not_ready" => Some(Self::NotReady),
            _ => None,
        }
    }
}

impl fmt::Display for PinnedScratchNearGpuValidationGateStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Reason for the near-GPU pinned scratch validation gate status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinnedScratchNearGpuValidationGateReason {
    /// GPU-local pinned scratch checkout was proven by the probe output.
    Ready,
    /// No PCI device with a NUMA node was visible.
    NoGpuWithNumaNode,
    /// The requested GPU BDF was not discovered.
    GpuMissing,
    /// The requested GPU BDF had no reported NUMA node.
    GpuNumaNodeUnavailable,
    /// GPU locality resolved to an empty node set.
    EmptyNodeSet,
    /// GPU locality resolved to multiple nodes.
    MultipleNodes,
    /// GPU locality was unavailable for an unclassified reason.
    UnresolvedLocality,
    /// Pool construction failed.
    ConstructorFailed,
    /// Checkout failed.
    CheckoutFailed,
    /// Allocation through the checked-out arena failed or was missing.
    AllocationFailed,
    /// Release failed or was missing.
    ReleaseFailed,
    /// Release did not leave an idle arena in pool stats.
    ReleaseDidNotLeaveIdleArena,
    /// Final stats did not show locked bytes remaining in the pool.
    LockedBytesMissing,
}

impl PinnedScratchNearGpuValidationGateReason {
    /// Returns a stable machine-readable reason string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::NoGpuWithNumaNode => "no_gpu_with_numa_node",
            Self::GpuMissing => "gpu_missing",
            Self::GpuNumaNodeUnavailable => "gpu_numa_node_unavailable",
            Self::EmptyNodeSet => "empty_node_set",
            Self::MultipleNodes => "multiple_nodes",
            Self::UnresolvedLocality => "unresolved_locality",
            Self::ConstructorFailed => "constructor_failed",
            Self::CheckoutFailed => "checkout_failed",
            Self::AllocationFailed => "allocation_failed",
            Self::ReleaseFailed => "release_failed",
            Self::ReleaseDidNotLeaveIdleArena => "release_did_not_leave_idle_arena",
            Self::LockedBytesMissing => "locked_bytes_missing",
        }
    }

    /// Parses a stable machine-readable reason string.
    #[must_use]
    pub fn from_str_token(value: &str) -> Option<Self> {
        match value {
            "ready" => Some(Self::Ready),
            "no_gpu_with_numa_node" => Some(Self::NoGpuWithNumaNode),
            "gpu_missing" => Some(Self::GpuMissing),
            "gpu_numa_node_unavailable" => Some(Self::GpuNumaNodeUnavailable),
            "empty_node_set" => Some(Self::EmptyNodeSet),
            "multiple_nodes" => Some(Self::MultipleNodes),
            "unresolved_locality" => Some(Self::UnresolvedLocality),
            "constructor_failed" => Some(Self::ConstructorFailed),
            "checkout_failed" => Some(Self::CheckoutFailed),
            "allocation_failed" => Some(Self::AllocationFailed),
            "release_failed" => Some(Self::ReleaseFailed),
            "release_did_not_leave_idle_arena" => Some(Self::ReleaseDidNotLeaveIdleArena),
            "locked_bytes_missing" => Some(Self::LockedBytesMissing),
            _ => None,
        }
    }
}

impl fmt::Display for PinnedScratchNearGpuValidationGateReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Parsed near-GPU pinned scratch validation gate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinnedScratchNearGpuValidationGate {
    /// Final gate status.
    pub status: PinnedScratchNearGpuValidationGateStatus,
    /// Reason for the status.
    pub reason: PinnedScratchNearGpuValidationGateReason,
    /// Parsed near-GPU pinned scratch probe output.
    pub probe: PinnedScratchNearGpuProbeOutput,
}

/// Status and reason parsed from a near-GPU pinned scratch validation gate line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PinnedScratchNearGpuValidationGateVerdict {
    /// Parsed gate status.
    pub status: PinnedScratchNearGpuValidationGateStatus,
    /// Parsed gate reason.
    pub reason: PinnedScratchNearGpuValidationGateReason,
}

/// Mapped scratch transparent huge page validation gate status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappedScratchThpValidationGateStatus {
    /// Huge page adoption was proven by probe output.
    Ready,
    /// Required observation evidence is unavailable.
    Unavailable,
    /// Huge page adoption was not proven by probe output.
    NotReady,
}

impl MappedScratchThpValidationGateStatus {
    /// Returns a stable machine-readable status string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Unavailable => "unavailable",
            Self::NotReady => "not_ready",
        }
    }

    /// Parses a stable machine-readable status string.
    #[must_use]
    pub fn from_str_token(value: &str) -> Option<Self> {
        match value {
            "ready" => Some(Self::Ready),
            "unavailable" => Some(Self::Unavailable),
            "not_ready" => Some(Self::NotReady),
            _ => None,
        }
    }
}

impl fmt::Display for MappedScratchThpValidationGateStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Reason for the mapped scratch transparent huge page validation gate status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappedScratchThpValidationGateReason {
    /// Huge page adoption was proven by probe output.
    Ready,
    /// The probe target does not support live THP evidence.
    UnsupportedPlatform,
    /// Observation evidence was unavailable or incomplete.
    ObservationUnavailable,
    /// The probe requested `no_hugepage` mode.
    NoHugePageMode,
    /// The kernel rejected or failed the advice request.
    AdviceFailed,
    /// The mapping was observed with the base page size.
    BasePageSize,
}

impl MappedScratchThpValidationGateReason {
    /// Returns a stable machine-readable reason string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::UnsupportedPlatform => "unsupported_platform",
            Self::ObservationUnavailable => "observation_unavailable",
            Self::NoHugePageMode => "no_hugepage_mode",
            Self::AdviceFailed => "advice_failed",
            Self::BasePageSize => "base_page_size",
        }
    }

    /// Parses a stable machine-readable reason string.
    #[must_use]
    pub fn from_str_token(value: &str) -> Option<Self> {
        match value {
            "ready" => Some(Self::Ready),
            "unsupported_platform" => Some(Self::UnsupportedPlatform),
            "observation_unavailable" => Some(Self::ObservationUnavailable),
            "no_hugepage_mode" => Some(Self::NoHugePageMode),
            "advice_failed" => Some(Self::AdviceFailed),
            "base_page_size" => Some(Self::BasePageSize),
            _ => None,
        }
    }
}

impl fmt::Display for MappedScratchThpValidationGateReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Parsed mapped scratch transparent huge page validation gate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MappedScratchThpValidationGate {
    /// Final gate status.
    pub status: MappedScratchThpValidationGateStatus,
    /// Reason for the status.
    pub reason: MappedScratchThpValidationGateReason,
    /// Parsed mapped scratch THP probe output.
    pub probe: MappedScratchThpProbeOutput,
}

/// Status and reason parsed from a mapped scratch THP validation gate line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MappedScratchThpValidationGateVerdict {
    /// Parsed gate status.
    pub status: MappedScratchThpValidationGateStatus,
    /// Parsed gate reason.
    pub reason: MappedScratchThpValidationGateReason,
}

/// Mapped scratch THP benchmark fault sample validation gate status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappedScratchThpFaultSampleValidationGateStatus {
    /// The benchmark log contains a complete available fault sample set.
    Ready,
    /// The sample set was complete but process fault counters were unavailable.
    Unavailable,
}

impl MappedScratchThpFaultSampleValidationGateStatus {
    /// Returns a stable machine-readable status string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Unavailable => "unavailable",
        }
    }

    /// Parses a stable machine-readable status string.
    #[must_use]
    pub fn from_str_token(value: &str) -> Option<Self> {
        match value {
            "ready" => Some(Self::Ready),
            "unavailable" => Some(Self::Unavailable),
            _ => None,
        }
    }
}

impl fmt::Display for MappedScratchThpFaultSampleValidationGateStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Reason for the mapped scratch THP benchmark fault sample gate status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappedScratchThpFaultSampleValidationGateReason {
    /// All required samples are present and available.
    Ready,
    /// One or more samples reported unavailable process fault counters.
    FaultCountersUnavailable,
}

impl MappedScratchThpFaultSampleValidationGateReason {
    /// Returns a stable machine-readable reason string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::FaultCountersUnavailable => "fault_counters_unavailable",
        }
    }

    /// Parses a stable machine-readable reason string.
    #[must_use]
    pub fn from_str_token(value: &str) -> Option<Self> {
        match value {
            "ready" => Some(Self::Ready),
            "fault_counters_unavailable" => Some(Self::FaultCountersUnavailable),
            _ => None,
        }
    }
}

impl fmt::Display for MappedScratchThpFaultSampleValidationGateReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Parsed mapped scratch THP benchmark fault sample validation gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MappedScratchThpFaultSampleValidationGate {
    /// Final gate status.
    pub status: MappedScratchThpFaultSampleValidationGateStatus,
    /// Reason for the status.
    pub reason: MappedScratchThpFaultSampleValidationGateReason,
    /// Parsed benchmark fault samples.
    pub samples: MappedScratchThpFaultSamples,
}

/// Status and reason parsed from a mapped scratch THP fault sample gate line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MappedScratchThpFaultSampleValidationGateVerdict {
    /// Parsed gate status.
    pub status: MappedScratchThpFaultSampleValidationGateStatus,
    /// Parsed gate reason.
    pub reason: MappedScratchThpFaultSampleValidationGateReason,
}

/// Mapped scratch THP benchmark fault sample comparison line status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappedScratchThpFaultSampleComparisonStatus {
    /// Fault sample comparison is available.
    Available,
    /// Fault sample comparison is unavailable.
    Unavailable,
}

impl MappedScratchThpFaultSampleComparisonStatus {
    /// Returns a stable machine-readable status string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Unavailable => "unavailable",
        }
    }
}

impl fmt::Display for MappedScratchThpFaultSampleComparisonStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Reason for the mapped scratch THP benchmark fault sample comparison status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappedScratchThpFaultSampleComparisonReason {
    /// Comparison is available.
    Ready,
    /// Process fault counters were unavailable in at least one required sample.
    FaultCountersUnavailable,
    /// Samples were marked ready, but the defensive comparison could not be built.
    ComparisonUnavailable,
}

impl MappedScratchThpFaultSampleComparisonReason {
    /// Returns a stable machine-readable reason string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::FaultCountersUnavailable => "fault_counters_unavailable",
            Self::ComparisonUnavailable => "comparison_unavailable",
        }
    }
}

impl fmt::Display for MappedScratchThpFaultSampleComparisonReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Displayable mapped scratch THP benchmark fault sample comparison line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MappedScratchThpFaultSampleComparisonOutput {
    /// Final comparison status.
    pub status: MappedScratchThpFaultSampleComparisonStatus,
    /// Reason for the comparison status.
    pub reason: MappedScratchThpFaultSampleComparisonReason,
    /// Computed comparison, present when status is available.
    pub comparison: Option<MappedScratchThpFaultSampleComparison>,
}

impl fmt::Display for PinnedScratchValidationGate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "pinned_scratch_validation_gate={} reason={}",
            self.status, self.reason
        )
    }
}

impl fmt::Display for PinnedScratchValidationGateVerdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "pinned_scratch_validation_gate={} reason={}",
            self.status, self.reason
        )
    }
}

impl fmt::Display for PinnedScratchNearGpuValidationGate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "pinned_scratch_near_gpu_validation_gate={} reason={}",
            self.status, self.reason
        )
    }
}

impl fmt::Display for PinnedScratchNearGpuValidationGateVerdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "pinned_scratch_near_gpu_validation_gate={} reason={}",
            self.status, self.reason
        )
    }
}

impl fmt::Display for MappedScratchThpValidationGate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "mapped_scratch_thp_validation_gate={} reason={}",
            self.status, self.reason
        )
    }
}

impl fmt::Display for MappedScratchThpValidationGateVerdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "mapped_scratch_thp_validation_gate={} reason={}",
            self.status, self.reason
        )
    }
}

impl fmt::Display for MappedScratchThpFaultSampleValidationGate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "mapped_scratch_thp_fault_sample_validation_gate={} reason={}",
            self.status, self.reason
        )
    }
}

impl fmt::Display for MappedScratchThpFaultSampleValidationGateVerdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "mapped_scratch_thp_fault_sample_validation_gate={} reason={}",
            self.status, self.reason
        )
    }
}

impl fmt::Display for MappedScratchThpFaultSampleComparisonOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "mapped_scratch_thp_fault_sample_comparison={} reason={}",
            self.status, self.reason
        )?;

        if let Some(comparison) = self.comparison {
            write!(
                f,
                " default_minor_faults_delta={} hugepage_minor_faults_delta={} no_hugepage_minor_faults_delta={} hugepage_vs_default_minor_faults_delta={} hugepage_vs_no_hugepage_minor_faults_delta={} major_faults_observed={}",
                comparison.default_minor_faults_delta,
                comparison.hugepage_minor_faults_delta,
                comparison.no_hugepage_minor_faults_delta,
                comparison.hugepage_vs_default_minor_faults_delta,
                comparison.hugepage_vs_no_hugepage_minor_faults_delta,
                comparison.major_faults_observed
            )?;
        }

        Ok(())
    }
}

impl PinnedScratchValidationGateVerdict {
    /// Builds a verdict only when the status and reason are coherent.
    ///
    /// # Errors
    ///
    /// Returns an error when the reason is not valid for the status.
    pub fn from_parts(
        status: PinnedScratchValidationGateStatus,
        reason: PinnedScratchValidationGateReason,
    ) -> Result<Self, PinnedScratchValidationGateLineParseError> {
        let verdict = Self { status, reason };
        if verdict.is_consistent() {
            Ok(verdict)
        } else {
            Err(PinnedScratchValidationGateLineParseError::InconsistentVerdict { status, reason })
        }
    }

    /// Returns true when the reason is valid for the status.
    #[must_use]
    pub fn is_consistent(self) -> bool {
        matches!(
            (self.status, self.reason),
            (
                PinnedScratchValidationGateStatus::Ready,
                PinnedScratchValidationGateReason::Ready
            ) | (
                PinnedScratchValidationGateStatus::NotReady,
                PinnedScratchValidationGateReason::CheckoutFailed
                    | PinnedScratchValidationGateReason::AllocationFailed
                    | PinnedScratchValidationGateReason::ReleaseFailed
                    | PinnedScratchValidationGateReason::ReuseCheckoutFailed
                    | PinnedScratchValidationGateReason::ReuseReleaseFailed
                    | PinnedScratchValidationGateReason::ReleaseDidNotLeaveIdleArena
                    | PinnedScratchValidationGateReason::ReuseNotObserved
                    | PinnedScratchValidationGateReason::LockedBytesMissing
            )
        )
    }
}

impl PinnedScratchNearGpuValidationGateVerdict {
    /// Builds a verdict only when the status and reason are coherent.
    ///
    /// # Errors
    ///
    /// Returns an error when the reason is not valid for the status.
    pub fn from_parts(
        status: PinnedScratchNearGpuValidationGateStatus,
        reason: PinnedScratchNearGpuValidationGateReason,
    ) -> Result<Self, PinnedScratchNearGpuValidationGateLineParseError> {
        let verdict = Self { status, reason };
        if verdict.is_consistent() {
            Ok(verdict)
        } else {
            Err(
                PinnedScratchNearGpuValidationGateLineParseError::InconsistentVerdict {
                    status,
                    reason,
                },
            )
        }
    }

    /// Returns true when the reason is valid for the status.
    #[must_use]
    pub fn is_consistent(self) -> bool {
        matches!(
            (self.status, self.reason),
            (
                PinnedScratchNearGpuValidationGateStatus::Ready,
                PinnedScratchNearGpuValidationGateReason::Ready
            ) | (
                PinnedScratchNearGpuValidationGateStatus::Unavailable,
                PinnedScratchNearGpuValidationGateReason::NoGpuWithNumaNode
                    | PinnedScratchNearGpuValidationGateReason::GpuMissing
                    | PinnedScratchNearGpuValidationGateReason::GpuNumaNodeUnavailable
                    | PinnedScratchNearGpuValidationGateReason::EmptyNodeSet
                    | PinnedScratchNearGpuValidationGateReason::MultipleNodes
                    | PinnedScratchNearGpuValidationGateReason::UnresolvedLocality
            ) | (
                PinnedScratchNearGpuValidationGateStatus::NotReady,
                PinnedScratchNearGpuValidationGateReason::ConstructorFailed
                    | PinnedScratchNearGpuValidationGateReason::CheckoutFailed
                    | PinnedScratchNearGpuValidationGateReason::AllocationFailed
                    | PinnedScratchNearGpuValidationGateReason::ReleaseFailed
                    | PinnedScratchNearGpuValidationGateReason::ReleaseDidNotLeaveIdleArena
                    | PinnedScratchNearGpuValidationGateReason::LockedBytesMissing
            )
        )
    }
}

impl MappedScratchThpValidationGateVerdict {
    /// Builds a verdict only when the status and reason are coherent.
    ///
    /// # Errors
    ///
    /// Returns an error when the reason is not valid for the status.
    pub fn from_parts(
        status: MappedScratchThpValidationGateStatus,
        reason: MappedScratchThpValidationGateReason,
    ) -> Result<Self, MappedScratchThpValidationGateLineParseError> {
        let verdict = Self { status, reason };
        if verdict.is_consistent() {
            Ok(verdict)
        } else {
            Err(
                MappedScratchThpValidationGateLineParseError::InconsistentVerdict {
                    status,
                    reason,
                },
            )
        }
    }

    /// Returns true when the reason is valid for the status.
    #[must_use]
    pub fn is_consistent(self) -> bool {
        matches!(
            (self.status, self.reason),
            (
                MappedScratchThpValidationGateStatus::Ready,
                MappedScratchThpValidationGateReason::Ready
            ) | (
                MappedScratchThpValidationGateStatus::Unavailable,
                MappedScratchThpValidationGateReason::UnsupportedPlatform
                    | MappedScratchThpValidationGateReason::ObservationUnavailable
            ) | (
                MappedScratchThpValidationGateStatus::NotReady,
                MappedScratchThpValidationGateReason::NoHugePageMode
                    | MappedScratchThpValidationGateReason::AdviceFailed
                    | MappedScratchThpValidationGateReason::BasePageSize
            )
        )
    }
}

impl MappedScratchThpFaultSampleValidationGateVerdict {
    /// Builds a verdict only when the status and reason are coherent.
    ///
    /// # Errors
    ///
    /// Returns an error when the reason is not valid for the status.
    pub fn from_parts(
        status: MappedScratchThpFaultSampleValidationGateStatus,
        reason: MappedScratchThpFaultSampleValidationGateReason,
    ) -> Result<Self, MappedScratchThpFaultSampleValidationGateLineParseError> {
        let verdict = Self { status, reason };
        if verdict.is_consistent() {
            Ok(verdict)
        } else {
            Err(
                MappedScratchThpFaultSampleValidationGateLineParseError::InconsistentVerdict {
                    status,
                    reason,
                },
            )
        }
    }

    /// Returns true when the reason is valid for the status.
    #[must_use]
    pub fn is_consistent(self) -> bool {
        matches!(
            (self.status, self.reason),
            (
                MappedScratchThpFaultSampleValidationGateStatus::Ready,
                MappedScratchThpFaultSampleValidationGateReason::Ready
            ) | (
                MappedScratchThpFaultSampleValidationGateStatus::Unavailable,
                MappedScratchThpFaultSampleValidationGateReason::FaultCountersUnavailable
            )
        )
    }
}

/// Error returned when parsing a pinned scratch validation gate line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PinnedScratchValidationGateLineParseError {
    /// The line does not contain a `pinned_scratch_validation_gate=` token.
    MissingStatus,
    /// The line does not contain a `reason=` token.
    MissingReason,
    /// The line contains a duplicate `pinned_scratch_validation_gate=` token.
    DuplicateStatus,
    /// The line contains a duplicate `reason=` token.
    DuplicateReason,
    /// The line contains a token outside the pinned scratch validation gate schema.
    InvalidToken(String),
    /// The gate status token is not recognized.
    UnknownStatus(String),
    /// The gate reason token is not recognized.
    UnknownReason(String),
    /// The status and reason tokens are individually valid but inconsistent together.
    InconsistentVerdict {
        /// Parsed gate status.
        status: PinnedScratchValidationGateStatus,
        /// Parsed gate reason.
        reason: PinnedScratchValidationGateReason,
    },
}

/// Error returned when parsing a near-GPU pinned scratch validation gate line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PinnedScratchNearGpuValidationGateLineParseError {
    /// The line does not contain a `pinned_scratch_near_gpu_validation_gate=` token.
    MissingStatus,
    /// The line does not contain a `reason=` token.
    MissingReason,
    /// The line contains a duplicate `pinned_scratch_near_gpu_validation_gate=` token.
    DuplicateStatus,
    /// The line contains a duplicate `reason=` token.
    DuplicateReason,
    /// The line contains a token outside the near-GPU validation gate schema.
    InvalidToken(String),
    /// The gate status token is not recognized.
    UnknownStatus(String),
    /// The gate reason token is not recognized.
    UnknownReason(String),
    /// The status and reason tokens are individually valid but inconsistent together.
    InconsistentVerdict {
        /// Parsed gate status.
        status: PinnedScratchNearGpuValidationGateStatus,
        /// Parsed gate reason.
        reason: PinnedScratchNearGpuValidationGateReason,
    },
}

/// Error returned when parsing a mapped scratch THP validation gate line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MappedScratchThpValidationGateLineParseError {
    /// The line does not contain a `mapped_scratch_thp_validation_gate=` token.
    MissingStatus,
    /// The line does not contain a `reason=` token.
    MissingReason,
    /// The line contains a duplicate `mapped_scratch_thp_validation_gate=` token.
    DuplicateStatus,
    /// The line contains a duplicate `reason=` token.
    DuplicateReason,
    /// The line contains a token outside the mapped scratch THP gate schema.
    InvalidToken(String),
    /// The gate status token is not recognized.
    UnknownStatus(String),
    /// The gate reason token is not recognized.
    UnknownReason(String),
    /// The status and reason tokens are individually valid but inconsistent together.
    InconsistentVerdict {
        /// Parsed gate status.
        status: MappedScratchThpValidationGateStatus,
        /// Parsed gate reason.
        reason: MappedScratchThpValidationGateReason,
    },
}

/// Error returned when parsing a mapped scratch THP fault sample gate line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MappedScratchThpFaultSampleValidationGateLineParseError {
    /// The line does not contain a `mapped_scratch_thp_fault_sample_validation_gate=` token.
    MissingStatus,
    /// The line does not contain a `reason=` token.
    MissingReason,
    /// The line contains a duplicate `mapped_scratch_thp_fault_sample_validation_gate=` token.
    DuplicateStatus,
    /// The line contains a duplicate `reason=` token.
    DuplicateReason,
    /// The line contains a token outside the fault sample gate schema.
    InvalidToken(String),
    /// The gate status token is not recognized.
    UnknownStatus(String),
    /// The gate reason token is not recognized.
    UnknownReason(String),
    /// The status and reason tokens are individually valid but inconsistent together.
    InconsistentVerdict {
        /// Parsed gate status.
        status: MappedScratchThpFaultSampleValidationGateStatus,
        /// Parsed gate reason.
        reason: MappedScratchThpFaultSampleValidationGateReason,
    },
}

impl fmt::Display for PinnedScratchValidationGateLineParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingStatus => f.write_str("missing pinned_scratch_validation_gate token"),
            Self::MissingReason => f.write_str("missing reason token"),
            Self::DuplicateStatus => f.write_str("duplicate pinned_scratch_validation_gate token"),
            Self::DuplicateReason => f.write_str("duplicate reason token"),
            Self::InvalidToken(token) => {
                write!(f, "invalid pinned scratch validation gate token: {token}")
            }
            Self::UnknownStatus(status) => {
                write!(f, "unknown pinned scratch validation gate status: {status}")
            }
            Self::UnknownReason(reason) => {
                write!(f, "unknown pinned scratch validation gate reason: {reason}")
            }
            Self::InconsistentVerdict { status, reason } => {
                write!(
                    f,
                    "inconsistent pinned scratch validation gate: {status} {reason}"
                )
            }
        }
    }
}

impl std::error::Error for PinnedScratchValidationGateLineParseError {}

impl fmt::Display for PinnedScratchNearGpuValidationGateLineParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingStatus => {
                f.write_str("missing pinned_scratch_near_gpu_validation_gate token")
            }
            Self::MissingReason => f.write_str("missing reason token"),
            Self::DuplicateStatus => {
                f.write_str("duplicate pinned_scratch_near_gpu_validation_gate token")
            }
            Self::DuplicateReason => f.write_str("duplicate reason token"),
            Self::InvalidToken(token) => {
                write!(
                    f,
                    "invalid pinned scratch near-GPU validation gate token: {token}"
                )
            }
            Self::UnknownStatus(status) => {
                write!(
                    f,
                    "unknown pinned scratch near-GPU validation gate status: {status}"
                )
            }
            Self::UnknownReason(reason) => {
                write!(
                    f,
                    "unknown pinned scratch near-GPU validation gate reason: {reason}"
                )
            }
            Self::InconsistentVerdict { status, reason } => {
                write!(
                    f,
                    "inconsistent pinned scratch near-GPU validation gate: {status} {reason}"
                )
            }
        }
    }
}

impl std::error::Error for PinnedScratchNearGpuValidationGateLineParseError {}

impl fmt::Display for MappedScratchThpValidationGateLineParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingStatus => f.write_str("missing mapped_scratch_thp_validation_gate token"),
            Self::MissingReason => f.write_str("missing reason token"),
            Self::DuplicateStatus => {
                f.write_str("duplicate mapped_scratch_thp_validation_gate token")
            }
            Self::DuplicateReason => f.write_str("duplicate reason token"),
            Self::InvalidToken(token) => {
                write!(
                    f,
                    "invalid mapped scratch THP validation gate token: {token}"
                )
            }
            Self::UnknownStatus(status) => {
                write!(
                    f,
                    "unknown mapped scratch THP validation gate status: {status}"
                )
            }
            Self::UnknownReason(reason) => {
                write!(
                    f,
                    "unknown mapped scratch THP validation gate reason: {reason}"
                )
            }
            Self::InconsistentVerdict { status, reason } => {
                write!(
                    f,
                    "inconsistent mapped scratch THP validation gate: {status} {reason}"
                )
            }
        }
    }
}

impl std::error::Error for MappedScratchThpValidationGateLineParseError {}

impl fmt::Display for MappedScratchThpFaultSampleValidationGateLineParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingStatus => {
                f.write_str("missing mapped_scratch_thp_fault_sample_validation_gate token")
            }
            Self::MissingReason => f.write_str("missing reason token"),
            Self::DuplicateStatus => {
                f.write_str("duplicate mapped_scratch_thp_fault_sample_validation_gate token")
            }
            Self::DuplicateReason => f.write_str("duplicate reason token"),
            Self::InvalidToken(token) => write!(
                f,
                "invalid mapped scratch THP fault sample validation gate token: {token}"
            ),
            Self::UnknownStatus(status) => write!(
                f,
                "unknown mapped scratch THP fault sample validation gate status: {status}"
            ),
            Self::UnknownReason(reason) => write!(
                f,
                "unknown mapped scratch THP fault sample validation gate reason: {reason}"
            ),
            Self::InconsistentVerdict { status, reason } => write!(
                f,
                "inconsistent mapped scratch THP fault sample validation gate: {status} {reason}"
            ),
        }
    }
}

impl std::error::Error for MappedScratchThpFaultSampleValidationGateLineParseError {}

/// Error returned when extracting a pinned scratch validation gate from output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PinnedScratchValidationGateOutputParseError {
    /// The output does not contain a `pinned_scratch_validation_gate=` line.
    MissingGateLine,
    /// The output contains more than one `pinned_scratch_validation_gate=` line.
    DuplicateGateLine,
    /// The discovered gate line is malformed.
    Line(PinnedScratchValidationGateLineParseError),
}

/// Error returned when extracting a mapped scratch THP validation gate from output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MappedScratchThpValidationGateOutputParseError {
    /// The output does not contain a `mapped_scratch_thp_validation_gate=` line.
    MissingGateLine,
    /// The output contains more than one `mapped_scratch_thp_validation_gate=` line.
    DuplicateGateLine,
    /// The discovered gate line is malformed.
    Line(MappedScratchThpValidationGateLineParseError),
}

/// Error returned when extracting a mapped scratch THP fault sample gate from output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MappedScratchThpFaultSampleValidationGateOutputParseError {
    /// The output does not contain a `mapped_scratch_thp_fault_sample_validation_gate=` line.
    MissingGateLine,
    /// The output contains more than one `mapped_scratch_thp_fault_sample_validation_gate=` line.
    DuplicateGateLine,
    /// The discovered gate line is malformed.
    Line(MappedScratchThpFaultSampleValidationGateLineParseError),
}

impl fmt::Display for PinnedScratchValidationGateOutputParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingGateLine => f.write_str("missing pinned_scratch_validation_gate line"),
            Self::DuplicateGateLine => f.write_str("duplicate pinned_scratch_validation_gate line"),
            Self::Line(source) => {
                write!(f, "invalid pinned_scratch_validation_gate line: {source}")
            }
        }
    }
}

impl std::error::Error for PinnedScratchValidationGateOutputParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Line(source) => Some(source),
            Self::MissingGateLine | Self::DuplicateGateLine => None,
        }
    }
}

impl fmt::Display for MappedScratchThpValidationGateOutputParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingGateLine => f.write_str("missing mapped_scratch_thp_validation_gate line"),
            Self::DuplicateGateLine => {
                f.write_str("duplicate mapped_scratch_thp_validation_gate line")
            }
            Self::Line(source) => {
                write!(
                    f,
                    "invalid mapped_scratch_thp_validation_gate line: {source}"
                )
            }
        }
    }
}

impl std::error::Error for MappedScratchThpValidationGateOutputParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Line(source) => Some(source),
            Self::MissingGateLine | Self::DuplicateGateLine => None,
        }
    }
}

impl fmt::Display for MappedScratchThpFaultSampleValidationGateOutputParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingGateLine => {
                f.write_str("missing mapped_scratch_thp_fault_sample_validation_gate line")
            }
            Self::DuplicateGateLine => {
                f.write_str("duplicate mapped_scratch_thp_fault_sample_validation_gate line")
            }
            Self::Line(source) => write!(
                f,
                "invalid mapped_scratch_thp_fault_sample_validation_gate line: {source}"
            ),
        }
    }
}

impl std::error::Error for MappedScratchThpFaultSampleValidationGateOutputParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Line(source) => Some(source),
            Self::MissingGateLine | Self::DuplicateGateLine => None,
        }
    }
}

/// Error returned when evaluating pinned scratch pool probe output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PinnedScratchValidationGateParseError {
    /// Pinned scratch pool probe output was missing or malformed.
    Probe(PinnedScratchPoolProbeOutputParseError),
}

/// Error returned when evaluating near-GPU pinned scratch probe output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PinnedScratchNearGpuValidationGateParseError {
    /// Near-GPU pinned scratch probe output was missing or malformed.
    Probe(PinnedScratchNearGpuProbeOutputParseError),
}

/// Error returned when evaluating mapped scratch THP probe output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MappedScratchThpValidationGateParseError {
    /// Mapped scratch THP probe output was missing or malformed.
    Probe(MappedScratchThpProbeOutputParseError),
}

/// Error returned when evaluating mapped scratch THP benchmark fault samples.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MappedScratchThpFaultSampleValidationGateParseError {
    /// Benchmark fault sample output was missing or malformed.
    Samples(MappedScratchThpFaultSamplesParseError),
}

impl fmt::Display for PinnedScratchValidationGateParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Probe(source) => write!(f, "invalid pinned scratch pool output: {source}"),
        }
    }
}

impl std::error::Error for PinnedScratchValidationGateParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Probe(source) => Some(source),
        }
    }
}

impl fmt::Display for PinnedScratchNearGpuValidationGateParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Probe(source) => {
                write!(f, "invalid near-GPU pinned scratch output: {source}")
            }
        }
    }
}

impl std::error::Error for PinnedScratchNearGpuValidationGateParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Probe(source) => Some(source),
        }
    }
}

impl fmt::Display for MappedScratchThpValidationGateParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Probe(source) => write!(f, "invalid mapped scratch THP output: {source}"),
        }
    }
}

impl std::error::Error for MappedScratchThpValidationGateParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Probe(source) => Some(source),
        }
    }
}

impl fmt::Display for MappedScratchThpFaultSampleValidationGateParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Samples(source) => {
                write!(
                    f,
                    "invalid mapped scratch THP fault sample output: {source}"
                )
            }
        }
    }
}

impl std::error::Error for MappedScratchThpFaultSampleValidationGateParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Samples(source) => Some(source),
        }
    }
}

impl PinnedScratchValidationGate {
    /// Builds a gate from parsed pinned scratch pool probe output.
    #[must_use]
    pub fn from_probe(probe: PinnedScratchPoolProbeOutput) -> Self {
        let reason = pinned_scratch_not_ready_reason(&probe);
        let status = if reason == PinnedScratchValidationGateReason::Ready {
            PinnedScratchValidationGateStatus::Ready
        } else {
            PinnedScratchValidationGateStatus::NotReady
        };

        Self {
            status,
            reason,
            probe,
        }
    }

    /// Returns true only when host page-locked scratch reuse is proven.
    #[must_use]
    pub fn is_ready(self) -> bool {
        self.status == PinnedScratchValidationGateStatus::Ready
    }
}

impl PinnedScratchNearGpuValidationGate {
    /// Builds a gate from parsed near-GPU pinned scratch probe output.
    #[must_use]
    pub fn from_probe(probe: PinnedScratchNearGpuProbeOutput) -> Self {
        let reason = pinned_scratch_near_gpu_gate_reason(&probe);
        let status = match reason {
            PinnedScratchNearGpuValidationGateReason::Ready => {
                PinnedScratchNearGpuValidationGateStatus::Ready
            }
            PinnedScratchNearGpuValidationGateReason::NoGpuWithNumaNode
            | PinnedScratchNearGpuValidationGateReason::GpuMissing
            | PinnedScratchNearGpuValidationGateReason::GpuNumaNodeUnavailable
            | PinnedScratchNearGpuValidationGateReason::EmptyNodeSet
            | PinnedScratchNearGpuValidationGateReason::MultipleNodes
            | PinnedScratchNearGpuValidationGateReason::UnresolvedLocality => {
                PinnedScratchNearGpuValidationGateStatus::Unavailable
            }
            PinnedScratchNearGpuValidationGateReason::ConstructorFailed
            | PinnedScratchNearGpuValidationGateReason::CheckoutFailed
            | PinnedScratchNearGpuValidationGateReason::AllocationFailed
            | PinnedScratchNearGpuValidationGateReason::ReleaseFailed
            | PinnedScratchNearGpuValidationGateReason::ReleaseDidNotLeaveIdleArena
            | PinnedScratchNearGpuValidationGateReason::LockedBytesMissing => {
                PinnedScratchNearGpuValidationGateStatus::NotReady
            }
        };

        Self {
            status,
            reason,
            probe,
        }
    }

    /// Returns true only when GPU-local pinned scratch checkout is proven.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.status == PinnedScratchNearGpuValidationGateStatus::Ready
    }
}

impl MappedScratchThpValidationGate {
    /// Builds a gate from parsed mapped scratch THP probe output.
    #[must_use]
    pub fn from_probe(probe: MappedScratchThpProbeOutput) -> Self {
        let reason = mapped_scratch_thp_gate_reason(&probe);
        let status = match reason {
            MappedScratchThpValidationGateReason::Ready => {
                MappedScratchThpValidationGateStatus::Ready
            }
            MappedScratchThpValidationGateReason::UnsupportedPlatform
            | MappedScratchThpValidationGateReason::ObservationUnavailable => {
                MappedScratchThpValidationGateStatus::Unavailable
            }
            MappedScratchThpValidationGateReason::NoHugePageMode
            | MappedScratchThpValidationGateReason::AdviceFailed
            | MappedScratchThpValidationGateReason::BasePageSize => {
                MappedScratchThpValidationGateStatus::NotReady
            }
        };

        Self {
            status,
            reason,
            probe,
        }
    }

    /// Returns true only when huge page adoption is proven.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.status == MappedScratchThpValidationGateStatus::Ready
    }
}

impl MappedScratchThpFaultSampleValidationGate {
    /// Builds a gate from parsed mapped scratch THP benchmark fault samples.
    #[must_use]
    pub fn from_samples(samples: MappedScratchThpFaultSamples) -> Self {
        let reason = mapped_scratch_thp_fault_sample_gate_reason(&samples);
        let status = match reason {
            MappedScratchThpFaultSampleValidationGateReason::Ready => {
                MappedScratchThpFaultSampleValidationGateStatus::Ready
            }
            MappedScratchThpFaultSampleValidationGateReason::FaultCountersUnavailable => {
                MappedScratchThpFaultSampleValidationGateStatus::Unavailable
            }
        };

        Self {
            status,
            reason,
            samples,
        }
    }

    /// Returns true only when all required fault samples are available.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.status == MappedScratchThpFaultSampleValidationGateStatus::Ready
    }

    /// Builds the stable comparison output line for this gate.
    #[must_use]
    pub fn comparison_output(&self) -> MappedScratchThpFaultSampleComparisonOutput {
        if let Some(comparison) = self.samples.comparison() {
            return MappedScratchThpFaultSampleComparisonOutput {
                status: MappedScratchThpFaultSampleComparisonStatus::Available,
                reason: MappedScratchThpFaultSampleComparisonReason::Ready,
                comparison: Some(comparison),
            };
        }

        let reason = match self.reason {
            MappedScratchThpFaultSampleValidationGateReason::Ready => {
                MappedScratchThpFaultSampleComparisonReason::ComparisonUnavailable
            }
            MappedScratchThpFaultSampleValidationGateReason::FaultCountersUnavailable => {
                MappedScratchThpFaultSampleComparisonReason::FaultCountersUnavailable
            }
        };

        MappedScratchThpFaultSampleComparisonOutput {
            status: MappedScratchThpFaultSampleComparisonStatus::Unavailable,
            reason,
            comparison: None,
        }
    }
}

/// Parses pinned scratch pool probe output and returns the validation gate.
///
/// # Errors
///
/// Returns an error when the probe output is missing required stable lines or
/// contains malformed stable lines.
pub fn evaluate_pinned_scratch_validation_output(
    output: &str,
) -> Result<PinnedScratchValidationGate, PinnedScratchValidationGateParseError> {
    let probe = parse_pinned_scratch_pool_probe_output(output)
        .map_err(PinnedScratchValidationGateParseError::Probe)?;
    Ok(PinnedScratchValidationGate::from_probe(probe))
}

/// Parses near-GPU pinned scratch probe output and returns the validation gate.
///
/// # Errors
///
/// Returns an error when the probe output is missing required stable lines or
/// contains malformed stable lines.
pub fn evaluate_pinned_scratch_near_gpu_validation_output(
    output: &str,
) -> Result<PinnedScratchNearGpuValidationGate, PinnedScratchNearGpuValidationGateParseError> {
    let probe = parse_pinned_scratch_near_gpu_probe_output(output)
        .map_err(PinnedScratchNearGpuValidationGateParseError::Probe)?;
    Ok(PinnedScratchNearGpuValidationGate::from_probe(probe))
}

/// Parses mapped scratch THP probe output and returns the validation gate.
///
/// # Errors
///
/// Returns an error when the probe output is missing required stable lines or
/// contains malformed stable lines.
pub fn evaluate_mapped_scratch_thp_validation_output(
    output: &str,
) -> Result<MappedScratchThpValidationGate, MappedScratchThpValidationGateParseError> {
    let probe = parse_mapped_scratch_thp_probe_output(output)
        .map_err(MappedScratchThpValidationGateParseError::Probe)?;
    Ok(MappedScratchThpValidationGate::from_probe(probe))
}

/// Parses mapped scratch THP benchmark output and returns the fault sample gate.
///
/// # Errors
///
/// Returns an error when the output is missing required fault sample lines or
/// contains malformed stable fault sample lines.
pub fn evaluate_mapped_scratch_thp_fault_sample_validation_output(
    output: &str,
) -> Result<
    MappedScratchThpFaultSampleValidationGate,
    MappedScratchThpFaultSampleValidationGateParseError,
> {
    let samples = parse_mapped_scratch_thp_fault_samples_output(output)
        .map_err(MappedScratchThpFaultSampleValidationGateParseError::Samples)?;
    Ok(MappedScratchThpFaultSampleValidationGate::from_samples(
        samples,
    ))
}

/// Parses a pinned scratch validation gate verdict line.
///
/// The expected format is `pinned_scratch_validation_gate=<status> reason=<reason>`.
///
/// # Errors
///
/// Returns an error when the line is missing required tokens, contains duplicate
/// tokens, contains unsupported tokens, uses an unknown status or reason, or
/// combines a status with an incoherent reason.
pub fn parse_pinned_scratch_validation_gate_line(
    line: &str,
) -> Result<PinnedScratchValidationGateVerdict, PinnedScratchValidationGateLineParseError> {
    let mut status_token = None;
    let mut reason_token = None;

    for token in line.split_whitespace() {
        let Some((key, value)) = token.split_once('=') else {
            return Err(PinnedScratchValidationGateLineParseError::InvalidToken(
                token.to_owned(),
            ));
        };

        match key {
            "pinned_scratch_validation_gate" => {
                if status_token.replace(value).is_some() {
                    return Err(PinnedScratchValidationGateLineParseError::DuplicateStatus);
                }
            }
            "reason" => {
                if reason_token.replace(value).is_some() {
                    return Err(PinnedScratchValidationGateLineParseError::DuplicateReason);
                }
            }
            _ => {
                return Err(PinnedScratchValidationGateLineParseError::InvalidToken(
                    token.to_owned(),
                ));
            }
        }
    }

    let status_token =
        status_token.ok_or(PinnedScratchValidationGateLineParseError::MissingStatus)?;
    let reason_token =
        reason_token.ok_or(PinnedScratchValidationGateLineParseError::MissingReason)?;

    let status =
        PinnedScratchValidationGateStatus::from_str_token(status_token).ok_or_else(|| {
            PinnedScratchValidationGateLineParseError::UnknownStatus(status_token.to_owned())
        })?;
    let reason =
        PinnedScratchValidationGateReason::from_str_token(reason_token).ok_or_else(|| {
            PinnedScratchValidationGateLineParseError::UnknownReason(reason_token.to_owned())
        })?;

    PinnedScratchValidationGateVerdict::from_parts(status, reason)
}

/// Parses a near-GPU pinned scratch validation gate verdict line.
///
/// The expected format is
/// `pinned_scratch_near_gpu_validation_gate=<status> reason=<reason>`.
///
/// # Errors
///
/// Returns an error when the line is missing required tokens, contains duplicate
/// tokens, contains unsupported tokens, uses an unknown status or reason, or
/// combines a status with an incoherent reason.
pub fn parse_pinned_scratch_near_gpu_validation_gate_line(
    line: &str,
) -> Result<
    PinnedScratchNearGpuValidationGateVerdict,
    PinnedScratchNearGpuValidationGateLineParseError,
> {
    let mut status_token = None;
    let mut reason_token = None;

    for token in line.split_whitespace() {
        let Some((key, value)) = token.split_once('=') else {
            return Err(
                PinnedScratchNearGpuValidationGateLineParseError::InvalidToken(token.to_owned()),
            );
        };

        match key {
            "pinned_scratch_near_gpu_validation_gate" => {
                if status_token.replace(value).is_some() {
                    return Err(PinnedScratchNearGpuValidationGateLineParseError::DuplicateStatus);
                }
            }
            "reason" => {
                if reason_token.replace(value).is_some() {
                    return Err(PinnedScratchNearGpuValidationGateLineParseError::DuplicateReason);
                }
            }
            _ => {
                return Err(
                    PinnedScratchNearGpuValidationGateLineParseError::InvalidToken(
                        token.to_owned(),
                    ),
                );
            }
        }
    }

    let status_token =
        status_token.ok_or(PinnedScratchNearGpuValidationGateLineParseError::MissingStatus)?;
    let reason_token =
        reason_token.ok_or(PinnedScratchNearGpuValidationGateLineParseError::MissingReason)?;

    let status = PinnedScratchNearGpuValidationGateStatus::from_str_token(status_token)
        .ok_or_else(|| {
            PinnedScratchNearGpuValidationGateLineParseError::UnknownStatus(status_token.to_owned())
        })?;
    let reason = PinnedScratchNearGpuValidationGateReason::from_str_token(reason_token)
        .ok_or_else(|| {
            PinnedScratchNearGpuValidationGateLineParseError::UnknownReason(reason_token.to_owned())
        })?;

    PinnedScratchNearGpuValidationGateVerdict::from_parts(status, reason)
}

/// Parses a mapped scratch THP validation gate verdict line.
///
/// The expected format is
/// `mapped_scratch_thp_validation_gate=<status> reason=<reason>`.
///
/// # Errors
///
/// Returns an error when the line is missing required tokens, contains duplicate
/// tokens, contains unsupported tokens, uses an unknown status or reason, or
/// combines a status with an incoherent reason.
pub fn parse_mapped_scratch_thp_validation_gate_line(
    line: &str,
) -> Result<MappedScratchThpValidationGateVerdict, MappedScratchThpValidationGateLineParseError> {
    let mut status_token = None;
    let mut reason_token = None;

    for token in line.split_whitespace() {
        let Some((key, value)) = token.split_once('=') else {
            return Err(MappedScratchThpValidationGateLineParseError::InvalidToken(
                token.to_owned(),
            ));
        };

        match key {
            "mapped_scratch_thp_validation_gate" => {
                if status_token.replace(value).is_some() {
                    return Err(MappedScratchThpValidationGateLineParseError::DuplicateStatus);
                }
            }
            "reason" => {
                if reason_token.replace(value).is_some() {
                    return Err(MappedScratchThpValidationGateLineParseError::DuplicateReason);
                }
            }
            _ => {
                return Err(MappedScratchThpValidationGateLineParseError::InvalidToken(
                    token.to_owned(),
                ));
            }
        }
    }

    let status_token =
        status_token.ok_or(MappedScratchThpValidationGateLineParseError::MissingStatus)?;
    let reason_token =
        reason_token.ok_or(MappedScratchThpValidationGateLineParseError::MissingReason)?;

    let status =
        MappedScratchThpValidationGateStatus::from_str_token(status_token).ok_or_else(|| {
            MappedScratchThpValidationGateLineParseError::UnknownStatus(status_token.to_owned())
        })?;
    let reason =
        MappedScratchThpValidationGateReason::from_str_token(reason_token).ok_or_else(|| {
            MappedScratchThpValidationGateLineParseError::UnknownReason(reason_token.to_owned())
        })?;

    MappedScratchThpValidationGateVerdict::from_parts(status, reason)
}

/// Parses a mapped scratch THP fault sample validation gate verdict line.
///
/// The expected format is
/// `mapped_scratch_thp_fault_sample_validation_gate=<status> reason=<reason>`.
///
/// # Errors
///
/// Returns an error when the line is missing required tokens, contains duplicate
/// tokens, contains unsupported tokens, uses an unknown status or reason, or
/// combines a status with an incoherent reason.
pub fn parse_mapped_scratch_thp_fault_sample_validation_gate_line(
    line: &str,
) -> Result<
    MappedScratchThpFaultSampleValidationGateVerdict,
    MappedScratchThpFaultSampleValidationGateLineParseError,
> {
    let mut status_token = None;
    let mut reason_token = None;

    for token in line.split_whitespace() {
        let Some((key, value)) = token.split_once('=') else {
            return Err(
                MappedScratchThpFaultSampleValidationGateLineParseError::InvalidToken(
                    token.to_owned(),
                ),
            );
        };

        match key {
            "mapped_scratch_thp_fault_sample_validation_gate" => {
                if status_token.replace(value).is_some() {
                    return Err(
                        MappedScratchThpFaultSampleValidationGateLineParseError::DuplicateStatus,
                    );
                }
            }
            "reason" => {
                if reason_token.replace(value).is_some() {
                    return Err(
                        MappedScratchThpFaultSampleValidationGateLineParseError::DuplicateReason,
                    );
                }
            }
            _ => {
                return Err(
                    MappedScratchThpFaultSampleValidationGateLineParseError::InvalidToken(
                        token.to_owned(),
                    ),
                );
            }
        }
    }

    let status_token = status_token
        .ok_or(MappedScratchThpFaultSampleValidationGateLineParseError::MissingStatus)?;
    let reason_token = reason_token
        .ok_or(MappedScratchThpFaultSampleValidationGateLineParseError::MissingReason)?;

    let status = MappedScratchThpFaultSampleValidationGateStatus::from_str_token(status_token)
        .ok_or_else(|| {
            MappedScratchThpFaultSampleValidationGateLineParseError::UnknownStatus(
                status_token.to_owned(),
            )
        })?;
    let reason = MappedScratchThpFaultSampleValidationGateReason::from_str_token(reason_token)
        .ok_or_else(|| {
            MappedScratchThpFaultSampleValidationGateLineParseError::UnknownReason(
                reason_token.to_owned(),
            )
        })?;

    MappedScratchThpFaultSampleValidationGateVerdict::from_parts(status, reason)
}

/// Extracts a pinned scratch validation gate verdict from multiline output.
///
/// # Errors
///
/// Returns an error when the output has no pinned scratch validation gate line,
/// has more than one pinned scratch validation gate line, or contains a
/// malformed pinned scratch validation gate line.
pub fn parse_pinned_scratch_validation_gate_output(
    output: &str,
) -> Result<PinnedScratchValidationGateVerdict, PinnedScratchValidationGateOutputParseError> {
    let mut gate = None;

    for line in output.lines().map(str::trim) {
        if !line.starts_with("pinned_scratch_validation_gate=") {
            continue;
        }

        if gate.is_some() {
            return Err(PinnedScratchValidationGateOutputParseError::DuplicateGateLine);
        }

        gate = Some(
            parse_pinned_scratch_validation_gate_line(line)
                .map_err(PinnedScratchValidationGateOutputParseError::Line)?,
        );
    }

    gate.ok_or(PinnedScratchValidationGateOutputParseError::MissingGateLine)
}

/// Extracts a mapped scratch THP validation gate verdict from multiline output.
///
/// # Errors
///
/// Returns an error when the output has no mapped scratch THP validation gate
/// line, has more than one mapped scratch THP validation gate line, or contains
/// a malformed mapped scratch THP validation gate line.
pub fn parse_mapped_scratch_thp_validation_gate_output(
    output: &str,
) -> Result<MappedScratchThpValidationGateVerdict, MappedScratchThpValidationGateOutputParseError> {
    let mut gate = None;

    for line in output.lines().map(str::trim) {
        if !line.starts_with("mapped_scratch_thp_validation_gate=") {
            continue;
        }

        if gate.is_some() {
            return Err(MappedScratchThpValidationGateOutputParseError::DuplicateGateLine);
        }

        gate = Some(
            parse_mapped_scratch_thp_validation_gate_line(line)
                .map_err(MappedScratchThpValidationGateOutputParseError::Line)?,
        );
    }

    gate.ok_or(MappedScratchThpValidationGateOutputParseError::MissingGateLine)
}

/// Extracts a mapped scratch THP fault sample gate verdict from multiline output.
///
/// # Errors
///
/// Returns an error when the output has no mapped scratch THP fault sample gate
/// line, has more than one mapped scratch THP fault sample gate line, or
/// contains a malformed mapped scratch THP fault sample gate line.
pub fn parse_mapped_scratch_thp_fault_sample_validation_gate_output(
    output: &str,
) -> Result<
    MappedScratchThpFaultSampleValidationGateVerdict,
    MappedScratchThpFaultSampleValidationGateOutputParseError,
> {
    let mut gate = None;

    for line in output.lines().map(str::trim) {
        if !line.starts_with("mapped_scratch_thp_fault_sample_validation_gate=") {
            continue;
        }

        if gate.is_some() {
            return Err(
                MappedScratchThpFaultSampleValidationGateOutputParseError::DuplicateGateLine,
            );
        }

        gate = Some(
            parse_mapped_scratch_thp_fault_sample_validation_gate_line(line)
                .map_err(MappedScratchThpFaultSampleValidationGateOutputParseError::Line)?,
        );
    }

    gate.ok_or(MappedScratchThpFaultSampleValidationGateOutputParseError::MissingGateLine)
}

fn pinned_scratch_not_ready_reason(
    probe: &PinnedScratchPoolProbeOutput,
) -> PinnedScratchValidationGateReason {
    if probe.checkout.status != PinnedScratchPoolProbeStatus::Ok {
        return PinnedScratchValidationGateReason::CheckoutFailed;
    }

    match probe.allocation {
        Some(event) if event.status == PinnedScratchPoolProbeStatus::Ok => {}
        _ => return PinnedScratchValidationGateReason::AllocationFailed,
    }
    match probe.release {
        Some(event) if event.status == PinnedScratchPoolProbeStatus::Ok => {}
        _ => return PinnedScratchValidationGateReason::ReleaseFailed,
    }
    match probe.reuse_checkout {
        Some(event) if event.status == PinnedScratchPoolProbeStatus::Ok => {}
        _ => return PinnedScratchValidationGateReason::ReuseCheckoutFailed,
    }
    match probe.reuse_release {
        Some(event) if event.status == PinnedScratchPoolProbeStatus::Ok => {}
        _ => return PinnedScratchValidationGateReason::ReuseReleaseFailed,
    }

    match probe.after_release_stats {
        Some(stats) if stats.idle >= 1 => {}
        _ => return PinnedScratchValidationGateReason::ReleaseDidNotLeaveIdleArena,
    }
    match probe.after_reuse_checkout_stats {
        Some(stats) if stats.reused_arenas >= 1 => {}
        _ => return PinnedScratchValidationGateReason::ReuseNotObserved,
    }
    match probe.after_reuse_release_stats {
        Some(stats) if stats.locked_bytes > 0 => {}
        _ => return PinnedScratchValidationGateReason::LockedBytesMissing,
    }

    PinnedScratchValidationGateReason::Ready
}

fn pinned_scratch_near_gpu_gate_reason(
    probe: &PinnedScratchNearGpuProbeOutput,
) -> PinnedScratchNearGpuValidationGateReason {
    match probe.pool.status {
        PinnedScratchNearGpuProbeStatus::Unavailable => {
            return near_gpu_unavailable_gate_reason(probe.pool.reason.as_deref());
        }
        PinnedScratchNearGpuProbeStatus::Error => {
            return PinnedScratchNearGpuValidationGateReason::ConstructorFailed;
        }
        PinnedScratchNearGpuProbeStatus::Ok => {}
    }

    match probe.checkout {
        Some(event) if event.status == PinnedScratchPoolProbeStatus::Ok => {}
        _ => return PinnedScratchNearGpuValidationGateReason::CheckoutFailed,
    }
    match probe.allocation {
        Some(event) if event.status == PinnedScratchPoolProbeStatus::Ok => {}
        _ => return PinnedScratchNearGpuValidationGateReason::AllocationFailed,
    }
    match probe.release {
        Some(event) if event.status == PinnedScratchPoolProbeStatus::Ok => {}
        _ => return PinnedScratchNearGpuValidationGateReason::ReleaseFailed,
    }
    match probe.after_release_stats {
        Some(stats) if stats.idle >= 1 => {}
        _ => return PinnedScratchNearGpuValidationGateReason::ReleaseDidNotLeaveIdleArena,
    }
    match probe.after_release_stats {
        Some(stats) if stats.locked_bytes > 0 => {}
        _ => return PinnedScratchNearGpuValidationGateReason::LockedBytesMissing,
    }

    PinnedScratchNearGpuValidationGateReason::Ready
}

fn mapped_scratch_thp_gate_reason(
    probe: &MappedScratchThpProbeOutput,
) -> MappedScratchThpValidationGateReason {
    let Some(mode) = probe.mode else {
        return MappedScratchThpValidationGateReason::UnsupportedPlatform;
    };
    if mode != MappedScratchHugePageAdvice::HugePage {
        return MappedScratchThpValidationGateReason::NoHugePageMode;
    }

    match probe.advice_status {
        Some(MappedScratchThpAdviceStatus::Ok) => {}
        Some(MappedScratchThpAdviceStatus::Error) | None => {
            return MappedScratchThpValidationGateReason::AdviceFailed;
        }
    }

    match probe.observation {
        Some(MappedScratchThpObservation::Yes) => MappedScratchThpValidationGateReason::Ready,
        Some(MappedScratchThpObservation::No) => MappedScratchThpValidationGateReason::BasePageSize,
        Some(MappedScratchThpObservation::Unknown) | None => {
            MappedScratchThpValidationGateReason::ObservationUnavailable
        }
    }
}

fn mapped_scratch_thp_fault_sample_gate_reason(
    samples: &MappedScratchThpFaultSamples,
) -> MappedScratchThpFaultSampleValidationGateReason {
    let statuses = [
        samples.default.status,
        samples.hugepage.status,
        samples.no_hugepage.status,
    ];
    if statuses
        .iter()
        .all(|status| *status == MappedScratchThpFaultSampleStatus::Available)
    {
        MappedScratchThpFaultSampleValidationGateReason::Ready
    } else {
        MappedScratchThpFaultSampleValidationGateReason::FaultCountersUnavailable
    }
}

fn near_gpu_unavailable_gate_reason(
    reason: Option<&str>,
) -> PinnedScratchNearGpuValidationGateReason {
    match reason {
        Some("no_gpu_with_numa_node") => {
            PinnedScratchNearGpuValidationGateReason::NoGpuWithNumaNode
        }
        Some("gpu_missing") => PinnedScratchNearGpuValidationGateReason::GpuMissing,
        Some("gpu_numa_node_unavailable") => {
            PinnedScratchNearGpuValidationGateReason::GpuNumaNodeUnavailable
        }
        Some("empty_node_set") => PinnedScratchNearGpuValidationGateReason::EmptyNodeSet,
        Some("multiple_nodes") => PinnedScratchNearGpuValidationGateReason::MultipleNodes,
        Some("unresolved" | "unresolved_locality") | None => {
            PinnedScratchNearGpuValidationGateReason::UnresolvedLocality
        }
        Some(_) => PinnedScratchNearGpuValidationGateReason::UnresolvedLocality,
    }
}

#[cfg(test)]
mod pinned_scratch_tests {
    use super::{
        evaluate_mapped_scratch_thp_fault_sample_validation_output,
        evaluate_mapped_scratch_thp_validation_output,
        evaluate_pinned_scratch_near_gpu_validation_output,
        evaluate_pinned_scratch_validation_output,
        parse_mapped_scratch_thp_fault_sample_validation_gate_line,
        parse_mapped_scratch_thp_fault_sample_validation_gate_output,
        parse_mapped_scratch_thp_validation_gate_line,
        parse_mapped_scratch_thp_validation_gate_output,
        parse_pinned_scratch_near_gpu_validation_gate_line,
        parse_pinned_scratch_validation_gate_line, parse_pinned_scratch_validation_gate_output,
        MappedScratchThpFaultSampleComparisonReason, MappedScratchThpFaultSampleComparisonStatus,
        MappedScratchThpFaultSampleValidationGateLineParseError,
        MappedScratchThpFaultSampleValidationGateOutputParseError,
        MappedScratchThpFaultSampleValidationGateParseError,
        MappedScratchThpFaultSampleValidationGateReason,
        MappedScratchThpFaultSampleValidationGateStatus,
        MappedScratchThpFaultSampleValidationGateVerdict,
        MappedScratchThpValidationGateLineParseError,
        MappedScratchThpValidationGateOutputParseError, MappedScratchThpValidationGateParseError,
        MappedScratchThpValidationGateReason, MappedScratchThpValidationGateStatus,
        MappedScratchThpValidationGateVerdict, PinnedScratchNearGpuValidationGateLineParseError,
        PinnedScratchNearGpuValidationGateParseError, PinnedScratchNearGpuValidationGateReason,
        PinnedScratchNearGpuValidationGateStatus, PinnedScratchNearGpuValidationGateVerdict,
        PinnedScratchValidationGateLineParseError, PinnedScratchValidationGateOutputParseError,
        PinnedScratchValidationGateParseError, PinnedScratchValidationGateReason,
        PinnedScratchValidationGateStatus, PinnedScratchValidationGateVerdict,
    };

    const READY_OUTPUT: &str = "\
arena_capacity=16384
max_locked_bytes=40958
pool_stats phase=initial locked_bytes=0 checked_out=0 idle=0 created_arenas=0 reused_arenas=0 checkout_count=0 release_count=0
pool_checkout=ok handle=0
checked_out_mapping_start=0xffffab7fb000
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

    const NEAR_GPU_READY_OUTPUT: &str = "\
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

    const THP_READY_OUTPUT: &str = "\
mapped_scratch_thp=started mode=hugepage
mapping_start=0xffff8753f000
mapping_len=4198399
base_page_kb=4
thp_advice=ok mode=hugepage
touched=1025
numa_maps=available entries=42
numa_maps_match=containing
kernel_page_kb=2048
thp_observed=yes reason=kernel_page_size
";

    const THP_UNAVAILABLE_OUTPUT: &str = "\
mapped_scratch_thp=started mode=hugepage
mapping_start=0xffff8753f000
mapping_len=4198399
base_page_kb=4
thp_advice=ok mode=hugepage
touched=1025
numa_maps=unavailable
thp_observed=unknown reason=numa_maps_unavailable
";

    const THP_FAULT_SAMPLE_READY_OUTPUT: &str = "\
Gnuplot not found, using plotters backend
fault_sample=default status=available iterations=8 minor_faults_delta=16400 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
fault_sample=hugepage status=available iterations=8 minor_faults_delta=8224 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
fault_sample=no_hugepage status=available iterations=8 minor_faults_delta=16400 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
Benchmarking mapped_scratch_write_touch_4mib_default
";

    const THP_FAULT_SAMPLE_UNAVAILABLE_OUTPUT: &str = "\
fault_sample=default status=available iterations=8 minor_faults_delta=16400 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
fault_sample=hugepage status=unavailable
fault_sample=no_hugepage status=available iterations=8 minor_faults_delta=16400 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
";

    #[test]
    fn reports_ready_pinned_scratch_gate_from_probe_output() {
        let gate = evaluate_pinned_scratch_validation_output(READY_OUTPUT).expect("gate");

        assert_eq!(gate.status, PinnedScratchValidationGateStatus::Ready);
        assert_eq!(gate.reason, PinnedScratchValidationGateReason::Ready);
        assert_eq!(
            gate.to_string(),
            "pinned_scratch_validation_gate=ready reason=ready"
        );
        assert!(gate.is_ready());
    }

    #[test]
    fn reports_ready_pinned_scratch_near_gpu_gate_from_probe_output() {
        let gate = evaluate_pinned_scratch_near_gpu_validation_output(NEAR_GPU_READY_OUTPUT)
            .expect("gate");

        assert_eq!(gate.status, PinnedScratchNearGpuValidationGateStatus::Ready);
        assert_eq!(gate.reason, PinnedScratchNearGpuValidationGateReason::Ready);
        assert_eq!(
            gate.to_string(),
            "pinned_scratch_near_gpu_validation_gate=ready reason=ready"
        );
        assert!(gate.is_ready());
    }

    #[test]
    fn reports_ready_mapped_scratch_thp_gate_from_probe_output() {
        let gate = evaluate_mapped_scratch_thp_validation_output(THP_READY_OUTPUT).expect("gate");

        assert_eq!(gate.status, MappedScratchThpValidationGateStatus::Ready);
        assert_eq!(gate.reason, MappedScratchThpValidationGateReason::Ready);
        assert_eq!(
            gate.to_string(),
            "mapped_scratch_thp_validation_gate=ready reason=ready"
        );
        assert!(gate.is_ready());
    }

    #[test]
    fn reports_unavailable_mapped_scratch_thp_gate_from_probe_output() {
        let gate =
            evaluate_mapped_scratch_thp_validation_output(THP_UNAVAILABLE_OUTPUT).expect("gate");

        assert_eq!(
            gate.status,
            MappedScratchThpValidationGateStatus::Unavailable
        );
        assert_eq!(
            gate.reason,
            MappedScratchThpValidationGateReason::ObservationUnavailable
        );
        assert_eq!(
            gate.to_string(),
            "mapped_scratch_thp_validation_gate=unavailable reason=observation_unavailable"
        );
        assert!(!gate.is_ready());
    }

    #[test]
    fn reports_ready_mapped_scratch_thp_fault_sample_gate_from_benchmark_output() {
        let gate = evaluate_mapped_scratch_thp_fault_sample_validation_output(
            THP_FAULT_SAMPLE_READY_OUTPUT,
        )
        .expect("gate");

        assert_eq!(
            gate.status,
            MappedScratchThpFaultSampleValidationGateStatus::Ready
        );
        assert_eq!(
            gate.reason,
            MappedScratchThpFaultSampleValidationGateReason::Ready
        );
        assert_eq!(
            gate.to_string(),
            "mapped_scratch_thp_fault_sample_validation_gate=ready reason=ready"
        );
        let comparison = gate.comparison_output();
        assert_eq!(
            comparison.status,
            MappedScratchThpFaultSampleComparisonStatus::Available
        );
        assert_eq!(
            comparison.reason,
            MappedScratchThpFaultSampleComparisonReason::Ready
        );
        assert_eq!(
            comparison.to_string(),
            "mapped_scratch_thp_fault_sample_comparison=available reason=ready default_minor_faults_delta=16400 hugepage_minor_faults_delta=8224 no_hugepage_minor_faults_delta=16400 hugepage_vs_default_minor_faults_delta=-8176 hugepage_vs_no_hugepage_minor_faults_delta=-8176 major_faults_observed=false"
        );
        assert!(gate.is_ready());
    }

    #[test]
    fn reports_unavailable_mapped_scratch_thp_fault_sample_gate() {
        let gate = evaluate_mapped_scratch_thp_fault_sample_validation_output(
            THP_FAULT_SAMPLE_UNAVAILABLE_OUTPUT,
        )
        .expect("gate");

        assert_eq!(
            gate.status,
            MappedScratchThpFaultSampleValidationGateStatus::Unavailable
        );
        assert_eq!(
            gate.reason,
            MappedScratchThpFaultSampleValidationGateReason::FaultCountersUnavailable
        );
        assert_eq!(
            gate.to_string(),
            "mapped_scratch_thp_fault_sample_validation_gate=unavailable reason=fault_counters_unavailable"
        );
        let comparison = gate.comparison_output();
        assert_eq!(
            comparison.status,
            MappedScratchThpFaultSampleComparisonStatus::Unavailable
        );
        assert_eq!(
            comparison.reason,
            MappedScratchThpFaultSampleComparisonReason::FaultCountersUnavailable
        );
        assert_eq!(
            comparison.to_string(),
            "mapped_scratch_thp_fault_sample_comparison=unavailable reason=fault_counters_unavailable"
        );
        assert!(!gate.is_ready());
    }

    #[test]
    fn reports_mapped_scratch_thp_fault_sample_parse_errors() {
        let error = evaluate_mapped_scratch_thp_fault_sample_validation_output(
            "fault_sample=default status=unavailable\n",
        )
        .expect_err("missing samples");

        assert!(matches!(
            error,
            MappedScratchThpFaultSampleValidationGateParseError::Samples(_)
        ));
    }

    #[test]
    fn reports_unavailable_pinned_scratch_near_gpu_gate_from_probe_output() {
        let output = "\
topology_nodes=0
topology_pci_devices=0
near_gpu_pool=unavailable reason=no_gpu_with_numa_node
";

        let gate = evaluate_pinned_scratch_near_gpu_validation_output(output).expect("gate");

        assert_eq!(
            gate.status,
            PinnedScratchNearGpuValidationGateStatus::Unavailable
        );
        assert_eq!(
            gate.reason,
            PinnedScratchNearGpuValidationGateReason::NoGpuWithNumaNode
        );
        assert!(!gate.is_ready());
    }

    #[test]
    fn reports_checkout_failure_as_not_ready() {
        let output = "\
arena_capacity=16384
max_locked_bytes=40958
pool_stats phase=initial locked_bytes=0 checked_out=0 idle=0 created_arenas=0 reused_arenas=0 checkout_count=0 release_count=0
pool_checkout=error
pool_checkout_error=pinned scratch pool arena failed
pool_stats phase=checkout_error locked_bytes=0 checked_out=0 idle=0 created_arenas=0 reused_arenas=0 checkout_count=0 release_count=0
";

        let gate = evaluate_pinned_scratch_validation_output(output).expect("gate");

        assert_eq!(gate.status, PinnedScratchValidationGateStatus::NotReady);
        assert_eq!(
            gate.reason,
            PinnedScratchValidationGateReason::CheckoutFailed
        );
        assert!(!gate.is_ready());
    }

    #[test]
    fn reports_mapped_scratch_thp_failures_as_not_ready() {
        let no_hugepage_output = "\
mapped_scratch_thp=started mode=no_hugepage
thp_advice=ok mode=no_hugepage
touched=1025
kernel_page_kb=4
thp_observed=no reason=base_page_size
";
        let gate = evaluate_mapped_scratch_thp_validation_output(no_hugepage_output).expect("gate");
        assert_eq!(gate.status, MappedScratchThpValidationGateStatus::NotReady);
        assert_eq!(
            gate.reason,
            MappedScratchThpValidationGateReason::NoHugePageMode
        );

        let advice_error_output = "\
mapped_scratch_thp=started mode=hugepage
thp_advice=error mode=hugepage
thp_advice_error=madvise failed
";
        let gate =
            evaluate_mapped_scratch_thp_validation_output(advice_error_output).expect("gate");
        assert_eq!(
            gate.reason,
            MappedScratchThpValidationGateReason::AdviceFailed
        );

        let base_page_output = THP_READY_OUTPUT
            .replace("kernel_page_kb=2048", "kernel_page_kb=4")
            .replace(
                "thp_observed=yes reason=kernel_page_size",
                "thp_observed=no reason=base_page_size",
            );
        let gate = evaluate_mapped_scratch_thp_validation_output(&base_page_output).expect("gate");
        assert_eq!(
            gate.reason,
            MappedScratchThpValidationGateReason::BasePageSize
        );
    }

    #[test]
    fn reports_pinned_scratch_near_gpu_failures_as_not_ready() {
        let constructor_error = "\
topology_nodes=2
topology_pci_devices=4
gpu_bdf=0000:65:00.0
arena_capacity=16384
max_locked_bytes=40958
near_gpu_pool=error
near_gpu_pool_error=pinned scratch pool arena failed
";
        let gate =
            evaluate_pinned_scratch_near_gpu_validation_output(constructor_error).expect("gate");
        assert_eq!(
            gate.status,
            PinnedScratchNearGpuValidationGateStatus::NotReady
        );
        assert_eq!(
            gate.reason,
            PinnedScratchNearGpuValidationGateReason::ConstructorFailed
        );

        let checkout_error =
            NEAR_GPU_READY_OUTPUT.replace("pool_checkout=ok handle=0", "pool_checkout=error");
        let gate =
            evaluate_pinned_scratch_near_gpu_validation_output(&checkout_error).expect("gate");
        assert_eq!(
            gate.reason,
            PinnedScratchNearGpuValidationGateReason::CheckoutFailed
        );

        let allocation_error = NEAR_GPU_READY_OUTPUT.replace(
            "checked_out_allocation=ok bytes=256",
            "checked_out_allocation=error",
        );
        let gate =
            evaluate_pinned_scratch_near_gpu_validation_output(&allocation_error).expect("gate");
        assert_eq!(
            gate.reason,
            PinnedScratchNearGpuValidationGateReason::AllocationFailed
        );

        let release_error =
            NEAR_GPU_READY_OUTPUT.replace("pool_release=ok handle=0", "pool_release=error");
        let gate =
            evaluate_pinned_scratch_near_gpu_validation_output(&release_error).expect("gate");
        assert_eq!(
            gate.reason,
            PinnedScratchNearGpuValidationGateReason::ReleaseFailed
        );
    }

    #[test]
    fn reports_pinned_scratch_near_gpu_accounting_failures_as_not_ready() {
        let no_idle_after_release = NEAR_GPU_READY_OUTPUT.replace(
            "phase=after_release locked_bytes=20479 checked_out=0 idle=1",
            "phase=after_release locked_bytes=20479 checked_out=0 idle=0",
        );
        let gate = evaluate_pinned_scratch_near_gpu_validation_output(&no_idle_after_release)
            .expect("gate");
        assert_eq!(
            gate.reason,
            PinnedScratchNearGpuValidationGateReason::ReleaseDidNotLeaveIdleArena
        );

        let no_locked_bytes = NEAR_GPU_READY_OUTPUT.replace(
            "phase=after_release locked_bytes=20479",
            "phase=after_release locked_bytes=0",
        );
        let gate =
            evaluate_pinned_scratch_near_gpu_validation_output(&no_locked_bytes).expect("gate");
        assert_eq!(
            gate.reason,
            PinnedScratchNearGpuValidationGateReason::LockedBytesMissing
        );
    }

    #[test]
    fn reports_near_gpu_probe_parse_errors() {
        let error = evaluate_pinned_scratch_near_gpu_validation_output(
            "topology_pci_devices=0\nnear_gpu_pool=unavailable reason=no_gpu_with_numa_node\n",
        )
        .expect_err("missing topology nodes");

        assert!(matches!(
            error,
            PinnedScratchNearGpuValidationGateParseError::Probe(_)
        ));
    }

    #[test]
    fn reports_mapped_scratch_thp_unavailable_platform() {
        let gate = evaluate_mapped_scratch_thp_validation_output(
            "mapped_scratch_thp=unsupported-platform\n",
        )
        .expect("gate");

        assert_eq!(
            gate.status,
            MappedScratchThpValidationGateStatus::Unavailable
        );
        assert_eq!(
            gate.reason,
            MappedScratchThpValidationGateReason::UnsupportedPlatform
        );
    }

    #[test]
    fn reports_mapped_scratch_thp_probe_parse_errors() {
        let error = evaluate_mapped_scratch_thp_validation_output(
            "mapped_scratch_thp=started mode=hugepage\nthp_advice=ok mode=hugepage\n",
        )
        .expect_err("missing touched");

        assert!(matches!(
            error,
            MappedScratchThpValidationGateParseError::Probe(_)
        ));
    }

    #[test]
    fn reports_event_failures_as_not_ready() {
        let allocation_error = READY_OUTPUT.replace(
            "checked_out_allocation=ok bytes=256",
            "checked_out_allocation=error",
        );
        let gate = evaluate_pinned_scratch_validation_output(&allocation_error).expect("gate");
        assert_eq!(
            gate.reason,
            PinnedScratchValidationGateReason::AllocationFailed
        );

        let release_error = READY_OUTPUT.replace("pool_release=ok handle=0", "pool_release=error");
        let gate = evaluate_pinned_scratch_validation_output(&release_error).expect("gate");
        assert_eq!(
            gate.reason,
            PinnedScratchValidationGateReason::ReleaseFailed
        );

        let reuse_checkout_error = READY_OUTPUT.replace(
            "pool_reuse_checkout=ok handle=1",
            "pool_reuse_checkout=error",
        );
        let gate = evaluate_pinned_scratch_validation_output(&reuse_checkout_error).expect("gate");
        assert_eq!(
            gate.reason,
            PinnedScratchValidationGateReason::ReuseCheckoutFailed
        );

        let reuse_release_error =
            READY_OUTPUT.replace("pool_reuse_release=ok handle=1", "pool_reuse_release=error");
        let gate = evaluate_pinned_scratch_validation_output(&reuse_release_error).expect("gate");
        assert_eq!(
            gate.reason,
            PinnedScratchValidationGateReason::ReuseReleaseFailed
        );
    }

    #[test]
    fn reports_accounting_failures_as_not_ready() {
        let no_idle_after_release = READY_OUTPUT.replace(
            "phase=after_release locked_bytes=20479 checked_out=0 idle=1",
            "phase=after_release locked_bytes=20479 checked_out=0 idle=0",
        );
        let gate = evaluate_pinned_scratch_validation_output(&no_idle_after_release).expect("gate");
        assert_eq!(
            gate.reason,
            PinnedScratchValidationGateReason::ReleaseDidNotLeaveIdleArena
        );

        let no_reuse_observed = READY_OUTPUT.replace(
            "phase=after_reuse_checkout locked_bytes=20479 checked_out=1 idle=0 created_arenas=1 reused_arenas=1",
            "phase=after_reuse_checkout locked_bytes=20479 checked_out=1 idle=0 created_arenas=1 reused_arenas=0",
        );
        let gate = evaluate_pinned_scratch_validation_output(&no_reuse_observed).expect("gate");
        assert_eq!(
            gate.reason,
            PinnedScratchValidationGateReason::ReuseNotObserved
        );

        let no_locked_bytes = READY_OUTPUT.replace(
            "phase=after_reuse_release locked_bytes=20479",
            "phase=after_reuse_release locked_bytes=0",
        );
        let gate = evaluate_pinned_scratch_validation_output(&no_locked_bytes).expect("gate");
        assert_eq!(
            gate.reason,
            PinnedScratchValidationGateReason::LockedBytesMissing
        );
    }

    #[test]
    fn reports_probe_parse_errors() {
        let error = evaluate_pinned_scratch_validation_output("pool_checkout=ok handle=0\n")
            .expect_err("missing initial stats");

        assert!(matches!(
            error,
            PinnedScratchValidationGateParseError::Probe(_)
        ));
    }

    #[test]
    fn parses_pinned_scratch_validation_gate_lines() {
        assert_eq!(
            parse_pinned_scratch_validation_gate_line(
                "pinned_scratch_validation_gate=ready reason=ready"
            )
            .expect("ready"),
            PinnedScratchValidationGateVerdict {
                status: PinnedScratchValidationGateStatus::Ready,
                reason: PinnedScratchValidationGateReason::Ready,
            }
        );
        assert_eq!(
            parse_pinned_scratch_validation_gate_line(
                "pinned_scratch_validation_gate=not_ready reason=reuse_not_observed"
            )
            .expect("not ready"),
            PinnedScratchValidationGateVerdict {
                status: PinnedScratchValidationGateStatus::NotReady,
                reason: PinnedScratchValidationGateReason::ReuseNotObserved,
            }
        );
        assert_eq!(
            PinnedScratchValidationGateStatus::Ready.to_string(),
            "ready"
        );
        assert_eq!(
            PinnedScratchValidationGateReason::LockedBytesMissing.to_string(),
            "locked_bytes_missing"
        );
        assert!(PinnedScratchValidationGateVerdict {
            status: PinnedScratchValidationGateStatus::NotReady,
            reason: PinnedScratchValidationGateReason::AllocationFailed,
        }
        .is_consistent());
        assert!(!PinnedScratchValidationGateVerdict {
            status: PinnedScratchValidationGateStatus::Ready,
            reason: PinnedScratchValidationGateReason::AllocationFailed,
        }
        .is_consistent());
    }

    #[test]
    fn parses_pinned_scratch_near_gpu_validation_gate_lines() {
        assert_eq!(
            parse_pinned_scratch_near_gpu_validation_gate_line(
                "pinned_scratch_near_gpu_validation_gate=ready reason=ready"
            )
            .expect("ready"),
            PinnedScratchNearGpuValidationGateVerdict {
                status: PinnedScratchNearGpuValidationGateStatus::Ready,
                reason: PinnedScratchNearGpuValidationGateReason::Ready,
            }
        );
        assert_eq!(
            parse_pinned_scratch_near_gpu_validation_gate_line(
                "pinned_scratch_near_gpu_validation_gate=unavailable reason=gpu_missing"
            )
            .expect("unavailable"),
            PinnedScratchNearGpuValidationGateVerdict {
                status: PinnedScratchNearGpuValidationGateStatus::Unavailable,
                reason: PinnedScratchNearGpuValidationGateReason::GpuMissing,
            }
        );
        assert_eq!(
            parse_pinned_scratch_near_gpu_validation_gate_line(
                "pinned_scratch_near_gpu_validation_gate=not_ready reason=constructor_failed"
            )
            .expect("not ready"),
            PinnedScratchNearGpuValidationGateVerdict {
                status: PinnedScratchNearGpuValidationGateStatus::NotReady,
                reason: PinnedScratchNearGpuValidationGateReason::ConstructorFailed,
            }
        );
        assert_eq!(
            PinnedScratchNearGpuValidationGateStatus::Unavailable.to_string(),
            "unavailable"
        );
        assert_eq!(
            PinnedScratchNearGpuValidationGateReason::GpuNumaNodeUnavailable.to_string(),
            "gpu_numa_node_unavailable"
        );
    }

    #[test]
    fn parses_mapped_scratch_thp_validation_gate_lines() {
        assert_eq!(
            parse_mapped_scratch_thp_validation_gate_line(
                "mapped_scratch_thp_validation_gate=ready reason=ready"
            )
            .expect("ready"),
            MappedScratchThpValidationGateVerdict {
                status: MappedScratchThpValidationGateStatus::Ready,
                reason: MappedScratchThpValidationGateReason::Ready,
            }
        );
        assert_eq!(
            parse_mapped_scratch_thp_validation_gate_line(
                "mapped_scratch_thp_validation_gate=unavailable reason=observation_unavailable"
            )
            .expect("unavailable"),
            MappedScratchThpValidationGateVerdict {
                status: MappedScratchThpValidationGateStatus::Unavailable,
                reason: MappedScratchThpValidationGateReason::ObservationUnavailable,
            }
        );
        assert_eq!(
            parse_mapped_scratch_thp_validation_gate_line(
                "mapped_scratch_thp_validation_gate=not_ready reason=base_page_size"
            )
            .expect("not ready"),
            MappedScratchThpValidationGateVerdict {
                status: MappedScratchThpValidationGateStatus::NotReady,
                reason: MappedScratchThpValidationGateReason::BasePageSize,
            }
        );
        assert_eq!(
            MappedScratchThpValidationGateStatus::Unavailable.to_string(),
            "unavailable"
        );
        assert_eq!(
            MappedScratchThpValidationGateReason::AdviceFailed.to_string(),
            "advice_failed"
        );
    }

    #[test]
    fn parses_mapped_scratch_thp_fault_sample_validation_gate_lines() {
        assert_eq!(
            parse_mapped_scratch_thp_fault_sample_validation_gate_line(
                "mapped_scratch_thp_fault_sample_validation_gate=ready reason=ready"
            )
            .expect("ready"),
            MappedScratchThpFaultSampleValidationGateVerdict {
                status: MappedScratchThpFaultSampleValidationGateStatus::Ready,
                reason: MappedScratchThpFaultSampleValidationGateReason::Ready,
            }
        );
        assert_eq!(
            parse_mapped_scratch_thp_fault_sample_validation_gate_line(
                "mapped_scratch_thp_fault_sample_validation_gate=unavailable reason=fault_counters_unavailable"
            )
            .expect("unavailable"),
            MappedScratchThpFaultSampleValidationGateVerdict {
                status: MappedScratchThpFaultSampleValidationGateStatus::Unavailable,
                reason: MappedScratchThpFaultSampleValidationGateReason::FaultCountersUnavailable,
            }
        );
        assert_eq!(
            MappedScratchThpFaultSampleValidationGateStatus::Unavailable.to_string(),
            "unavailable"
        );
        assert_eq!(
            MappedScratchThpFaultSampleValidationGateReason::FaultCountersUnavailable.to_string(),
            "fault_counters_unavailable"
        );
        assert!(MappedScratchThpFaultSampleValidationGateVerdict {
            status: MappedScratchThpFaultSampleValidationGateStatus::Unavailable,
            reason: MappedScratchThpFaultSampleValidationGateReason::FaultCountersUnavailable,
        }
        .is_consistent());
        assert!(!MappedScratchThpFaultSampleValidationGateVerdict {
            status: MappedScratchThpFaultSampleValidationGateStatus::Ready,
            reason: MappedScratchThpFaultSampleValidationGateReason::FaultCountersUnavailable,
        }
        .is_consistent());
    }

    #[test]
    fn rejects_invalid_pinned_scratch_near_gpu_validation_gate_lines() {
        assert_eq!(
            parse_pinned_scratch_near_gpu_validation_gate_line("reason=ready")
                .expect_err("missing status"),
            PinnedScratchNearGpuValidationGateLineParseError::MissingStatus
        );
        assert_eq!(
            parse_pinned_scratch_near_gpu_validation_gate_line(
                "pinned_scratch_near_gpu_validation_gate=ready"
            )
            .expect_err("missing reason"),
            PinnedScratchNearGpuValidationGateLineParseError::MissingReason
        );
        assert_eq!(
            parse_pinned_scratch_near_gpu_validation_gate_line(
                "pinned_scratch_near_gpu_validation_gate=maybe reason=ready"
            )
            .expect_err("unknown status"),
            PinnedScratchNearGpuValidationGateLineParseError::UnknownStatus("maybe".to_owned())
        );
        assert_eq!(
            parse_pinned_scratch_near_gpu_validation_gate_line(
                "pinned_scratch_near_gpu_validation_gate=ready reason=gpu_missing"
            )
            .expect_err("inconsistent verdict"),
            PinnedScratchNearGpuValidationGateLineParseError::InconsistentVerdict {
                status: PinnedScratchNearGpuValidationGateStatus::Ready,
                reason: PinnedScratchNearGpuValidationGateReason::GpuMissing,
            }
        );
    }

    #[test]
    fn rejects_invalid_mapped_scratch_thp_validation_gate_lines() {
        assert_eq!(
            parse_mapped_scratch_thp_validation_gate_line("reason=ready")
                .expect_err("missing status"),
            MappedScratchThpValidationGateLineParseError::MissingStatus
        );
        assert_eq!(
            parse_mapped_scratch_thp_validation_gate_line(
                "mapped_scratch_thp_validation_gate=ready"
            )
            .expect_err("missing reason"),
            MappedScratchThpValidationGateLineParseError::MissingReason
        );
        assert_eq!(
            parse_mapped_scratch_thp_validation_gate_line(
                "mapped_scratch_thp_validation_gate=maybe reason=ready"
            )
            .expect_err("unknown status"),
            MappedScratchThpValidationGateLineParseError::UnknownStatus("maybe".to_owned())
        );
        assert_eq!(
            parse_mapped_scratch_thp_validation_gate_line(
                "mapped_scratch_thp_validation_gate=ready reason=observation_unavailable"
            )
            .expect_err("inconsistent verdict"),
            MappedScratchThpValidationGateLineParseError::InconsistentVerdict {
                status: MappedScratchThpValidationGateStatus::Ready,
                reason: MappedScratchThpValidationGateReason::ObservationUnavailable,
            }
        );
    }

    #[test]
    fn rejects_invalid_mapped_scratch_thp_fault_sample_validation_gate_lines() {
        assert_eq!(
            parse_mapped_scratch_thp_fault_sample_validation_gate_line("reason=ready")
                .expect_err("missing status"),
            MappedScratchThpFaultSampleValidationGateLineParseError::MissingStatus
        );
        assert_eq!(
            parse_mapped_scratch_thp_fault_sample_validation_gate_line(
                "mapped_scratch_thp_fault_sample_validation_gate=ready"
            )
            .expect_err("missing reason"),
            MappedScratchThpFaultSampleValidationGateLineParseError::MissingReason
        );
        assert_eq!(
            parse_mapped_scratch_thp_fault_sample_validation_gate_line(
                "mapped_scratch_thp_fault_sample_validation_gate=maybe reason=ready"
            )
            .expect_err("unknown status"),
            MappedScratchThpFaultSampleValidationGateLineParseError::UnknownStatus(
                "maybe".to_owned()
            )
        );
        assert_eq!(
            parse_mapped_scratch_thp_fault_sample_validation_gate_line(
                "mapped_scratch_thp_fault_sample_validation_gate=ready reason=maybe"
            )
            .expect_err("unknown reason"),
            MappedScratchThpFaultSampleValidationGateLineParseError::UnknownReason(
                "maybe".to_owned()
            )
        );
        assert_eq!(
            parse_mapped_scratch_thp_fault_sample_validation_gate_line(
                "mapped_scratch_thp_fault_sample_validation_gate=ready reason=ready extra=true"
            )
            .expect_err("extra token"),
            MappedScratchThpFaultSampleValidationGateLineParseError::InvalidToken(
                "extra=true".to_owned()
            )
        );
        assert_eq!(
            parse_mapped_scratch_thp_fault_sample_validation_gate_line(
                "mapped_scratch_thp_fault_sample_validation_gate=ready mapped_scratch_thp_fault_sample_validation_gate=unavailable reason=ready"
            )
            .expect_err("duplicate status"),
            MappedScratchThpFaultSampleValidationGateLineParseError::DuplicateStatus
        );
        assert_eq!(
            parse_mapped_scratch_thp_fault_sample_validation_gate_line(
                "mapped_scratch_thp_fault_sample_validation_gate=ready reason=ready reason=fault_counters_unavailable"
            )
            .expect_err("duplicate reason"),
            MappedScratchThpFaultSampleValidationGateLineParseError::DuplicateReason
        );
        assert_eq!(
            parse_mapped_scratch_thp_fault_sample_validation_gate_line(
                "mapped_scratch_thp_fault_sample_validation_gate=ready reason=fault_counters_unavailable"
            )
            .expect_err("inconsistent verdict"),
            MappedScratchThpFaultSampleValidationGateLineParseError::InconsistentVerdict {
                status: MappedScratchThpFaultSampleValidationGateStatus::Ready,
                reason: MappedScratchThpFaultSampleValidationGateReason::FaultCountersUnavailable,
            }
        );
    }

    #[test]
    fn rejects_invalid_pinned_scratch_validation_gate_lines() {
        assert_eq!(
            parse_pinned_scratch_validation_gate_line("reason=ready").expect_err("missing status"),
            PinnedScratchValidationGateLineParseError::MissingStatus
        );
        assert_eq!(
            parse_pinned_scratch_validation_gate_line("pinned_scratch_validation_gate=ready")
                .expect_err("missing reason"),
            PinnedScratchValidationGateLineParseError::MissingReason
        );
        assert_eq!(
            parse_pinned_scratch_validation_gate_line(
                "pinned_scratch_validation_gate=maybe reason=ready"
            )
            .expect_err("unknown status"),
            PinnedScratchValidationGateLineParseError::UnknownStatus("maybe".to_owned())
        );
        assert_eq!(
            parse_pinned_scratch_validation_gate_line(
                "pinned_scratch_validation_gate=ready reason=maybe"
            )
            .expect_err("unknown reason"),
            PinnedScratchValidationGateLineParseError::UnknownReason("maybe".to_owned())
        );
        assert_eq!(
            parse_pinned_scratch_validation_gate_line(
                "pinned_scratch_validation_gate=ready reason=ready extra=true"
            )
            .expect_err("extra token"),
            PinnedScratchValidationGateLineParseError::InvalidToken("extra=true".to_owned())
        );
        assert_eq!(
            parse_pinned_scratch_validation_gate_line(
                "pinned_scratch_validation_gate=ready pinned_scratch_validation_gate=not_ready reason=ready"
            )
            .expect_err("duplicate status"),
            PinnedScratchValidationGateLineParseError::DuplicateStatus
        );
        assert_eq!(
            parse_pinned_scratch_validation_gate_line(
                "pinned_scratch_validation_gate=ready reason=ready reason=checkout_failed"
            )
            .expect_err("duplicate reason"),
            PinnedScratchValidationGateLineParseError::DuplicateReason
        );
        assert_eq!(
            parse_pinned_scratch_validation_gate_line(
                "pinned_scratch_validation_gate=ready reason=checkout_failed"
            )
            .expect_err("inconsistent verdict"),
            PinnedScratchValidationGateLineParseError::InconsistentVerdict {
                status: PinnedScratchValidationGateStatus::Ready,
                reason: PinnedScratchValidationGateReason::CheckoutFailed,
            }
        );
    }

    #[test]
    fn parses_pinned_scratch_validation_gate_from_output() {
        let output = "\
pool_checkout=ok handle=0
pinned_scratch_validation_gate=ready reason=ready
";

        assert_eq!(
            parse_pinned_scratch_validation_gate_output(output).expect("gate"),
            PinnedScratchValidationGateVerdict {
                status: PinnedScratchValidationGateStatus::Ready,
                reason: PinnedScratchValidationGateReason::Ready,
            }
        );
    }

    #[test]
    fn parses_mapped_scratch_thp_validation_gate_from_output() {
        let output = "\
mapped_scratch_thp=started mode=hugepage
mapped_scratch_thp_validation_gate=unavailable reason=observation_unavailable
";

        assert_eq!(
            parse_mapped_scratch_thp_validation_gate_output(output).expect("gate"),
            MappedScratchThpValidationGateVerdict {
                status: MappedScratchThpValidationGateStatus::Unavailable,
                reason: MappedScratchThpValidationGateReason::ObservationUnavailable,
            }
        );
    }

    #[test]
    fn parses_mapped_scratch_thp_fault_sample_validation_gate_from_output() {
        let output = "\
fault_sample=default status=available iterations=8 minor_faults_delta=1 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
mapped_scratch_thp_fault_sample_validation_gate=ready reason=ready
";

        assert_eq!(
            parse_mapped_scratch_thp_fault_sample_validation_gate_output(output).expect("gate"),
            MappedScratchThpFaultSampleValidationGateVerdict {
                status: MappedScratchThpFaultSampleValidationGateStatus::Ready,
                reason: MappedScratchThpFaultSampleValidationGateReason::Ready,
            }
        );
    }

    #[test]
    fn rejects_invalid_pinned_scratch_validation_gate_output() {
        assert_eq!(
            parse_pinned_scratch_validation_gate_output("pool_checkout=ok handle=0\n")
                .expect_err("missing gate"),
            PinnedScratchValidationGateOutputParseError::MissingGateLine
        );
        assert_eq!(
            parse_pinned_scratch_validation_gate_output(
                "pinned_scratch_validation_gate=ready reason=ready\npinned_scratch_validation_gate=not_ready reason=checkout_failed\n"
            )
            .expect_err("duplicate gate"),
            PinnedScratchValidationGateOutputParseError::DuplicateGateLine
        );
        assert_eq!(
            parse_pinned_scratch_validation_gate_output(
                "pinned_scratch_validation_gate=maybe reason=ready\n"
            )
            .expect_err("bad gate"),
            PinnedScratchValidationGateOutputParseError::Line(
                PinnedScratchValidationGateLineParseError::UnknownStatus("maybe".to_owned())
            )
        );
    }

    #[test]
    fn rejects_invalid_mapped_scratch_thp_validation_gate_output() {
        assert_eq!(
            parse_mapped_scratch_thp_validation_gate_output("mapped_scratch_thp=started\n")
                .expect_err("missing gate"),
            MappedScratchThpValidationGateOutputParseError::MissingGateLine
        );
        assert_eq!(
            parse_mapped_scratch_thp_validation_gate_output(
                "mapped_scratch_thp_validation_gate=ready reason=ready\nmapped_scratch_thp_validation_gate=not_ready reason=base_page_size\n"
            )
            .expect_err("duplicate gate"),
            MappedScratchThpValidationGateOutputParseError::DuplicateGateLine
        );
        assert_eq!(
            parse_mapped_scratch_thp_validation_gate_output(
                "mapped_scratch_thp_validation_gate=maybe reason=ready\n"
            )
            .expect_err("bad gate"),
            MappedScratchThpValidationGateOutputParseError::Line(
                MappedScratchThpValidationGateLineParseError::UnknownStatus("maybe".to_owned())
            )
        );
    }

    #[test]
    fn rejects_invalid_mapped_scratch_thp_fault_sample_validation_gate_output() {
        assert_eq!(
            parse_mapped_scratch_thp_fault_sample_validation_gate_output(
                "fault_sample=default status=unavailable\n"
            )
            .expect_err("missing gate"),
            MappedScratchThpFaultSampleValidationGateOutputParseError::MissingGateLine
        );
        assert_eq!(
            parse_mapped_scratch_thp_fault_sample_validation_gate_output(
                "mapped_scratch_thp_fault_sample_validation_gate=ready reason=ready\nmapped_scratch_thp_fault_sample_validation_gate=unavailable reason=fault_counters_unavailable\n"
            )
            .expect_err("duplicate gate"),
            MappedScratchThpFaultSampleValidationGateOutputParseError::DuplicateGateLine
        );
        assert_eq!(
            parse_mapped_scratch_thp_fault_sample_validation_gate_output(
                "mapped_scratch_thp_fault_sample_validation_gate=maybe reason=ready\n"
            )
            .expect_err("bad line"),
            MappedScratchThpFaultSampleValidationGateOutputParseError::Line(
                MappedScratchThpFaultSampleValidationGateLineParseError::UnknownStatus(
                    "maybe".to_owned()
                )
            )
        );
    }
}

#[cfg(target_os = "linux")]
pub mod linux {
    //! Linux placement validation gate helpers.

    use std::fmt;

    use locus_observe::{
        parse_numa_placement_proof_output, parse_numa_placement_readiness_output,
        NumaPlacementProof, NumaPlacementProofOutputParseError, NumaPlacementProofStatus,
        NumaPlacementReadinessOutputParseError, NumaPlacementValidationReadiness,
    };
    use locus_sys::linux::{
        parse_linux_numa_policy_readiness_output, LinuxNumaPolicyReadiness,
        LinuxNumaPolicyReadinessOutputParseError,
    };

    /// Probe outputs required for a combined placement validation gate.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PlacementValidationOutputs<'a> {
        /// Full output from `locus-sys --example mbind_region`.
        pub memory_policy_output: &'a str,
        /// Full output from `locus-observe --example locality_environment`.
        pub placement_readiness_output: &'a str,
        /// Full output from `locus-alloc --example mapped_scratch_bind`.
        pub placement_proof_output: &'a str,
    }

    /// Combined placement validation gate status.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum PlacementValidationGateStatus {
        /// All required proof conditions are satisfied.
        Verified,
        /// The environment is not ready to prove placement.
        NotReady,
        /// Evidence was available but did not prove placement.
        Unverified,
        /// Primary placement proof evidence was unavailable.
        Unavailable,
    }

    impl PlacementValidationGateStatus {
        /// Returns a stable machine-readable status string.
        #[must_use]
        pub fn as_str(self) -> &'static str {
            match self {
                Self::Verified => "verified",
                Self::NotReady => "not_ready",
                Self::Unverified => "unverified",
                Self::Unavailable => "unavailable",
            }
        }

        /// Parses a stable machine-readable status string.
        #[must_use]
        pub fn from_str_token(value: &str) -> Option<Self> {
            match value {
                "verified" => Some(Self::Verified),
                "not_ready" => Some(Self::NotReady),
                "unverified" => Some(Self::Unverified),
                "unavailable" => Some(Self::Unavailable),
                _ => None,
            }
        }
    }

    impl fmt::Display for PlacementValidationGateStatus {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.as_str())
        }
    }

    /// Reason for the combined placement validation gate status.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum PlacementValidationGateReason {
        /// All required proof conditions are satisfied.
        Verified,
        /// Linux memory policy application readiness was not ready.
        MemoryPolicyNotReady,
        /// Locality evidence readiness was not ready.
        PlacementEvidenceNotReady,
        /// Placement proof evidence was unavailable.
        PlacementProofUnavailable,
        /// Placement proof evidence was present but unverified.
        PlacementProofUnverified,
    }

    impl PlacementValidationGateReason {
        /// Returns a stable machine-readable reason string.
        #[must_use]
        pub fn as_str(self) -> &'static str {
            match self {
                Self::Verified => "verified",
                Self::MemoryPolicyNotReady => "memory_policy_not_ready",
                Self::PlacementEvidenceNotReady => "placement_evidence_not_ready",
                Self::PlacementProofUnavailable => "placement_proof_unavailable",
                Self::PlacementProofUnverified => "placement_proof_unverified",
            }
        }

        /// Parses a stable machine-readable reason string.
        #[must_use]
        pub fn from_str_token(value: &str) -> Option<Self> {
            match value {
                "verified" => Some(Self::Verified),
                "memory_policy_not_ready" => Some(Self::MemoryPolicyNotReady),
                "placement_evidence_not_ready" => Some(Self::PlacementEvidenceNotReady),
                "placement_proof_unavailable" => Some(Self::PlacementProofUnavailable),
                "placement_proof_unverified" => Some(Self::PlacementProofUnverified),
                _ => None,
            }
        }
    }

    impl fmt::Display for PlacementValidationGateReason {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.as_str())
        }
    }

    /// Parsed combined placement validation gate verdict.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PlacementValidationGate {
        /// Final gate status.
        pub status: PlacementValidationGateStatus,
        /// Reason for the status.
        pub reason: PlacementValidationGateReason,
        /// Parsed Linux memory-policy readiness.
        pub memory_policy: LinuxNumaPolicyReadiness,
        /// Parsed locality evidence readiness.
        pub placement_readiness: NumaPlacementValidationReadiness,
        /// Parsed mapped arena placement proof.
        pub placement_proof: NumaPlacementProof,
    }

    /// Status and reason parsed from a placement validation gate line.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PlacementValidationGateVerdict {
        /// Parsed gate status.
        pub status: PlacementValidationGateStatus,
        /// Parsed gate reason.
        pub reason: PlacementValidationGateReason,
    }

    impl fmt::Display for PlacementValidationGate {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "placement_validation_gate={} reason={}",
                self.status, self.reason
            )
        }
    }

    impl fmt::Display for PlacementValidationGateVerdict {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "placement_validation_gate={} reason={}",
                self.status, self.reason
            )
        }
    }

    impl PlacementValidationGateVerdict {
        /// Builds a verdict only when the status and reason are coherent.
        ///
        /// # Errors
        ///
        /// Returns an error when the reason is not valid for the status.
        pub fn from_parts(
            status: PlacementValidationGateStatus,
            reason: PlacementValidationGateReason,
        ) -> Result<Self, PlacementValidationGateLineParseError> {
            let verdict = Self { status, reason };
            if verdict.is_consistent() {
                Ok(verdict)
            } else {
                Err(PlacementValidationGateLineParseError::InconsistentVerdict { status, reason })
            }
        }

        /// Returns true when the reason is valid for the status.
        #[must_use]
        pub fn is_consistent(self) -> bool {
            matches!(
                (self.status, self.reason),
                (
                    PlacementValidationGateStatus::Verified,
                    PlacementValidationGateReason::Verified
                ) | (
                    PlacementValidationGateStatus::NotReady,
                    PlacementValidationGateReason::MemoryPolicyNotReady
                        | PlacementValidationGateReason::PlacementEvidenceNotReady
                ) | (
                    PlacementValidationGateStatus::Unverified,
                    PlacementValidationGateReason::PlacementProofUnverified
                ) | (
                    PlacementValidationGateStatus::Unavailable,
                    PlacementValidationGateReason::PlacementProofUnavailable
                )
            )
        }
    }

    /// Error returned when parsing a placement validation gate line.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum PlacementValidationGateLineParseError {
        /// The line does not contain a `placement_validation_gate=` token.
        MissingStatus,
        /// The line does not contain a `reason=` token.
        MissingReason,
        /// The line contains a duplicate `placement_validation_gate=` token.
        DuplicateStatus,
        /// The line contains a duplicate `reason=` token.
        DuplicateReason,
        /// The line contains a token outside the placement validation gate schema.
        InvalidToken(String),
        /// The gate status token is not recognized.
        UnknownStatus(String),
        /// The gate reason token is not recognized.
        UnknownReason(String),
        /// The status and reason tokens are individually valid but inconsistent together.
        InconsistentVerdict {
            /// Parsed gate status.
            status: PlacementValidationGateStatus,
            /// Parsed gate reason.
            reason: PlacementValidationGateReason,
        },
    }

    impl fmt::Display for PlacementValidationGateLineParseError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::MissingStatus => f.write_str("missing placement_validation_gate token"),
                Self::MissingReason => f.write_str("missing reason token"),
                Self::DuplicateStatus => f.write_str("duplicate placement_validation_gate token"),
                Self::DuplicateReason => f.write_str("duplicate reason token"),
                Self::InvalidToken(token) => {
                    write!(f, "invalid placement validation gate token: {token}")
                }
                Self::UnknownStatus(status) => {
                    write!(f, "unknown placement validation gate status: {status}")
                }
                Self::UnknownReason(reason) => {
                    write!(f, "unknown placement validation gate reason: {reason}")
                }
                Self::InconsistentVerdict { status, reason } => {
                    write!(
                        f,
                        "inconsistent placement validation gate: {status} {reason}"
                    )
                }
            }
        }
    }

    impl std::error::Error for PlacementValidationGateLineParseError {}

    /// Error returned when extracting a placement validation gate from multiline output.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum PlacementValidationGateOutputParseError {
        /// The output does not contain a `placement_validation_gate=` line.
        MissingGateLine,
        /// The output contains more than one `placement_validation_gate=` line.
        DuplicateGateLine,
        /// The discovered gate line is malformed.
        Line(PlacementValidationGateLineParseError),
    }

    impl fmt::Display for PlacementValidationGateOutputParseError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::MissingGateLine => f.write_str("missing placement_validation_gate line"),
                Self::DuplicateGateLine => f.write_str("duplicate placement_validation_gate line"),
                Self::Line(source) => {
                    write!(f, "invalid placement_validation_gate line: {source}")
                }
            }
        }
    }

    impl std::error::Error for PlacementValidationGateOutputParseError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            match self {
                Self::Line(source) => Some(source),
                Self::MissingGateLine | Self::DuplicateGateLine => None,
            }
        }
    }

    impl PlacementValidationGate {
        /// Builds a combined gate from parsed verdicts.
        #[must_use]
        pub fn from_verdicts(
            memory_policy: LinuxNumaPolicyReadiness,
            placement_readiness: NumaPlacementValidationReadiness,
            placement_proof: NumaPlacementProof,
        ) -> Self {
            let (status, reason) = if !memory_policy.is_ready() {
                (
                    PlacementValidationGateStatus::NotReady,
                    PlacementValidationGateReason::MemoryPolicyNotReady,
                )
            } else if !placement_readiness.is_ready() {
                (
                    PlacementValidationGateStatus::NotReady,
                    PlacementValidationGateReason::PlacementEvidenceNotReady,
                )
            } else {
                match placement_proof.status {
                    NumaPlacementProofStatus::Verified => (
                        PlacementValidationGateStatus::Verified,
                        PlacementValidationGateReason::Verified,
                    ),
                    NumaPlacementProofStatus::Unverified => (
                        PlacementValidationGateStatus::Unverified,
                        PlacementValidationGateReason::PlacementProofUnverified,
                    ),
                    NumaPlacementProofStatus::Unavailable => (
                        PlacementValidationGateStatus::Unavailable,
                        PlacementValidationGateReason::PlacementProofUnavailable,
                    ),
                }
            };

            Self {
                status,
                reason,
                memory_policy,
                placement_readiness,
                placement_proof,
            }
        }

        /// Returns true only when all proof conditions are satisfied.
        #[must_use]
        pub fn is_verified(self) -> bool {
            self.status == PlacementValidationGateStatus::Verified
        }
    }

    /// Errors from parsing combined placement validation probe output.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum PlacementValidationGateParseError {
        /// Memory-policy readiness output was missing or malformed.
        MemoryPolicy(LinuxNumaPolicyReadinessOutputParseError),
        /// Placement evidence readiness output was missing or malformed.
        PlacementReadiness(NumaPlacementReadinessOutputParseError),
        /// Placement proof output was missing or malformed.
        PlacementProof(NumaPlacementProofOutputParseError),
    }

    impl fmt::Display for PlacementValidationGateParseError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::MemoryPolicy(source) => write!(f, "invalid memory policy output: {source}"),
                Self::PlacementReadiness(source) => {
                    write!(f, "invalid placement readiness output: {source}")
                }
                Self::PlacementProof(source) => {
                    write!(f, "invalid placement proof output: {source}")
                }
            }
        }
    }

    impl std::error::Error for PlacementValidationGateParseError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            match self {
                Self::MemoryPolicy(source) => Some(source),
                Self::PlacementReadiness(source) => Some(source),
                Self::PlacementProof(source) => Some(source),
            }
        }
    }

    /// Parses probe outputs and returns the combined placement validation gate.
    ///
    /// # Errors
    ///
    /// Returns an error when any probe output is missing its final verdict line
    /// or contains a malformed final verdict line.
    pub fn evaluate_placement_validation_outputs(
        outputs: PlacementValidationOutputs<'_>,
    ) -> Result<PlacementValidationGate, PlacementValidationGateParseError> {
        let memory_policy = parse_linux_numa_policy_readiness_output(outputs.memory_policy_output)
            .map_err(PlacementValidationGateParseError::MemoryPolicy)?;
        let placement_readiness =
            parse_numa_placement_readiness_output(outputs.placement_readiness_output)
                .map_err(PlacementValidationGateParseError::PlacementReadiness)?;
        let placement_proof = parse_numa_placement_proof_output(outputs.placement_proof_output)
            .map_err(PlacementValidationGateParseError::PlacementProof)?;

        Ok(PlacementValidationGate::from_verdicts(
            memory_policy,
            placement_readiness,
            placement_proof,
        ))
    }

    /// Parses a placement validation gate verdict line.
    ///
    /// The expected format is `placement_validation_gate=<status> reason=<reason>`.
    ///
    /// # Errors
    ///
    /// Returns an error when the line is missing required tokens, contains duplicate
    /// tokens, contains unsupported tokens, uses an unknown status or reason, or
    /// combines a status with an incoherent reason.
    pub fn parse_placement_validation_gate_line(
        line: &str,
    ) -> Result<PlacementValidationGateVerdict, PlacementValidationGateLineParseError> {
        let mut status_token = None;
        let mut reason_token = None;

        for token in line.split_whitespace() {
            let Some((key, value)) = token.split_once('=') else {
                return Err(PlacementValidationGateLineParseError::InvalidToken(
                    token.to_owned(),
                ));
            };

            match key {
                "placement_validation_gate" => {
                    if status_token.replace(value).is_some() {
                        return Err(PlacementValidationGateLineParseError::DuplicateStatus);
                    }
                }
                "reason" => {
                    if reason_token.replace(value).is_some() {
                        return Err(PlacementValidationGateLineParseError::DuplicateReason);
                    }
                }
                _ => {
                    return Err(PlacementValidationGateLineParseError::InvalidToken(
                        token.to_owned(),
                    ));
                }
            }
        }

        let status_token =
            status_token.ok_or(PlacementValidationGateLineParseError::MissingStatus)?;
        let reason_token =
            reason_token.ok_or(PlacementValidationGateLineParseError::MissingReason)?;

        let status =
            PlacementValidationGateStatus::from_str_token(status_token).ok_or_else(|| {
                PlacementValidationGateLineParseError::UnknownStatus(status_token.to_owned())
            })?;
        let reason =
            PlacementValidationGateReason::from_str_token(reason_token).ok_or_else(|| {
                PlacementValidationGateLineParseError::UnknownReason(reason_token.to_owned())
            })?;

        PlacementValidationGateVerdict::from_parts(status, reason)
    }

    /// Extracts the final placement validation gate verdict from multiline output.
    ///
    /// # Errors
    ///
    /// Returns an error when the output has no placement validation gate line,
    /// has more than one placement validation gate line, or contains a
    /// malformed placement validation gate line.
    pub fn parse_placement_validation_gate_output(
        output: &str,
    ) -> Result<PlacementValidationGateVerdict, PlacementValidationGateOutputParseError> {
        let mut gate = None;

        for line in output.lines().map(str::trim) {
            if !line.starts_with("placement_validation_gate=") {
                continue;
            }

            if gate.is_some() {
                return Err(PlacementValidationGateOutputParseError::DuplicateGateLine);
            }

            gate = Some(
                parse_placement_validation_gate_line(line)
                    .map_err(PlacementValidationGateOutputParseError::Line)?,
            );
        }

        gate.ok_or(PlacementValidationGateOutputParseError::MissingGateLine)
    }

    #[cfg(test)]
    mod tests {
        use locus_observe::{
            NumaPlacementProof, NumaPlacementProofReason, NumaPlacementProofStatus,
            NumaPlacementValidationReadiness, NumaPlacementValidationReadinessReason,
            NumaPlacementValidationReadinessStatus,
        };
        use locus_sys::linux::{
            LinuxNumaPolicyReadiness, LinuxNumaPolicyReadinessReason,
            LinuxNumaPolicyReadinessStatus,
        };

        use super::{
            evaluate_placement_validation_outputs, parse_placement_validation_gate_line,
            parse_placement_validation_gate_output, PlacementValidationGate,
            PlacementValidationGateLineParseError, PlacementValidationGateOutputParseError,
            PlacementValidationGateParseError, PlacementValidationGateReason,
            PlacementValidationGateStatus, PlacementValidationGateVerdict,
            PlacementValidationOutputs,
        };

        #[test]
        fn reports_verified_gate_from_ready_inputs() {
            let gate = PlacementValidationGate::from_verdicts(
                LinuxNumaPolicyReadiness {
                    status: LinuxNumaPolicyReadinessStatus::Ready,
                    reason: LinuxNumaPolicyReadinessReason::Ready,
                },
                NumaPlacementValidationReadiness {
                    status: NumaPlacementValidationReadinessStatus::Ready,
                    reason: NumaPlacementValidationReadinessReason::Ready,
                },
                NumaPlacementProof {
                    status: NumaPlacementProofStatus::Verified,
                    reason: NumaPlacementProofReason::Verified,
                },
            );

            assert_eq!(gate.status, PlacementValidationGateStatus::Verified);
            assert_eq!(gate.reason, PlacementValidationGateReason::Verified);
            assert_eq!(gate.status.to_string(), "verified");
            assert_eq!(gate.reason.to_string(), "verified");
            assert_eq!(
                gate.to_string(),
                "placement_validation_gate=verified reason=verified"
            );
            assert!(gate.is_verified());
        }

        #[test]
        fn prioritizes_readiness_before_proof_status() {
            let not_ready_policy = LinuxNumaPolicyReadiness {
                status: LinuxNumaPolicyReadinessStatus::NotReady,
                reason: LinuxNumaPolicyReadinessReason::PermissionDenied,
            };
            let ready_evidence = NumaPlacementValidationReadiness {
                status: NumaPlacementValidationReadinessStatus::Ready,
                reason: NumaPlacementValidationReadinessReason::Ready,
            };
            let unavailable_proof = NumaPlacementProof {
                status: NumaPlacementProofStatus::Unavailable,
                reason: NumaPlacementProofReason::NumaMapsUnavailable,
            };

            let gate = PlacementValidationGate::from_verdicts(
                not_ready_policy,
                ready_evidence,
                unavailable_proof,
            );

            assert_eq!(gate.status, PlacementValidationGateStatus::NotReady);
            assert_eq!(
                gate.reason,
                PlacementValidationGateReason::MemoryPolicyNotReady
            );
            assert!(!gate.is_verified());
        }

        #[test]
        fn evaluates_docker_probe_outputs_as_not_ready() {
            let gate = evaluate_placement_validation_outputs(PlacementValidationOutputs {
                memory_policy_output: "\
mbind=error mbind syscall failed: Operation not permitted (os error 1)
memory_policy_readiness=not_ready reason=permission_denied
touched=4
",
                placement_readiness_output: "\
numa_maps=unavailable
cgroup_numa_stat=unavailable
node_numastat=unavailable
placement_validation_readiness=not_ready reason=numa_maps_unavailable
",
                placement_proof_output: "\
mapping_start=0xffff98744000
mapping_len=20479
mapped_scratch_bind=error mapped scratch arena NUMA policy failed: mbind syscall failed: Operation not permitted (os error 1)
memory_policy_readiness=not_ready reason=permission_denied
touched=5
home_node=0
cgroup_numa_delta=unavailable
node_numastat_delta=unavailable
numa_maps=unavailable
placement_proof=unavailable reason=numa_maps_unavailable
",
            })
            .expect("gate");

            assert_eq!(gate.status, PlacementValidationGateStatus::NotReady);
            assert_eq!(
                gate.reason,
                PlacementValidationGateReason::MemoryPolicyNotReady
            );
            assert_eq!(
                gate.placement_proof.status,
                NumaPlacementProofStatus::Unavailable
            );
        }

        #[test]
        fn reports_probe_output_parse_errors() {
            let error = evaluate_placement_validation_outputs(PlacementValidationOutputs {
                memory_policy_output: "mbind=ok\n",
                placement_readiness_output: "placement_validation_readiness=ready reason=ready\n",
                placement_proof_output: "placement_proof=verified reason=verified\n",
            })
            .expect_err("missing memory policy readiness");

            assert!(matches!(
                error,
                PlacementValidationGateParseError::MemoryPolicy(_)
            ));
        }

        #[test]
        fn parses_placement_validation_gate_lines() {
            assert_eq!(
                parse_placement_validation_gate_line(
                    "placement_validation_gate=verified reason=verified"
                )
                .expect("verified"),
                PlacementValidationGateVerdict {
                    status: PlacementValidationGateStatus::Verified,
                    reason: PlacementValidationGateReason::Verified,
                }
            );
            assert_eq!(
                parse_placement_validation_gate_line(
                    "placement_validation_gate=verified reason=verified"
                )
                .expect("verified")
                .to_string(),
                "placement_validation_gate=verified reason=verified"
            );
            assert_eq!(
                parse_placement_validation_gate_line(
                    "placement_validation_gate=not_ready reason=memory_policy_not_ready"
                )
                .expect("not ready"),
                PlacementValidationGateVerdict {
                    status: PlacementValidationGateStatus::NotReady,
                    reason: PlacementValidationGateReason::MemoryPolicyNotReady,
                }
            );
            assert_eq!(
                parse_placement_validation_gate_line(
                    "placement_validation_gate=unverified reason=placement_proof_unverified"
                )
                .expect("unverified"),
                PlacementValidationGateVerdict {
                    status: PlacementValidationGateStatus::Unverified,
                    reason: PlacementValidationGateReason::PlacementProofUnverified,
                }
            );
            assert_eq!(
                parse_placement_validation_gate_line(
                    "placement_validation_gate=unavailable reason=placement_proof_unavailable"
                )
                .expect("unavailable"),
                PlacementValidationGateVerdict {
                    status: PlacementValidationGateStatus::Unavailable,
                    reason: PlacementValidationGateReason::PlacementProofUnavailable,
                }
            );
            assert!(PlacementValidationGateVerdict {
                status: PlacementValidationGateStatus::NotReady,
                reason: PlacementValidationGateReason::PlacementEvidenceNotReady,
            }
            .is_consistent());
            assert!(!PlacementValidationGateVerdict {
                status: PlacementValidationGateStatus::Verified,
                reason: PlacementValidationGateReason::MemoryPolicyNotReady,
            }
            .is_consistent());
        }

        #[test]
        fn rejects_invalid_placement_validation_gate_lines() {
            assert_eq!(
                parse_placement_validation_gate_line("reason=verified")
                    .expect_err("missing status"),
                PlacementValidationGateLineParseError::MissingStatus
            );
            assert_eq!(
                parse_placement_validation_gate_line("placement_validation_gate=verified")
                    .expect_err("missing reason"),
                PlacementValidationGateLineParseError::MissingReason
            );
            assert_eq!(
                parse_placement_validation_gate_line(
                    "placement_validation_gate=maybe reason=verified"
                )
                .expect_err("unknown status"),
                PlacementValidationGateLineParseError::UnknownStatus("maybe".to_owned())
            );
            assert_eq!(
                parse_placement_validation_gate_line(
                    "placement_validation_gate=verified reason=maybe"
                )
                .expect_err("unknown reason"),
                PlacementValidationGateLineParseError::UnknownReason("maybe".to_owned())
            );
            assert_eq!(
                parse_placement_validation_gate_line(
                    "placement_validation_gate=verified reason=verified extra=true"
                )
                .expect_err("extra token"),
                PlacementValidationGateLineParseError::InvalidToken("extra=true".to_owned())
            );
            assert_eq!(
                parse_placement_validation_gate_line(
                    "placement_validation_gate=verified placement_validation_gate=not_ready reason=verified"
                )
                .expect_err("duplicate status"),
                PlacementValidationGateLineParseError::DuplicateStatus
            );
            assert_eq!(
                parse_placement_validation_gate_line(
                    "placement_validation_gate=verified reason=verified reason=memory_policy_not_ready"
                )
                .expect_err("duplicate reason"),
                PlacementValidationGateLineParseError::DuplicateReason
            );
            assert_eq!(
                parse_placement_validation_gate_line(
                    "placement_validation_gate=verified reason=memory_policy_not_ready"
                )
                .expect_err("inconsistent verdict"),
                PlacementValidationGateLineParseError::InconsistentVerdict {
                    status: PlacementValidationGateStatus::Verified,
                    reason: PlacementValidationGateReason::MemoryPolicyNotReady,
                }
            );
        }

        #[test]
        fn parses_placement_validation_gate_from_output() {
            let output = "\
mapping_start=0xffff854a3000
mapping_len=20479
memory_policy_readiness=not_ready reason=permission_denied
touched=5
home_node=0
numa_maps=unavailable
cgroup_numa_stat=unavailable
node_numastat=unavailable
placement_validation_readiness=not_ready reason=numa_maps_unavailable
placement_proof=unavailable reason=numa_maps_unavailable
placement_validation_gate=not_ready reason=memory_policy_not_ready
";

            assert_eq!(
                parse_placement_validation_gate_output(output).expect("gate"),
                PlacementValidationGateVerdict {
                    status: PlacementValidationGateStatus::NotReady,
                    reason: PlacementValidationGateReason::MemoryPolicyNotReady,
                }
            );
        }

        #[test]
        fn rejects_invalid_placement_validation_gate_output() {
            assert_eq!(
                parse_placement_validation_gate_output(
                    "placement_proof=verified reason=verified\n"
                )
                .expect_err("missing gate"),
                PlacementValidationGateOutputParseError::MissingGateLine
            );
            assert_eq!(
                parse_placement_validation_gate_output(
                    "placement_validation_gate=verified reason=verified\nplacement_validation_gate=not_ready reason=memory_policy_not_ready\n"
                )
                .expect_err("duplicate gate"),
                PlacementValidationGateOutputParseError::DuplicateGateLine
            );
            assert_eq!(
                parse_placement_validation_gate_output(
                    "placement_validation_gate=maybe reason=verified\n"
                )
                .expect_err("bad gate"),
                PlacementValidationGateOutputParseError::Line(
                    PlacementValidationGateLineParseError::UnknownStatus("maybe".to_owned())
                )
            );
            assert_eq!(
                parse_placement_validation_gate_output(
                    "placement_validation_gate=not_ready reason=verified\n"
                )
                .expect_err("inconsistent gate"),
                PlacementValidationGateOutputParseError::Line(
                    PlacementValidationGateLineParseError::InconsistentVerdict {
                        status: PlacementValidationGateStatus::NotReady,
                        reason: PlacementValidationGateReason::Verified,
                    }
                )
            );
        }
    }
}
