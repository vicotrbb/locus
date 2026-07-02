# Postulate 0069: Validation Gate Display

Date: 2026-07-02

## Statement

The combined placement validation gate should expose a stable display representation for its final machine-readable line.

## Rationale

The file-based and live validation examples print the same final gate schema:

```text
placement_validation_gate=<status> reason=<reason>
```

Centralizing that rendering in `Display` keeps examples, tests, and parser expectations aligned. It also gives downstream automation a single formatting contract when it already holds a parsed or evaluated gate value.

## Experiment

Implement `Display` for `PlacementValidationGate` and `PlacementValidationGateVerdict`. Update the validation examples to print the gate value directly, then cover the rendered line in focused tests.

## Expected Result

The examples should keep printing the same final gate line. Host formatting, workspace tests, clippy, and Docker `locus-validate` tests should pass.
