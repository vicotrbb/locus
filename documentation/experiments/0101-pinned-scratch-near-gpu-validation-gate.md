# Experiment 0101: Pinned Scratch Near-GPU Validation Gate

Date: 2026-07-02

## Postulate

[Postulate 0093: Pinned Scratch Near-GPU Validation Gate](../postulates/0093-pinned-scratch-near-gpu-validation-gate.md)

## Change

Added near-GPU pinned scratch validation gate types and evaluators to `locus-validate`.

The gate prints:

```text
pinned_scratch_near_gpu_validation_gate=<status> reason=<reason>
```

It reports:

- `ready` when the near-GPU probe selected a topology-backed pool home node and checkout, allocation, release, idle accounting, and locked-byte accounting succeeded;
- `unavailable` when the probe reported unavailable GPU PCI NUMA locality;
- `not_ready` when constructor, checkout, allocation, release, or accounting failed after topology selection.

## Commands

```text
cargo test -p locus-validate
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Results

Focused validation tests passed:

```text
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Workspace tests passed:

```text
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Clippy initially rejected an unnested or-pattern in the unavailable reason mapping. After simplifying the match pattern, clippy passed with `-D warnings`.

## Conclusion

The postulate survives. Near-GPU pinned scratch output now has a CI-friendly validation verdict that separates missing topology from real allocator or accounting failures.

This is validation infrastructure rather than an allocator benchmark. It enables the next live or file-based gate experiment to compare hosts with and without visible GPU PCI NUMA locality using one stable verdict line.
