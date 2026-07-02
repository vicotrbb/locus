# Postulate 0075: Placement Verdict Display

Date: 2026-07-02

## Statement

The placement readiness and placement proof verdicts should expose stable display representations for their final machine-readable lines.

## Rationale

The locality environment, mapped scratch, and live validation examples print these schemas:

```text
placement_validation_readiness=<status> reason=<reason>
placement_proof=<status> reason=<reason>
```

Centralizing these renderings in `Display` keeps probe output aligned with parser expectations and matches the existing combined gate display contract.

## Experiment

Implement `Display` for `NumaPlacementValidationReadiness` and `NumaPlacementProof`. Update probe print sites to print verdict values directly, then cover the rendered lines in focused tests.

## Expected Result

The probes should keep printing the same placement readiness and placement proof lines. Host validation and Docker `locus-observe` tests should pass.
