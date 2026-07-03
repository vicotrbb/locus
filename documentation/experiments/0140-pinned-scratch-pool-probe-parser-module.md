# Experiment 0140: Pinned Scratch Pool Probe Parser Module

Date: 2026-07-03

Postulate: `documentation/postulates/0132-pinned-scratch-pool-probe-parser-module.md`

## Question

Can the pinned scratch pool probe parser move out of `crates/locus-alloc/src/lib.rs` into a focused module while preserving the public `locus_alloc::*` API, near-GPU parser behavior, downstream validation behavior, and real Docker gate output?

## Change

Moved the pinned scratch pool probe parser subsystem into:

- `crates/locus-alloc/src/pinned_scratch_pool_probe.rs`

The new module owns:

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
- `parse_pinned_scratch_pool_probe_output`;
- crate-private event and stats line classifiers used by near-GPU parsing;
- private numeric, duplicate-field, required-field, and output aggregation helpers;
- focused pool probe parser tests.

`crates/locus-alloc/src/lib.rs` now keeps the root API source-compatible through `pub use` re-exports. The near-GPU parser still consumes the shared pool event and stats lines through the extracted module.

## Size Result

| File | Lines before | Lines after |
| --- | ---: | ---: |
| `crates/locus-alloc/src/lib.rs` | 1868 | 892 |
| `crates/locus-alloc/src/pinned_scratch_pool_probe.rs` | 0 | 996 |

Root `lib.rs` shrank by 976 lines while the total allocator source size changed only by module boilerplate and formatting.

## Validation

Commands:

```sh
cargo fmt --all
cargo test -p locus-alloc
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -q -p locus-validate --example live_pinned_scratch_validation_gate
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -q -p locus-validate --example live_pinned_scratch_near_gpu_validation_gate
```

Results:

- `cargo test -p locus-alloc`: passed, 59 tests.
- `cargo test --workspace`: passed, 153 unit tests plus doc tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- Docker live pinned scratch gate: passed with `pinned_scratch_validation_gate=ready reason=ready`.
- Docker live near-GPU pinned scratch gate: passed with `pinned_scratch_near_gpu_validation_gate=unavailable reason=no_gpu_with_numa_node`.

Docker base pinned scratch evidence:

```text
pool_stats phase=initial locked_bytes=0 checked_out=0 idle=0 created_arenas=0 reused_arenas=0 checkout_count=0 release_count=0
pool_checkout=ok handle=0
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

Docker near-GPU evidence:

```text
topology_nodes=0
topology_pci_devices=0
near_gpu_pool=unavailable reason=no_gpu_with_numa_node
pinned_scratch_near_gpu_validation_gate=unavailable reason=no_gpu_with_numa_node
```

The near-GPU result is expected in this Docker container because no GPU PCI locality is visible. It still proves the near-GPU gate can parse the shared pool parser API after extraction and classify unavailable topology correctly.

## Conclusion

Postulate 0132 survived.

The pool probe parser now has a focused module and focused tests, the root API remains source-compatible, near-GPU parsing still consumes shared pool lines, and Docker validation gates still produce the expected stable outputs.
