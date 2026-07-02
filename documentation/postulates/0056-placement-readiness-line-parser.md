# Postulate 0056: Placement Readiness Line Parser

Date: 2026-07-02

## Statement

The final placement validation readiness line should be parsed by a shared observe-layer helper.

## Rationale

The locality environment probe now prints a stable final readiness line:

```text
placement_validation_readiness=<status> reason=<reason>
```

Validation automation should consume the same typed `NumaPlacementValidationReadiness` model used by the probe. A shared parser keeps status and reason tokens aligned with the readiness types and catches malformed probe output.

## Experiment

Add a parser for the final readiness line in `locus-observe`. Cover ready and not-ready lines, plus missing, duplicate, unknown, and extra-token failures.

## Expected Result

The parser should return `NumaPlacementValidationReadiness` for valid readiness lines and typed parse errors for invalid lines. Workspace tests and clippy should pass.
