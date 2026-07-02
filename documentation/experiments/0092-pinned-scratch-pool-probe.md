# Experiment 0092: Pinned Scratch Pool Probe

Date: 2026-07-02

## Postulate

[Postulate 0084](../postulates/0084-pinned-scratch-pool-probe.md) claims that the pinned scratch pool should have a small command-line probe that exercises checkout, allocation, release, and reuse with stable output.

## Change

Added the `pinned_scratch_pool` example to `locus-alloc` and documented it in the README probe list.

The example prints:

- pool capacity and locked-byte budget;
- pool stats before checkout;
- checkout status and handle id;
- checked-out mapping identity;
- allocation status;
- release status;
- reuse checkout status;
- pool stats after each phase.

## Commands

```text
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-alloc --example pinned_scratch_pool
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-alloc
```

## Results

`cargo fmt --all` passed.

`cargo test --workspace` passed on the host:

```text
locus-alloc: 32 passed
locus-core: 9 passed
locus-observe: 27 passed
locus-sys: 6 passed
locus-topology: 2 passed
locus-validate: 0 passed
doc tests: passed
```

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Docker `cargo run -p locus-alloc --example pinned_scratch_pool` output:

```text
arena_capacity=16384
max_locked_bytes=40958
pool_stats phase=initial locked_bytes=0 checked_out=0 idle=0 created_arenas=0 reused_arenas=0 checkout_count=0 release_count=0
pool_checkout=ok handle=0
checked_out_mapping_start=0xffffbc46e000
checked_out_mapping_len=20479
checked_out_allocation=ok bytes=256
pool_stats phase=after_checkout locked_bytes=20479 checked_out=1 idle=0 created_arenas=1 reused_arenas=0 checkout_count=1 release_count=0
pool_release=ok handle=0
pool_stats phase=after_release locked_bytes=20479 checked_out=0 idle=1 created_arenas=1 reused_arenas=0 checkout_count=1 release_count=1
pool_reuse_checkout=ok handle=1
pool_stats phase=after_reuse_checkout locked_bytes=20479 checked_out=1 idle=0 created_arenas=1 reused_arenas=1 checkout_count=2 release_count=1
pool_reuse_release=ok handle=1
pool_stats phase=after_reuse_release locked_bytes=20479 checked_out=0 idle=1 created_arenas=1 reused_arenas=1 checkout_count=2 release_count=2
```

Docker `cargo test -p locus-alloc` passed:

```text
locus-alloc: 33 passed
doc tests: passed
```

## Conclusion

The postulate survived. The pinned scratch pool now has an executable probe that shows successful page-locked checkout, allocation, release, and reuse in Docker.

The output confirms that reuse keeps `created_arenas=1` and increases `reused_arenas=1`. This is still host page-locked memory only, not CUDA registered or GPU-near placement evidence.
