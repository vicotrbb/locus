# Experiment 0156: Best Results Evidence Status

Date: 2026-07-03

## Postulate

[Postulate 0148](../postulates/0148-best-results-evidence-status.md)
claimed that the best benchmark results note should preserve the strongest
observed timings and explicitly mark when a result is not yet backed by the
kernel evidence needed for allocator guidance.

## Change

Updated
`documentation/dev-notes/2026-07-03-best-benchmark-results.md`
to add:

- an `Evidence status` column for benchmark rows;
- a `Best Validation Results` section;
- the mapped scratch THP `smaps` fallback result as the current strongest THP
  page-size validation evidence;
- the mixed-size remote-free controller preservation result as behavior
  evidence rather than a new timing best.

The fast THP-advised first-touch row remains in the benchmark table, but it is
now labeled as timing-only for huge page adoption until a same-run measurement
joins timing with page-size proof.

## Validation

Commands:

```bash
rg -n "THP-advised|smaps|Evidence status|Best Validation Results" documentation/dev-notes/2026-07-03-best-benchmark-results.md
git diff --check
```

Results:

- The updated note contains the expected evidence-status and validation-result
  sections.
- Dash scan over touched documentation passed with no em dash or en dash.
- `git diff --check` passed.

## Interpretation

The postulate survived.

The best-results note still preserves the fastest observed timings, but it now
prevents the highest-risk misread: treating a THP advice timing as proof that
the mapping used huge pages. The note also preserves the current negative THP
validation result, which is useful because it proves the probe can produce a
kernel page-size verdict in Docker even when `numa_maps` is unavailable.

## Next Step

The next THP benchmark pass should capture timing, fault samples, and
page-size evidence from the same Linux run before changing the best-results
status from timing-only to allocator guidance.
