# Development Note: Initial Rust Foundation

Date: 2026-07-02

## Scope

This change creates the first Rust workspace for Locus and implements a small, testable foundation:

- Linux CPU-list parsing.
- NUMA and PCI topology data models.
- Linux sysfs discovery for NUMA nodes and PCI locality.
- Initial placement policy modeling.
- A safe node-tagged scratch arena.
- A Criterion benchmark harness for the scratch arena reset cycle and a default `Vec<u8>` allocation baseline.

## Validation Plan

Run:

```sh
cargo test --workspace
cargo bench -p locus-alloc --bench scratch_arena
```

The first command validates parsing, policy selection, sysfs fixture discovery, and arena behavior. The second command records allocator reset-cycle timing for a simple scratch workload.

## Notes

The scratch arena is intentionally not NUMA-bound yet. It records the intended home node and validates arena lifetime behavior without introducing raw allocation or Linux memory-policy calls.
