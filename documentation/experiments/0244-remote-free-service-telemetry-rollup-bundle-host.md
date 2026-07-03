# Experiment 0244: Remote-Free Service Telemetry Rollup Bundle Host

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0236-remote-free-service-telemetry-rollup-bundle-host.md`

The postulate said that directory rollup bundle rows could carry each
bundle's capture host metadata without changing release-check verdict
semantics.

## Change

Directory rollup bundle validations now carry optional summary host metadata.
The rollup builder copies that metadata into each bundle row when the
underlying summary can be parsed, and the rollup writer emits a bundle-level
`host` object only when metadata is present.

The release checker remains count-focused. It still validates schema,
aggregate counts, failed bundle statuses, timing ranges, and bundle row count.
It does not include host metadata in the pass or fail decision.

## Commands

```text
cargo fmt --all --check
cargo test -p locus-validate collection_summary -- --nocapture
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --dir target/locus-evidence/remote-free-service-summary-host-json --write-rollup
sed -n '1,220p' target/locus-evidence/remote-free-service-summary-host-json/collection-summary-rollup.json
wc -c target/locus-evidence/remote-free-service-summary-host-json/collection-summary-rollup.json
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup target/locus-evidence/remote-free-service-summary-host-json/collection-summary-rollup.json
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --dir target/locus-evidence/remote-free-service-summary-json --write-rollup
sed -n '1,220p' target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json
wc -c target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace
git diff --check
rg -n "$(printf '\342\200\224')" documentation crates || true
```

## Results

Focused library tests passed:

```text
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

Focused example tests passed:

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The host-bearing evidence root wrote a 694-byte rollup artifact with a
bundle-level host object:

```text
remote_free_service_telemetry_collection_summary_rollup root=target/locus-evidence/remote-free-service-summary-host-json summaries=1 valid_bundles=1 drifted_summaries=0 missing_artifacts=0 other_failures=0 timing_ranges=1
remote_free_service_telemetry_collection_summary_rollup_artifact=written path=target/locus-evidence/remote-free-service-summary-host-json/collection-summary-rollup.json bytes=694
```

The relevant bundle row was:

```json
{
  "host": {
    "arch": "aarch64",
    "hostname": null,
    "os": "macos"
  },
  "run_id": "apply-confirm-summary-host-1783084007",
  "status": "valid",
  "summary": "apply-confirm-summary-host-1783084007/collection-summary.json",
  "timing_ranges": 1
}
```

The artifact-only release check still reported the same count-focused verdict:

```text
remote_free_service_telemetry_collection_summary_rollup_check=ok path=target/locus-evidence/remote-free-service-summary-host-json/collection-summary-rollup.json summaries=1 valid_bundles=1 timing_ranges=1 bundles=1
```

The older no-host evidence root still wrote a 591-byte artifact. Its bundle row
omitted `host`, while the rollup refresh host remained present:

```text
remote_free_service_telemetry_collection_summary_rollup root=target/locus-evidence/remote-free-service-summary-json summaries=1 valid_bundles=1 drifted_summaries=0 missing_artifacts=0 other_failures=0 timing_ranges=1
remote_free_service_telemetry_collection_summary_rollup_artifact=written path=target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json bytes=591
```

The old artifact also passed the release check:

```text
remote_free_service_telemetry_collection_summary_rollup_check=ok path=target/locus-evidence/remote-free-service-summary-json/collection-summary-rollup.json summaries=1 valid_bundles=1 timing_ranges=1 bundles=1
```

The first broad clippy run caught two code-quality issues:

```text
this function has too many lines (103/100)
called `map(<f>).unwrap_or(<a>)` on a `Result` value
```

The test fixture was extracted into a helper and the identity read switched to
`map_or`. Final broad gates passed:

```text
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace
```

## Interpretation

The postulate survived.

Bundle-level host metadata gives rollup readers the capture host for each
evidence bundle, while old no-host bundles remain valid and release checking
keeps the same verdict semantics.

The artifact size delta is clear in the real evidence:

- host-bearing summary root: 694 bytes, with bundle `host`;
- old no-host summary root: 591 bytes, without bundle `host`.

## Next Question

Can rollup release checks optionally report host coverage counts without
making host metadata part of the verdict?
