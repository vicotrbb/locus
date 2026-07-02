# Experiment 0012: Logical KV Block Table

Date: 2026-07-02

## Postulate

See `documentation/postulates/0009-logical-kv-block-table.md`.

## Commands

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench scratch_arena
```

## Results

Executed on 2026-07-02.

`cargo test --workspace` passed:

- `locus-alloc`: 15 unit tests passed.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 7 unit tests passed.
- `locus-topology`: 2 unit tests passed.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Benchmark command used for the recorded sample:

```sh
cargo bench -p locus-alloc --bench scratch_arena -- --sample-size 10 --warm-up-time 1 --measurement-time 1
```

Criterion reported:

- `scratch_arena_reset_cycle_64x256b`: 200.43 ns to 201.74 ns.
- `vec_allocation_cycle_64x256b`: 619.86 ns to 623.22 ns.
- `request_scratch_cycle_16x64x256b`: 11.040 us to 11.095 us.
- `request_vec_allocation_cycle_16x64x256b`: 12.102 us to 12.152 us.
- `request_scratch_pool_cycle_16x64x256b`: 3.3153 us to 3.3350 us.
- `kv_block_pool_cycle_256x4k`: 1.2161 us to 1.2394 us.
- `kv_vec_allocation_cycle_256x4k`: 16.822 us to 16.932 us.
- `kv_block_table_append_release_128x16tokens`: 1.9010 us to 1.9066 us.
- `kv_vec_table_allocation_128x4k`: 8.2849 us to 8.3182 us.

## Conclusion

The postulate survived. Locus now has a safe logical KV sequence table that maps token growth to fixed-size block handles and releases sequence-owned blocks in bulk.

This remains a foundation step. It does not implement prefix sharing, eviction, physical page placement, GPU kernel compatibility, or model-specific KV layout metadata.

## Next Questions

- Should logical token ranges map to block handles through a compact page-table API?
- Should prefix-sharing support be modeled as immutable shared block spans?
