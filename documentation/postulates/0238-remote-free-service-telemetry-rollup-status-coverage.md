# Postulate 0238: Remote-Free Service Telemetry Rollup Status Coverage

Date: 2026-07-03

## Claim

Release-check output can report bundle status coverage by status without
rescanning evidence directories.

## Rationale

The rollup release checker already reads every persisted bundle row to validate
aggregate counts and failed statuses. That same artifact-only pass can expose
the status distribution for dashboards and triage logs.

This should not change verdict semantics. A rollup with drifted, missing, or
other-failure rows should still fail because failed rows exist, not because the
status coverage fields exist. A clean rollup should still pass and report zero
failed status counts.

## Test

Add status coverage fields to the release-check report and failed-bundles
error output:

- valid bundle rows;
- drifted summary rows;
- missing artifact rows;
- other failure rows.

Focused tests should prove:

- clean rollups pass and report zero failed status counts;
- mixed valid and drifted rollups fail and still report the status coverage
  that was computed before the verdict;
- real rollup artifacts report status coverage without directory rescans.

## Expected Outcome

The postulate survives if status coverage appears in release-check output for
clean artifacts and in failed-bundles errors for failed artifacts, while pass
or fail behavior remains driven by the existing status and count rules.
