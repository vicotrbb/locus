# Postulate 0033: Bind Probe Placement Verified Output

Date: 2026-07-02

## Statement

The mapped scratch bind probe should print an explicit `placement_verified` field derived from the conservative placement proof helper.

## Rationale

`placement_status` is descriptive, but validation automation benefits from a boolean that is true only when all reported pages for the matched mapping are on the expected node. The probe should expose that value without weakening the proof condition.

## Experiment

Update matched `numa_maps` output to include:

- `placement_verified`;
- expected-node pages;
- other-node pages;
- total pages.

## Expected Result

The probe should compile under all-target checks. In the current Docker environment, the matched `numa_maps` path is not reached because `numa_maps` remains unavailable.
