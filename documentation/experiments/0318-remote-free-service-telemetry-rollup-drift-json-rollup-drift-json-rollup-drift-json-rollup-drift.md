# Experiment 0318: Remote-Free Service Telemetry Rollup Drift JSON Rollup Drift JSON Rollup Drift JSON Rollup Drift

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0310-remote-free-service-telemetry-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift.md`

The postulate said that saved repeated-check dashboard archive drift verdict
rollup drift verdict rollup drift verdict rollup drift verdict rollup JSON can
verify against saved source verdict records so release dashboards can reject
stale rollup recheck cohort artifacts.

## Change

No Rust code change was needed. The validation example already exposes strict
and report verification modes for saved rollup recheck cohort rollup JSON:

```text
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against <saved-rollup-recheck-verdict-log.txt> <saved-rollup-log.txt>
remote_free_service_telemetry_summary_validate --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against-report <saved-rollup-recheck-verdict-log.txt> <saved-rollup-log.txt>
```

The strict verifier uses:

```text
verify_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_log
```

The report path uses:

```text
check_remote_free_service_telemetry_collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup_json_log
```

## Commands

```text
mkdir -p target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift
cp target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json.log target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json.log
cp target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-parser/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup.log target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup.log
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json.log target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup.log > target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-strict.log
python3 - <<'PY'
import json
from pathlib import Path
root = Path('target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift')
src = root / 'matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup.log'
out = root / 'stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup.log'
lines = []
for line in src.read_text().splitlines():
    if line.startswith('{'):
        value = json.loads(line)
        if value.get('schema') == 'locus.remote_free_service.telemetry.collection_summary_rollup_check_log_summary_verification_rollup_verification_summary_verification_rollup.v1':
            value['records'] = 1
            line = json.dumps(value, separators=(',', ':'), sort_keys=True)
    lines.append(line)
out.write_text('\n'.join(lines) + '\n')
PY
set +e
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json.log target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup.log > target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-strict.log 2>&1
cmd_rc=$?
printf '%s\n' "$cmd_rc" > target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-strict-status.txt
set -e
cargo run -p locus-validate --example remote_free_service_telemetry_summary_validate -- --rollup-check-json-summary-verdict-rollup-verify-against-json-summary-verify-against-json-rollup-verify-against-json-rollup-verify-against-json-rollup-verify-against-report target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json.log target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup.log > target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-report.log
cargo fmt --all
cargo test -p locus-validate collection_summary -- --nocapture
cargo test -p locus-validate --example remote_free_service_telemetry_summary_validate -- --nocapture
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace --quiet
git diff --check
rg -n "$(printf '\342\200\224')" documentation crates || true
```

## Results

The matched strict check emitted:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification_rollup=ok records=2 matched=1 drifted=1 drift_records=1 drift_matched=0 drift_drifted=0 drift_drift_records=0 drift_drift_rollup_hosts_present=0 drift_drift_rollup_hosts_missing=0 drift_drift_bundle_hosts=0 drift_drift_bundle_hosts_missing=0 drift_drift_status_valid_bundles=0 drift_drift_status_drifted_summaries=0 drift_drift_status_missing_artifacts=0 drift_drift_status_other_failures=0
```

The controlled stale strict check exited with status `1` and reported:

```text
Error: CountDrift { field: "records", expected: 2, actual: 1 }
```

The stale report mode emitted:

```text
remote_free_service_telemetry_collection_summary_rollup_check_log_summary_json_verification_rollup_verification_summary_verification_rollup_verification=drifted field=records expected=2 actual=1 expected_records=2 actual_records=1
```

The artifacts are saved at:

```text
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift/combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift/matched-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-strict.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-strict.log
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-strict-status.txt
target/locus-evidence/remote-free-service-rollup-drift-json-rollup-drift-json-rollup-drift-json-rollup-drift/stale-combined-rollup-verification-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-verdict-json-rollup-report.log
```

Focused collection-summary tests passed:

```text
test result: ok. 108 passed; 0 failed; 0 ignored; 0 measured; 90 filtered out
```

Focused example tests passed:

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Final broad gates passed:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo test --workspace --quiet
git diff --check
rg -n "$(printf '\342\200\224')" documentation crates || true
```

The full workspace suite reported:

```text
test result: ok. 191 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 198 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Interpretation

The postulate survived.

Saved repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict rollup drift verdict rollup JSON verifies against saved
source verdict records. The real archive matches, while a controlled stale
`records=1` rollup fails strict verification with `CountDrift` expected `2`
and actual `1`.

This is dashboard verifier summary verdict rollup check rollup drift verdict
cohort rollup drift verdict artifact cohort rollup drift verdict cohort rollup
drift check evidence, not allocator speed evidence.

## Next Question

Can saved repeated-check dashboard archive drift verdict rollup drift verdict
rollup drift verdict rollup drift verdict rollup drift reports emit compact
JSON verdicts so release dashboards can preserve typed rollup recheck cohort
verification outcomes?
