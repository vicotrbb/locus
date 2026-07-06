use std::fmt;

use locus::{
    parse_mapped_scratch_thp_fault_samples_output, MappedScratchThpFaultSampleStatus,
    MappedScratchThpFaultSamples, MappedScratchThpFaultSamplesParseError,
};

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

/// Error returned when evaluating mapped scratch THP benchmark fault samples.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MappedScratchThpFaultSampleValidationGateParseError {
    /// Benchmark fault sample output was missing or malformed.
    Samples(MappedScratchThpFaultSamplesParseError),
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

#[cfg(test)]
mod tests {
    use super::{
        evaluate_mapped_scratch_thp_fault_sample_validation_output,
        parse_mapped_scratch_thp_fault_sample_validation_gate_line,
        parse_mapped_scratch_thp_fault_sample_validation_gate_output,
        MappedScratchThpFaultSampleValidationGateLineParseError,
        MappedScratchThpFaultSampleValidationGateOutputParseError,
        MappedScratchThpFaultSampleValidationGateParseError,
        MappedScratchThpFaultSampleValidationGateReason,
        MappedScratchThpFaultSampleValidationGateStatus,
        MappedScratchThpFaultSampleValidationGateVerdict,
    };
    use crate::{
        MappedScratchThpFaultSampleComparisonReason, MappedScratchThpFaultSampleComparisonStatus,
    };

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
    fn reports_ready_gate_from_benchmark_output() {
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
    fn reports_unavailable_gate() {
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
    fn reports_parse_errors() {
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
    fn parses_gate_lines() {
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
    fn rejects_invalid_gate_lines() {
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
    fn parses_gate_from_output() {
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
    fn rejects_invalid_gate_output() {
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
