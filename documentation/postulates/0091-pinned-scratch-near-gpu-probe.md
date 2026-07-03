# Postulate 0091: Pinned Scratch Near-GPU Probe

Date: 2026-07-02

## Statement

The near-GPU pinned scratch constructor should have a runnable probe against discovered sysfs topology.

## Rationale

`PinnedScratchPool::new_near_gpu` connects a GPU BDF hint to a host NUMA node through discovered PCI topology, but it is currently covered only by unit tests with synthetic topology. A small probe should expose how the constructor behaves in real host and Docker environments.

This remains host topology and page-lock probing only. It does not select a CUDA device, register host memory with CUDA, prove DMA behavior, or validate page placement.

## Experiment

Add a `pinned_scratch_near_gpu` example to `locus-alloc` that:

- discovers sysfs topology;
- accepts an optional GPU BDF argument;
- otherwise picks the first discovered PCI device with a reported NUMA node;
- creates a `PinnedScratchPool` with `new_near_gpu`;
- prints stable topology, constructor, checkout, allocation, release, and stats lines;
- exits successfully with `near_gpu_pool=unavailable` when no suitable GPU topology is visible.

## Expected Result

The example should compile under all-target checks and run in Docker without failing when no GPU PCI locality is exposed.
