use std::fmt;

use locus::{
    parse_mapped_scratch_thp_probe_output, MappedScratchHugePageAdvice,
    MappedScratchThpAdviceStatus, MappedScratchThpObservation, MappedScratchThpProbeOutput,
    MappedScratchThpProbeOutputParseError,
};

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

/// Error returned when evaluating mapped scratch THP probe output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MappedScratchThpValidationGateParseError {
    /// Mapped scratch THP probe output was missing or malformed.
    Probe(MappedScratchThpProbeOutputParseError),
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

#[cfg(test)]
mod tests {
    use super::{
        evaluate_mapped_scratch_thp_validation_output,
        parse_mapped_scratch_thp_validation_gate_line,
        parse_mapped_scratch_thp_validation_gate_output,
        MappedScratchThpValidationGateLineParseError,
        MappedScratchThpValidationGateOutputParseError, MappedScratchThpValidationGateParseError,
        MappedScratchThpValidationGateReason, MappedScratchThpValidationGateStatus,
        MappedScratchThpValidationGateVerdict,
    };

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

    const THP_SMAPS_BASE_PAGE_OUTPUT: &str = "\
mapped_scratch_thp=started mode=hugepage
mapping_start=0xffff8753f000
mapping_len=4198399
base_page_kb=4
thp_advice=ok mode=hugepage
touched=1025
numa_maps=unavailable
smaps=available entries=25
smaps_match=containing_range
smaps_range=0xffff8753f000-0xffff87940000
kernel_page_kb=4
thp_observed=no reason=base_page_size
";

    #[test]
    fn reports_ready_gate_from_probe_output() {
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
    fn reports_unavailable_gate_from_probe_output() {
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
    fn reports_not_ready_gate_from_smaps_fallback_output() {
        let gate = evaluate_mapped_scratch_thp_validation_output(THP_SMAPS_BASE_PAGE_OUTPUT)
            .expect("gate");

        assert_eq!(gate.status, MappedScratchThpValidationGateStatus::NotReady);
        assert_eq!(
            gate.reason,
            MappedScratchThpValidationGateReason::BasePageSize
        );
        assert_eq!(
            gate.to_string(),
            "mapped_scratch_thp_validation_gate=not_ready reason=base_page_size"
        );
    }

    #[test]
    fn reports_failures_as_not_ready() {
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
    fn reports_unavailable_platform() {
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
    fn reports_probe_parse_errors() {
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
    fn parses_gate_lines() {
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
    fn rejects_invalid_gate_lines() {
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
    fn parses_gate_from_output() {
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
    fn rejects_invalid_gate_output() {
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
}
