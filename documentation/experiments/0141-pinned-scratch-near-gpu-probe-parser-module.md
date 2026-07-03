# Experiment 0141: Pinned Scratch Near-GPU Probe Parser Module

Date: 2026-07-03

Postulate: `documentation/postulates/0133-pinned-scratch-near-gpu-probe-parser-module.md`

## Question

Can the near-GPU pinned scratch probe parser move out of `crates/locus-alloc/src/lib.rs` into a focused module while preserving the public `locus_alloc::*` API, shared pool parser behavior, downstream validation behavior, and real Docker gate output?

## Change

Moved the near-GPU pinned scratch probe parser subsystem into:

- `crates/locus-alloc/src/pinned_scratch_near_gpu_probe.rs`

The new module owns:

- `PinnedScratchNearGpuProbeStatus`;
- `PinnedScratchNearGpuPoolLine`;
- `PinnedScratchNearGpuProbeOutput`;
- `PinnedScratchNearGpuProbeLineParseError`;
- `PinnedScratchNearGpuProbeOutputParseError`;
- `parse_pinned_scratch_near_gpu_probe_pool_line`;
- `parse_pinned_scratch_near_gpu_probe_output`;
- private near-GPU numeric, duplicate-field, required-field, event, and stats helpers;
- focused near-GPU parser tests.

The module consumes shared pool event and stats parsers from `pinned_scratch_pool_probe.rs`. `crates/locus-alloc/src/lib.rs` now keeps the near-GPU API source-compatible through `pub use` re-exports.

## Size Result

| File | Lines before | Lines after |
| --- | ---: | ---: |
| `crates/locus-alloc/src/lib.rs` | 892 | 74 |
| `crates/locus-alloc/src/pinned_scratch_near_gpu_probe.rs` | 0 | 830 |

Root `lib.rs` shrank by 818 lines and is now primarily a module and re-export surface plus the shared alignment constant.

## Validation

Commands:

```sh
cargo fmt --all
cargo test -p locus-alloc
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -q -p locus-validate --example live_pinned_scratch_near_gpu_validation_gate
```

Results:

- `cargo test -p locus-alloc`: passed, 59 tests.
- `cargo test --workspace`: passed, 153 unit tests plus doc tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- Docker live near-GPU pinned scratch gate: passed with `pinned_scratch_near_gpu_validation_gate=unavailable reason=no_gpu_with_numa_node`.

Docker near-GPU evidence:

```text
topology_nodes=0
topology_pci_devices=0
near_gpu_pool=unavailable reason=no_gpu_with_numa_node
pinned_scratch_near_gpu_validation_gate=unavailable reason=no_gpu_with_numa_node
```

The unavailable result is expected in this Docker container because no GPU PCI locality is visible. The result still verifies the stable near-GPU output path and downstream validation parser after extraction.

## Conclusion

Postulate 0133 survived.

The near-GPU parser now has a focused module and focused tests, the root API remains source-compatible, the parser still consumes shared pool event and stats lines, and Docker validation still produces the expected unavailable gate output on this container.
