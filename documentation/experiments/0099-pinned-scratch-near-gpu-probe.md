# Experiment 0099: Pinned Scratch Near-GPU Probe

Date: 2026-07-02

## Postulate

[Postulate 0091: Pinned Scratch Near-GPU Probe](../postulates/0091-pinned-scratch-near-gpu-probe.md)

## Change

Added a `pinned_scratch_near_gpu` example for `locus-alloc`.

The probe discovers sysfs topology, accepts an optional GPU BDF, otherwise selects the first discovered PCI device with a NUMA node, and creates a `PinnedScratchPool` through `PinnedScratchPool::new_near_gpu`. It prints stable topology, constructor, checkout, allocation, release, and pool accounting lines when a topology-backed GPU locality target is available. When no GPU PCI locality is visible, it exits successfully with an unavailable line.

This is still a host topology and page-lock probe. It does not register memory with CUDA, select a CUDA device, prove DMA locality, or prove final page placement.

## Commands

```text
cargo run -p locus-alloc --example pinned_scratch_near_gpu
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-alloc --example pinned_scratch_near_gpu
```

## Results

Local probe output:

```text
topology_nodes=0
topology_pci_devices=0
near_gpu_pool=unavailable reason=no_gpu_with_numa_node
```

Workspace tests passed:

```text
test result: ok. 41 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Clippy passed with `-D warnings`.

Docker probe output:

```text
topology_nodes=0
topology_pci_devices=0
near_gpu_pool=unavailable reason=no_gpu_with_numa_node
```

## Conclusion

The postulate survives for the current environment. The near-GPU constructor now has a runnable real-topology probe, and the probe handles hosts or containers without visible PCI NUMA locality without failing.

The current host and Docker container did not expose GPU PCI locality, so this experiment did not exercise a successful `new_near_gpu` checkout path. A later experiment should run this probe on a machine or container with visible PCI devices that report NUMA nodes, then pair it with placement proof from `numa_maps` or cgroup NUMA evidence.
