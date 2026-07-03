# Postulate 0093: Pinned Scratch Near-GPU Validation Gate

Date: 2026-07-02

## Statement

Near-GPU pinned scratch probe output should evaluate to a stable validation gate.

## Rationale

The near-GPU probe and parser expose whether a host or container has usable PCI NUMA locality for a GPU-like device and whether a topology-backed pinned scratch pool can checkout, allocate, release, and retain locked pool accounting. A validation gate turns that parsed output into a single CI-friendly verdict.

The gate should distinguish:

- `ready`, when GPU-local topology selected a pool home node and the reduced checkout path succeeded;
- `unavailable`, when no suitable GPU PCI NUMA locality was visible;
- `not_ready`, when topology was available but construction, checkout, allocation, release, or accounting failed.

## Expected Result

Unit tests should cover ready, unavailable, constructor failure, checkout failure, event failure, accounting failure, malformed probe output, and stable gate-line parsing.
