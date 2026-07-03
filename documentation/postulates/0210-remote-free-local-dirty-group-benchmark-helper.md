# Postulate 0210: Remote-Free Local Dirty Group Benchmark Helper

Date: 2026-07-03

## Claim

The service-window benchmark can factor the repeated local dirty-buffer group
collection assertions into a dedicated helper module without changing measured
allocation counters, dirty flush counters, or missing-owner guard behavior.

## Rationale

Experiments 0214 through 0217 added manual, integrated, bounded, and
registry-validated local dirty-buffer group paths. Their successful collection
steps now repeat the same capacity reuse checks, duplicate-mark assertions,
flush counter assertions, and tracker-empty assertions in the main
service-window harness. That makes the large harness harder to inspect and
makes future variants easier to wire incorrectly.

A small helper module can centralize the shared invariants while leaving the
main harness responsible for service-window routing and aggregate checks. The
helper should keep the same real allocation sequence and avoid unsafe
missing-owner allocation for unbounded local group APIs.

## Experiment

Extract the local dirty-buffer group collection paths into a dedicated
benchmark helper module that:

- exposes an explicit mode enum for manual, integrated, bounded, and validated
  local dirty-buffer group paths;
- preserves per-window duplicate mark, flush, capacity reuse, and tracker
  assertions;
- preserves the bounded and validated missing-owner rejection checks that prove
  no sparse local buffer is allocated;
- keeps the existing direct group missing-owner path away from unbounded
  `usize::MAX` vector indexing;
- compiles and runs the same focused service-window benchmark target.

## Falsification

The postulate fails if any service-window counters change, if dirty group flush
totals change, if missing-owner rejection semantics weaken, if benchmark
compilation fails, or if the refactor makes the main harness less organized.

## Expected Value

If the postulate survives, future local dirty-buffer group experiments can add
one small mode-specific marking path while reusing the same correctness and
allocation-invariant assertions.
