//! Linux validation helpers.

mod placement_validation_gate;

pub use placement_validation_gate::{
    evaluate_placement_validation_outputs, parse_placement_validation_gate_line,
    parse_placement_validation_gate_output, PlacementValidationGate,
    PlacementValidationGateLineParseError, PlacementValidationGateOutputParseError,
    PlacementValidationGateParseError, PlacementValidationGateReason,
    PlacementValidationGateStatus, PlacementValidationGateVerdict, PlacementValidationOutputs,
};
