# Postulate 0148: Best Results Evidence Status

Date: 2026-07-03

## Claim

The best benchmark results note should preserve the strongest observed timings
and explicitly mark when a result is not yet backed by the kernel evidence
needed for allocator guidance.

## Rationale

The best-results note is meant to be reused for allocator design, README
material, release notes, and deeper benchmark planning. It currently records
the largest THP timing delta, while the newer mapped scratch THP `smaps`
fallback experiment showed that Docker can provide kernel page-size evidence
and that the tested mapping still used 4 KiB pages.

The fast THP-advice timing remains useful, but it should be labeled as
unverified for huge page adoption until repeated runs can join timing, fault
samples, and page-size evidence from the same environment.

## Experiment

Update the best-results note to:

- keep the fastest observed allocator timing rows;
- add an evidence status column;
- mark the THP-advised timing as requiring same-run page-size proof;
- add the recent `smaps` fallback as the best observability result for THP
  readiness;
- keep remote-free controller results as behavior-preserving evidence when they
  do not beat the current best timing.

## Falsification

The postulate is weakened if the note becomes harder to scan, duplicates full
experiment logs, or hides the fastest observed timings behind too much
qualification.

## Expected Value

If the postulate survives, future readers can reuse the best measurements
without accidentally treating an advice-only THP timing as proof of huge page
adoption.
