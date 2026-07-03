use std::fmt;

use locus_alloc::{
    parse_mapped_scratch_thp_page_samples_output, MappedScratchThpObservation,
    MappedScratchThpPageSampleSource, MappedScratchThpPageSampleStatus,
    MappedScratchThpPageSamples, MappedScratchThpPageSamplesParseError,
};

use crate::{
    evaluate_mapped_scratch_thp_fault_sample_validation_output,
    MappedScratchThpFaultSampleComparisonOutput, MappedScratchThpFaultSampleValidationGate,
    MappedScratchThpFaultSampleValidationGateParseError,
    MappedScratchThpFaultSampleValidationGateStatus,
};

/// Mapped scratch THP benchmark evidence report status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappedScratchThpBenchmarkEvidenceStatus {
    /// Page-size samples and fault samples were available.
    Ready,
    /// One or more evidence families were unavailable.
    Unavailable,
}

impl MappedScratchThpBenchmarkEvidenceStatus {
    /// Returns a stable machine-readable status string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Unavailable => "unavailable",
        }
    }
}

impl fmt::Display for MappedScratchThpBenchmarkEvidenceStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Reason for the mapped scratch THP benchmark evidence report status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappedScratchThpBenchmarkEvidenceReason {
    /// All required evidence was available.
    Ready,
    /// One or more page-size samples were unavailable.
    PageSamplesUnavailable,
    /// Fault sample evidence was unavailable.
    FaultSamplesUnavailable,
}

impl MappedScratchThpBenchmarkEvidenceReason {
    /// Returns a stable machine-readable reason string.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::PageSamplesUnavailable => "page_samples_unavailable",
            Self::FaultSamplesUnavailable => "fault_samples_unavailable",
        }
    }
}

impl fmt::Display for MappedScratchThpBenchmarkEvidenceReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Parsed mapped scratch THP benchmark evidence report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MappedScratchThpBenchmarkEvidenceReport {
    /// Final report status.
    pub status: MappedScratchThpBenchmarkEvidenceStatus,
    /// Reason for the report status.
    pub reason: MappedScratchThpBenchmarkEvidenceReason,
    /// Parsed page-size samples.
    pub page_samples: MappedScratchThpPageSamples,
    /// Fault sample validation gate.
    pub fault_gate: MappedScratchThpFaultSampleValidationGate,
    /// Fault sample comparison output.
    pub fault_comparison: MappedScratchThpFaultSampleComparisonOutput,
    /// Parsed Criterion timing intervals.
    pub timings: MappedScratchThpBenchmarkTimings,
}

/// Criterion timing interval for one mapped scratch THP benchmark case.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MappedScratchThpTimingInterval {
    /// Lower bound in picoseconds.
    pub lower_ps: u128,
    /// Point estimate in picoseconds.
    pub estimate_ps: u128,
    /// Upper bound in picoseconds.
    pub upper_ps: u128,
}

/// Criterion timing intervals for mapped scratch THP benchmark cases.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MappedScratchThpBenchmarkTimings {
    /// Default mapping timing interval.
    pub default: MappedScratchThpTimingInterval,
    /// Hugepage advice timing interval.
    pub hugepage: MappedScratchThpTimingInterval,
    /// No-hugepage advice timing interval.
    pub no_hugepage: MappedScratchThpTimingInterval,
}

impl MappedScratchThpBenchmarkEvidenceReport {
    /// Builds a report from parsed page-size samples and fault sample gate.
    #[must_use]
    pub fn from_parts(
        page_samples: MappedScratchThpPageSamples,
        fault_gate: MappedScratchThpFaultSampleValidationGate,
        timings: MappedScratchThpBenchmarkTimings,
    ) -> Self {
        let fault_comparison = fault_gate.comparison_output();
        let reason = benchmark_evidence_reason(&page_samples, &fault_gate);
        let status = match reason {
            MappedScratchThpBenchmarkEvidenceReason::Ready => {
                MappedScratchThpBenchmarkEvidenceStatus::Ready
            }
            MappedScratchThpBenchmarkEvidenceReason::PageSamplesUnavailable
            | MappedScratchThpBenchmarkEvidenceReason::FaultSamplesUnavailable => {
                MappedScratchThpBenchmarkEvidenceStatus::Unavailable
            }
        };

        Self {
            status,
            reason,
            page_samples,
            fault_gate,
            fault_comparison,
            timings,
        }
    }

