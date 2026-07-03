# Experiment 0103: Live Pinned Scratch Near-GPU Validation Gate

Date: 2026-07-02

## Postulate

[Postulate 0095: Live Pinned Scratch Near-GPU Validation Gate](../postulates/0095-live-pinned-scratch-near-gpu-validation-gate.md)

## Change

Added `live_pinned_scratch_near_gpu_validation_gate` to `locus-validate`.

The example discovers sysfs topology, accepts an optional GPU BDF, emits the same stable near-GPU pinned scratch probe lines as `pinned_scratch_near_gpu`, and evaluates those exact lines into:

```text
pinned_scratch_near_gpu_validation_gate=<status> reason=<reason>
```

This remains host topology and page-lock validation only. It does not register memory with CUDA, select a CUDA device, prove DMA behavior, or prove final page placement.

## Commands

```text
cargo run -p locus-validate --example live_pinned_scratch_near_gpu_validation_gate
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-validate --example live_pinned_scratch_near_gpu_validation_gate
```

## Results

Local live gate output:

```text
topology_nodes=0
topology_pci_devices=0
near_gpu_pool=unavailable reason=no_gpu_with_numa_node
pinned_scratch_near_gpu_validation_gate=unavailable reason=no_gpu_with_numa_node
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

Clippy initially rejected the live example because `main` was too large. After splitting construction, checkout, allocation, and release handling into helpers, clippy passed with `-D warnings`.

Docker live gate output:

```text
topology_nodes=0
topology_pci_devices=0
near_gpu_pool=unavailable reason=no_gpu_with_numa_node
pinned_scratch_near_gpu_validation_gate=unavailable reason=no_gpu_with_numa_node
```

## Conclusion

The postulate survives. Near-GPU pinned scratch validation now has a live one-command probe and gate for host qualification and CI workflows.

The current host and Docker container still do not expose GPU PCI NUMA locality, so this remains an unavailable-topology result rather than a successful GPU-local checkout proof. A future run on a GPU-visible NUMA host should exercise the `ready` path and pair it with placement evidence.
