use std::{collections::BTreeMap, fmt};

use crate::MappedScratchThpBenchmarkEvidenceStatus;

/// Page-size evidence cohort for a repeated mapped scratch THP report run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MappedScratchThpBenchmarkEvidenceRunCohort {
    /// Every report line had the same hugepage page-size evidence.
    Consistent {
        /// Hugepage observation value shared by all report lines.
        hugepage_observed: String,
        /// Hugepage evidence source shared by all report lines.
        hugepage_source: String,
        /// Hugepage kernel page size shared by all report lines.
        hugepage_kernel_page_kb: String,
    },
    /// Report lines had different hugepage page-size evidence.
    Mixed,
}

impl MappedScratchThpBenchmarkEvidenceRunCohort {
    /// Returns a stable machine-readable cohort label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Consistent { .. } => "consistent",
            Self::Mixed => "mixed",
        }
    }
}

/// Summary for repeated compact mapped scratch THP benchmark evidence reports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MappedScratchThpBenchmarkEvidenceRunSummary {
    /// Number of compact report lines.
    pub reports: usize,
    /// Number of ready report lines.
    pub ready_reports: usize,
    /// Number of unavailable report lines.
    pub unavailable_reports: usize,
    /// Number of report lines with observed hugepage adoption.
    pub hugepage_adoption_reports: usize,
    /// Number of report lines with base-page hugepage advice observations.
    pub base_page_reports: usize,
    /// Number of report lines that observed major faults.
    pub major_fault_reports: usize,
    /// Minimum default estimate timing in picoseconds.
    pub default_time_estimate_min_ps: u128,
    /// Maximum default estimate timing in picoseconds.
    pub default_time_estimate_max_ps: u128,
    /// Minimum hugepage estimate timing in picoseconds.
    pub hugepage_time_estimate_min_ps: u128,
    /// Maximum hugepage estimate timing in picoseconds.
    pub hugepage_time_estimate_max_ps: u128,
    /// Minimum no-hugepage estimate timing in picoseconds.
    pub no_hugepage_time_estimate_min_ps: u128,
    /// Maximum no-hugepage estimate timing in picoseconds.
    pub no_hugepage_time_estimate_max_ps: u128,
    /// Minimum hugepage minus default estimate timing in picoseconds.
    pub hugepage_vs_default_time_estimate_min_delta_ps: i128,
    /// Maximum hugepage minus default estimate timing in picoseconds.
    pub hugepage_vs_default_time_estimate_max_delta_ps: i128,
    /// Page-size evidence cohort classification.
    pub page_evidence_cohort: MappedScratchThpBenchmarkEvidenceRunCohort,
}

impl fmt::Display for MappedScratchThpBenchmarkEvidenceRunSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "mapped_scratch_thp_benchmark_evidence_runs=ready reports={} ready_reports={} unavailable_reports={} hugepage_adoption_reports={} base_page_reports={} major_fault_reports={} default_time_estimate_min_ps={} default_time_estimate_max_ps={} hugepage_time_estimate_min_ps={} hugepage_time_estimate_max_ps={} no_hugepage_time_estimate_min_ps={} no_hugepage_time_estimate_max_ps={} hugepage_vs_default_time_estimate_min_delta_ps={} hugepage_vs_default_time_estimate_max_delta_ps={} page_evidence_cohort={}",
            self.reports,
            self.ready_reports,
            self.unavailable_reports,
            self.hugepage_adoption_reports,
            self.base_page_reports,
            self.major_fault_reports,
            self.default_time_estimate_min_ps,
            self.default_time_estimate_max_ps,
            self.hugepage_time_estimate_min_ps,
            self.hugepage_time_estimate_max_ps,
            self.no_hugepage_time_estimate_min_ps,
            self.no_hugepage_time_estimate_max_ps,
            self.hugepage_vs_default_time_estimate_min_delta_ps,
            self.hugepage_vs_default_time_estimate_max_delta_ps,
            self.page_evidence_cohort.as_str()
        )?;

        if let MappedScratchThpBenchmarkEvidenceRunCohort::Consistent {
            hugepage_observed,
            hugepage_source,
            hugepage_kernel_page_kb,
        } = &self.page_evidence_cohort
        {
            write!(
                f,
                " cohort_hugepage_observed={hugepage_observed} cohort_hugepage_source={hugepage_source} cohort_hugepage_kernel_page_kb={hugepage_kernel_page_kb}"
            )?;
        }

        Ok(())
    }
}