    /// Returns true when page-size and fault-sample evidence are available.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.status == MappedScratchThpBenchmarkEvidenceStatus::Ready
    }

    /// Returns the hugepage advice observation from page-size evidence.
    #[must_use]
    pub fn hugepage_observation(&self) -> MappedScratchThpObservation {
        self.page_samples.hugepage.observation
    }

    /// Returns true when hugepage advice produced observed larger kernel pages.
    #[must_use]
    pub fn observed_hugepage_adoption(&self) -> bool {
        self.hugepage_observation() == MappedScratchThpObservation::Yes
    }
}

impl fmt::Display for MappedScratchThpBenchmarkEvidenceReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let page_samples_status = if page_samples_available(&self.page_samples) {
            "available"
        } else {
            "unavailable"
        };
        let hugepage = &self.page_samples.hugepage;
        let hugepage_kernel_page_kb = hugepage
            .kernel_page_kb
            .map_or_else(|| "unknown".to_owned(), |value| value.to_string());

        write!(
            f,
            "mapped_scratch_thp_benchmark_evidence={} reason={} page_samples={} fault_samples={} hugepage_observed={} hugepage_reason={} hugepage_source={} hugepage_kernel_page_kb={} hugepage_adoption={} fault_comparison={}",
            self.status,
            self.reason,
            page_samples_status,
            self.fault_gate.status,
            hugepage.observation,
            hugepage.reason,
            hugepage.source,
            hugepage_kernel_page_kb,
            self.observed_hugepage_adoption(),
            self.fault_comparison.status
        )?;

        if let Some(comparison) = self.fault_comparison.comparison {
            write!(
                f,
                " hugepage_vs_default_minor_faults_delta={} hugepage_vs_no_hugepage_minor_faults_delta={} major_faults_observed={}",
                comparison.hugepage_vs_default_minor_faults_delta,
                comparison.hugepage_vs_no_hugepage_minor_faults_delta,
                comparison.major_faults_observed
            )?;
        }

        write!(
            f,
            " default_time_lower_ps={} default_time_estimate_ps={} default_time_upper_ps={} hugepage_time_lower_ps={} hugepage_time_estimate_ps={} hugepage_time_upper_ps={} no_hugepage_time_lower_ps={} no_hugepage_time_estimate_ps={} no_hugepage_time_upper_ps={}",
            self.timings.default.lower_ps,
            self.timings.default.estimate_ps,
            self.timings.default.upper_ps,
            self.timings.hugepage.lower_ps,
            self.timings.hugepage.estimate_ps,
            self.timings.hugepage.upper_ps,
            self.timings.no_hugepage.lower_ps,
            self.timings.no_hugepage.estimate_ps,
            self.timings.no_hugepage.upper_ps
        )?;

        Ok(())
    }
}

/// Error returned when parsing a mapped scratch THP benchmark evidence report.
#[derive(Debug)]
pub enum MappedScratchThpBenchmarkEvidenceReportParseError {
    /// Page-size sample output was missing or malformed.
    PageSamples(MappedScratchThpPageSamplesParseError),
    /// Fault sample output was missing or malformed.
    FaultSamples(MappedScratchThpFaultSampleValidationGateParseError),
    /// Criterion timing output was missing or malformed.
    Timings(MappedScratchThpBenchmarkTimingsParseError),
}

impl fmt::Display for MappedScratchThpBenchmarkEvidenceReportParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PageSamples(source) => {
                write!(
                    f,
                    "invalid mapped scratch THP benchmark page samples: {source}"
                )
            }
            Self::FaultSamples(source) => {
                write!(
                    f,
                    "invalid mapped scratch THP benchmark fault samples: {source}"
                )
            }
            Self::Timings(source) => {
                write!(f, "invalid mapped scratch THP benchmark timings: {source}")
            }
        }
    }
}

