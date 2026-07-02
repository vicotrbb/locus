# Locus

Locus is an experimental Rust memory locality runtime for AI inference workloads. The initial foundation focuses on explicit domain allocators, Linux topology discovery, placement policy modeling, and measured allocator experiments.

The project deliberately starts without a process-wide allocator replacement. Early work is organized around safe Rust APIs that make memory class, placement, and lifetime explicit.

## Current Foundation

- `locus-core`: topology data types, Linux CPU-list parsing, and placement policy models.
- `locus-topology`: Linux sysfs discovery for NUMA nodes and PCI device locality.
- `locus-observe`: parsers for Linux NUMA locality evidence.
- `locus-sys`: narrow unsafe boundary for owned mappings, page touching, and Linux NUMA policy probes.
- `locus-alloc`: safe node-tagged scratch arenas, request scratch pools, and KV block foundations.

## Validation

Run the foundation tests:

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Run the scratch arena benchmark harness, including a default `Vec<u8>` allocation baseline:

```sh
cargo bench -p locus-alloc --bench scratch_arena
```

Run the current Linux-oriented sys probes through Docker:

```sh
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-sys
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-sys --example mbind_region
```

The `mbind_region` example reports whether the current Linux environment permits `mbind`, then write-touches the mapped pages. Some containers return `EPERM`; that is recorded as environment evidence, not treated as placement success.

## Research Loop

Every meaningful allocator experiment should have:

- a postulate recorded before implementation;
- an ADR or development note for design decisions;
- focused tests and benchmarks;
- an experiment log with commands, results, and follow-up questions.
