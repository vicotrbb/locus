# Postulate 0053: Typed Unavailable Placement Proof

Date: 2026-07-02

## Statement

Unavailable placement evidence should be represented by the same typed proof verdict used for verified and unverified placement.

## Rationale

The mapped scratch bind probe now prints a final proof line, but the unavailable `numa_maps` case is still an ad hoc string in the probe. Future automation should consume one proof status and reason model for all primary evidence outcomes.

Adding an unavailable status to the observe-layer proof verdict keeps probe output consistent and makes unavailable evidence distinct from evidence that is present but unverified.

## Experiment

Extend `NumaPlacementProofStatus` and `NumaPlacementProofReason` so `numa_maps` unavailability can be represented by `NumaPlacementProof`. Update the mapped scratch bind probe to print unavailable proof output through that helper.

## Expected Result

The helper should pass focused unit tests. Docker should preserve its current unavailable placement result while routing the final proof line through the typed verdict.
