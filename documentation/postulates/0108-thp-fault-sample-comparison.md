# Postulate 0108: THP Fault Sample Comparison

Date: 2026-07-03

## Statement

Mapped scratch THP benchmark fault samples should expose a typed comparison summary so benchmark reports can interpret minor-fault deltas without duplicating arithmetic.

## Rationale

The benchmark now emits stable `fault_sample=` lines, and `locus-alloc` can parse those lines into typed samples. Experiment notes still compare deltas manually, which is easy to get wrong and hard for future automation to reuse.

The comparison must stay conservative. A lower minor-fault delta in the `hugepage` advice sample is useful supporting evidence, but it is not proof of transparent huge page adoption. Page-size evidence from the THP probe remains the adoption proof.

## Experiment

Add a small comparison summary to `locus-alloc` that reports:

- default, `hugepage`, and `no_hugepage` process minor-fault deltas;
- `hugepage` minus default process minor-fault delta;
- `hugepage` minus `no_hugepage` process minor-fault delta;
- whether any process or child major-fault delta was observed.

The summary should return no value when any required sample is unavailable or incomplete.

## Expected Result

Focused tests should compute the comparison from current benchmark sample output, flag major-fault observations, reject unavailable samples by returning no comparison, and keep workspace validation clean.
