use std::fmt;

use locus::{
    parse_pinned_scratch_near_gpu_probe_output, PinnedScratchNearGpuProbeOutput,
    PinnedScratchNearGpuProbeOutputParseError, PinnedScratchNearGpuProbeStatus,
    PinnedScratchPoolProbeStatus,
};

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

/// Error returned when evaluating near-GPU pinned scratch probe output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PinnedScratchNearGpuValidationGateParseError {
    /// Near-GPU pinned scratch probe output was missing or malformed.
    Probe(PinnedScratchNearGpuProbeOutputParseError),
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
mod tests {
    use super::{
        evaluate_pinned_scratch_near_gpu_validation_output,
        parse_pinned_scratch_near_gpu_validation_gate_line,
        PinnedScratchNearGpuValidationGateLineParseError,
        PinnedScratchNearGpuValidationGateParseError, PinnedScratchNearGpuValidationGateReason,
        PinnedScratchNearGpuValidationGateStatus, PinnedScratchNearGpuValidationGateVerdict,
    };

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
}
