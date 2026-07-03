use std::fmt;

use locus_alloc::MappedScratchThpFaultSampleComparison;

use crate::{
    MappedScratchThpFaultSampleValidationGate, MappedScratchThpFaultSampleValidationGateReason,
};

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

    /// Parses a stable machine-readable reason string.
    #[must_use]
    pub fn from_str_token(value: &str) -> Option<Self> {
        match value {
            "ready" => Some(Self::Ready),
            "fault_counters_unavailable" => Some(Self::FaultCountersUnavailable),
            "comparison_unavailable" => Some(Self::ComparisonUnavailable),
            _ => None,
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

impl MappedScratchThpFaultSampleComparisonOutput {
    /// Builds a comparison output only when the status, reason, and fields agree.
    ///
    /// # Errors
    ///
    /// Returns an error when the reason or comparison payload is not valid for
    /// the status.
    pub fn from_parts(
        status: MappedScratchThpFaultSampleComparisonStatus,
        reason: MappedScratchThpFaultSampleComparisonReason,
        comparison: Option<MappedScratchThpFaultSampleComparison>,
    ) -> Result<Self, MappedScratchThpFaultSampleComparisonLineParseError> {
        let output = Self {
            status,
            reason,
            comparison,
        };
        if output.is_consistent() {
            Ok(output)
        } else {
            Err(
                MappedScratchThpFaultSampleComparisonLineParseError::InconsistentComparison {
                    status,
                    reason,
                    comparison_present: comparison.is_some(),
                },
            )
        }
    }

    /// Returns true when the reason and comparison payload are valid for the status.
    #[must_use]
    pub fn is_consistent(self) -> bool {
        matches!(
            (self.status, self.reason, self.comparison.is_some()),
            (
                MappedScratchThpFaultSampleComparisonStatus::Available,
                MappedScratchThpFaultSampleComparisonReason::Ready,
                true
            ) | (
                MappedScratchThpFaultSampleComparisonStatus::Unavailable,
                MappedScratchThpFaultSampleComparisonReason::FaultCountersUnavailable
                    | MappedScratchThpFaultSampleComparisonReason::ComparisonUnavailable,
                false
            )
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

impl MappedScratchThpFaultSampleValidationGate {
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

/// Error returned when parsing a mapped scratch THP fault sample comparison line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MappedScratchThpFaultSampleComparisonLineParseError {
    /// The line does not contain a `mapped_scratch_thp_fault_sample_comparison=` token.
    MissingStatus,
    /// The line does not contain a `reason=` token.
    MissingReason,
    /// The line does not contain a field required for an available comparison.
    MissingField(&'static str),
    /// The line contains a duplicate field token.
    DuplicateField(&'static str),
    /// The line contains a token outside the comparison schema.
    InvalidToken(String),
    /// The comparison status token is not recognized.
    UnknownStatus(String),
    /// The comparison reason token is not recognized.
    UnknownReason(String),
    /// A numeric field could not be parsed.
    InvalidNumber {
        /// Field name.
        field: &'static str,
        /// Invalid value.
        value: String,
    },
    /// A boolean field could not be parsed.
    InvalidBool {
        /// Field name.
        field: &'static str,
        /// Invalid value.
        value: String,
    },
    /// An unavailable comparison line contained a comparison payload field.
    UnexpectedField {
        /// Parsed comparison status.
        status: MappedScratchThpFaultSampleComparisonStatus,
        /// Unexpected field.
        field: &'static str,
    },
    /// The status, reason, and comparison payload are individually valid but inconsistent together.
    InconsistentComparison {
        /// Parsed comparison status.
        status: MappedScratchThpFaultSampleComparisonStatus,
        /// Parsed comparison reason.
        reason: MappedScratchThpFaultSampleComparisonReason,
        /// Whether a comparison payload was present.
        comparison_present: bool,
    },
}

impl fmt::Display for MappedScratchThpFaultSampleComparisonLineParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingStatus => {
                f.write_str("missing mapped_scratch_thp_fault_sample_comparison token")
            }
            Self::MissingReason => f.write_str("missing reason token"),
            Self::MissingField(field) => {
                write!(f, "missing mapped scratch THP fault sample comparison field: {field}")
            }
            Self::DuplicateField(field) => {
                write!(
                    f,
                    "duplicate mapped scratch THP fault sample comparison field: {field}"
                )
            }
            Self::InvalidToken(token) => write!(
                f,
                "invalid mapped scratch THP fault sample comparison token: {token}"
            ),
            Self::UnknownStatus(status) => write!(
                f,
                "unknown mapped scratch THP fault sample comparison status: {status}"
            ),
            Self::UnknownReason(reason) => write!(
                f,
                "unknown mapped scratch THP fault sample comparison reason: {reason}"
            ),
            Self::InvalidNumber { field, value } => write!(
                f,
                "invalid mapped scratch THP fault sample comparison number for {field}: {value}"
            ),
            Self::InvalidBool { field, value } => write!(
                f,
                "invalid mapped scratch THP fault sample comparison bool for {field}: {value}"
            ),
            Self::UnexpectedField { status, field } => write!(
                f,
                "unexpected mapped scratch THP fault sample comparison field for {status}: {field}"
            ),
            Self::InconsistentComparison {
                status,
                reason,
                comparison_present,
            } => write!(
                f,
                "inconsistent mapped scratch THP fault sample comparison: {status} {reason} comparison_present={comparison_present}"
            ),
        }
    }
}

impl std::error::Error for MappedScratchThpFaultSampleComparisonLineParseError {}

/// Error returned when extracting a mapped scratch THP fault sample comparison from output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MappedScratchThpFaultSampleComparisonOutputParseError {
    /// The output does not contain a `mapped_scratch_thp_fault_sample_comparison=` line.
    MissingComparisonLine,
    /// The output contains more than one `mapped_scratch_thp_fault_sample_comparison=` line.
    DuplicateComparisonLine,
    /// The discovered comparison line is malformed.
    Line(MappedScratchThpFaultSampleComparisonLineParseError),
}

impl fmt::Display for MappedScratchThpFaultSampleComparisonOutputParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingComparisonLine => {
                f.write_str("missing mapped_scratch_thp_fault_sample_comparison line")
            }
            Self::DuplicateComparisonLine => {
                f.write_str("duplicate mapped_scratch_thp_fault_sample_comparison line")
            }
            Self::Line(source) => write!(
                f,
                "invalid mapped_scratch_thp_fault_sample_comparison line: {source}"
            ),
        }
    }
}

