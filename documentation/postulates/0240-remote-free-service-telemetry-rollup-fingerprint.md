# Postulate 0240: Remote-Free Service Telemetry Rollup Fingerprint

Date: 2026-07-03

## Claim

Release-check output can expose a stable evidence fingerprint for rollup
artifacts without introducing cryptographic dependencies.

## Rationale

Release-check output now includes artifact path, schema, byte count, bundle
counts, host coverage, and status coverage. A compact fingerprint over the
exact artifact text would make it easier to compare copied logs, CI output,
and local artifacts without opening the JSON file again.

This fingerprint is for evidence identity and drift triage. It must not be
described as cryptographic integrity, tamper resistance, or provenance proof.
The validation verdict must still come from schema, counts, failed statuses,
timing ranges, and bundle rows.

## Test

Add a deterministic dependency-free fingerprint to the release-check report.

Focused tests should prove:

- the fingerprint is computed from the exact artifact bytes read by the
  checker;
- changing artifact text changes the fingerprint;
- the fingerprint appears only on successful release-check output;
- unsupported schemas still fail before producing an ok report.

Real evidence should validate both current rollup artifacts and record their
fingerprints.

## Expected Outcome

The postulate survives if release checks print stable artifact fingerprints for
passing artifacts while preserving all existing validation behavior and without
adding dependencies.