/// Error returned when summarizing compact mapped scratch THP report lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MappedScratchThpBenchmarkEvidenceRunSummaryParseError {
    /// No compact report lines were found.
    Empty,
    /// A non-empty line was not a compact report line.
    InvalidLine(String),
    /// A key appeared more than once in one line.
    DuplicateField(String),
    /// A required key was missing.
    MissingField(&'static str),
    /// A status field was not recognized.
    InvalidStatus(String),
    /// A boolean field was not recognized.
    InvalidBool {
        /// Field name.
        field: &'static str,
        /// Invalid boolean value.
        value: String,
    },
    /// An unsigned integer field was not recognized.
    InvalidUnsigned {
        /// Field name.
        field: &'static str,
        /// Invalid unsigned value.
        value: String,
    },
    /// A timing delta overflowed signed representation.
    DeltaOverflow,
}

impl fmt::Display for MappedScratchThpBenchmarkEvidenceRunSummaryParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("missing compact mapped scratch THP report lines"),
            Self::InvalidLine(line) => {
                write!(f, "invalid compact mapped scratch THP report line: {line}")
            }
            Self::DuplicateField(field) => {
                write!(
                    f,
                    "duplicate compact mapped scratch THP report field: {field}"
                )
            }
            Self::MissingField(field) => {
                write!(
                    f,
                    "missing compact mapped scratch THP report field: {field}"
                )
            }
            Self::InvalidStatus(value) => {
                write!(f, "invalid compact mapped scratch THP status: {value}")
            }
            Self::InvalidBool { field, value } => {
                write!(
                    f,
                    "invalid compact mapped scratch THP bool {field}: {value}"
                )
            }
            Self::InvalidUnsigned { field, value } => {
                write!(
                    f,
                    "invalid compact mapped scratch THP unsigned {field}: {value}"
                )
            }
            Self::DeltaOverflow => {
                f.write_str("compact mapped scratch THP timing delta overflowed")
            }
        }
    }
}

impl std::error::Error for MappedScratchThpBenchmarkEvidenceRunSummaryParseError {}

/// Summarizes compact mapped scratch THP benchmark evidence report lines.
///
/// # Errors
///
/// Returns an error when no compact report lines are present or when a report
/// line is malformed.
pub fn summarize_mapped_scratch_thp_benchmark_evidence_report_lines(
    output: &str,
) -> Result<
    MappedScratchThpBenchmarkEvidenceRunSummary,
    MappedScratchThpBenchmarkEvidenceRunSummaryParseError,
> {
    let mut reports = Vec::new();
    for line in output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        if !line.starts_with("mapped_scratch_thp_benchmark_evidence=") {
            return Err(
                MappedScratchThpBenchmarkEvidenceRunSummaryParseError::InvalidLine(line.to_owned()),
            );
        }
        reports.push(parse_compact_report_line(line)?);
    }

    let first = reports
        .first()
        .ok_or(MappedScratchThpBenchmarkEvidenceRunSummaryParseError::Empty)?;
    let mut summary = MappedScratchThpBenchmarkEvidenceRunSummary {
        reports: 0,
        ready_reports: 0,
        unavailable_reports: 0,
        hugepage_adoption_reports: 0,
        base_page_reports: 0,
        major_fault_reports: 0,
        default_time_estimate_min_ps: first.default_time_estimate_ps,
        default_time_estimate_max_ps: first.default_time_estimate_ps,
        hugepage_time_estimate_min_ps: first.hugepage_time_estimate_ps,
        hugepage_time_estimate_max_ps: first.hugepage_time_estimate_ps,
        no_hugepage_time_estimate_min_ps: first.no_hugepage_time_estimate_ps,
        no_hugepage_time_estimate_max_ps: first.no_hugepage_time_estimate_ps,
        hugepage_vs_default_time_estimate_min_delta_ps: first
            .hugepage_vs_default_time_estimate_delta_ps,
        hugepage_vs_default_time_estimate_max_delta_ps: first
            .hugepage_vs_default_time_estimate_delta_ps,
        page_evidence_cohort: MappedScratchThpBenchmarkEvidenceRunCohort::Consistent {
            hugepage_observed: first.hugepage_observed.clone(),
            hugepage_source: first.hugepage_source.clone(),
            hugepage_kernel_page_kb: first.hugepage_kernel_page_kb.clone(),
        },
    };
    let first_page_evidence = first.page_evidence_key();

    for report in &reports {
        summary.reports += 1;
        match report.status {
            MappedScratchThpBenchmarkEvidenceStatus::Ready => summary.ready_reports += 1,
            MappedScratchThpBenchmarkEvidenceStatus::Unavailable => {
                summary.unavailable_reports += 1;
            }
        }
        if report.hugepage_adoption {
            summary.hugepage_adoption_reports += 1;
        }
        if report.hugepage_observed == "no" {
            summary.base_page_reports += 1;
        }
        if report.major_faults_observed {
            summary.major_fault_reports += 1;
        }

        summary.default_time_estimate_min_ps = summary
            .default_time_estimate_min_ps
            .min(report.default_time_estimate_ps);
        summary.default_time_estimate_max_ps = summary
            .default_time_estimate_max_ps
            .max(report.default_time_estimate_ps);
        summary.hugepage_time_estimate_min_ps = summary
            .hugepage_time_estimate_min_ps
            .min(report.hugepage_time_estimate_ps);
        summary.hugepage_time_estimate_max_ps = summary
            .hugepage_time_estimate_max_ps
            .max(report.hugepage_time_estimate_ps);
        summary.no_hugepage_time_estimate_min_ps = summary
            .no_hugepage_time_estimate_min_ps
            .min(report.no_hugepage_time_estimate_ps);
        summary.no_hugepage_time_estimate_max_ps = summary
            .no_hugepage_time_estimate_max_ps
            .max(report.no_hugepage_time_estimate_ps);
        summary.hugepage_vs_default_time_estimate_min_delta_ps = summary
            .hugepage_vs_default_time_estimate_min_delta_ps
            .min(report.hugepage_vs_default_time_estimate_delta_ps);
        summary.hugepage_vs_default_time_estimate_max_delta_ps = summary
            .hugepage_vs_default_time_estimate_max_delta_ps
            .max(report.hugepage_vs_default_time_estimate_delta_ps);

        if report.page_evidence_key() != first_page_evidence {
            summary.page_evidence_cohort = MappedScratchThpBenchmarkEvidenceRunCohort::Mixed;
        }
    }

    Ok(summary)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompactReportLine {
    status: MappedScratchThpBenchmarkEvidenceStatus,
    hugepage_observed: String,
    hugepage_source: String,
    hugepage_kernel_page_kb: String,
    hugepage_adoption: bool,
    major_faults_observed: bool,
    default_time_estimate_ps: u128,
    hugepage_time_estimate_ps: u128,
    no_hugepage_time_estimate_ps: u128,
    hugepage_vs_default_time_estimate_delta_ps: i128,
}

