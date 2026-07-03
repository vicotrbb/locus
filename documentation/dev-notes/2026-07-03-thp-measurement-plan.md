# THP Measurement Plan

Date: 2026-07-03

## Current Evidence

Locus now has three pieces of mapped scratch transparent huge page evidence:

- `mapped_scratch_thp` emits advisory status and page-size observation lines.
- `mapped_scratch_thp_validation_gate` classifies captured output.
- `live_mapped_scratch_thp_validation_gate` collects live evidence and prints the final gate.
- `mapped_scratch_write_touch_4mib_*` benchmarks compare default mapping, `hugepage` advice, and `no_hugepage` advice for first-touch cost.

The first Docker benchmark sample showed a much faster `hugepage` advice case, but the live Docker gate still reported `unavailable reason=observation_unavailable` because `numa_maps` evidence was unavailable.

## Measurement Rule

Do not treat THP benchmark speedups as huge page adoption proof unless the same environment also reports:

```text
mapped_scratch_thp_validation_gate=ready reason=ready
```

If the gate reports `unavailable`, benchmark results are still useful as environment baseline data, but they do not prove the mapping used larger pages.

## Repeatable Run Set

Use this minimal run set before interpreting a THP performance result:

```text
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-validate --example live_mapped_scratch_thp_validation_gate
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo bench -p locus-alloc --bench scratch_arena -- mapped_scratch_write_touch_4mib --sample-size 30
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-validate --example live_mapped_scratch_thp_validation_gate
```

Run the gate before and after the benchmark so an environment change is visible. Increase sample size for any result that will influence allocator policy.

## Next Questions

- Which Linux host or container configuration exposes `numa_maps` page-size evidence for this mapping?
- Does the `hugepage` advice case remain faster across repeated runs and larger sample sizes?
- Does ready gate evidence correlate with the faster first-touch benchmark result?
- Should the benchmark emit or capture the validation gate result alongside Criterion output?
