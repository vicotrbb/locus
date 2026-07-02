# Postulate 0073: Placement Proof Consistency

Date: 2026-07-02

## Statement

The mapped scratch placement proof parser should reject inconsistent status and reason pairs.

## Rationale

The placement proof line is the primary mapping-specific evidence contract. A contradictory line such as `placement_proof=verified reason=policy_not_applied` should not be accepted as a coherent proof result.

Rejecting inconsistent pairs keeps malformed proof output from entering the combined validation gate.

## Experiment

Add consistency rules for `NumaPlacementProof`:

- `verified` must use `verified`;
- `unverified` must use `policy_not_applied`, `mapping_missing`, `no_pages_reported`, `partial_pages_on_expected_node`, or `no_pages_on_expected_node`;
- `unavailable` must use `numa_maps_unavailable`.

Update line and output parser tests to reject inconsistent pairs.

## Expected Result

The parser should continue accepting valid placement proof output and reject inconsistent status and reason combinations. Workspace validation should pass.
