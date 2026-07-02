# Postulate 0057: Linux Memory Policy Readiness

Date: 2026-07-02

## Statement

The Linux `mbind` probe should print a typed memory-policy readiness verdict in addition to the raw syscall result.

## Rationale

Placement validation requires two separate capabilities: policy application and locality evidence. The locality environment probe now reports evidence readiness, but `mbind_region` still leaves memory-policy permission as a raw success or error string.

A typed readiness verdict in `locus-sys` can classify successful policy application, permission denial, invalid node input, and other syscall failures without weakening the narrow syscall boundary.

## Experiment

Add a Linux NUMA policy readiness helper to `locus-sys` and update the `mbind_region` example to print:

```text
memory_policy_readiness=<status> reason=<reason>
```

## Expected Result

The helper should pass focused tests. The Docker `mbind_region` example should still report the current `Operation not permitted` result and should end with a not-ready memory-policy verdict.
