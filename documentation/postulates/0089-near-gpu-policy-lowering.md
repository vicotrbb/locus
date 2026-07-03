# Postulate 0089: Near-GPU Policy Lowering

Date: 2026-07-02

## Statement

`PlacementPolicy::NearGpu` should lower to a concrete NUMA policy when discovered PCI locality identifies the GPU's NUMA node.

## Rationale

The core policy layer can already choose `NearGpu` for pinned host memory, and topology discovery records PCI `numa_node` values from sysfs. The pinned scratch pool still receives an explicit `NodeId`, so callers need a deterministic bridge from a GPU BDF hint to the NUMA node used for host page-locked staging pools.

This remains a host locality policy step. It does not register memory with CUDA, select a CUDA device, prove GPU DMA behavior, or validate page placement.

## Experiment

Add a helper in `locus-core` that resolves a `LocalityDecision` against discovered `Topology`:

- preserve non-`NearGpu` policies unchanged;
- lower `NearGpu(<bdf>)` to `Bind(<gpu numa node>)` when the PCI device is discovered with a NUMA node;
- fall back to `Local` when the GPU is missing from topology;
- fall back to `Local` when the GPU has no reported NUMA node;
- provide explicit reason strings for each result.

## Expected Result

The helper should pass focused policy tests and keep workspace validation green.
