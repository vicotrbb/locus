# Experiment 0088: Mapped Scratch Lock Probe

Date: 2026-07-02

## Postulate

See `documentation/postulates/0080-mapped-scratch-lock-probe.md`.

## Commands

```sh
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-alloc --example mapped_scratch_lock
```

## Results

Executed on 2026-07-02.

`cargo fmt --all` completed successfully.

`cargo test --workspace` passed on the host:

- `locus-alloc`: 24 host-visible unit tests passed.
- `locus-core`: 9 unit tests passed.
- `locus-observe`: 27 unit tests passed.
- `locus-sys`: 6 host-visible unit tests passed.
- `locus-topology`: 2 unit tests passed.
- `locus-validate`: 0 host-visible unit tests passed because the combined gate is Linux-gated.
- Doc tests completed with no failures.

`cargo clippy --workspace --all-targets -- -D warnings` passed.

Docker `cargo run -p locus-alloc --example mapped_scratch_lock` output:

```text
mapping_start=0xffffac3ff000
mapping_len=20479
touched=5
page_lock=ok
page_unlock=ok
```

## Conclusion

The postulate survived. The mapped scratch lock probe records page-lock behavior in Docker and confirms that a small mapped arena can be locked and unlocked in the current container environment.

This validates the OS page-lock portion of future pinned host staging work. It does not validate GPU DMA registration or GPU-near placement.
