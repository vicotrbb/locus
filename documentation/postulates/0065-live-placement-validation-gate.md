# Postulate 0065: Live Placement Validation Gate

Date: 2026-07-02

## Statement

Locus should provide a one-command live placement validation gate that attempts the mapped scratch placement workflow and prints the combined verdict.

## Rationale

The file-based gate is useful for captured outputs, but a future permitted Linux host should be able to run one command that attempts memory policy application, touches pages, inspects locality evidence, and prints the final combined gate. This reduces shell orchestration around the three lower-level probes while preserving the same conservative proof conditions.

The live gate belongs in `locus-validate` because it combines allocator, system, and observability evidence.

## Experiment

Add a Linux-only `live_placement_validation_gate` example to `locus-validate`. It should:

- allocate a mapped scratch arena on node 0;
- attempt Linux memory policy binding;
- write-touch mapped pages;
- inspect availability of `numa_maps`, cgroup NUMA stats, and node `numastat`;
- classify the mapped arena placement proof;
- print `placement_validation_gate=<status> reason=<reason>`.

## Expected Result

The example should compile under workspace checks. In Docker it should report a not-ready gate because `mbind` is denied and `numa_maps` is unavailable.
