use std::fmt;

use crate::{MappedScratchThpFaultSampleMode, MappedScratchThpObservation};

/// Stable mapped scratch THP benchmark page-size sample status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappedScratchThpPageSampleStatus {
    /// Kernel page-size evidence was collected.
    Available,
    /// Kernel page-size evidence was unavailable.
    Unavailable,
}

impl MappedScratchThpPageSampleStatus {
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

impl fmt::Display for MappedScratchThpPageSampleStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Kernel evidence source for a mapped scratch THP page-size sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappedScratchThpPageSampleSource {
    /// Evidence came from `/proc/<pid>/numa_maps`.
    NumaMaps,
    /// Evidence came from `/proc/<pid>/smaps`.
    Smaps,
    /// No page-size evidence source was available.
    None,
}

impl MappedScratchThpPageSampleSource {
    /// Returns a stable machine-readable source string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NumaMaps => "numa_maps",
            Self::Smaps => "smaps",
            Self::None => "none",
        }
    }

    /// Parses a stable machine-readable source string.
    #[must_use]
    pub fn from_str_token(value: &str) -> Option<Self> {
        match value {
            "numa_maps" => Some(Self::NumaMaps),
            "smaps" => Some(Self::Smaps),
            "none" => Some(Self::None),
            _ => None,
        }
    }
}

impl fmt::Display for MappedScratchThpPageSampleSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Parsed mapped scratch THP benchmark page-size sample line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MappedScratchThpPageSampleLine {
    /// Sample mode.
    pub mode: MappedScratchThpFaultSampleMode,
    /// Evidence availability.
    pub status: MappedScratchThpPageSampleStatus,
    /// Evidence source.
    pub source: MappedScratchThpPageSampleSource,
    /// Observed kernel page size in KiB.
    pub kernel_page_kb: Option<usize>,
    /// THP observation derived from the page-size comparison.
    pub observation: MappedScratchThpObservation,
    /// Stable observation reason.
    pub reason: String,
}

/// Parsed mapped scratch THP benchmark page-size samples.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MappedScratchThpPageSamples {
    /// Default mode page sample.
    pub default: MappedScratchThpPageSampleLine,
    /// Hugepage advice mode page sample.
    pub hugepage: MappedScratchThpPageSampleLine,
    /// No-hugepage advice mode page sample.
    pub no_hugepage: MappedScratchThpPageSampleLine,
}

/// Error returned when parsing a mapped scratch THP page-size sample line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MappedScratchThpPageSampleLineParseError {
    /// A required field is missing.
    MissingField(&'static str),
    /// A field appears more than once.
    DuplicateField(&'static str),
    /// A token does not contain `=`.
    InvalidToken(String),
    /// A field is not part of the stable schema.
    UnknownField(String),
    /// The mode token is not recognized.
    UnknownMode(String),
    /// The status token is not recognized.
    UnknownStatus(String),
    /// The source token is not recognized.
    UnknownSource(String),
    /// The observation token is not recognized.
    UnknownObservation(String),
    /// A numeric field is malformed.
    InvalidNumber {
        /// Field name.
        field: &'static str,
        /// Rejected value.
        value: String,
    },
    /// The line is syntactically valid but internally inconsistent.
    Inconsistent {
        /// Reason for the inconsistency.
        reason: &'static str,
    },
}

impl fmt::Display for MappedScratchThpPageSampleLineParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingField(field) => {
                write!(f, "missing mapped scratch THP page sample field: {field}")
            }
            Self::DuplicateField(field) => {
                write!(f, "duplicate mapped scratch THP page sample field: {field}")
            }
            Self::InvalidToken(token) => {
                write!(f, "invalid mapped scratch THP page sample token: {token}")
            }
            Self::UnknownField(field) => {
                write!(f, "unknown mapped scratch THP page sample field: {field}")
            }
            Self::UnknownMode(mode) => {
                write!(f, "unknown mapped scratch THP page sample mode: {mode}")
            }
            Self::UnknownStatus(status) => {
                write!(f, "unknown mapped scratch THP page sample status: {status}")
            }
            Self::UnknownSource(source) => {
                write!(f, "unknown mapped scratch THP page sample source: {source}")
            }
            Self::UnknownObservation(observation) => {
                write!(
                    f,
                    "unknown mapped scratch THP page sample observation: {observation}"
                )
            }
            Self::InvalidNumber { field, value } => write!(
                f,
                "invalid mapped scratch THP page sample number for {field}: {value}"
            ),
            Self::Inconsistent { reason } => {
                write!(f, "inconsistent mapped scratch THP page sample: {reason}")
            }
        }
    }
}

impl std::error::Error for MappedScratchThpPageSampleLineParseError {}