impl std::error::Error for MappedScratchThpBenchmarkEvidenceReportParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::PageSamples(source) => Some(source),
            Self::FaultSamples(source) => Some(source),
            Self::Timings(source) => Some(source),
        }
    }
}

/// Error returned when parsing mapped scratch THP Criterion timing intervals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MappedScratchThpBenchmarkTimingsParseError {
    /// A required benchmark timing interval is missing.
    MissingBenchmark(&'static str),
    /// A benchmark timing interval appeared more than once.
    DuplicateBenchmark(&'static str),
    /// A timing line did not contain a bracketed interval.
    MissingInterval(String),
    /// A timing interval had an unexpected field count.
    InvalidInterval(String),
    /// A timing value was malformed.
    InvalidValue(String),
    /// A timing unit is not supported.
    UnknownUnit(String),
    /// A numeric conversion overflowed.
    Overflow(String),
}

impl fmt::Display for MappedScratchThpBenchmarkTimingsParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingBenchmark(name) => {
                write!(f, "missing mapped scratch THP benchmark timing: {name}")
            }
            Self::DuplicateBenchmark(name) => {
                write!(f, "duplicate mapped scratch THP benchmark timing: {name}")
            }
            Self::MissingInterval(line) => {
                write!(f, "missing Criterion timing interval: {line}")
            }
            Self::InvalidInterval(line) => {
                write!(f, "invalid Criterion timing interval: {line}")
            }
            Self::InvalidValue(value) => {
                write!(f, "invalid Criterion timing value: {value}")
            }
            Self::UnknownUnit(unit) => {
                write!(f, "unknown Criterion timing unit: {unit}")
            }
            Self::Overflow(value) => {
                write!(f, "Criterion timing value overflowed: {value}")
            }
        }
    }
}

impl std::error::Error for MappedScratchThpBenchmarkTimingsParseError {}

/// Parses mapped scratch THP benchmark output into one evidence report.
///
/// # Errors
///
/// Returns an error when page-size sample lines or fault sample lines are
/// missing, duplicated, or malformed.
pub fn parse_mapped_scratch_thp_benchmark_evidence_report_output(
    output: &str,
) -> Result<
    MappedScratchThpBenchmarkEvidenceReport,
    MappedScratchThpBenchmarkEvidenceReportParseError,
> {
    let page_samples = parse_mapped_scratch_thp_page_samples_output(output)
        .map_err(MappedScratchThpBenchmarkEvidenceReportParseError::PageSamples)?;
    let fault_gate = evaluate_mapped_scratch_thp_fault_sample_validation_output(output)
        .map_err(MappedScratchThpBenchmarkEvidenceReportParseError::FaultSamples)?;
    let timings = parse_mapped_scratch_thp_benchmark_timings(output)
        .map_err(MappedScratchThpBenchmarkEvidenceReportParseError::Timings)?;

    Ok(MappedScratchThpBenchmarkEvidenceReport::from_parts(
        page_samples,
        fault_gate,
        timings,
    ))
}

fn benchmark_evidence_reason(
    page_samples: &MappedScratchThpPageSamples,
    fault_gate: &MappedScratchThpFaultSampleValidationGate,
) -> MappedScratchThpBenchmarkEvidenceReason {
    if !page_samples_available(page_samples) {
        return MappedScratchThpBenchmarkEvidenceReason::PageSamplesUnavailable;
    }

    if fault_gate.status != MappedScratchThpFaultSampleValidationGateStatus::Ready {
        return MappedScratchThpBenchmarkEvidenceReason::FaultSamplesUnavailable;
    }

    MappedScratchThpBenchmarkEvidenceReason::Ready
}

