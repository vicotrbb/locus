# Postulate 0222: Remote-Free Service Telemetry Evidence Summary JSON

Date: 2026-07-03

## Claim

The remote-free service telemetry collector can emit a compact JSON summary
beside `manifest.txt` and `validation-summary.txt` that records collection
mode, run count, benchmark filters or saved-output inputs, Criterion arguments,
and artifact byte counts without weakening the existing manifest-backed
validation path.

## Rationale

The collector now creates durable evidence bundles, but follow-up tooling still
has to inspect filenames, parse command lines from experiment notes, or read
large captured output files to understand what a run contains. A compact JSON
summary should make repeated cohorts easier to index and compare while leaving
the text manifest as the authoritative validation input.

## Test

Add `collection-summary.json` to each evidence directory. It should include:

- a schema string;
- the collection mode;
- the run id;
- the number of captured outputs;
- the Criterion arguments used for direct benchmark capture;
- one artifact entry per captured output plus `manifest.txt` and
  `validation-summary.txt`;
- byte counts read from the files after they are written.

Validate it with a real repeated direct capture of
`remote_free_service_runtime_apply_confirm` and inspect the JSON summary
instead of relying only on the larger captured benchmark outputs.

## Expected Outcome

The postulate survives if the real repeated direct capture produces a valid
`collection-summary.json` whose artifact byte counts match filesystem metadata
and whose run count matches the manifest-backed validation summary.
