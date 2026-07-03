# Experiment 0100: Pinned Scratch Near-GPU Output Parser

Date: 2026-07-02

## Postulate

[Postulate 0092: Pinned Scratch Near-GPU Output Parser](../postulates/0092-pinned-scratch-near-gpu-output-parser.md)

## Change

Added parser types and functions for `pinned_scratch_near_gpu` output in `locus-alloc`.

The parser classifies `near_gpu_pool=ok`, `near_gpu_pool=unavailable`, and `near_gpu_pool=error`. For successful constructor output, it parses the reduced checkout path used by the near-GPU probe and reuses the existing pinned scratch pool event and stats parsers.

The first implementation compiled and passed focused tests, but clippy rejected the multiline output parser as too large. The parser was then split into a small public entry point, a private output-field accumulator, a per-line parser, and a finish step.

## Commands

```text
cargo test -p locus-alloc
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p locus-alloc --example pinned_scratch_near_gpu
```

## Results

Focused allocation tests passed:

```text
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Clippy passed with `-D warnings` after the parser refactor.

Workspace tests passed:

```text
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Local probe output:

```text
topology_nodes=0
topology_pci_devices=0
near_gpu_pool=unavailable reason=no_gpu_with_numa_node
```

## Conclusion

The postulate survives. The near-GPU pinned scratch probe now has a structured parser for stable output, with unit coverage for successful construction, unavailable topology, constructor error, malformed pool lines, duplicate top-level fields, malformed numbers, and missing successful checkout events.

This increment is parser and validation infrastructure, so no allocator benchmark was run. The next measurement-oriented step should use this parser in a validation gate or benchmark harness that can compare near-GPU pool setup and checkout behavior across visible GPU-local NUMA topologies.
