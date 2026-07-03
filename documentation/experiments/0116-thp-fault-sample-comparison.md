# Experiment 0116: THP Fault Sample Comparison

Date: 2026-07-03

## Postulate

[Postulate 0108: THP Fault Sample Comparison](../postulates/0108-thp-fault-sample-comparison.md)

## Change

Added a typed comparison summary for mapped scratch THP benchmark fault samples in `locus-alloc`.

The change includes:

- `MappedScratchThpFaultSampleComparison`;
- `MappedScratchThpFaultSamples::comparison`;
- focused tests for comparison arithmetic, unavailable samples, and major-fault observation.

The comparison reports default, `hugepage`, and `no_hugepage` process minor-fault deltas, plus `hugepage` minus each control delta. It also flags nonzero process or child major-fault deltas.

The API returns no comparison when any sample is unavailable or missing required fields. This keeps incomplete evidence explicit instead of treating missing counters as zero.

## Commands

```text
cargo fmt --all
cargo test -p locus-alloc
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-alloc
```

## Results

Focused `locus-alloc` tests passed:

- host: 58 unit tests passed, plus doc tests.
- Docker: 60 unit tests passed, plus doc tests. The Docker count includes Linux-only mapped scratch tests.

Workspace validation passed:

- `cargo test --workspace`: 141 unit tests passed across workspace crates, plus doc tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed after renaming private helper fields to satisfy `clippy::struct_field_names`.

The current sample fixture compares:

- default process minor-fault delta: 16400;
- `hugepage` process minor-fault delta: 8224;
- `no_hugepage` process minor-fault delta: 16400;
- `hugepage` minus default: -8176;
- `hugepage` minus `no_hugepage`: -8176;
- major faults observed: false.

## Conclusion

The postulate survived. Benchmark fault sample interpretation now has one typed calculation path in `locus-alloc`, which should reduce duplicated arithmetic in future validation and report aggregation.

This remains supporting evidence only. Lower minor-fault deltas can help explain a benchmark run, but THP adoption still requires page-size evidence from the mapped scratch THP probe.

## Next Questions

- Should `locus-validate` print this comparison summary beside the fault sample gate verdict?
- Should future benchmark output include enough run metadata to join timing estimates, fault comparisons, and THP page-size evidence into one report?
