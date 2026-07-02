# Postulate 0068: Validation Gate Verdict Consistency

Date: 2026-07-02

## Statement

The combined placement validation gate parser should reject inconsistent status and reason pairs.

## Rationale

The final gate line is the highest-level validation verdict. Token parsing alone is not enough if it accepts contradictory output such as `placement_validation_gate=verified reason=memory_policy_not_ready`.

Rejecting inconsistent pairs makes automation stricter and keeps malformed validation output from being treated as a coherent verdict.

## Experiment

Add consistency rules for `PlacementValidationGateVerdict`:

- `verified` must use `verified`;
- `not_ready` must use `memory_policy_not_ready` or `placement_evidence_not_ready`;
- `unverified` must use `placement_proof_unverified`;
- `unavailable` must use `placement_proof_unavailable`.

Update line and output parser tests to reject inconsistent pairs.

## Expected Result

The parser should continue accepting valid gate output and reject inconsistent status and reason combinations. Workspace validation and Docker `locus-validate` tests should pass.
