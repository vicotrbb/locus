# Postulate 0297: Remote-Free Service Telemetry Rollup Check Log Summary Verdict Rollup Verification Summary Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON

Date: 2026-07-03

## Claim

Repeated-check dashboard archive drift verdict rollup drift reports can emit
compact JSON verdicts so release dashboards can store matched and stale cohort
rollup outcomes as machine readable records.

## Rationale

Experiment 0304 proved that archived repeated-check dashboard archive drift
verdict rollup JSON verifies against saved archive drift verdict records. The
next release-dashboard need is a compact machine readable verdict for that
rollup drift check itself, so dashboard archives can keep both the human
summary and the typed `matched` or `drifted` result.

The formatter should preserve the same typed counters as the drift checker,
including expected records, actual records, matched status, and the first drift
field. The parser should reload the emitted JSON line so stored dashboard
records can be rechecked without relying only on console text.

## Test

Run the compact JSON verdict mode against the real Experiment 0302 source
records and real Experiment 0303 archived rollup. Then rerun it against the
controlled stale `records=1` archived rollup from Experiment 0304.

Parse both saved JSON verdict logs back through the validation example.

## Expected Outcome

The postulate survives if the real archive emits and reloads a JSON verdict
with `status=matched`, `matched=true`, `drift=null`, and two expected and
actual records, while the stale archive emits and reloads a JSON verdict with
`status=drifted`, `matched=false`, `field=records`, expected `2`, and actual
`1`.
