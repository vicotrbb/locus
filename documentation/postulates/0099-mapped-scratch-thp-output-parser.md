# Postulate 0099: Mapped Scratch THP Output Parser

Date: 2026-07-02

## Statement

The mapped scratch THP probe should have a focused parser for its stable advice and observation lines.

## Rationale

`mapped_scratch_thp` prints machine-readable lines that separate accepted advice from observed page-size evidence. Automation should be able to consume those lines without scraping free-form error text or assuming that `madvise` acceptance means transparent huge page adoption.

Parsing the stable lines makes future validation gates and benchmarks able to classify:

- which advice mode was requested;
- whether the kernel accepted the THP advice;
- whether the probe observed a larger page size, base page size, or unknown evidence.

## Experiment

Add parser types and functions in `locus-alloc` for full multiline `mapped_scratch_thp` output.

The parser should accept both `hugepage` and `no_hugepage` modes, require `touched` and `thp_observed` only when advice succeeded, and preserve the observation reason as a stable token.

## Expected Result

The parser should accept the current Docker outputs, reject malformed or duplicate stable lines, and keep advice acceptance separate from huge page observation.
