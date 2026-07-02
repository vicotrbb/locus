# Experiment 0095: Pinned Scratch Validation Gate

Date: 2026-07-02

## Postulate

[Postulate 0087](../postulates/0087-pinned-scratch-validation-gate.md) claims that the pinned scratch pool probe should feed a validation-layer gate that reports whether host page-locked scratch reuse is ready.

## Change

Added a host page-locked scratch validation gate to `locus-validate`.

The gate consumes parsed `pinned_scratch_pool` output and prints:

```text
pinned_scratch_validation_gate=<status> reason=<reason>
```

The gate reports `ready reason=ready` only when checkout, allocation, release, reuse checkout, reuse release, idle arena accounting, reuse accounting, and retained locked bytes are all observed. It remains a host page-lock readiness gate only. It does not claim CUDA host registration, GPU-near placement, DMA readiness, or async transfer safety.

Added:

- `PinnedScratchValidationGateStatus`;
- `PinnedScratchValidationGateReason`;
- `PinnedScratchValidationGate`;
- `PinnedScratchValidationGateVerdict`;
- line and output parsers for the gate verdict;
- `evaluate_pinned_scratch_validation_output`;
- a file-based `pinned_scratch_validation_gate` example.

Updated the README with the captured-output validation command.

## Commands

```text
cargo fmt --all
cargo test -p locus-validate
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-validate
docker run --rm -v "$PWD":/work -w /work rust:1.96 sh -lc 'cargo run -p locus-alloc --example pinned_scratch_pool > /tmp/pinned-scratch.out && cat /tmp/pinned-scratch.out && cargo run -p locus-validate --example pinned_scratch_validation_gate /tmp/pinned-scratch.out'
docker run --rm -v "$PWD":/work -w /work rust:1.96 sh -lc '/usr/local/cargo/bin/cargo run -p locus-alloc --example pinned_scratch_pool > /tmp/pinned-scratch.out && cat /tmp/pinned-scratch.out && /usr/local/cargo/bin/cargo run -p locus-validate --example pinned_scratch_validation_gate /tmp/pinned-scratch.out'
```

## Results

`cargo fmt --all` passed.

`cargo test -p locus-validate` passed:

```text
locus-validate: 9 passed
doc tests: passed
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

Docker `cargo test -p locus-validate` passed:

```text
locus-validate: 17 passed
doc tests: passed
```

The first Docker live-flow command failed before running the probes because `cargo` was not on `PATH` inside `sh -lc`:

```text
sh: 1: cargo: not found
```

The corrected Docker live-flow command used `/usr/local/cargo/bin/cargo` and passed. Probe output:

```text
arena_capacity=16384
max_locked_bytes=40958
pool_stats phase=initial locked_bytes=0 checked_out=0 idle=0 created_arenas=0 reused_arenas=0 checkout_count=0 release_count=0
pool_checkout=ok handle=0
checked_out_mapping_start=0xffff92121000
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

The postulate survived. Locus now has a validation-layer verdict for host page-locked scratch pool reuse.

This is still not a GPU staging proof. The gate confirms that small host page-locked scratch checkout, allocation, release, and reuse worked in the current environment.
