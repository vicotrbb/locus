use std::fmt;

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

/// Parses one stable mapped scratch THP benchmark fault sample line.
///
/// # Errors
///
/// Returns an error when the line is outside the stable fault sample schema.
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

#[cfg(test)]
mod tests {
    use super::{
        parse_mapped_scratch_thp_fault_sample_line, parse_mapped_scratch_thp_fault_samples_output,
        MappedScratchThpFaultSampleComparison, MappedScratchThpFaultSampleLine,
        MappedScratchThpFaultSampleLineParseError, MappedScratchThpFaultSampleMode,
        MappedScratchThpFaultSampleStatus, MappedScratchThpFaultSamples,
        MappedScratchThpFaultSamplesParseError,
    };

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
}
