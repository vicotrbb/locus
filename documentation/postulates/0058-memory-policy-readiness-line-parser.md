# Postulate 0058: Memory Policy Readiness Line Parser

Date: 2026-07-02

## Statement

The final Linux memory-policy readiness line should be parsed by a shared `locus-sys` helper.

## Rationale

The `mbind_region` probe now prints a stable final readiness line:

```text
memory_policy_readiness=<status> reason=<reason>
```

Validation automation should consume the same typed `LinuxNumaPolicyReadiness` model used by the probe. A shared parser keeps accepted tokens aligned with the readiness types and catches malformed output.

## Experiment

Add a parser for the final memory-policy readiness line in `locus-sys`. Cover ready and not-ready lines, plus missing, duplicate, unknown, and extra-token failures.

## Expected Result

The parser should return `LinuxNumaPolicyReadiness` for valid readiness lines and typed parse errors for invalid lines. Host workspace validation and Docker Linux tests should pass.
