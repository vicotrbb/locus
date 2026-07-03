# Experiment 0235: Remote-Free Service Telemetry Rollup Bundle Table

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0227-remote-free-service-telemetry-rollup-bundle-table.md`

The postulate said that the remote-free service telemetry rollup artifact could
include a compact per-bundle table with summary paths, run ids, validation
status, and timing range counts while staying small enough for release checks.

## Change

Extended `collection-summary-rollup.json` to schema:

```text
locus.remote_free_service.telemetry.collection_summary_rollup.v2
```

The artifact now includes a sorted `bundles` array. Each row records:

- relative `summary` path;
- `run_id` when the summary can be parsed;
- validation `status`;
- `timing_ranges` for valid bundles.

The existing terminal rollup line is unchanged.

## Commands

```text
cargo fmt --all --check
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --dir target/locus-evidence/remote-free-service-summary-json --write-rollup
cargo test -p locus-validate collection_summary -- --nocapture
sed -n '1,160p' target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json
wc -c target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json
find target/locus-evidence/remote-free-service-summary-json -maxdepth 2 -name 'collection-summary*.json' -print | sort
```

## Results

The focused example tests passed, including the rollup artifact and classified
bundle row checks:

```text
test tests::reports_matching_validation_summary ... ok
test tests::rejects_drifted_validation_summary ... ok
test tests::writes_directory_rollup_artifact ... ok
test tests::rolls_up_valid_drifted_and_missing_bundles ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The real evidence root produced:

```text
remote_free_service_telemetry_collection_summary_rollup root=target/locus-evidence/remote-free-service-summary-json summaries=1 valid_bundles=1 drifted_summaries=0 missing_artifacts=0 other_failures=0 timing_ranges=1
remote_free_service_telemetry_collection_summary_rollup_artifact=written path=target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json bytes=511
```

The persisted artifact was:

```text
{
  "bundles": [
    {
      "run_id": "apply-confirm-summary-1783084007-13676",
      "status": "valid",
      "summary": "apply-confirm-summary-1783084007-13676/collection-summary.json",
      "timing_ranges": 1
    }
  ],
  "drifted_summaries": 0,
  "missing_artifacts": 0,
  "other_failures": 0,
  "root": "target/locus-evidence/remote-free-service-summary-json",
  "schema": "locus.remote_free_service.telemetry.collection_summary_rollup.v2",
  "summaries": 1,
  "timing_ranges": 1,
  "valid_bundles": 1
}
```

The artifact byte count was:

```text
511 target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json
```

## Interpretation

The postulate survived.

The rollup artifact now identifies the exact validated bundle without rerunning
the scanner. The current real evidence root still fits under the 512-byte
target while carrying the run id, relative summary path, status, and timing
range count.

## Next Question

Can release checks consume the rollup artifact directly and reject drifted,
missing, or otherwise failed bundle rows without scanning the evidence tree?
