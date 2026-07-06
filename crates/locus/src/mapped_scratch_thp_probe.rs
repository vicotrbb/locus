use std::fmt;

use crate::MappedScratchHugePageAdvice;

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

#[cfg(test)]
mod tests {
    use crate::MappedScratchHugePageAdvice;

    use super::{
        parse_mapped_scratch_thp_probe_output, MappedScratchThpAdviceStatus,
        MappedScratchThpObservation, MappedScratchThpProbeOutput,
        MappedScratchThpProbeOutputParseError, MappedScratchThpProbeRunStatus,
    };

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

        let smaps_fallback_output = "\
mapped_scratch_thp=started mode=hugepage
thp_advice=ok mode=hugepage
touched=1025
numa_maps=unavailable
smaps=available entries=42
smaps_match=containing_range
smaps_range=0x1000-0x5000
kernel_page_kb=2048
thp_observed=yes reason=kernel_page_size
";

        assert_eq!(
            parse_mapped_scratch_thp_probe_output(smaps_fallback_output)
                .expect("smaps fallback output"),
            MappedScratchThpProbeOutput {
                run_status: MappedScratchThpProbeRunStatus::Started,
                mode: Some(MappedScratchHugePageAdvice::HugePage),
                advice_status: Some(MappedScratchThpAdviceStatus::Ok),
                touched: Some(1025),
                kernel_page_kb: Some(2048),
                observation: Some(MappedScratchThpObservation::Yes),
                observation_reason: Some("kernel_page_size".to_owned()),
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
}
