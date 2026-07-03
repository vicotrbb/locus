# Postulate 0237: Remote-Free Service Telemetry Rollup Host Coverage

Date: 2026-07-03

## Claim

Release checks can report rollup host and bundle host coverage counts without
making host metadata part of the release-check verdict.

## Rationale

Experiments 0241 through 0244 added optional host metadata to rollup refresh
context, collection summaries, validator output, and rollup bundle rows. The
release-check output still reports only count and status verdict fields, so a
human or dashboard cannot quickly tell whether a persisted rollup has host
context coverage.

Host metadata remains evidence-triage context. It should help explain where
benchmark evidence came from, but it should not decide whether the persisted
rollup passes. Older artifacts without bundle host metadata must keep passing.

## Test

Add non-verdict host coverage fields to the release-check report:

- whether the rollup artifact has a rollup-level host object;
- how many bundle rows have a bundle-level host object;
- how many bundle rows are missing bundle-level host objects.

Focused tests should prove:

- old no-host bundle rows still pass and report zero bundle host coverage;
- host-bearing bundle rows pass and report full bundle host coverage;
- host coverage does not participate in count drift or failed-row verdicts.

Real evidence should validate both the host-bearing and old no-host rollup
artifacts.

## Expected Outcome

The postulate survives if host coverage appears in release-check output while
the same artifacts still pass or fail only by schema, counts, statuses, timing
ranges, and bundle row count.
