# Experiment 0011: Node-Tagged KV Block Pool

Date: 2026-07-02

## Postulate

See `documentation/postulates/0008-node-tagged-kv-block-pool.md`.

## Commands

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench scratch_arena
```

## Results

Executed on 2026-07-02.

`cargo test --workspace` passed:

- `locus-alloc`: 12 unit tests passed.
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

- `scratch_arena_reset_cycle_64x256b`: 200.16 ns to 200.93 ns.
- `vec_allocation_cycle_64x256b`: 627.07 ns to 632.77 ns.
- `request_scratch_cycle_16x64x256b`: 10.380 us to 10.431 us.
- `request_vec_allocation_cycle_16x64x256b`: 12.207 us to 12.275 us.
- `request_scratch_pool_cycle_16x64x256b`: 3.6275 us to 3.6406 us.
- `kv_block_pool_cycle_256x4k`: 1.1499 us to 1.1556 us.
- `kv_vec_allocation_cycle_256x4k`: 16.840 us to 16.917 us.

## Conclusion

The postulate survived. A fixed-size KV block pool gives Locus a safe, benchmarked block reuse primitive with stale-handle detection and clear accounting.

The result is not a full KV-cache manager. There is no logical sequence block table, prefix sharing, page placement validation, or GPU integration yet.

## Next Questions

- Should the next KV step add a logical sequence-to-block table?
- Should KV block size be parameterized by model layer, head count, and head dimension metadata?
