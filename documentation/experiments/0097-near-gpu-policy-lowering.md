# Experiment 0097: Near-GPU Policy Lowering

Date: 2026-07-02

## Postulate

[Postulate 0089](../postulates/0089-near-gpu-policy-lowering.md) claims that `PlacementPolicy::NearGpu` should lower to a concrete NUMA policy when discovered PCI locality identifies the GPU's NUMA node.

## Change

Added `resolve_topology_policy` to `locus-core`.

The helper:

- preserves non-`NearGpu` policies unchanged;
- lowers `NearGpu(<bdf>)` to `Bind(<gpu numa node>)` when topology contains the PCI device with a reported NUMA node;
- falls back to `Local` when the PCI device is not discovered;
- falls back to `Local` when the PCI device has no reported NUMA node;
- returns explicit reason strings for each result.

This is a deterministic policy-lowering step only. It does not call Linux memory policy APIs, register memory with CUDA, select a CUDA device, prove GPU DMA behavior, or validate page placement.

## Commands

```text
cargo fmt --all
cargo test -p locus-core
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Results

`cargo fmt --all` passed.

`cargo test -p locus-core` passed:

```text
locus-core: 13 passed
doc tests: passed
```

`cargo test --workspace` passed:

```text
locus-alloc: 38 passed
locus-core: 13 passed
locus-observe: 27 passed
locus-sys: 6 passed
locus-topology: 2 passed
locus-validate: 9 passed
doc tests: passed
```

`cargo clippy --workspace --all-targets -- -D warnings` passed.

## Conclusion

The postulate survived. The core policy layer can now turn a logical near-GPU pinned-host placement request into a concrete host NUMA bind policy when sysfs PCI topology provides the GPU's NUMA node.

This gives future pinned staging pools a deterministic way to choose their host node from a GPU BDF hint. It still does not prove NUMA placement or GPU transfer readiness.
