# Postulate 0248: Remote-Free Service Telemetry Rollup Check Log Summary JSON Verdict

Date: 2026-07-03

## Claim

Saved-log summary drift verification can emit a compact JSON verdict that
dashboards can ingest for both matched and drifted archived summary records.

## Rationale

Experiment 0255 proved that an archived summary JSON line can be checked
against the source rollup-check JSON records it claims to summarize. The
current command is human-readable on success and exits with an error on drift.

Dashboards need a stable verdict artifact that records whether the archived
summary matched, which counter drifted when it did not match, and the expected
and actual typed summaries. That lets CI archive one machine-readable verdict
beside the source log and summary log.

## Test

Add a typed verification report and compact JSON formatter for saved-log
summary drift verification.

Focused tests should prove:

- matching source and archived summary logs produce a matched verdict;
- drifted archived summary logs produce a drifted verdict without losing the
  expected and actual typed summaries;
- the verdict JSON is single-line and schema-tagged;
- grouped `expected`, `actual`, and `drift` fields match the typed verdict.

Real evidence should regenerate the combined rollup-check log and archived
summary log, emit a matched JSON verdict, then emit a drifted JSON verdict
from a controlled `records=1` edit.

## Expected Outcome

The postulate survives if the verifier can produce stable machine-readable
verdict JSON for both accepted and drifted dashboard summary artifacts.