/// Error returned when extracting mapped scratch THP page-size samples.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MappedScratchThpPageSamplesParseError {
    /// A required sample line is missing.
    MissingSample(MappedScratchThpFaultSampleMode),
    /// A mode appears more than once.
    DuplicateSample(MappedScratchThpFaultSampleMode),
    /// A sample line is malformed.
    Line {
        /// One-based line number.
        line: usize,
        /// Source error.
        source: MappedScratchThpPageSampleLineParseError,
    },
}

impl fmt::Display for MappedScratchThpPageSamplesParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSample(mode) => {
                write!(f, "missing mapped scratch THP page sample: {mode}")
            }
            Self::DuplicateSample(mode) => {
                write!(f, "duplicate mapped scratch THP page sample: {mode}")
            }
            Self::Line { line, source } => {
                write!(
                    f,
                    "invalid mapped scratch THP page sample on line {line}: {source}"
                )
            }
        }
    }
}

impl std::error::Error for MappedScratchThpPageSamplesParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Line { source, .. } => Some(source),
            Self::MissingSample(_) | Self::DuplicateSample(_) => None,
        }
    }
}

/// Parses one `thp_page_sample=` benchmark line.
///
/// # Errors
///
/// Returns an error when the line is missing required fields, contains
/// malformed tokens, or reports inconsistent availability and observation
/// values.
pub fn parse_mapped_scratch_thp_page_sample_line(
    line: &str,
) -> Result<MappedScratchThpPageSampleLine, MappedScratchThpPageSampleLineParseError> {
    let mut fields = MappedScratchThpPageSampleFields::default();

    for token in line.split_whitespace() {
        let Some((key, value)) = token.split_once('=') else {
            return Err(MappedScratchThpPageSampleLineParseError::InvalidToken(
                token.to_owned(),
            ));
        };

        match key {
            "thp_page_sample" => {
                let mode =
                    MappedScratchThpFaultSampleMode::from_str_token(value).ok_or_else(|| {
                        MappedScratchThpPageSampleLineParseError::UnknownMode(value.to_owned())
                    })?;
                set_field(&mut fields.mode, "thp_page_sample", mode)?;
            }
            "status" => {
                let status =
                    MappedScratchThpPageSampleStatus::from_str_token(value).ok_or_else(|| {
                        MappedScratchThpPageSampleLineParseError::UnknownStatus(value.to_owned())
                    })?;
                set_field(&mut fields.status, "status", status)?;
            }
            "source" => {
                let source =
                    MappedScratchThpPageSampleSource::from_str_token(value).ok_or_else(|| {
                        MappedScratchThpPageSampleLineParseError::UnknownSource(value.to_owned())
                    })?;
                set_field(&mut fields.source, "source", source)?;
            }
            "kernel_page_kb" => {
                if fields.kernel_page_kb_seen {
                    return Err(MappedScratchThpPageSampleLineParseError::DuplicateField(
                        "kernel_page_kb",
                    ));
                }
                fields.kernel_page_kb_seen = true;
                if value != "unknown" {
                    fields.kernel_page_kb = Some(parse_usize("kernel_page_kb", value)?);
                }
            }
            "thp_observed" => {
                let observation =
                    MappedScratchThpObservation::from_str_token(value).ok_or_else(|| {
                        MappedScratchThpPageSampleLineParseError::UnknownObservation(
                            value.to_owned(),
                        )
                    })?;
                set_field(&mut fields.observation, "thp_observed", observation)?;
            }
            "reason" => {
                set_field(&mut fields.reason, "reason", value.to_owned())?;
            }
            _ => {
                return Err(MappedScratchThpPageSampleLineParseError::UnknownField(
                    key.to_owned(),
                ));
            }
        }
    }

    fields.finish()
}

/// Extracts mapped scratch THP page-size samples from multiline benchmark output.
///
/// # Errors
///
/// Returns an error when any `thp_page_sample=` line is malformed, duplicated,
/// or when any required mode is missing.
pub fn parse_mapped_scratch_thp_page_samples_output(
    output: &str,
) -> Result<MappedScratchThpPageSamples, MappedScratchThpPageSamplesParseError> {
    let mut default = None;
    let mut hugepage = None;
    let mut no_hugepage = None;

    for (index, line) in output.lines().enumerate() {
        let line = line.trim();
        if !line.starts_with("thp_page_sample=") {
            continue;
        }

        let sample = parse_mapped_scratch_thp_page_sample_line(line).map_err(|source| {
            MappedScratchThpPageSamplesParseError::Line {
                line: index + 1,
                source,
            }
        })?;

        match sample.mode {
            MappedScratchThpFaultSampleMode::Default => {
                set_sample(&mut default, sample)?;
            }
            MappedScratchThpFaultSampleMode::HugePage => {
                set_sample(&mut hugepage, sample)?;
            }
            MappedScratchThpFaultSampleMode::NoHugePage => {
                set_sample(&mut no_hugepage, sample)?;
            }
        }
    }

    Ok(MappedScratchThpPageSamples {
        default: default.ok_or(MappedScratchThpPageSamplesParseError::MissingSample(
            MappedScratchThpFaultSampleMode::Default,
        ))?,
        hugepage: hugepage.ok_or(MappedScratchThpPageSamplesParseError::MissingSample(
            MappedScratchThpFaultSampleMode::HugePage,
        ))?,
        no_hugepage: no_hugepage.ok_or(MappedScratchThpPageSamplesParseError::MissingSample(
            MappedScratchThpFaultSampleMode::NoHugePage,
        ))?,
    })
}

