# Postulate 0071: Memory Policy Readiness Consistency

Date: 2026-07-02

## Statement

The Linux memory-policy readiness parser should reject inconsistent status and reason pairs.

## Rationale

The readiness line is an automation contract for deciding whether a placement proof can proceed. Token parsing alone is not enough if it accepts contradictory output such as `memory_policy_readiness=ready reason=permission_denied`.

Rejecting incoherent pairs keeps captured probe output from being treated as valid readiness evidence.

## Experiment

Add consistency rules for `LinuxNumaPolicyReadiness`:

- `ready` must use `ready`;
- `not_ready` must use `invalid_node`, `permission_denied`, or `syscall_failed`.

Update line and output parser tests to reject inconsistent pairs.

## Expected Result

The parser should continue accepting valid memory-policy readiness output and reject inconsistent status and reason combinations. Workspace validation and Docker `locus-sys` tests should pass.
