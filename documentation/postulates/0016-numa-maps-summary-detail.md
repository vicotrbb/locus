# Postulate 0016: Numa Maps Summary Detail

Date: 2026-07-02

## Statement

`numa_maps` summaries should include policy and kernel page-size aggregation, not only node page totals, so future placement experiments can distinguish policy application from page materialization and page-size behavior.

## Rationale

A placement experiment can fail in several different ways: the policy may not be applied, pages may not be materialized, or pages may use unexpected page sizes. Recording only pages by node loses that context. The summary layer should expose enough structure for experiment logs to compare policy, node, and page-size evidence consistently.

## Experiment

Extend `NumaMapsSummary` with:

- mapping counts by policy token;
- page totals by `kernelpagesize_kB`;
- fixture coverage for mixed policies and page sizes.

## Expected Result

The summary should remain fixture-testable and provide richer evidence for later mapped arena placement experiments.
