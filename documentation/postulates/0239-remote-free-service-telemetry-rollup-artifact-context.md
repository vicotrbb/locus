# Postulate 0239: Remote-Free Service Telemetry Rollup Artifact Context

Date: 2026-07-03

## Claim

Release-check output can report artifact byte count and schema version context
without weakening artifact validation.

## Rationale

Release-check logs now report counts, host coverage, and status coverage, but
they still lack two basic artifact facts: the exact byte count read from disk
and the accepted schema string. Both values are already available during the
artifact-only validation path. Reporting them should help compare release
artifacts without requiring a separate `wc` or JSON inspection step.

This context must not soften validation. The checker should still reject
unsupported schema strings before producing an ok report, and byte count should
describe the exact artifact text that was parsed.

## Test

Add release-check report fields for:

- accepted schema string;
- artifact byte count read from disk.

Focused tests should prove:

- successful checks report the expected schema string;
- successful checks report the exact text byte count;
- unsupported schema strings still fail instead of producing context output.

Real evidence should validate both current rollup artifacts and report their
byte counts.

## Expected Outcome

The postulate survives if release-check output includes schema and byte-count
context for passing artifacts while unexpected schemas remain hard failures.
