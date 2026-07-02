# Postulate 0066: Live Gate Evidence Lines

Date: 2026-07-02

## Statement

The live placement validation gate should print individual evidence-source availability lines before its combined verdict.

## Rationale

The live gate already prints `placement_validation_readiness`, but a not-ready verdict is easier to diagnose when the output also shows which evidence sources were available. This keeps the one-command path self-describing and reduces the need to rerun lower-level probes after a failed gate.

## Experiment

Update `live_placement_validation_gate` to print:

- `numa_maps=available` or `numa_maps=unavailable`;
- `cgroup_numa_stat=available` or `cgroup_numa_stat=unavailable`;
- `node_numastat=available` or `node_numastat=unavailable`.

## Expected Result

The example should compile under workspace checks. Docker should preserve the not-ready combined gate and show all three evidence sources as unavailable.
