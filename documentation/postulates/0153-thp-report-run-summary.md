# Postulate 0153: THP Report Run Summary

Date: 2026-07-03

## Claim

Repeated THP benchmark evidence should be summarized from compact report lines
inside `locus-validate` instead of being left as manual spreadsheet work.

## Rationale

The compact report line now joins page-size evidence, fault deltas, and timing
intervals from one benchmark log. Repeated runs are still hard to compare
because a reader must manually check whether all runs came from the same
page-size evidence state before interpreting the timing intervals.

A small report-line summary can improve this without changing allocator policy:
count ready reports, count observed hugepage adoption, count major-fault
observations, and print min and max timing estimates. It should also classify
whether the page-size evidence is consistent across the run set.

## Experiment

Add a parser and example that accept one or more compact
`mapped_scratch_thp_benchmark_evidence=` report lines and emit one summary line
with:

- report counts;
- ready and unavailable counts;
- hugepage adoption and base-page counts;
- major-fault observation counts;
- min and max estimate timings for default, hugepage, and no-hugepage cases;
- min and max hugepage-vs-default estimate deltas;
- a page-evidence cohort label.

## Falsification

The postulate is weakened if the summary accepts malformed report lines, hides
mixed page-size evidence, or makes timing look policy-ready when the run set is
too small or heterogeneous.

## Expected Value

If the postulate survives, repeated THP runs can be compared as a typed,
machine-readable artifact while preserving the distinction between performance
data and huge page adoption proof.