#[derive(Default)]
struct MappedScratchThpPageSampleFields {
    mode: Option<MappedScratchThpFaultSampleMode>,
    status: Option<MappedScratchThpPageSampleStatus>,
    source: Option<MappedScratchThpPageSampleSource>,
    kernel_page_kb: Option<usize>,
    kernel_page_kb_seen: bool,
    observation: Option<MappedScratchThpObservation>,
    reason: Option<String>,
}

impl MappedScratchThpPageSampleFields {
    fn finish(
        self,
    ) -> Result<MappedScratchThpPageSampleLine, MappedScratchThpPageSampleLineParseError> {
        let mode = self
            .mode
            .ok_or(MappedScratchThpPageSampleLineParseError::MissingField(
                "thp_page_sample",
            ))?;
        let status = self
            .status
            .ok_or(MappedScratchThpPageSampleLineParseError::MissingField(
                "status",
            ))?;
        let source = self
            .source
            .ok_or(MappedScratchThpPageSampleLineParseError::MissingField(
                "source",
            ))?;
        let observation =
            self.observation
                .ok_or(MappedScratchThpPageSampleLineParseError::MissingField(
                    "thp_observed",
                ))?;
        let reason = self
            .reason
            .ok_or(MappedScratchThpPageSampleLineParseError::MissingField(
                "reason",
            ))?;

        if !self.kernel_page_kb_seen {
            return Err(MappedScratchThpPageSampleLineParseError::MissingField(
                "kernel_page_kb",
            ));
        }

        match status {
            MappedScratchThpPageSampleStatus::Available => {
                if source == MappedScratchThpPageSampleSource::None {
                    return Err(MappedScratchThpPageSampleLineParseError::Inconsistent {
                        reason: "available sample cannot use source=none",
                    });
                }
                if self.kernel_page_kb.is_none() {
                    return Err(MappedScratchThpPageSampleLineParseError::Inconsistent {
                        reason: "available sample requires numeric kernel_page_kb",
                    });
                }
                if observation == MappedScratchThpObservation::Unknown {
                    return Err(MappedScratchThpPageSampleLineParseError::Inconsistent {
                        reason: "available sample requires yes or no observation",
                    });
                }
            }
            MappedScratchThpPageSampleStatus::Unavailable => {
                if self.kernel_page_kb.is_some() {
                    return Err(MappedScratchThpPageSampleLineParseError::Inconsistent {
                        reason: "unavailable sample cannot include numeric kernel_page_kb",
                    });
                }
                if observation != MappedScratchThpObservation::Unknown {
                    return Err(MappedScratchThpPageSampleLineParseError::Inconsistent {
                        reason: "unavailable sample requires unknown observation",
                    });
                }
            }
        }

        Ok(MappedScratchThpPageSampleLine {
            mode,
            status,
            source,
            kernel_page_kb: self.kernel_page_kb,
            observation,
            reason,
        })
    }
}

fn parse_usize(
    field: &'static str,
    value: &str,
) -> Result<usize, MappedScratchThpPageSampleLineParseError> {
    value.parse().map_err(
        |_| MappedScratchThpPageSampleLineParseError::InvalidNumber {
            field,
            value: value.to_owned(),
        },
    )
}

fn set_field<T>(
    slot: &mut Option<T>,
    field: &'static str,
    value: T,
) -> Result<(), MappedScratchThpPageSampleLineParseError> {
    if slot.replace(value).is_some() {
        return Err(MappedScratchThpPageSampleLineParseError::DuplicateField(
            field,
        ));
    }
    Ok(())
}