fn parse_mapped_scratch_thp_benchmark_timings(
    output: &str,
) -> Result<MappedScratchThpBenchmarkTimings, MappedScratchThpBenchmarkTimingsParseError> {
    let mut default = None;
    let mut hugepage = None;
    let mut no_hugepage = None;
    let mut current = None;

    for line in output.lines().map(str::trim) {
        match line {
            "mapped_scratch_write_touch_4mib_default" => {
                current = Some("mapped_scratch_write_touch_4mib_default");
                continue;
            }
            "mapped_scratch_write_touch_4mib_hugepage_advice" => {
                current = Some("mapped_scratch_write_touch_4mib_hugepage_advice");
                continue;
            }
            "mapped_scratch_write_touch_4mib_no_hugepage_advice" => {
                current = Some("mapped_scratch_write_touch_4mib_no_hugepage_advice");
                continue;
            }
            _ => {}
        }

        let Some(name) = current else {
            continue;
        };

        if !line.starts_with("time:") {
            continue;
        }

        let interval = parse_criterion_timing_interval(line)?;
        match name {
            "mapped_scratch_write_touch_4mib_default" => {
                set_timing(&mut default, name, interval)?;
            }
            "mapped_scratch_write_touch_4mib_hugepage_advice" => {
                set_timing(&mut hugepage, name, interval)?;
            }
            "mapped_scratch_write_touch_4mib_no_hugepage_advice" => {
                set_timing(&mut no_hugepage, name, interval)?;
            }
            _ => {}
        }
        current = None;
    }

    Ok(MappedScratchThpBenchmarkTimings {
        default: default.ok_or(
            MappedScratchThpBenchmarkTimingsParseError::MissingBenchmark(
                "mapped_scratch_write_touch_4mib_default",
            ),
        )?,
        hugepage: hugepage.ok_or(
            MappedScratchThpBenchmarkTimingsParseError::MissingBenchmark(
                "mapped_scratch_write_touch_4mib_hugepage_advice",
            ),
        )?,
        no_hugepage: no_hugepage.ok_or(
            MappedScratchThpBenchmarkTimingsParseError::MissingBenchmark(
                "mapped_scratch_write_touch_4mib_no_hugepage_advice",
            ),
        )?,
    })
}

fn parse_criterion_timing_interval(
    line: &str,
) -> Result<MappedScratchThpTimingInterval, MappedScratchThpBenchmarkTimingsParseError> {
    let start = line.find('[').ok_or_else(|| {
        MappedScratchThpBenchmarkTimingsParseError::MissingInterval(line.to_owned())
    })?;
    let end = line.find(']').ok_or_else(|| {
        MappedScratchThpBenchmarkTimingsParseError::MissingInterval(line.to_owned())
    })?;
    if end <= start {
        return Err(MappedScratchThpBenchmarkTimingsParseError::InvalidInterval(
            line.to_owned(),
        ));
    }

    let tokens = line[start + 1..end].split_whitespace().collect::<Vec<_>>();
    if tokens.len() != 6 {
        return Err(MappedScratchThpBenchmarkTimingsParseError::InvalidInterval(
            line.to_owned(),
        ));
    }

    Ok(MappedScratchThpTimingInterval {
        lower_ps: parse_timing_value_ps(tokens[0], tokens[1])?,
        estimate_ps: parse_timing_value_ps(tokens[2], tokens[3])?,
        upper_ps: parse_timing_value_ps(tokens[4], tokens[5])?,
    })
}

fn parse_timing_value_ps(
    value: &str,
    unit: &str,
) -> Result<u128, MappedScratchThpBenchmarkTimingsParseError> {
    let scale_ps = match unit {
        "ps" => 1,
        "ns" => 1_000,
        "us" | "\u{00b5}s" => 1_000_000,
        "ms" => 1_000_000_000,
        "s" => 1_000_000_000_000,
        _ => {
            return Err(MappedScratchThpBenchmarkTimingsParseError::UnknownUnit(
                unit.to_owned(),
            ));
        }
    };

    parse_decimal_scaled(value, scale_ps)
}

