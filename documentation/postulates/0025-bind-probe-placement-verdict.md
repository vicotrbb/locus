# Postulate 0025: Bind Probe Placement Verdict

Date: 2026-07-02

## Statement

The mapped scratch bind probe should print the placement classifier verdict when it can match the arena mapping to `numa_maps`.

## Rationale

Raw per-node page counts are useful, but probes should produce a stable verdict that can be compared across benchmark runs and automation. The `NumaPlacementEvidence` classifier exists for this purpose, and the live probe should consume it directly.

## Experiment

Update the bind probe to:

- build `NumaPlacementEvidence` from an exact or containing `numa_maps` match;
- print a stable placement status string;
- print expected node, expected-node pages, total pages, and raw node pages;
- preserve explicit unavailable behavior when `numa_maps` cannot be read.

## Expected Result

The probe should compile under all-target checks. In the current Docker environment, the verdict path is not reached because `numa_maps` remains unavailable.
