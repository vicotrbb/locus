# Postulate 0074: Memory Readiness Display

Date: 2026-07-02

## Statement

The Linux memory-policy readiness verdict should expose a stable display representation for its final machine-readable line.

## Rationale

Multiple probes print the same readiness schema:

```text
memory_policy_readiness=<status> reason=<reason>
```

Centralizing that rendering in `Display` keeps probe output aligned with parser expectations and reduces duplicated formatting in examples.

## Experiment

Implement `Display` for `LinuxNumaPolicyReadiness`. Update probes that already hold a readiness verdict to print the value directly, then cover the rendered line in focused tests.

## Expected Result

The probes should keep printing the same memory-policy readiness line. Host validation and Docker `locus-sys` tests should pass.