fn parse_decimal_scaled(
    value: &str,
    scale: u128,
) -> Result<u128, MappedScratchThpBenchmarkTimingsParseError> {
    let (whole, fractional) = value.split_once('.').unwrap_or((value, ""));
    if whole.is_empty()
        || !whole.chars().all(|value| value.is_ascii_digit())
        || !fractional.chars().all(|value| value.is_ascii_digit())
    {
        return Err(MappedScratchThpBenchmarkTimingsParseError::InvalidValue(
            value.to_owned(),
        ));
    }

    let whole = whole
        .parse::<u128>()
        .map_err(|_| MappedScratchThpBenchmarkTimingsParseError::InvalidValue(value.to_owned()))?;
    let whole_scaled = whole
        .checked_mul(scale)
        .ok_or_else(|| MappedScratchThpBenchmarkTimingsParseError::Overflow(value.to_owned()))?;

    if fractional.is_empty() {
        return Ok(whole_scaled);
    }

    let fractional_value = fractional
        .parse::<u128>()
        .map_err(|_| MappedScratchThpBenchmarkTimingsParseError::InvalidValue(value.to_owned()))?;
    let divisor = 10_u128
        .checked_pow(
            u32::try_from(fractional.len()).map_err(|_| {
                MappedScratchThpBenchmarkTimingsParseError::Overflow(value.to_owned())
            })?,
        )
        .ok_or_else(|| MappedScratchThpBenchmarkTimingsParseError::Overflow(value.to_owned()))?;
    let fractional_scaled = fractional_value
        .checked_mul(scale)
        .ok_or_else(|| MappedScratchThpBenchmarkTimingsParseError::Overflow(value.to_owned()))?
        / divisor;

    whole_scaled
        .checked_add(fractional_scaled)
        .ok_or_else(|| MappedScratchThpBenchmarkTimingsParseError::Overflow(value.to_owned()))
}

fn set_timing(
    slot: &mut Option<MappedScratchThpTimingInterval>,
    name: &'static str,
    interval: MappedScratchThpTimingInterval,
) -> Result<(), MappedScratchThpBenchmarkTimingsParseError> {
    if slot.replace(interval).is_some() {
        return Err(MappedScratchThpBenchmarkTimingsParseError::DuplicateBenchmark(name));
    }
    Ok(())
}

fn page_samples_available(samples: &MappedScratchThpPageSamples) -> bool {
    [
        samples.default.status,
        samples.hugepage.status,
        samples.no_hugepage.status,
    ]
    .into_iter()
    .all(|status| status == MappedScratchThpPageSampleStatus::Available)
        && [
            samples.default.source,
            samples.hugepage.source,
            samples.no_hugepage.source,
        ]
        .into_iter()
        .all(|source| source != MappedScratchThpPageSampleSource::None)
}

#[cfg(test)]
mod tests {
    use super::{
        parse_mapped_scratch_thp_benchmark_evidence_report_output,
        MappedScratchThpBenchmarkEvidenceReason, MappedScratchThpBenchmarkEvidenceStatus,
    };
    use locus_alloc::MappedScratchThpObservation;

    const READY_BASE_PAGE_REPORT: &str = "\
thp_page_sample=default status=available source=smaps kernel_page_kb=4 thp_observed=no reason=base_page_size
thp_page_sample=hugepage status=available source=smaps kernel_page_kb=4 thp_observed=no reason=base_page_size
thp_page_sample=no_hugepage status=available source=smaps kernel_page_kb=4 thp_observed=no reason=base_page_size
fault_sample=default status=available iterations=8 minor_faults_delta=16400 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
fault_sample=hugepage status=available iterations=8 minor_faults_delta=8224 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
fault_sample=no_hugepage status=available iterations=8 minor_faults_delta=16400 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
";

    const READY_HUGEPAGE_REPORT: &str = "\
thp_page_sample=default status=available source=smaps kernel_page_kb=4 thp_observed=no reason=base_page_size
thp_page_sample=hugepage status=available source=smaps kernel_page_kb=2048 thp_observed=yes reason=kernel_page_size
thp_page_sample=no_hugepage status=available source=smaps kernel_page_kb=4 thp_observed=no reason=base_page_size
fault_sample=default status=available iterations=8 minor_faults_delta=16400 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
fault_sample=hugepage status=available iterations=8 minor_faults_delta=8224 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
fault_sample=no_hugepage status=available iterations=8 minor_faults_delta=16400 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
";

    const CRITERION_TIMINGS: &str = "\
mapped_scratch_write_touch_4mib_default
                        time:   [878.54 \u{00b5}s 1.0696 ms 1.4043 ms]

mapped_scratch_write_touch_4mib_hugepage_advice
                        time:   [31.610 \u{00b5}s 31.839 \u{00b5}s 32.391 \u{00b5}s]

mapped_scratch_write_touch_4mib_no_hugepage_advice
                        time:   [804.15 \u{00b5}s 813.29 \u{00b5}s 818.20 \u{00b5}s]
";