impl std::error::Error for MappedScratchThpFaultSampleComparisonOutputParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Line(source) => Some(source),
            Self::MissingComparisonLine | Self::DuplicateComparisonLine => None,
        }
    }
}

/// Parses a mapped scratch THP fault sample comparison line.
///
/// The available format is
/// `mapped_scratch_thp_fault_sample_comparison=available reason=ready ...`.
/// Unavailable lines include only status and reason.
///
/// # Errors
///
/// Returns an error when required fields are missing, duplicated, malformed, or
/// inconsistent with the comparison status.
pub fn parse_mapped_scratch_thp_fault_sample_comparison_line(
    line: &str,
) -> Result<
    MappedScratchThpFaultSampleComparisonOutput,
    MappedScratchThpFaultSampleComparisonLineParseError,
> {
    let mut fields = MappedScratchThpFaultSampleComparisonLineFields::default();

    for token in line.split_whitespace() {
        fields.parse_token(token)?;
    }

    fields.finish()
}

/// Extracts a mapped scratch THP fault sample comparison from multiline output.
///
/// # Errors
///
/// Returns an error when the output has no mapped scratch THP fault sample
/// comparison line, has more than one mapped scratch THP fault sample
/// comparison line, or contains a malformed comparison line.
pub fn parse_mapped_scratch_thp_fault_sample_comparison_output(
    output: &str,
) -> Result<
    MappedScratchThpFaultSampleComparisonOutput,
    MappedScratchThpFaultSampleComparisonOutputParseError,
> {
    let mut comparison = None;

    for line in output.lines().map(str::trim) {
        if !line.starts_with("mapped_scratch_thp_fault_sample_comparison=") {
            continue;
        }

        if comparison.is_some() {
            return Err(
                MappedScratchThpFaultSampleComparisonOutputParseError::DuplicateComparisonLine,
            );
        }

        comparison = Some(
            parse_mapped_scratch_thp_fault_sample_comparison_line(line)
                .map_err(MappedScratchThpFaultSampleComparisonOutputParseError::Line)?,
        );
    }

    comparison.ok_or(MappedScratchThpFaultSampleComparisonOutputParseError::MissingComparisonLine)
}

