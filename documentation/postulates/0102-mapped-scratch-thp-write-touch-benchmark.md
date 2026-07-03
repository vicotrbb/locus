# Postulate 0102: Mapped Scratch THP Write-Touch Benchmark

Date: 2026-07-03

## Statement

Mapped scratch transparent huge page advice should have benchmark coverage for first-touch cost.

## Rationale

The THP probes can show whether the kernel accepts advice and whether larger page evidence is visible, but they do not measure the cost of using the advice path. First-touch latency matters for inference runtimes because arena materialization can appear on request setup, batch growth, or staging-buffer preparation paths.

Adding benchmark cases for default mapping behavior, `hugepage` advice, and `no_hugepage` advice makes the THP path measurable against the existing mapped scratch and `Vec` write-touch baselines.

## Experiment

Extend the `scratch_arena` benchmark harness with Linux-only 4 MiB mapped scratch write-touch cases:

- default mapped scratch write-touch;
- mapped scratch with `hugepage` advice;
- mapped scratch with `no_hugepage` advice.

## Expected Result

The benchmark should compile on all targets, run on Linux, and skip THP-specific cases on non-Linux targets. The first recorded run should be treated as baseline evidence rather than a performance claim.
