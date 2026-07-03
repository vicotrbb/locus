//! Validation helpers that combine Locus probe verdicts.

mod mapped_scratch_thp_gate;
mod pinned_scratch_gate;
mod pinned_scratch_near_gpu_gate;
mod thp_fault_sample_comparison;
mod thp_fault_sample_gate;
mod thp_fault_sample_report;

pub use mapped_scratch_thp_gate::{
    evaluate_mapped_scratch_thp_validation_output, parse_mapped_scratch_thp_validation_gate_line,
    parse_mapped_scratch_thp_validation_gate_output, MappedScratchThpValidationGate,
    MappedScratchThpValidationGateLineParseError, MappedScratchThpValidationGateOutputParseError,
    MappedScratchThpValidationGateParseError, MappedScratchThpValidationGateReason,
    MappedScratchThpValidationGateStatus, MappedScratchThpValidationGateVerdict,
};
pub use pinned_scratch_gate::{
    evaluate_pinned_scratch_validation_output, parse_pinned_scratch_validation_gate_line,
    parse_pinned_scratch_validation_gate_output, PinnedScratchValidationGate,
    PinnedScratchValidationGateLineParseError, PinnedScratchValidationGateOutputParseError,
    PinnedScratchValidationGateParseError, PinnedScratchValidationGateReason,
    PinnedScratchValidationGateStatus, PinnedScratchValidationGateVerdict,
};
pub use pinned_scratch_near_gpu_gate::{
    evaluate_pinned_scratch_near_gpu_validation_output,
    parse_pinned_scratch_near_gpu_validation_gate_line, PinnedScratchNearGpuValidationGate,
    PinnedScratchNearGpuValidationGateLineParseError, PinnedScratchNearGpuValidationGateParseError,
    PinnedScratchNearGpuValidationGateReason, PinnedScratchNearGpuValidationGateStatus,
    PinnedScratchNearGpuValidationGateVerdict,
};
pub use thp_fault_sample_comparison::{
    parse_mapped_scratch_thp_fault_sample_comparison_line,
    parse_mapped_scratch_thp_fault_sample_comparison_output,
    MappedScratchThpFaultSampleComparisonLineParseError,
    MappedScratchThpFaultSampleComparisonOutput,
    MappedScratchThpFaultSampleComparisonOutputParseError,
    MappedScratchThpFaultSampleComparisonReason, MappedScratchThpFaultSampleComparisonStatus,
};
pub use thp_fault_sample_gate::{
    evaluate_mapped_scratch_thp_fault_sample_validation_output,
    parse_mapped_scratch_thp_fault_sample_validation_gate_line,
    parse_mapped_scratch_thp_fault_sample_validation_gate_output,
    MappedScratchThpFaultSampleValidationGate,
    MappedScratchThpFaultSampleValidationGateLineParseError,
    MappedScratchThpFaultSampleValidationGateOutputParseError,
    MappedScratchThpFaultSampleValidationGateParseError,
    MappedScratchThpFaultSampleValidationGateReason,
    MappedScratchThpFaultSampleValidationGateStatus,
    MappedScratchThpFaultSampleValidationGateVerdict,
};
pub use thp_fault_sample_report::{
    parse_mapped_scratch_thp_fault_sample_report_output, MappedScratchThpFaultSampleReport,
    MappedScratchThpFaultSampleReportParseError,
};

#[cfg(target_os = "linux")]
pub mod linux;
