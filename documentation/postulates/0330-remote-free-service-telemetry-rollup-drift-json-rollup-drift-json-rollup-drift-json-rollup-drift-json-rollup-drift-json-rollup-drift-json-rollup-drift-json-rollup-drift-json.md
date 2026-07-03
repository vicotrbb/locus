# Postulate 0330: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON

Date: 2026-07-03

## Claim

Repeated-check dashboard archive drift verdict rollup drift verdict rollup
drift verdict rollup drift verdict rollup drift verdict rollup drift verdict
rollup recheck verdict rollup drift reports can emit compact JSON verdicts so
release dashboards can archive summary recheck decisions.

## Rationale

Experiment 0337 proved that archived aggregate rollup recheck cohort rollup
recheck verdict rollup drift JSON verifies against saved source verdict
records and rejects a controlled stale `records=1` archive. Release dashboards
also need a compact, machine-readable verdict record for those matched and
stale recheck decisions, plus a parser path that can reload the saved verdict
record later.

The verifier should emit one matched JSON verdict for the real archive and one
drifted JSON verdict for the controlled stale archive. The parser should reload
both records into typed reports without requiring the original source logs.

## Test

Run the JSON verdict mode against the saved source verdict records and the
matched saved rollup JSON artifact:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against-json <saved-json-verdict-log.txt> <saved-rollup-log.txt>
```

Run the same mode against a controlled stale rollup copy with top-level
`records=1`:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against-json <saved-json-verdict-log.txt> <stale-rollup-log.txt>
```

Then reload each saved JSON verdict record with the parser-only mode:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-verify <saved-json-verdict-output.txt>
```

## Expected Outcome

The postulate survives if the matched JSON verdict emits and reloads as
`status=matched`, `matched=true`, and `records=2`, while the controlled stale
JSON verdict emits and reloads as `status=drifted`, `matched=false`,
`field=records`, expected `2`, and actual `1`.
