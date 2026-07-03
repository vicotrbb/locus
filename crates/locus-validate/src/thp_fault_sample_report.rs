use std::fmt;

use crate::{
    parse_mapped_scratch_thp_fault_sample_comparison_output,
    parse_mapped_scratch_thp_fault_sample_validation_gate_output,
    MappedScratchThpFaultSampleComparisonOutput,
    MappedScratchThpFaultSampleComparisonOutputParseError,
    MappedScratchThpFaultSampleComparisonReason, MappedScratchThpFaultSampleComparisonStatus,
    MappedScratchThpFaultSampleValidationGateOutputParseError,
    MappedScratchThpFaultSampleValidationGateReason,
    MappedScratchThpFaultSampleValidationGateStatus,
    MappedScratchThpFaultSampleValidationGateVerdict,
};

/// Parsed mapped scratch THP benchmark fault sample report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MappedScratchThpFaultSampleReport {
    /// Parsed fault sample availability gate verdict.
    pub gate: MappedScratchThpFaultSampleValidationGateVerdict,
    /// Parsed fault sample comparison output.
    pub comparison: MappedScratchThpFaultSampleComparisonOutput,
}

impl MappedScratchThpFaultSampleReport {
    /// Builds a report only when the gate and comparison outputs agree.
    ///
    /// # Errors
    ///
    /// Returns an error when the two parsed lines are individually valid but
    /// contradictory when read as one report.
    pub fn from_parts(
        gate: MappedScratchThpFaultSampleValidationGateVerdict,
        comparison: MappedScratchThpFaultSampleComparisonOutput,
    ) -> Result<Self, MappedScratchThpFaultSampleReportParseError> {
        let report = Self { gate, comparison };
        if report.is_consistent() {
            Ok(report)
        } else {
            Err(MappedScratchThpFaultSampleReportParseError::Inconsistent {
                gate_status: gate.status,
                gate_reason: gate.reason,
                comparison_status: comparison.status,
                comparison_reason: comparison.reason,
            })
        }
    }

    /// Returns true when the gate and comparison lines can describe the same report.
    #[must_use]
    pub fn is_consistent(self) -> bool {
        matches!(
            (
                self.gate.status,
                self.gate.reason,
                self.comparison.status,
                self.comparison.reason,
                self.comparison.comparison.is_some(),
            ),
            (
                MappedScratchThpFaultSampleValidationGateStatus::Ready,
                MappedScratchThpFaultSampleValidationGateReason::Ready,
                MappedScratchThpFaultSampleComparisonStatus::Available,
                MappedScratchThpFaultSampleComparisonReason::Ready,
                true
            ) | (
                MappedScratchThpFaultSampleValidationGateStatus::Ready,
                MappedScratchThpFaultSampleValidationGateReason::Ready,
                MappedScratchThpFaultSampleComparisonStatus::Unavailable,
                MappedScratchThpFaultSampleComparisonReason::ComparisonUnavailable,
                false
            ) | (
                MappedScratchThpFaultSampleValidationGateStatus::Unavailable,
                MappedScratchThpFaultSampleValidationGateReason::FaultCountersUnavailable,
                MappedScratchThpFaultSampleComparisonStatus::Unavailable,
                MappedScratchThpFaultSampleComparisonReason::FaultCountersUnavailable,
                false
            )
        )
    }
}

/// Error returned when parsing a mapped scratch THP fault sample report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MappedScratchThpFaultSampleReportParseError {
    /// The fault sample gate line is missing or malformed.
    Gate(MappedScratchThpFaultSampleValidationGateOutputParseError),
    /// The fault sample comparison line is missing or malformed.
    Comparison(MappedScratchThpFaultSampleComparisonOutputParseError),
    /// The gate and comparison lines are individually valid but contradictory together.
    Inconsistent {
        /// Parsed gate status.
        gate_status: MappedScratchThpFaultSampleValidationGateStatus,
        /// Parsed gate reason.
        gate_reason: MappedScratchThpFaultSampleValidationGateReason,
        /// Parsed comparison status.
        comparison_status: MappedScratchThpFaultSampleComparisonStatus,
        /// Parsed comparison reason.
        comparison_reason: MappedScratchThpFaultSampleComparisonReason,
    },
}

impl fmt::Display for MappedScratchThpFaultSampleReportParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Gate(source) => write!(
                f,
                "invalid mapped scratch THP fault sample report gate: {source}"
            ),
            Self::Comparison(source) => write!(
                f,
                "invalid mapped scratch THP fault sample report comparison: {source}"
            ),
            Self::Inconsistent {
                gate_status,
                gate_reason,
                comparison_status,
                comparison_reason,
            } => write!(
                f,
                "inconsistent mapped scratch THP fault sample report: gate={gate_status} reason={gate_reason} comparison={comparison_status} comparison_reason={comparison_reason}"
            ),
        }
    }
}

impl std::error::Error for MappedScratchThpFaultSampleReportParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Gate(source) => Some(source),
            Self::Comparison(source) => Some(source),
            Self::Inconsistent { .. } => None,
        }
    }
}

/// Parses mapped scratch THP fault sample validation command output as one report.
///
/// # Errors
///
/// Returns an error when either required line is missing, duplicated, malformed,
/// or inconsistent with the other line.
pub fn parse_mapped_scratch_thp_fault_sample_report_output(
    output: &str,
) -> Result<MappedScratchThpFaultSampleReport, MappedScratchThpFaultSampleReportParseError> {
    let gate = parse_mapped_scratch_thp_fault_sample_validation_gate_output(output)
        .map_err(MappedScratchThpFaultSampleReportParseError::Gate)?;
    let comparison = parse_mapped_scratch_thp_fault_sample_comparison_output(output)
        .map_err(MappedScratchThpFaultSampleReportParseError::Comparison)?;

    MappedScratchThpFaultSampleReport::from_parts(gate, comparison)
}

