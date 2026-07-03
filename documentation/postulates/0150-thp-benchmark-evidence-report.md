# Postulate 0150: THP Benchmark Evidence Report

Date: 2026-07-03

## Claim

Locus should provide a validation report that parses one mapped scratch THP
benchmark log and summarizes page-size evidence, fault-counter evidence, and
whether the hugepage-advice timing can be treated as huge page adoption
evidence.

## Rationale

The `scratch_arena` benchmark now emits `thp_page_sample=` lines before
existing `fault_sample=` lines and Criterion timing. Those lines make one log
much more useful, but downstream readers still need to combine them correctly.

The correct rule is conservative: timing and lower minor-fault counts are not
proof of huge page adoption unless the hugepage page sample reports
`thp_observed=yes reason=kernel_page_size`.

## Experiment

Add a focused `locus-validate` report module and example command that:

- parses `thp_page_sample=` lines through `locus-alloc`;
- evaluates the existing fault-sample gate and comparison;
- emits one stable `mapped_scratch_thp_benchmark_evidence=` line;
- reports `ready` only when page samples and fault samples are both available;
- reports whether hugepage adoption was observed from page-size evidence.

## Falsification

The postulate is weakened if the report can be confused by advice-only timing,
accepts missing page samples as ready, or duplicates the detailed benchmark log
instead of emitting a compact summary.

## Expected Value

If the postulate survives, future THP benchmark logs can be classified by one
command before anyone copies timing numbers into design guidance.
