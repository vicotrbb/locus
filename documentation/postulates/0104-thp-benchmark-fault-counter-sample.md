# Postulate 0104: THP Benchmark Fault Counter Sample

Date: 2026-07-03

## Statement

The mapped scratch THP write-touch benchmark should print process fault counter deltas for each advice mode before Criterion timing so first-touch measurements have page-fault context.

## Rationale

The current THP write-touch benchmark reports timing only. The most recent Docker run showed a large timing difference between default allocation and `hugepage` advice, but timing alone does not explain whether page fault behavior also changed.

`locus-observe` now parses `/proc/self/stat` minor and major fault counters. Sampling those counters around a small fixed set of benchmark-like iterations creates supporting evidence without changing Criterion's measured loops.

## Experiment

Add Linux-only benchmark diagnostics that:

- run a small fixed sample for the default, `hugepage`, and `no_hugepage` modes;
- sample process fault counters before and after the sample;
- print stable `fault_sample=` lines with signed minor and major fault deltas;
- keep the Criterion benchmark names unchanged.

## Expected Result

The Docker benchmark should emit one fault sample line per mode, then continue to report the existing Criterion timing output. The sample is supporting evidence only and must not be treated as proof of huge page adoption.
