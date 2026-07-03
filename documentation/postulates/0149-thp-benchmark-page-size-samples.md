# Postulate 0149: THP Benchmark Page-Size Samples

Date: 2026-07-03

## Claim

The mapped scratch THP benchmark should emit a small page-size evidence sample
for each advice mode before Criterion timing so benchmark logs can join timing,
fault counters, and kernel page-size evidence from the same run.

## Rationale

The current benchmark emits fault counter samples and Criterion timing. The
latest `smaps` fallback work showed that page-size evidence is available in
Docker even when `numa_maps` is unavailable, and that accepted hugepage advice
can still leave the mapping at 4 KiB pages.

Timing and fault counters are not enough to prove huge page adoption. A stable
page-size sample line lets future validation tools reject advice-only timing
claims while preserving useful benchmark output.

## Experiment

Add a typed `thp_page_sample=` parser in `locus-alloc` and make the
`scratch_arena` benchmark print one page-size sample for each mode:

- `default`;
- `hugepage`;
- `no_hugepage`.

Each sample should create a real mapped scratch arena, apply the mode's advice,
write-touch the pages, inspect `numa_maps` first and `smaps` second, and report
whether the observed kernel page size exceeds the base page size.

## Falsification

The postulate is weakened if the benchmark output becomes ambiguous, if the
sample cannot be parsed, or if the added sampling path changes Criterion case
names or measured loops.

## Expected Value

If the postulate survives, a single benchmark log will contain the key evidence
needed to avoid treating THP advice timing as huge page adoption proof.
