# Postulate 0225: Remote-Free Service Telemetry Summary Directory Rollup

Date: 2026-07-03

## Claim

The remote-free service telemetry summary validator can validate every
`collection-summary.json` under an evidence directory and emit a compact rollup
that counts valid bundles, drifted validation summaries, missing artifacts,
other failures, and timing ranges that survived validation.

## Rationale

Single-bundle validation now proves artifact byte counts, saved summary
integrity, and manifest-backed timing stability. Evidence directories can still
contain multiple run bundles, so review currently requires invoking the
validator once per bundle and manually combining outcomes.

A directory rollup gives repeated benchmark work a cheap audit surface while
preserving the existing single-bundle validation logic.

## Test

Extend `remote_free_service_telemetry_summary_validate` with:

```text
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --dir <evidence-root>
```

The mode should:

- recursively discover `collection-summary.json` files;
- validate each summary with the same single-bundle path;
- count valid bundles;
- count saved validation-summary drift separately;
- count missing artifacts separately;
- count other failures separately;
- sum timing ranges only from valid bundles.

Validate it against the real `remote-free-service-summary-json` evidence root
from Experiment 0230.

## Expected Outcome

The postulate survives if the real evidence directory rollup finds the saved
bundle, reports it as valid, reports zero drift and zero missing artifacts, and
counts the surviving timing range.
