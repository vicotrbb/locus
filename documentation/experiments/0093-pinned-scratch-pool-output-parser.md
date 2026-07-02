# Experiment 0093: Pinned Scratch Pool Output Parser

Date: 2026-07-02

## Postulate

[Postulate 0085](../postulates/0085-pinned-scratch-pool-output-parser.md) claims that the pinned scratch pool probe should have a typed parser for its stable event and accounting lines.

## Change

Added parser types and functions in `locus-alloc` for the `pinned_scratch_pool` probe:

- `PinnedScratchPoolProbeStatus`;
- `PinnedScratchPoolProbeEvent`;
- `PinnedScratchPoolProbePhase`;
- `PinnedScratchPoolProbeEventLine`;
- `PinnedScratchPoolProbeStatsLine`;
- `PinnedScratchPoolProbeOutput`;
- `PinnedScratchPoolProbeLineParseError`;
- `PinnedScratchPoolProbeOutputParseError`;
- `parse_pinned_scratch_pool_probe_event_line`;
- `parse_pinned_scratch_pool_probe_stats_line`;
- `parse_pinned_scratch_pool_probe_output`.

The multiline parser ignores free-form detail lines and only consumes stable event and `pool_stats` lines. Successful checkout output requires allocation, release, reuse checkout, reuse release, and the expected phase stats.

## Commands

```text
cargo fmt --all
cargo test -p locus-alloc
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-alloc
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-alloc --example pinned_scratch_pool
```

## Results

`cargo fmt --all` passed.

`cargo test -p locus-alloc` passed:

```text
locus-alloc: 38 passed
doc tests: passed
```

`cargo test --workspace` passed on the host:

```text
locus-alloc: 38 passed
locus-core: 9 passed
locus-observe: 27 passed
locus-sys: 6 passed
locus-topology: 2 passed
locus-validate: 0 passed
doc tests: passed
```

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Docker `cargo test -p locus-alloc` passed:

```text
locus-alloc: 39 passed
doc tests: passed
```

Docker `cargo run -p locus-alloc --example pinned_scratch_pool` output:

```text
arena_capacity=16384
max_locked_bytes=40958
pool_stats phase=initial locked_bytes=0 checked_out=0 idle=0 created_arenas=0 reused_arenas=0 checkout_count=0 release_count=0
pool_checkout=ok handle=0
checked_out_mapping_start=0xffffab7fb000
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

## Conclusion

The postulate survived. The pinned scratch pool probe now has a typed parser that can distinguish successful reuse output from checkout failure, missing required lines, duplicate stable lines, and malformed numeric fields.

This improves automation readiness for host page-locked staging work. It still does not prove CUDA host registration or GPU-near placement.
