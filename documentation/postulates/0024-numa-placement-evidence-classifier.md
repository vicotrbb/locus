# Postulate 0024: Numa Placement Evidence Classifier

Date: 2026-07-02

## Statement

Locus should classify matched `numa_maps` evidence against an expected NUMA node instead of leaving placement validation as ad hoc page-count inspection.

## Rationale

The current observability layer can parse page counts and find a mapping entry by address. A validation tool still has to decide whether the entry proves local placement, partial placement, remote placement, or no materialized pages. Encoding that decision in one tested helper makes future allocator probes and benchmarks less ambiguous.

## Experiment

Add a `NumaPlacementEvidence` helper that:

- accepts one parsed `numa_maps` entry and an expected node;
- totals pages across all node fields;
- records pages on the expected node;
- records pages on other nodes;
- classifies the result as all expected, partial expected, no expected, or no pages reported.

## Expected Result

Fixture tests should cover all four classifications. The helper should not claim successful placement unless every reported page is on the expected node.