impl CompactReportLine {
    fn page_evidence_key(&self) -> (&str, &str, &str) {
        (
            &self.hugepage_observed,
            &self.hugepage_source,
            &self.hugepage_kernel_page_kb,
        )
    }
}

fn parse_compact_report_line(
    line: &str,
) -> Result<CompactReportLine, MappedScratchThpBenchmarkEvidenceRunSummaryParseError> {
    let fields = parse_compact_report_fields(line)?;
    let status = parse_compact_report_status(required_field(
        &fields,
        "mapped_scratch_thp_benchmark_evidence",
    )?)?;
    let hugepage_observed = required_field(&fields, "hugepage_observed")?.to_owned();
    let hugepage_source = required_field(&fields, "hugepage_source")?.to_owned();
    let hugepage_kernel_page_kb = required_field(&fields, "hugepage_kernel_page_kb")?.to_owned();
    let hugepage_adoption = parse_compact_report_bool(
        "hugepage_adoption",
        required_field(&fields, "hugepage_adoption")?,
    )?;
    let major_faults_observed = match fields.get("major_faults_observed") {
        Some(value) => parse_compact_report_bool("major_faults_observed", value)?,
        None => false,
    };
    let default_time_estimate_ps = parse_compact_report_u128(&fields, "default_time_estimate_ps")?;
    let hugepage_time_estimate_ps =
        parse_compact_report_u128(&fields, "hugepage_time_estimate_ps")?;
    let no_hugepage_time_estimate_ps =
        parse_compact_report_u128(&fields, "no_hugepage_time_estimate_ps")?;
    let hugepage_vs_default_time_estimate_delta_ps =
        checked_timing_delta(hugepage_time_estimate_ps, default_time_estimate_ps)?;

    Ok(CompactReportLine {
        status,
        hugepage_observed,
        hugepage_source,
        hugepage_kernel_page_kb,
        hugepage_adoption,
        major_faults_observed,
        default_time_estimate_ps,
        hugepage_time_estimate_ps,
        no_hugepage_time_estimate_ps,
        hugepage_vs_default_time_estimate_delta_ps,
    })
}

