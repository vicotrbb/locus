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
}

impl MappedScratchThpBenchmarkEvidenceReport {
    /// Builds a report from parsed page-size samples and fault sample gate.
    #[must_use]
    pub fn from_parts(
        page_samples: MappedScratchThpPageSamples,
        fault_gate: MappedScratchThpFaultSampleValidationGate,
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
        }
    }
}

impl std::error::Error for MappedScratchThpBenchmarkEvidenceReportParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::PageSamples(source) => Some(source),
            Self::FaultSamples(source) => Some(source),
        }
    }
}

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

    Ok(MappedScratchThpBenchmarkEvidenceReport::from_parts(
        page_samples,
        fault_gate,
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

    #[test]
    fn parses_ready_base_page_report() {
        let report =
            parse_mapped_scratch_thp_benchmark_evidence_report_output(READY_BASE_PAGE_REPORT)
                .expect("report");

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
            "mapped_scratch_thp_benchmark_evidence=ready reason=ready page_samples=available fault_samples=ready hugepage_observed=no hugepage_reason=base_page_size hugepage_source=smaps hugepage_kernel_page_kb=4 hugepage_adoption=false fault_comparison=available hugepage_vs_default_minor_faults_delta=-8176 hugepage_vs_no_hugepage_minor_faults_delta=-8176 major_faults_observed=false"
        );
    }

    #[test]
    fn parses_ready_hugepage_report() {
        let report =
            parse_mapped_scratch_thp_benchmark_evidence_report_output(READY_HUGEPAGE_REPORT)
                .expect("report");

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
        let report = parse_mapped_scratch_thp_benchmark_evidence_report_output(
            "thp_page_sample=default status=available source=smaps kernel_page_kb=4 thp_observed=no reason=base_page_size
thp_page_sample=hugepage status=unavailable source=none kernel_page_kb=unknown thp_observed=unknown reason=observability_unavailable
thp_page_sample=no_hugepage status=available source=smaps kernel_page_kb=4 thp_observed=no reason=base_page_size
fault_sample=default status=available iterations=8 minor_faults_delta=16400 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
fault_sample=hugepage status=available iterations=8 minor_faults_delta=8224 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
fault_sample=no_hugepage status=available iterations=8 minor_faults_delta=16400 child_minor_faults_delta=0 major_faults_delta=0 child_major_faults_delta=0
",
        )
        .expect("report");

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
        let report = parse_mapped_scratch_thp_benchmark_evidence_report_output(
            "thp_page_sample=default status=available source=smaps kernel_page_kb=4 thp_observed=no reason=base_page_size
thp_page_sample=hugepage status=available source=smaps kernel_page_kb=4 thp_observed=no reason=base_page_size
thp_page_sample=no_hugepage status=available source=smaps kernel_page_kb=4 thp_observed=no reason=base_page_size
fault_sample=default status=unavailable
fault_sample=hugepage status=unavailable
fault_sample=no_hugepage status=unavailable
",
        )
        .expect("report");

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