#[derive(Default)]
struct MappedScratchThpFaultSampleComparisonLineFields {
    status: Option<MappedScratchThpFaultSampleComparisonStatus>,
    reason: Option<MappedScratchThpFaultSampleComparisonReason>,
    default_minor_faults_delta: Option<i128>,
    hugepage_minor_faults_delta: Option<i128>,
    no_hugepage_minor_faults_delta: Option<i128>,
    hugepage_vs_default_minor_faults_delta: Option<i128>,
    hugepage_vs_no_hugepage_minor_faults_delta: Option<i128>,
    major_faults_observed: Option<bool>,
}

impl MappedScratchThpFaultSampleComparisonLineFields {
    fn parse_token(
        &mut self,
        token: &str,
    ) -> Result<(), MappedScratchThpFaultSampleComparisonLineParseError> {
        let Some((key, value)) = token.split_once('=') else {
            return Err(
                MappedScratchThpFaultSampleComparisonLineParseError::InvalidToken(token.to_owned()),
            );
        };

        match key {
            "mapped_scratch_thp_fault_sample_comparison" => self.parse_status(value),
            "reason" => self.parse_reason(value),
            "default_minor_faults_delta" => {
                self.parse_i128_field("default_minor_faults_delta", value)
            }
            "hugepage_minor_faults_delta" => {
                self.parse_i128_field("hugepage_minor_faults_delta", value)
            }
            "no_hugepage_minor_faults_delta" => {
                self.parse_i128_field("no_hugepage_minor_faults_delta", value)
            }
            "hugepage_vs_default_minor_faults_delta" => {
                self.parse_i128_field("hugepage_vs_default_minor_faults_delta", value)
            }
            "hugepage_vs_no_hugepage_minor_faults_delta" => {
                self.parse_i128_field("hugepage_vs_no_hugepage_minor_faults_delta", value)
            }
            "major_faults_observed" => self.parse_major_faults_observed(value),
            _ => Err(
                MappedScratchThpFaultSampleComparisonLineParseError::InvalidToken(token.to_owned()),
            ),
        }
    }

    fn parse_status(
        &mut self,
        value: &str,
    ) -> Result<(), MappedScratchThpFaultSampleComparisonLineParseError> {
        let parsed = MappedScratchThpFaultSampleComparisonStatus::from_str_token(value)
            .ok_or_else(|| {
                MappedScratchThpFaultSampleComparisonLineParseError::UnknownStatus(value.to_owned())
            })?;
        set_field(
            &mut self.status,
            "mapped_scratch_thp_fault_sample_comparison",
            parsed,
        )
    }

    fn parse_reason(
        &mut self,
        value: &str,
    ) -> Result<(), MappedScratchThpFaultSampleComparisonLineParseError> {
        let parsed = MappedScratchThpFaultSampleComparisonReason::from_str_token(value)
            .ok_or_else(|| {
                MappedScratchThpFaultSampleComparisonLineParseError::UnknownReason(value.to_owned())
            })?;
        set_field(&mut self.reason, "reason", parsed)
    }

    fn parse_i128_field(
        &mut self,
        field: &'static str,
        value: &str,
    ) -> Result<(), MappedScratchThpFaultSampleComparisonLineParseError> {
        let parsed = value.parse::<i128>().map_err(|_| {
            MappedScratchThpFaultSampleComparisonLineParseError::InvalidNumber {
                field,
                value: value.to_owned(),
            }
        })?;

        match field {
            "default_minor_faults_delta" => {
                set_field(&mut self.default_minor_faults_delta, field, parsed)
            }
            "hugepage_minor_faults_delta" => {
                set_field(&mut self.hugepage_minor_faults_delta, field, parsed)
            }
            "no_hugepage_minor_faults_delta" => {
                set_field(&mut self.no_hugepage_minor_faults_delta, field, parsed)
            }
            "hugepage_vs_default_minor_faults_delta" => set_field(
                &mut self.hugepage_vs_default_minor_faults_delta,
                field,
                parsed,
            ),
            "hugepage_vs_no_hugepage_minor_faults_delta" => set_field(
                &mut self.hugepage_vs_no_hugepage_minor_faults_delta,
                field,
                parsed,
            ),
            _ => unreachable!("unsupported comparison numeric field"),
        }
    }

    fn parse_major_faults_observed(
        &mut self,
        value: &str,
    ) -> Result<(), MappedScratchThpFaultSampleComparisonLineParseError> {
        let parsed = match value {
            "true" => true,
            "false" => false,
            _ => {
                return Err(
                    MappedScratchThpFaultSampleComparisonLineParseError::InvalidBool {
                        field: "major_faults_observed",
                        value: value.to_owned(),
                    },
                );
            }
        };
        set_field(
            &mut self.major_faults_observed,
            "major_faults_observed",
            parsed,
        )
    }

