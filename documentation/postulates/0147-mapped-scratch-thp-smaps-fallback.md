# Postulate 0147: Mapped Scratch THP Smaps Fallback

Date: 2026-07-03

## Claim

The mapped scratch THP probe should fall back to `/proc/self/smaps`
`KernelPageSize` evidence when `/proc/self/numa_maps` is unavailable or does
not contain the target mapping.

## Rationale

The current probe validates transparent huge page behavior from
`numa_maps kernelpagesize_kB`. That is good primary evidence when available,
but containerized Linux runs can expose `/proc/self/smaps` while hiding or
omitting useful `numa_maps` rows.

`smaps` reports an explicit virtual address range and `KernelPageSize` value
for each mapping. That gives the probe another kernel-reported page-size source
without trusting the success of `madvise` as proof that THP materialized.

## Experiment

Add a focused `smaps` parser to `locus-observe` that extracts mapping ranges
and optional `KernelPageSize` values. Update the mapped scratch THP examples to:

- try `numa_maps` evidence first;
- fall back to `smaps` when `numa_maps` is unavailable or misses the mapping;
- emit exactly one final `thp_observed` verdict;
- keep `unknown` when neither file can prove the page size.

Validate the probe on the host and inside Docker, where the proc filesystem is
known to be available for real kernel evidence.

## Falsification

The postulate is weakened if `smaps` is unavailable in the practical validation
environment, if its parser cannot reliably match the target mapping range, or
if the fallback produces ambiguous probe output.

## Expected Value

If the postulate survives, THP readiness checks will fail with a kernel page
size reason instead of an observability-unavailable reason in more Linux
environments.