    const ASCII_UNIT_TIMINGS: &str = "\
mapped_scratch_write_touch_4mib_default
                        time:   [12 ps 34 ns 5 us]

mapped_scratch_write_touch_4mib_hugepage_advice
                        time:   [1.5 ms 2.0 ms 3.25 ms]

mapped_scratch_write_touch_4mib_no_hugepage_advice
                        time:   [1 s 1.25 s 2 s]
";

    #[test]
    fn parses_ready_base_page_report() {
        let output = format!("{READY_BASE_PAGE_REPORT}{CRITERION_TIMINGS}");
        let report =
            parse_mapped_scratch_thp_benchmark_evidence_report_output(&output).expect("report");

        assert_eq!(
            report.status,
            MappedScratchThpBenchmarkEvidenceStatus::Ready
        );
        assert_eq!(
            report.reason,
            MappedScratchThpBenchmarkEvidenceReason::Ready
        );
        assert!(report.is_ready());
        assert_eq!(
            report.hugepage_observation(),
            MappedScratchThpObservation::No
        );
        assert!(!report.observed_hugepage_adoption());
        assert_eq!(
            report.to_string(),
            "mapped_scratch_thp_benchmark_evidence=ready reason=ready page_samples=available fault_samples=ready hugepage_observed=no hugepage_reason=base_page_size hugepage_source=smaps hugepage_kernel_page_kb=4 hugepage_adoption=false fault_comparison=available hugepage_vs_default_minor_faults_delta=-8176 hugepage_vs_no_hugepage_minor_faults_delta=-8176 major_faults_observed=false default_time_lower_ps=878540000 default_time_estimate_ps=1069600000 default_time_upper_ps=1404300000 hugepage_time_lower_ps=31610000 hugepage_time_estimate_ps=31839000 hugepage_time_upper_ps=32391000 no_hugepage_time_lower_ps=804150000 no_hugepage_time_estimate_ps=813290000 no_hugepage_time_upper_ps=818200000"
        );
    }

    #[test]
    fn parses_ready_hugepage_report() {
        let output = format!("{READY_HUGEPAGE_REPORT}{CRITERION_TIMINGS}");
        let report =
            parse_mapped_scratch_thp_benchmark_evidence_report_output(&output).expect("report");

        assert_eq!(
            report.status,
            MappedScratchThpBenchmarkEvidenceStatus::Ready
        );
        assert_eq!(
            report.hugepage_observation(),
            MappedScratchThpObservation::Yes
        );
        assert!(report.observed_hugepage_adoption());
    }

    #[test]
    fn reports_unavailable_page_samples() {
        let output = format!(
            "{}{}",
            "thp_page_sample=default status=available source=smaps kernel_page_kb=4 thp_observed=no reason=base_page_size
thp_page_sample=hugepage status=unavailable source=none kernel_page_kb=unknown thp_observed=unknown reason=observability_unavailable
thp_page_sample=no_hugepage status=available source=smaps kernel_page_kb=4 thp_observed=no reason=base_page_size
fault_sample=default status=available iterations=8 minor_faults_delta=16400 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
fault_sample=hugepage status=available iterations=8 minor_faults_delta=8224 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
fault_sample=no_hugepage status=available iterations=8 minor_faults_delta=16400 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
",
            CRITERION_TIMINGS
        );
        let report =
            parse_mapped_scratch_thp_benchmark_evidence_report_output(&output).expect("report");

        assert_eq!(
            report.status,
            MappedScratchThpBenchmarkEvidenceStatus::Unavailable
        );
        assert_eq!(
            report.reason,
            MappedScratchThpBenchmarkEvidenceReason::PageSamplesUnavailable
        );
        assert!(!report.is_ready());
    }

