# Postulate 0090: Pinned Scratch Near-GPU Constructor

Date: 2026-07-02

## Statement

`PinnedScratchPool` should offer a constructor that selects its host NUMA node from a GPU BDF and discovered topology.

## Rationale

The core policy layer can lower `PlacementPolicy::NearGpu` to a concrete `Bind` policy when sysfs PCI locality reports a GPU NUMA node. The pinned scratch pool still requires callers to pass a `NodeId` directly. A small constructor can bridge those pieces so pinned host staging pools can be configured from the same GPU hint used by scheduler policy.

This remains host locality selection only. It does not select a CUDA device, register host memory with CUDA, prove DMA behavior, or validate page placement.

## Experiment

Add `PinnedScratchPool::new_near_gpu` that:

- accepts a GPU BDF and discovered `Topology`;
- builds a pinned-host placement request;
- lowers the near-GPU policy through `resolve_topology_policy`;
- creates the pool on the resolved NUMA node;
- returns an explicit error if the GPU is missing or has no reported NUMA node.

Add focused tests for successful node selection, missing GPU topology, and unknown GPU NUMA node.

## Expected Result

The constructor should compile without system calls, pass focused tests, and keep host and Docker validation green.
