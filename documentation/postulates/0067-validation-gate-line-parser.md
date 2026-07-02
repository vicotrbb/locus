# Postulate 0067: Validation Gate Line Parser

Date: 2026-07-02

## Statement

The final combined placement validation gate line should be parsed by a shared `locus-validate` helper.

## Rationale

The file-based and live validation gate examples print:

```text
placement_validation_gate=<status> reason=<reason>
```

Automation should be able to consume that final verdict with the same typed `PlacementValidationGateStatus` and `PlacementValidationGateReason` model used by the gate implementation. This completes the parser coverage for every final verdict line used in the NUMA placement validation path.

## Experiment

Add line and output parsers for `placement_validation_gate=<status> reason=<reason>` in `locus-validate`. Cover valid verified, not-ready, unverified, and unavailable statuses, plus missing, duplicate, malformed, and unknown-token failures.

## Expected Result

The parsers should pass focused Linux tests. Host workspace validation and Docker `locus-validate` tests should pass.
