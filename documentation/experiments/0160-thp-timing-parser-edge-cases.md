# Experiment 0160: THP Timing Parser Edge Cases

Date: 2026-07-03

## Postulate

[Postulate 0152](../postulates/0152-thp-timing-parser-edge-cases.md)
claimed that the THP benchmark evidence report should reject malformed or
ambiguous Criterion timing evidence instead of producing misleading timing
fields.

## Change

Added focused tests for the THP benchmark timing parser:

- picosecond normalization for `ps`, `ns`, ASCII `us`, `ms`, and `s`;
- missing required benchmark timing block rejection;
- duplicate benchmark timing block rejection;
- unknown timing unit rejection.

## Validation

Command:

```bash
cargo test -p locus-validate thp_benchmark_evidence_report
cargo test --workspace
```

Result:

- passed, 9 tests.
- passed, 185 tests plus doc tests across the workspace.

## Interpretation

The postulate survived. The timing parser now has focused coverage for supported
Criterion units and for failure cases that would otherwise make malformed
benchmark evidence look ready.