#[cfg(test)]
mod tests {
    use super::{
        parse_mapped_scratch_thp_fault_sample_report_output, MappedScratchThpFaultSampleReport,
        MappedScratchThpFaultSampleReportParseError,
    };
    use crate::{
        MappedScratchThpFaultSampleComparisonOutputParseError,
        MappedScratchThpFaultSampleComparisonReason, MappedScratchThpFaultSampleComparisonStatus,
        MappedScratchThpFaultSampleValidationGateOutputParseError,
        MappedScratchThpFaultSampleValidationGateReason,
        MappedScratchThpFaultSampleValidationGateStatus,
    };

    const READY_REPORT: &str = "\
mapped_scratch_thp_fault_sample_validation_gate=ready reason=ready
mapped_scratch_thp_fault_sample_comparison=available reason=ready default_minor_faults_delta=16400 hugepage_minor_faults_delta=8224 no_hugepage_minor_faults_delta=16400 hugepage_vs_default_minor_faults_delta=-8176 hugepage_vs_no_hugepage_minor_faults_delta=-8176 major_faults_observed=false
";

    const UNAVAILABLE_REPORT: &str = "\
mapped_scratch_thp_fault_sample_validation_gate=unavailable reason=fault_counters_unavailable
mapped_scratch_thp_fault_sample_comparison=unavailable reason=fault_counters_unavailable
";

    #[test]
    fn parses_ready_report() {
        let report =
            parse_mapped_scratch_thp_fault_sample_report_output(READY_REPORT).expect("report");

        assert_eq!(
            report.gate.status,
            MappedScratchThpFaultSampleValidationGateStatus::Ready
        );
        assert_eq!(
            report.gate.reason,
            MappedScratchThpFaultSampleValidationGateReason::Ready
        );
        assert_eq!(
            report.comparison.status,
            MappedScratchThpFaultSampleComparisonStatus::Available
        );
        assert_eq!(
            report.comparison.reason,
            MappedScratchThpFaultSampleComparisonReason::Ready
        );
        assert!(report.comparison.comparison.is_some());
        assert!(report.is_consistent());
    }

    #[test]
    fn parses_unavailable_report() {
        let report = parse_mapped_scratch_thp_fault_sample_report_output(UNAVAILABLE_REPORT)
            .expect("report");

        assert_eq!(
            report.gate.status,
            MappedScratchThpFaultSampleValidationGateStatus::Unavailable
        );
        assert_eq!(
            report.comparison.status,
            MappedScratchThpFaultSampleComparisonStatus::Unavailable
        );
        assert_eq!(report.comparison.comparison, None);
        assert!(report.is_consistent());
    }

    #[test]
    fn accepts_defensive_comparison_unavailable_report() {
        let report = parse_mapped_scratch_thp_fault_sample_report_output(
            "mapped_scratch_thp_fault_sample_validation_gate=ready reason=ready
mapped_scratch_thp_fault_sample_comparison=unavailable reason=comparison_unavailable
",
        )
        .expect("defensive report");

        assert_eq!(
            report.gate.status,
            MappedScratchThpFaultSampleValidationGateStatus::Ready
        );
        assert_eq!(
            report.comparison.reason,
            MappedScratchThpFaultSampleComparisonReason::ComparisonUnavailable
        );
        assert!(report.is_consistent());
    }

    #[test]
    fn builds_report_from_parts() {
        let parsed =
            parse_mapped_scratch_thp_fault_sample_report_output(READY_REPORT).expect("report");

        assert_eq!(
            MappedScratchThpFaultSampleReport::from_parts(parsed.gate, parsed.comparison)
                .expect("from parts"),
            parsed
        );
    }

    #[test]
    fn reports_missing_gate_errors() {
        assert_eq!(
            parse_mapped_scratch_thp_fault_sample_report_output(
                "mapped_scratch_thp_fault_sample_comparison=unavailable reason=fault_counters_unavailable\n"
            )
            .expect_err("missing gate"),
            MappedScratchThpFaultSampleReportParseError::Gate(
                MappedScratchThpFaultSampleValidationGateOutputParseError::MissingGateLine
            )
        );
    }

    #[test]
    fn reports_missing_comparison_errors() {
        assert_eq!(
            parse_mapped_scratch_thp_fault_sample_report_output(
                "mapped_scratch_thp_fault_sample_validation_gate=ready reason=ready\n"
            )
            .expect_err("missing comparison"),
            MappedScratchThpFaultSampleReportParseError::Comparison(
                MappedScratchThpFaultSampleComparisonOutputParseError::MissingComparisonLine
            )
        );
    }

    #[test]
    fn rejects_inconsistent_reports() {
        assert_eq!(
            parse_mapped_scratch_thp_fault_sample_report_output(
                "mapped_scratch_thp_fault_sample_validation_gate=ready reason=ready
mapped_scratch_thp_fault_sample_comparison=unavailable reason=fault_counters_unavailable
"
            )
            .expect_err("inconsistent report"),
            MappedScratchThpFaultSampleReportParseError::Inconsistent {
                gate_status: MappedScratchThpFaultSampleValidationGateStatus::Ready,
                gate_reason: MappedScratchThpFaultSampleValidationGateReason::Ready,
                comparison_status: MappedScratchThpFaultSampleComparisonStatus::Unavailable,
                comparison_reason:
                    MappedScratchThpFaultSampleComparisonReason::FaultCountersUnavailable,
            }
        );
    }
}
