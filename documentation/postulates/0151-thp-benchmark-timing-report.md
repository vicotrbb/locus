# Postulate 0151: THP Benchmark Timing Report

Date: 2026-07-03

## Claim

The mapped scratch THP benchmark evidence report should parse the three
Criterion timing intervals for default, hugepage advice, and no-hugepage advice
from the same benchmark log that provides page-size and fault-counter evidence.

## Rationale

The current evidence report correctly prevents advice-only THP claims, but it
still leaves timing intervals in the raw Criterion output. That means a future
reader has to manually join the report line with timing text.

Parsing only the three mapped scratch THP benchmark cases keeps the scope
narrow while making the compact report sufficient to answer the main question:
was hugepage advice fast, and did the same log prove huge page adoption?

## Experiment

Add focused Criterion timing parsing to the THP benchmark evidence report:

- parse `mapped_scratch_write_touch_4mib_default`;
- parse `mapped_scratch_write_touch_4mib_hugepage_advice`;
- parse `mapped_scratch_write_touch_4mib_no_hugepage_advice`;
- normalize lower, estimate, and upper timing bounds to integer picoseconds;
- emit the normalized fields in the report line.

## Falsification

The postulate is weakened if the parser accepts unrelated Criterion output,
misreads units, or makes the report too brittle for the benchmark output that
Criterion currently emits.

## Expected Value

If the postulate survives, a saved THP benchmark log can be reduced to one
machine-readable line containing page-size evidence, fault-counter comparison,
and timing intervals.