    fn finish(
        self,
    ) -> Result<
        MappedScratchThpFaultSampleComparisonOutput,
        MappedScratchThpFaultSampleComparisonLineParseError,
    > {
        let status = self
            .status
            .ok_or(MappedScratchThpFaultSampleComparisonLineParseError::MissingStatus)?;
        let reason = self
            .reason
            .ok_or(MappedScratchThpFaultSampleComparisonLineParseError::MissingReason)?;

        match status {
            MappedScratchThpFaultSampleComparisonStatus::Available => {
                self.finish_available(status, reason)
            }
            MappedScratchThpFaultSampleComparisonStatus::Unavailable => {
                self.finish_unavailable(status, reason)
            }
        }
    }

    fn finish_available(
        self,
        status: MappedScratchThpFaultSampleComparisonStatus,
        reason: MappedScratchThpFaultSampleComparisonReason,
    ) -> Result<
        MappedScratchThpFaultSampleComparisonOutput,
        MappedScratchThpFaultSampleComparisonLineParseError,
    > {
        let comparison = MappedScratchThpFaultSampleComparison {
            default_minor_faults_delta: required_i128(
                self.default_minor_faults_delta,
                "default_minor_faults_delta",
            )?,
            hugepage_minor_faults_delta: required_i128(
                self.hugepage_minor_faults_delta,
                "hugepage_minor_faults_delta",
            )?,
            no_hugepage_minor_faults_delta: required_i128(
                self.no_hugepage_minor_faults_delta,
                "no_hugepage_minor_faults_delta",
            )?,
            hugepage_vs_default_minor_faults_delta: required_i128(
                self.hugepage_vs_default_minor_faults_delta,
                "hugepage_vs_default_minor_faults_delta",
            )?,
            hugepage_vs_no_hugepage_minor_faults_delta: required_i128(
                self.hugepage_vs_no_hugepage_minor_faults_delta,
                "hugepage_vs_no_hugepage_minor_faults_delta",
            )?,
            major_faults_observed: self.major_faults_observed.ok_or(
                MappedScratchThpFaultSampleComparisonLineParseError::MissingField(
                    "major_faults_observed",
                ),
            )?,
        };

        MappedScratchThpFaultSampleComparisonOutput::from_parts(status, reason, Some(comparison))
    }

    fn finish_unavailable(
        self,
        status: MappedScratchThpFaultSampleComparisonStatus,
        reason: MappedScratchThpFaultSampleComparisonReason,
    ) -> Result<
        MappedScratchThpFaultSampleComparisonOutput,
        MappedScratchThpFaultSampleComparisonLineParseError,
    > {
        for (field, present) in [
            (
                "default_minor_faults_delta",
                self.default_minor_faults_delta.is_some(),
            ),
            (
                "hugepage_minor_faults_delta",
                self.hugepage_minor_faults_delta.is_some(),
            ),
            (
                "no_hugepage_minor_faults_delta",
                self.no_hugepage_minor_faults_delta.is_some(),
            ),
            (
                "hugepage_vs_default_minor_faults_delta",
                self.hugepage_vs_default_minor_faults_delta.is_some(),
            ),
            (
                "hugepage_vs_no_hugepage_minor_faults_delta",
                self.hugepage_vs_no_hugepage_minor_faults_delta.is_some(),
            ),
            (
                "major_faults_observed",
                self.major_faults_observed.is_some(),
            ),
        ] {
            if present {
                return Err(
                    MappedScratchThpFaultSampleComparisonLineParseError::UnexpectedField {
                        status,
                        field,
                    },
                );
            }
        }

        MappedScratchThpFaultSampleComparisonOutput::from_parts(status, reason, None)
    }
}

fn required_i128(
    value: Option<i128>,
    field: &'static str,
) -> Result<i128, MappedScratchThpFaultSampleComparisonLineParseError> {
    value.ok_or(MappedScratchThpFaultSampleComparisonLineParseError::MissingField(field))
}

fn set_field<T>(
    slot: &mut Option<T>,
    field: &'static str,
    value: T,
) -> Result<(), MappedScratchThpFaultSampleComparisonLineParseError> {
    if slot.replace(value).is_some() {
        return Err(MappedScratchThpFaultSampleComparisonLineParseError::DuplicateField(field));
    }
    Ok(())
}
