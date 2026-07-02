# Postulate 0061: Placement Readiness Output Parser

Date: 2026-07-02

## Statement

Validation automation should be able to extract the final placement validation readiness verdict from full locality environment probe output.

## Rationale

The locality environment probe prints availability lines for `numa_maps`, cgroup NUMA stats, node `numastat`, and a final `placement_validation_readiness` line. Scripts should consume the final typed readiness verdict without duplicating multiline scanning and token parsing.

An observe-layer output parser keeps the readiness-line selection rule explicit and rejects duplicate or malformed final readiness lines.

## Experiment

Add an output parser that scans multiline locality environment output for `placement_validation_readiness=<status> reason=<reason>`, returns `NumaPlacementValidationReadiness`, and rejects missing, duplicate, or malformed readiness lines.

## Expected Result

The parser should pass focused tests. Workspace tests, clippy, and the Docker locality environment probe should pass with the existing not-ready output.
