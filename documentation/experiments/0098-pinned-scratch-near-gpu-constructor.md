# Experiment 0098: Pinned Scratch Near-GPU Constructor

Date: 2026-07-02

## Postulate

[Postulate 0090](../postulates/0090-pinned-scratch-near-gpu-constructor.md) claims that `PinnedScratchPool` should offer a constructor that selects its host NUMA node from a GPU BDF and discovered topology.

## Change

Added `PinnedScratchPool::new_near_gpu`.

The constructor:

- accepts a GPU BDF and discovered `Topology`;
- builds a pinned-host placement request;
- lowers the near-GPU policy through `resolve_topology_policy`;
- creates the pool on the resolved NUMA node;
- returns `PinnedScratchPoolError::GpuLocalityUnavailable` when topology cannot resolve the GPU to one concrete NUMA node.

This constructor does not lock pages immediately. It only selects the pool home node. Page locking still happens lazily on checkout.

This remains host locality selection only. It does not select a CUDA device, register host memory with CUDA, prove DMA behavior, or validate page placement.

## Commands

```text
cargo fmt --all
cargo test -p locus-alloc
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-alloc
```

## Results

`cargo fmt --all` passed.

`cargo test -p locus-alloc` passed:

```text
locus-alloc: 41 passed
doc tests: passed
```

`cargo test --workspace` passed:

```text
locus-alloc: 41 passed
locus-core: 13 passed
locus-observe: 27 passed
locus-sys: 6 passed
locus-topology: 2 passed
locus-validate: 9 passed
doc tests: passed
```

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Docker `cargo test -p locus-alloc` passed:

```text
locus-alloc: 42 passed
doc tests: passed
```

Docker runs one additional Linux-only allocator test.

## Conclusion

The postulate survived. `PinnedScratchPool` can now be constructed from GPU PCI locality when topology reports a concrete NUMA node for the GPU.

This connects pinned host pool configuration to the core near-GPU policy model. It still does not prove NUMA placement, CUDA registration, or GPU transfer readiness.
