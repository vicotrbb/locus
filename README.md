# Locus

Locus is an experimental Rust memory locality runtime for AI inference workloads. The initial foundation focuses on explicit domain allocators, Linux topology discovery, placement policy modeling, and measured allocator experiments.

The project deliberately starts without a process-wide allocator replacement. Early work is organized around safe Rust APIs that make memory class, placement, and lifetime explicit.

## Current Foundation

- `locus-core`: topology data types, Linux CPU-list parsing, and placement policy models.
- `locus-topology`: Linux sysfs discovery for NUMA nodes and PCI device locality.
- `locus-observe`: parsers, summaries, deltas, and classifiers for Linux NUMA locality evidence.
- `locus-sys`: narrow unsafe boundary for owned mappings, page touching, and Linux NUMA policy probes.
- `locus-alloc`: safe node-tagged scratch arenas, request scratch pools, KV block foundations, and host page-locked scratch pools.
- `locus-validate`: combined validation gates for probe outputs.

## Validation

Run the foundation tests:

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Run the scratch arena benchmark harness, including a default `Vec<u8>` allocation baseline:

```sh
cargo bench -p locus-alloc --bench scratch_arena
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo bench -p locus-alloc --bench scratch_arena -- mapped_scratch_write_touch_4mib --sample-size 10
```

Run the current Linux-oriented sys probes through Docker:

```sh
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-sys
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-sys --example mbind_region
```

The `mbind_region` example reports whether the current Linux environment permits `mbind`, then write-touches the mapped pages. Some containers return `EPERM` as `memory_policy_readiness=not_ready reason=permission_denied`. In the current seccomp-unconfined Docker environment, `mbind` returns `ENOSYS` as `memory_policy_readiness=not_ready reason=syscall_unavailable`. Both are recorded as environment evidence, not treated as placement success.

Run the current locality evidence probes:

```sh
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-observe --example locality_environment
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-observe --example process_fault_counts
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-alloc --example mapped_scratch_bind
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-alloc --example mapped_scratch_lock
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-alloc --example mapped_scratch_thp
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-alloc --example pinned_scratch_pool
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-alloc --example pinned_scratch_near_gpu
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-validate --example live_pinned_scratch_validation_gate
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-validate --example live_pinned_scratch_near_gpu_validation_gate
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-validate --example live_mapped_scratch_thp_validation_gate
```

The `locality_environment` example reports whether `numa_maps`, cgroup `memory.numa_stat`, and node `numastat` are available. The `process_fault_counts` example reports `/proc/self/stat` minor and major fault counters for interpreting first-touch and THP runs. The `mapped_scratch_bind` example prints the mapped arena address, attempts `mbind`, write-touches pages, and, when the host exposes the evidence, correlates the mapping with `numa_maps` and cgroup NUMA deltas. The `mapped_scratch_lock` example validates the OS page-lock portion of future pinned host staging buffers. The `mapped_scratch_thp` example applies an opt-in transparent huge page hint, write-touches the arena, and reports `numa_maps` page-size evidence with a `smaps` fallback when available. `PinnedScratchPool` builds on that primitive with budgeted checkout and reuse of host page-locked mapped scratch arenas. The `pinned_scratch_pool` example prints stable checkout, allocation, release, reuse, and accounting lines. The `pinned_scratch_near_gpu` example tries to select the pool home node from discovered PCI GPU locality. The `live_pinned_scratch_validation_gate` example prints pinned scratch lines and a final host page-lock readiness gate. The `live_pinned_scratch_near_gpu_validation_gate` example prints near-GPU pinned scratch lines and a final near-GPU validation gate, but it does not yet register memory with CUDA or prove GPU-near placement. The `live_mapped_scratch_thp_validation_gate` example prints mapped scratch THP evidence and a final THP adoption gate.

Captured probe output can be classified with:

```sh
cargo run -p locus-validate --example pinned_scratch_validation_gate -- pinned-scratch.out
cargo run -p locus-validate --example pinned_scratch_near_gpu_validation_gate -- near-gpu-pinned-scratch.out
cargo run -p locus-validate --example mapped_scratch_thp_validation_gate -- mapped-scratch-thp.out
cargo run -p locus-validate --example mapped_scratch_thp_fault_sample_validation_gate -- mapped-scratch-thp-bench.out
cargo run -p locus-validate --example mapped_scratch_thp_fault_sample_report -- mapped-scratch-thp-fault-sample-validation.out
cargo run -p locus-validate --example mapped_scratch_thp_benchmark_evidence_report -- mapped-scratch-thp-bench.out
```

The mapped scratch THP fault sample validation command prints both a sample availability gate and a comparison line for the process minor-fault deltas when counters are usable. The fault-sample report command parses those two lines together and rejects contradictory saved output. The benchmark evidence report command parses `thp_page_sample=` and `fault_sample=` lines from one benchmark log, then reports whether hugepage adoption was observed from page-size evidence.

Captured outputs from `mbind_region`, `locality_environment`, and `mapped_scratch_bind` can be combined with:

```sh
cargo run -p locus-validate --example placement_validation_gate -- \
  memory-policy.out \
  placement-readiness.out \
  placement-proof.out
```

Or run the live combined gate directly on a Linux host or container:

```sh
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-validate --example live_placement_validation_gate
```

Successful NUMA placement is not claimed unless a permitted policy operation is followed by page-touching and matching placement evidence for the specific mapping.

## Research Loop

Every meaningful allocator experiment should have:

- a postulate recorded before implementation;
- an ADR or development note for design decisions;
- focused tests and benchmarks;
- an experiment log with commands, results, and follow-up questions.