fn set_sample(
    slot: &mut Option<MappedScratchThpPageSampleLine>,
    sample: MappedScratchThpPageSampleLine,
) -> Result<(), MappedScratchThpPageSamplesParseError> {
    if slot.is_some() {
        return Err(MappedScratchThpPageSamplesParseError::DuplicateSample(
            sample.mode,
        ));
    }
    *slot = Some(sample);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        MappedScratchThpFaultSampleMode, MappedScratchThpObservation,
        MappedScratchThpPageSampleSource, MappedScratchThpPageSampleStatus,
    };

    use super::{
        parse_mapped_scratch_thp_page_sample_line, parse_mapped_scratch_thp_page_samples_output,
        MappedScratchThpPageSampleLine, MappedScratchThpPageSampleLineParseError,
        MappedScratchThpPageSamplesParseError,
    };

    #[test]
    fn parses_available_page_sample_line() {
        let sample = parse_mapped_scratch_thp_page_sample_line(
            "thp_page_sample=hugepage status=available source=smaps kernel_page_kb=2048 thp_observed=yes reason=kernel_page_size",
        )
        .expect("sample");

        assert_eq!(
            sample,
            MappedScratchThpPageSampleLine {
                mode: MappedScratchThpFaultSampleMode::HugePage,
                status: MappedScratchThpPageSampleStatus::Available,
                source: MappedScratchThpPageSampleSource::Smaps,
                kernel_page_kb: Some(2048),
                observation: MappedScratchThpObservation::Yes,
                reason: "kernel_page_size".to_owned(),
            }
        );
    }

    #[test]
    fn parses_unavailable_page_sample_line() {
        let sample = parse_mapped_scratch_thp_page_sample_line(
            "thp_page_sample=default status=unavailable source=none kernel_page_kb=unknown thp_observed=unknown reason=observability_unavailable",
        )
        .expect("sample");

        assert_eq!(sample.mode, MappedScratchThpFaultSampleMode::Default);
        assert_eq!(sample.status, MappedScratchThpPageSampleStatus::Unavailable);
        assert_eq!(sample.source, MappedScratchThpPageSampleSource::None);
        assert_eq!(sample.kernel_page_kb, None);
        assert_eq!(sample.observation, MappedScratchThpObservation::Unknown);
    }

    #[test]
    fn parses_page_samples_output() {
        let samples = parse_mapped_scratch_thp_page_samples_output(
            "noise=true
thp_page_sample=default status=available source=numa_maps kernel_page_kb=4 thp_observed=no reason=base_page_size
thp_page_sample=hugepage status=available source=smaps kernel_page_kb=2048 thp_observed=yes reason=kernel_page_size
thp_page_sample=no_hugepage status=available source=smaps kernel_page_kb=4 thp_observed=no reason=base_page_size
",
        )
        .expect("samples");

        assert_eq!(
            samples.default.mode,
            MappedScratchThpFaultSampleMode::Default
        );
        assert_eq!(
            samples.hugepage.observation,
            MappedScratchThpObservation::Yes
        );
        assert_eq!(samples.no_hugepage.kernel_page_kb, Some(4));
    }

    #[test]
    fn rejects_inconsistent_page_sample_lines() {
        assert!(matches!(
            parse_mapped_scratch_thp_page_sample_line(
                "thp_page_sample=hugepage status=available source=none kernel_page_kb=4 thp_observed=no reason=base_page_size",
            )
            .expect_err("source none"),
            MappedScratchThpPageSampleLineParseError::Inconsistent { .. }
        ));

        assert!(matches!(
            parse_mapped_scratch_thp_page_sample_line(
                "thp_page_sample=hugepage status=available source=smaps kernel_page_kb=unknown thp_observed=no reason=base_page_size",
            )
            .expect_err("missing page size"),
            MappedScratchThpPageSampleLineParseError::Inconsistent { .. }
        ));

        assert!(matches!(
            parse_mapped_scratch_thp_page_sample_line(
                "thp_page_sample=hugepage status=unavailable source=none kernel_page_kb=4 thp_observed=unknown reason=observability_unavailable",
            )
            .expect_err("numeric unavailable"),
            MappedScratchThpPageSampleLineParseError::Inconsistent { .. }
        ));
    }

    #[test]
    fn rejects_invalid_page_samples_output() {
        assert_eq!(
            parse_mapped_scratch_thp_page_samples_output(
                "thp_page_sample=default status=unavailable source=none kernel_page_kb=unknown thp_observed=unknown reason=observability_unavailable\n",
            )
            .expect_err("missing hugepage"),
            MappedScratchThpPageSamplesParseError::MissingSample(
                MappedScratchThpFaultSampleMode::HugePage
            )
        );

        assert_eq!(
            parse_mapped_scratch_thp_page_samples_output(
                "thp_page_sample=default status=unavailable source=none kernel_page_kb=unknown thp_observed=unknown reason=observability_unavailable
thp_page_sample=default status=unavailable source=none kernel_page_kb=unknown thp_observed=unknown reason=observability_unavailable
",
            )
            .expect_err("duplicate"),
            MappedScratchThpPageSamplesParseError::DuplicateSample(
                MappedScratchThpFaultSampleMode::Default
            )
        );
    }
}