fn parse_compact_report_fields(
    line: &str,
) -> Result<BTreeMap<&str, &str>, MappedScratchThpBenchmarkEvidenceRunSummaryParseError> {
    let mut fields = BTreeMap::new();
    for token in line.split_whitespace() {
        let (key, value) = token.split_once('=').ok_or_else(|| {
            MappedScratchThpBenchmarkEvidenceRunSummaryParseError::InvalidLine(line.to_owned())
        })?;
        if fields.insert(key, value).is_some() {
            return Err(
                MappedScratchThpBenchmarkEvidenceRunSummaryParseError::DuplicateField(
                    key.to_owned(),
                ),
            );
        }
    }
    Ok(fields)
}

fn required_field<'a>(
    fields: &'a BTreeMap<&str, &str>,
    field: &'static str,
) -> Result<&'a str, MappedScratchThpBenchmarkEvidenceRunSummaryParseError> {
    fields
        .get(field)
        .copied()
        .ok_or(MappedScratchThpBenchmarkEvidenceRunSummaryParseError::MissingField(field))
}

fn parse_compact_report_status(
    value: &str,
) -> Result<
    MappedScratchThpBenchmarkEvidenceStatus,
    MappedScratchThpBenchmarkEvidenceRunSummaryParseError,
> {
    match value {
        "ready" => Ok(MappedScratchThpBenchmarkEvidenceStatus::Ready),
        "unavailable" => Ok(MappedScratchThpBenchmarkEvidenceStatus::Unavailable),
        _ => Err(
            MappedScratchThpBenchmarkEvidenceRunSummaryParseError::InvalidStatus(value.to_owned()),
        ),
    }
}

fn parse_compact_report_bool(
    field: &'static str,
    value: &str,
) -> Result<bool, MappedScratchThpBenchmarkEvidenceRunSummaryParseError> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(
            MappedScratchThpBenchmarkEvidenceRunSummaryParseError::InvalidBool {
                field,
                value: value.to_owned(),
            },
        ),
    }
}

fn parse_compact_report_u128(
    fields: &BTreeMap<&str, &str>,
    field: &'static str,
) -> Result<u128, MappedScratchThpBenchmarkEvidenceRunSummaryParseError> {
    let value = required_field(fields, field)?;
    value.parse::<u128>().map_err(|_| {
        MappedScratchThpBenchmarkEvidenceRunSummaryParseError::InvalidUnsigned {
            field,
            value: value.to_owned(),
        }
    })
}

