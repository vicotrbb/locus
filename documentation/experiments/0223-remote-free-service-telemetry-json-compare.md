# Experiment 0223: Remote-Free Service Telemetry JSON Compare

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0215-remote-free-service-telemetry-json-compare.md`

The postulate said that optional remote-free service telemetry JSON sample rows
could be consumed by a small Rust validation tool that compares two benchmark
outputs and reports counter drift before timing deltas are trusted.

## Change

Added `remote_free_service_sample_compare.rs` to `locus-validate` and exposed:

- `parse_remote_free_service_telemetry_sample_rows`;
- `compare_remote_free_service_telemetry_sample_outputs`;
- typed sample keys, rows, drift entries, comparison status, and comparison
  report types.

Added the file-based example command:

```text
cargo run -p locus-validate --example remote_free_service_sample_compare -- <baseline-output> <candidate-output>
```

The parser consumes only JSON rows with schema
`locus.remote_free_service.telemetry.sample.v1`, keys rows by `(benchmark,
sample)`, rejects duplicate sample keys, and compares every parsed field value
exactly. The example prints one compact comparison report and then one drift
line for each changed field or missing sample.

## Commands

```text
mkdir -p target/locus-evidence/remote-free-service-sample-compare
LOCUS_REMOTE_FREE_SERVICE_TELEMETRY_JSON=1 cargo bench -p locus-alloc --bench remote_free_service_telemetry remote_free_service_runtime_apply_confirm > target/locus-evidence/remote-free-service-sample-compare/apply-confirm-a.txt 2>&1
LOCUS_REMOTE_FREE_SERVICE_TELEMETRY_JSON=1 cargo bench -p locus-alloc --bench remote_free_service_telemetry remote_free_service_runtime_apply_confirm > target/locus-evidence/remote-free-service-sample-compare/apply-confirm-b.txt 2>&1
cargo run -p locus-validate --example remote_free_service_sample_compare -- target/locus-evidence/remote-free-service-sample-compare/apply-confirm-a.txt target/locus-evidence/remote-free-service-sample-compare/apply-confirm-b.txt
python3 - <<'PY'
from pathlib import Path
base = Path('target/locus-evidence/remote-free-service-sample-compare/apply-confirm-b.txt')
drift = Path('target/locus-evidence/remote-free-service-sample-compare/apply-confirm-drift.txt')
text = base.read_text()
text = text.replace('"submitted_count":768', '"submitted_count":769', 1)
drift.write_text(text)
PY
cargo run -p locus-validate --example remote_free_service_sample_compare -- target/locus-evidence/remote-free-service-sample-compare/apply-confirm-a.txt target/locus-evidence/remote-free-service-sample-compare/apply-confirm-drift.txt
cargo fmt --all --check
git diff --check
rg -n "$(printf '\342\200\224')" documentation crates || true
cargo test -p locus-validate remote_free_service -- --nocapture
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
```

## Results

The real same-benchmark comparison across two JSON-enabled
`remote_free_service_runtime_apply_confirm` runs reported stable counters:

```text
remote_free_service_telemetry_sample_compare=stable baseline_samples=2 candidate_samples=2 compared_samples=2 drift_entries=0
```

A controlled candidate copy with the first JSON `submitted_count` changed from
768 to 769 reported one drift entry:

```text
remote_free_service_telemetry_sample_compare=drift baseline_samples=2 candidate_samples=2 compared_samples=2 drift_entries=1
remote_free_service_telemetry_sample_drift benchmark=remote_free_service_runtime_apply_confirm sample=remote_free_service_runtime_apply_confirm_sample field=submitted_count baseline=768 candidate=769
```

Focused unit tests covered parsing JSON sample rows, stable comparisons,
field drift, missing samples, duplicate sample keys, missing JSON rows, and
unexpected schemas. The focused test run passed seven
`remote_free_service_sample_compare` tests.

The first full clippy run found one ownership issue: `parse_sample_row_value`
accepted a `serde_json::Value` by value without consuming it. The parser was
changed to borrow the value. The final clippy run passed with warnings denied.

Workspace tests passed after the fix. The final `locus_validate` test count
increased from 59 to 66 unit tests.

## Interpretation

The postulate survived.

The repository now has reusable Rust tooling for checking remote-free service
telemetry counter stability across saved benchmark outputs. The tool detects
real JSON rows emitted by `cargo bench`, reports stable same-shape runs, and
surfaces a controlled counter drift before any timing delta is considered.

This is deliberately a validation-layer tool. It depends on `serde_json` in
`locus-validate`, while the allocator crate keeps its benchmark-only JSON
emitter dependency-free.

## Next Question

Can the comparison tool also parse Criterion timing intervals from the same
saved outputs and emit a combined report that refuses timing deltas when
counter drift is present?
