# Postulate 0052: Placement Proof Verdict

Date: 2026-07-02

## Statement

Placement probes should print one final machine-readable proof verdict in addition to raw placement evidence.

## Rationale

The placement checklist defines the required primary evidence for verified NUMA placement, but the live probe currently emits separate facts. Automation and future benchmark harnesses should not infer proof status from several lines of output.

A small observe-layer verdict type can keep proof classification consistent while leaving system reads and allocator probes separate.

## Experiment

Add:

- `NumaPlacementProofStatus`;
- `NumaPlacementProofReason`;
- `NumaPlacementProof`;
- a constructor that combines memory-policy success and optional `NumaPlacementEvidence`;
- mapped scratch bind probe output of `placement_proof=<status> reason=<reason>`.

## Expected Result

The helper should pass focused unit tests. The mapped scratch bind probe should still report unavailable evidence in Docker, but it should now print a final `placement_proof` line when `numa_maps` is missing or unavailable.
