# Postulate 0054: Placement Proof Line Parser

Date: 2026-07-02

## Statement

The final mapped scratch bind `placement_proof` line should be parsed by a shared observe-layer helper instead of by downstream string splitting.

## Rationale

The bind probe now prints a stable final line:

```text
placement_proof=<status> reason=<reason>
```

Automation that validates locality behavior should consume the same typed `NumaPlacementProof` model used by the probe. A shared parser keeps accepted status and reason tokens aligned with the proof verdict types and makes malformed probe output fail loudly.

## Experiment

Add a parser for the final placement proof line in `locus-observe`. Cover verified, unverified, and unavailable proof lines, plus missing, duplicate, unknown, and extra-token failures.

## Expected Result

The parser should return `NumaPlacementProof` for valid probe lines and typed parse errors for invalid lines. Workspace tests and clippy should pass.
