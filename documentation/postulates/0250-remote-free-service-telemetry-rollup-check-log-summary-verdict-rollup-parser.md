# Postulate 0250: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Parser

Date: 2026-07-03

## Claim

Saved verdict rollup JSON lines can be parsed back into typed rollups so
archived dashboard rollups can be verified after publication.

## Rationale

Experiment 0257 added a compact JSON rollup for saved verdict records. Once a
dashboard publishes or archives that rollup, Locus needs a way to validate the
published artifact itself without rereading every original verdict record.

The parser should check schema, flat counters, status coverage, and drift-field
coverage. This catches stale or manually edited dashboard rollup records that
remain valid JSON but no longer carry internally consistent rollup data.

## Test

Add a public parser for verdict rollup JSON lines.

Focused tests should prove:

- formatting then parsing a verdict rollup round-trips to the same typed
  rollup;
- schema drift is rejected;
- missing grouped fields are rejected;
- grouped status coverage drift is rejected;
- grouped drift-field coverage drift is rejected.

Real evidence should parse the real mixed verdict rollup output back into the
same human-readable verdict rollup line.

## Expected Outcome

The postulate survives if archived verdict rollup JSON reconstructs typed
rollups and rejects malformed or internally inconsistent dashboard rollups.
