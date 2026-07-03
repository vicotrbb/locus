# Experiment 0102: Pinned Scratch Near-GPU Validation Example

Date: 2026-07-02

## Postulate

[Postulate 0094: Pinned Scratch Near-GPU Validation Example](../postulates/0094-pinned-scratch-near-gpu-validation-example.md)

## Change

Added `pinned_scratch_near_gpu_validation_gate` to `locus-validate`.

The example reads one captured `pinned_scratch_near_gpu` output file and prints:

```text
pinned_scratch_near_gpu_validation_gate=<status> reason=<reason>
```

The README now lists the file-based classification command beside the existing pinned scratch validation gate.

## Commands

```text
cargo run -p locus-alloc --example pinned_scratch_near_gpu > /tmp/locus-near-gpu-pinned-scratch.out
cat /tmp/locus-near-gpu-pinned-scratch.out
cargo run -p locus-validate --example pinned_scratch_near_gpu_validation_gate -- /tmp/locus-near-gpu-pinned-scratch.out
cargo test -p locus-validate
docker run --rm -v "$PWD":/work -w /work rust:1.96 sh -lc '/usr/local/cargo/bin/cargo run -p locus-alloc --example pinned_scratch_near_gpu > /tmp/near-gpu.out && cat /tmp/near-gpu.out && /usr/local/cargo/bin/cargo run -p locus-validate --example pinned_scratch_near_gpu_validation_gate -- /tmp/near-gpu.out'
```

## Results

Local captured probe output:

```text
topology_nodes=0
topology_pci_devices=0
near_gpu_pool=unavailable reason=no_gpu_with_numa_node
```

Local gate output:

```text
pinned_scratch_near_gpu_validation_gate=unavailable reason=no_gpu_with_numa_node
```

Focused validation tests passed:

```text
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The first Docker shell command used bare `cargo` inside `sh -lc` and failed with:

```text
sh: 1: cargo: not found
```

Rerunning with `/usr/local/cargo/bin/cargo` succeeded.

Docker captured probe output:

```text
topology_nodes=0
topology_pci_devices=0
near_gpu_pool=unavailable reason=no_gpu_with_numa_node
```

Docker gate output:

```text
pinned_scratch_near_gpu_validation_gate=unavailable reason=no_gpu_with_numa_node
```

## Conclusion

The postulate survives. Captured near-GPU pinned scratch probe output can now be classified from the command line, and the current host and Docker environment both classify as unavailable because no GPU PCI NUMA locality is visible.

This does not benchmark allocator behavior. It creates the command-line gate needed to compare future GPU-visible hosts and containers with one stable verdict line.
