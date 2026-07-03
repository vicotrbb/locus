# Experiment 0096: Live Pinned Scratch Validation Gate

Date: 2026-07-02

## Postulate

[Postulate 0088](../postulates/0088-live-pinned-scratch-validation-gate.md) claims that the host page-locked scratch validation gate should have a live one-command example.

## Change

Added `live_pinned_scratch_validation_gate` to `locus-validate`.

The example:

- creates a budgeted `PinnedScratchPool`;
- prints the stable pinned scratch pool probe lines;
- captures those same lines in memory;
- evaluates them through `evaluate_pinned_scratch_validation_output`;
- prints the final `pinned_scratch_validation_gate=<status> reason=<reason>` line.

Updated the README probe list with the live validation command.

This remains a host page-lock validation path only. It does not prove CUDA host registration, GPU-near placement, DMA readiness, or async transfer safety.

## Commands

```text
cargo fmt --all
cargo run -p locus-validate --example live_pinned_scratch_validation_gate
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-validate --example live_pinned_scratch_validation_gate
```

## Results

`cargo fmt --all` passed.

Host `cargo run -p locus-validate --example live_pinned_scratch_validation_gate` output:

```text
arena_capacity=16384
max_locked_bytes=40958
pool_stats phase=initial locked_bytes=0 checked_out=0 idle=0 created_arenas=0 reused_arenas=0 checkout_count=0 release_count=0
pool_checkout=ok handle=0
checked_out_mapping_start=0x100690000
checked_out_mapping_len=20479
checked_out_allocation=ok bytes=256
pool_stats phase=after_checkout locked_bytes=20479 checked_out=1 idle=0 created_arenas=1 reused_arenas=0 checkout_count=1 release_count=0
pool_release=ok handle=0
pool_stats phase=after_release locked_bytes=20479 checked_out=0 idle=1 created_arenas=1 reused_arenas=0 checkout_count=1 release_count=1
pool_reuse_checkout=ok handle=1
pool_stats phase=after_reuse_checkout locked_bytes=20479 checked_out=1 idle=0 created_arenas=1 reused_arenas=1 checkout_count=2 release_count=1
pool_reuse_release=ok handle=1
pool_stats phase=after_reuse_release locked_bytes=20479 checked_out=0 idle=1 created_arenas=1 reused_arenas=1 checkout_count=2 release_count=2
pinned_scratch_validation_gate=ready reason=ready
```

`cargo test --workspace` passed:

```text
locus-alloc: 38 passed
locus-core: 9 passed
locus-observe: 27 passed
locus-sys: 6 passed
locus-topology: 2 passed
locus-validate: 9 passed
doc tests: passed
```

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Docker `cargo run -p locus-validate --example live_pinned_scratch_validation_gate` output:

```text
arena_capacity=16384
max_locked_bytes=40958
pool_stats phase=initial locked_bytes=0 checked_out=0 idle=0 created_arenas=0 reused_arenas=0 checkout_count=0 release_count=0
pool_checkout=ok handle=0
checked_out_mapping_start=0xffffbf21f000
checked_out_mapping_len=20479
checked_out_allocation=ok bytes=256
pool_stats phase=after_checkout locked_bytes=20479 checked_out=1 idle=0 created_arenas=1 reused_arenas=0 checkout_count=1 release_count=0
pool_release=ok handle=0
pool_stats phase=after_release locked_bytes=20479 checked_out=0 idle=1 created_arenas=1 reused_arenas=0 checkout_count=1 release_count=1
pool_reuse_checkout=ok handle=1
pool_stats phase=after_reuse_checkout locked_bytes=20479 checked_out=1 idle=0 created_arenas=1 reused_arenas=1 checkout_count=2 release_count=1
pool_reuse_release=ok handle=1
pool_stats phase=after_reuse_release locked_bytes=20479 checked_out=0 idle=1 created_arenas=1 reused_arenas=1 checkout_count=2 release_count=2
pinned_scratch_validation_gate=ready reason=ready
```

## Conclusion

The postulate survived. Locus now has a one-command live validation path for host page-locked scratch pool reuse.

The live gate confirms that checkout, allocation, release, reuse, idle accounting, reuse accounting, and retained locked bytes are all observed in the current host and Docker environments. It still does not prove GPU staging readiness or NUMA placement.