    #[test]
    fn reports_unavailable_fault_samples() {
        let output = format!(
            "{}{}",
            "thp_page_sample=default status=available source=smaps kernel_page_kb=4 thp_observed=no reason=base_page_size
thp_page_sample=hugepage status=available source=smaps kernel_page_kb=4 thp_observed=no reason=base_page_size
thp_page_sample=no_hugepage status=available source=smaps kernel_page_kb=4 thp_observed=no reason=base_page_size
fault_sample=default status=unavailable
fault_sample=hugepage status=unavailable
fault_sample=no_hugepage status=unavailable
",
            CRITERION_TIMINGS
        );
        let report =
            parse_mapped_scratch_thp_benchmark_evidence_report_output(&output).expect("report");

        assert_eq!(
            report.status,
            MappedScratchThpBenchmarkEvidenceStatus::Unavailable
        );
        assert_eq!(
            report.reason,
            MappedScratchThpBenchmarkEvidenceReason::FaultSamplesUnavailable
        );
    }

    #[test]
    fn parses_timing_units_to_picoseconds() {
        let output = format!("{READY_BASE_PAGE_REPORT}{ASCII_UNIT_TIMINGS}");
        let report =
            parse_mapped_scratch_thp_benchmark_evidence_report_output(&output).expect("report");

        assert_eq!(report.timings.default.lower_ps, 12);
        assert_eq!(report.timings.default.estimate_ps, 34_000);
        assert_eq!(report.timings.default.upper_ps, 5_000_000);
        assert_eq!(report.timings.hugepage.lower_ps, 1_500_000_000);
        assert_eq!(report.timings.hugepage.estimate_ps, 2_000_000_000);
        assert_eq!(report.timings.hugepage.upper_ps, 3_250_000_000);
        assert_eq!(report.timings.no_hugepage.lower_ps, 1_000_000_000_000);
        assert_eq!(report.timings.no_hugepage.estimate_ps, 1_250_000_000_000);
        assert_eq!(report.timings.no_hugepage.upper_ps, 2_000_000_000_000);
    }

    #[test]
    fn rejects_missing_timing_block() {
        let output = format!(
            "{}{}",
            READY_BASE_PAGE_REPORT,
            "\
mapped_scratch_write_touch_4mib_default
                        time:   [878.54 us 1.0696 ms 1.4043 ms]

mapped_scratch_write_touch_4mib_hugepage_advice
                        time:   [31.610 us 31.839 us 32.391 us]
"
        );
        let error =
            parse_mapped_scratch_thp_benchmark_evidence_report_output(&output).expect_err("error");

        assert!(error
            .to_string()
            .contains("mapped_scratch_write_touch_4mib_no_hugepage_advice"));
    }

    #[test]
    fn rejects_duplicate_timing_block() {
        let output = format!(
            "{}{}{}",
            READY_BASE_PAGE_REPORT,
            CRITERION_TIMINGS,
            "\
mapped_scratch_write_touch_4mib_default
                        time:   [900 us 1 ms 2 ms]
"
        );
        let error =
            parse_mapped_scratch_thp_benchmark_evidence_report_output(&output).expect_err("error");

        assert!(error.to_string().contains("duplicate"));
        assert!(error
            .to_string()
            .contains("mapped_scratch_write_touch_4mib_default"));
    }

    #[test]
    fn rejects_unknown_timing_unit() {
        let output = format!(
            "{}{}",
            READY_BASE_PAGE_REPORT,
            "\
mapped_scratch_write_touch_4mib_default
                        time:   [878.54 ticks 1.0696 ms 1.4043 ms]

mapped_scratch_write_touch_4mib_hugepage_advice
                        time:   [31.610 us 31.839 us 32.391 us]

mapped_scratch_write_touch_4mib_no_hugepage_advice
                        time:   [804.15 us 813.29 us 818.20 us]
"
        );
        let error =
            parse_mapped_scratch_thp_benchmark_evidence_report_output(&output).expect_err("error");

        assert!(error.to_string().contains("unknown Criterion timing unit"));
        assert!(error.to_string().contains("ticks"));
    }

    #[test]
    fn rejects_missing_page_samples() {
        let error = parse_mapped_scratch_thp_benchmark_evidence_report_output(
            "fault_sample=default status=unavailable
fault_sample=hugepage status=unavailable
fault_sample=no_hugepage status=unavailable
",
        )
        .expect_err("missing page samples");

        assert!(error.to_string().contains("page samples"));
    }
}