fn checked_timing_delta(
    lhs: u128,
    rhs: u128,
) -> Result<i128, MappedScratchThpBenchmarkEvidenceRunSummaryParseError> {
    if lhs >= rhs {
        i128::try_from(lhs - rhs)
            .map_err(|_| MappedScratchThpBenchmarkEvidenceRunSummaryParseError::DeltaOverflow)
    } else {
        i128::try_from(rhs - lhs)
            .map(|value| -value)
            .map_err(|_| MappedScratchThpBenchmarkEvidenceRunSummaryParseError::DeltaOverflow)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        summarize_mapped_scratch_thp_benchmark_evidence_report_lines,
        MappedScratchThpBenchmarkEvidenceRunCohort,
    };

    #[derive(Debug, Clone, Copy)]
    struct CompactReportLineInput {
        status: &'static str,
        hugepage_observed: &'static str,
        hugepage_source: &'static str,
        hugepage_kernel_page_kb: &'static str,
        hugepage_adoption: bool,
        major_faults_observed: bool,
        default_time_estimate_ps: u128,
        hugepage_time_estimate_ps: u128,
        no_hugepage_time_estimate_ps: u128,
    }

    impl Default for CompactReportLineInput {
        fn default() -> Self {
            Self {
                status: "ready",
                hugepage_observed: "no",
                hugepage_source: "smaps",
                hugepage_kernel_page_kb: "4",
                hugepage_adoption: false,
                major_faults_observed: false,
                default_time_estimate_ps: 1_000,
                hugepage_time_estimate_ps: 300,
                no_hugepage_time_estimate_ps: 1_100,
            }
        }
    }

    fn compact_report_line(input: CompactReportLineInput) -> String {
        format!(
            "mapped_scratch_thp_benchmark_evidence={} hugepage_observed={} hugepage_source={} hugepage_kernel_page_kb={} hugepage_adoption={} major_faults_observed={} default_time_estimate_ps={} hugepage_time_estimate_ps={} no_hugepage_time_estimate_ps={}",
            input.status,
            input.hugepage_observed,
            input.hugepage_source,
            input.hugepage_kernel_page_kb,
            input.hugepage_adoption,
            input.major_faults_observed,
            input.default_time_estimate_ps,
            input.hugepage_time_estimate_ps,
            input.no_hugepage_time_estimate_ps
        )
    }

    #[test]
    fn summarizes_consistent_compact_report_lines() {
        let input = [
            compact_report_line(CompactReportLineInput::default()),
            compact_report_line(CompactReportLineInput {
                default_time_estimate_ps: 1_200,
                hugepage_time_estimate_ps: 400,
                no_hugepage_time_estimate_ps: 1_300,
                ..CompactReportLineInput::default()
            }),
            compact_report_line(CompactReportLineInput {
                status: "unavailable",
                major_faults_observed: true,
                default_time_estimate_ps: 900,
                hugepage_time_estimate_ps: 350,
                no_hugepage_time_estimate_ps: 1_000,
                ..CompactReportLineInput::default()
            }),
        ]
        .join("\n");
        let summary =
            summarize_mapped_scratch_thp_benchmark_evidence_report_lines(&input).expect("summary");

        assert_eq!(summary.reports, 3);
        assert_eq!(summary.ready_reports, 2);
        assert_eq!(summary.unavailable_reports, 1);
        assert_eq!(summary.hugepage_adoption_reports, 0);
        assert_eq!(summary.base_page_reports, 3);
        assert_eq!(summary.major_fault_reports, 1);
        assert_eq!(summary.default_time_estimate_min_ps, 900);
        assert_eq!(summary.default_time_estimate_max_ps, 1_200);
        assert_eq!(summary.hugepage_time_estimate_min_ps, 300);
        assert_eq!(summary.hugepage_time_estimate_max_ps, 400);
        assert_eq!(summary.no_hugepage_time_estimate_min_ps, 1_000);
        assert_eq!(summary.no_hugepage_time_estimate_max_ps, 1_300);
        assert_eq!(summary.hugepage_vs_default_time_estimate_min_delta_ps, -800);
        assert_eq!(summary.hugepage_vs_default_time_estimate_max_delta_ps, -550);
        assert_eq!(
            summary.to_string(),
            "mapped_scratch_thp_benchmark_evidence_runs=ready reports=3 ready_reports=2 unavailable_reports=1 hugepage_adoption_reports=0 base_page_reports=3 major_fault_reports=1 default_time_estimate_min_ps=900 default_time_estimate_max_ps=1200 hugepage_time_estimate_min_ps=300 hugepage_time_estimate_max_ps=400 no_hugepage_time_estimate_min_ps=1000 no_hugepage_time_estimate_max_ps=1300 hugepage_vs_default_time_estimate_min_delta_ps=-800 hugepage_vs_default_time_estimate_max_delta_ps=-550 page_evidence_cohort=consistent cohort_hugepage_observed=no cohort_hugepage_source=smaps cohort_hugepage_kernel_page_kb=4"
        );
    }

    #[test]
    fn summarizes_mixed_page_evidence_report_lines() {
        let input = [
            compact_report_line(CompactReportLineInput::default()),
            compact_report_line(CompactReportLineInput {
                hugepage_observed: "yes",
                hugepage_kernel_page_kb: "2048",
                hugepage_adoption: true,
                hugepage_time_estimate_ps: 1_200,
                ..CompactReportLineInput::default()
            }),
        ]
        .join("\n");
        let summary =
            summarize_mapped_scratch_thp_benchmark_evidence_report_lines(&input).expect("summary");

        assert_eq!(summary.hugepage_adoption_reports, 1);
        assert_eq!(summary.base_page_reports, 1);
        assert_eq!(
            summary.page_evidence_cohort,
            MappedScratchThpBenchmarkEvidenceRunCohort::Mixed
        );
        assert_eq!(summary.hugepage_vs_default_time_estimate_min_delta_ps, -700);
        assert_eq!(summary.hugepage_vs_default_time_estimate_max_delta_ps, 200);
        assert!(summary.to_string().ends_with("page_evidence_cohort=mixed"));
    }

    #[test]
    fn rejects_invalid_compact_report_summary_input() {
        let error = summarize_mapped_scratch_thp_benchmark_evidence_report_lines(
            "mapped_scratch_thp_benchmark_evidence=ready hugepage_observed=no hugepage_source=smaps hugepage_kernel_page_kb=4 hugepage_adoption=false",
        )
        .expect_err("error");

        assert!(error.to_string().contains("default_time_estimate_ps"));
    }

    #[test]
    fn rejects_duplicate_compact_report_summary_fields() {
        let line = format!(
            "{} hugepage_observed=yes",
            compact_report_line(CompactReportLineInput::default())
        );
        let error =
            summarize_mapped_scratch_thp_benchmark_evidence_report_lines(&line).expect_err("error");

        assert!(error.to_string().contains("duplicate"));
        assert!(error.to_string().contains("hugepage_observed"));
    }
}
