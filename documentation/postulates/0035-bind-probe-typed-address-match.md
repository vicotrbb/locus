# Postulate 0035: Bind Probe Typed Address Match

Date: 2026-07-02

## Statement

The mapped scratch bind probe should use the shared typed `numa_maps` address matcher instead of duplicating exact and containing lookup logic.

## Rationale

The probe previously encoded match precedence locally. Using `numa_maps_entry_for_address` keeps exact-start preference and containing-range fallback consistent across future probes, and the output can report the typed match kind directly.

## Experiment

Update the bind probe to:

- call `numa_maps_entry_for_address`;
- print the match kind from `NumaMapsAddressMatchKind`;
- keep placement evidence and verification output unchanged.

## Expected Result

The probe should compile under all-target checks. In the current Docker environment, the match path is not reached because `numa_maps` remains unavailable.
