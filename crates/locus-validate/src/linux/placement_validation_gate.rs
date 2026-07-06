//! Linux placement validation gate helpers.

use std::fmt;

use locus_alloc::sys::linux::{
    parse_linux_numa_policy_readiness_output, LinuxNumaPolicyReadiness,
    LinuxNumaPolicyReadinessOutputParseError,
};
use locus_observe::{
    parse_numa_placement_proof_output, parse_numa_placement_readiness_output, NumaPlacementProof,
    NumaPlacementProofOutputParseError, NumaPlacementProofStatus,
    NumaPlacementReadinessOutputParseError, NumaPlacementValidationReadiness,
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

    let status_token = status_token.ok_or(PlacementValidationGateLineParseError::MissingStatus)?;
    let reason_token = reason_token.ok_or(PlacementValidationGateLineParseError::MissingReason)?;

    let status = PlacementValidationGateStatus::from_str_token(status_token).ok_or_else(|| {
        PlacementValidationGateLineParseError::UnknownStatus(status_token.to_owned())
    })?;
    let reason = PlacementValidationGateReason::from_str_token(reason_token).ok_or_else(|| {
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
    use locus_alloc::sys::linux::{
        LinuxNumaPolicyReadiness, LinuxNumaPolicyReadinessReason, LinuxNumaPolicyReadinessStatus,
    };
    use locus_observe::{
        NumaPlacementProof, NumaPlacementProofReason, NumaPlacementProofStatus,
        NumaPlacementValidationReadiness, NumaPlacementValidationReadinessReason,
        NumaPlacementValidationReadinessStatus,
    };

    use super::{
        evaluate_placement_validation_outputs, parse_placement_validation_gate_line,
        parse_placement_validation_gate_output, PlacementValidationGate,
        PlacementValidationGateLineParseError, PlacementValidationGateOutputParseError,
        PlacementValidationGateParseError, PlacementValidationGateReason,
        PlacementValidationGateStatus, PlacementValidationGateVerdict, PlacementValidationOutputs,
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
            parse_placement_validation_gate_line("reason=verified").expect_err("missing status"),
            PlacementValidationGateLineParseError::MissingStatus
        );
        assert_eq!(
            parse_placement_validation_gate_line("placement_validation_gate=verified")
                .expect_err("missing reason"),
            PlacementValidationGateLineParseError::MissingReason
        );
        assert_eq!(
            parse_placement_validation_gate_line("placement_validation_gate=maybe reason=verified")
                .expect_err("unknown status"),
            PlacementValidationGateLineParseError::UnknownStatus("maybe".to_owned())
        );
        assert_eq!(
            parse_placement_validation_gate_line("placement_validation_gate=verified reason=maybe")
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
            parse_placement_validation_gate_output("placement_proof=verified reason=verified\n")
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
