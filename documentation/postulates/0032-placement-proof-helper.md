# Postulate 0032: Placement Proof Helper

Date: 2026-07-02

## Statement

`NumaPlacementEvidence` should expose a conservative helper for deciding whether matched mapping evidence proves all reported pages are on the expected node.

## Rationale

The status enum already encodes the placement classification, but probe and benchmark code should not repeatedly compare enum variants or re-sum remote pages. A small helper makes the success condition explicit and avoids treating partial placement as proof.

## Experiment

Add helper methods that:

- return true only for all-pages-on-expected-node evidence;
- report the number of pages found on other nodes;
- pass fixture assertions for full and partial placement cases.

## Expected Result

The helpers should pass workspace tests and clippy. They should preserve the rule that partial placement is not proof of successful NUMA placement.
