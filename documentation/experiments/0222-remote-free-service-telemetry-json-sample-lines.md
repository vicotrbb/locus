# Experiment 0222: Remote-Free Service Telemetry JSON Sample Lines

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0214-remote-free-service-telemetry-json-sample-lines.md`

The postulate said that the remote-free service telemetry benchmark target
could emit optional machine-readable JSON sample lines from the same sample
rows used for human benchmark review, while preserving default text output,
Criterion benchmark registration, and allocation counters.

## Change

Added `remote_free_service/sample_output.rs` and included it from the
`remote_free_service_telemetry` benchmark target. Every sample printer now
routes its existing text row through `print_sample_line`.

By default, `print_sample_line` prints the same text row as before. When
`LOCUS_REMOTE_FREE_SERVICE_TELEMETRY_JSON` is set to `1`, `true`, `yes`, `on`,
or `json`, it also emits one JSON object beside each text sample row. The JSON
object includes:

- `schema`;
- `benchmark`;
- `sample`;
- the original text `line`;
- parsed `fields` from the row's `key=value` tokens.

The helper parses boolean, integer, and decimal values into JSON scalars, and
escapes JSON strings without adding a new crate dependency.

## Commands

```text
cargo bench -p locus-alloc --bench remote_free_service_telemetry remote_free_service_runtime_apply_confirm 2>&1 | rg "(^\\{|^remote_free_service_.*sample|time:)"
LOCUS_REMOTE_FREE_SERVICE_TELEMETRY_JSON=1 cargo bench -p locus-alloc --bench remote_free_service_telemetry remote_free_service_runtime_apply_confirm 2>&1 | rg "(^\\{|^remote_free_service_.*sample|remote_free_service_runtime_apply_confirm|time:)"
LOCUS_REMOTE_FREE_SERVICE_TELEMETRY_JSON=1 cargo bench -p locus-alloc --bench remote_free_service_telemetry remote_free_service_runtime_apply_confirm 2>&1 | python3 -c '...json parser and counter assertions...'
LOCUS_REMOTE_FREE_SERVICE_TELEMETRY_JSON=1 cargo bench -p locus-alloc --bench remote_free_service_telemetry remote_free_service_runtime_dirty_local_buffer_group_validated_collection_sequence 2>&1 | python3 -c '...json parser and counter assertions...'
LOCUS_REMOTE_FREE_SERVICE_TELEMETRY_JSON=1 cargo bench -p locus-alloc --bench remote_free_service_telemetry -- --list 2>&1 | python3 -c '...json parser and row-count assertions...'
LOCUS_REMOTE_FREE_SERVICE_TELEMETRY_JSON=1 cargo bench -p locus-alloc --bench remote_free_service_telemetry remote_free_service_runtime_dirty_local_buffer_group_validated_collection_sequence 2>&1 | rg "(^\\{|^remote_free_service_.*sample|time:)"
cargo fmt --all --check
git diff --check
rg -n "$(printf '\342\200\224')" documentation crates || true
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Results

Default focused output stayed text-only for the runtime apply-confirm path:

```text
remote_free_service_runtime_apply_confirm_sample windows=3 initial_queue_capacity=128 installed_queue_capacity=256 final_queue_capacity=256 submitted_count=768 drained_count=768 released_bytes=3145728 policy_drains=12 drain_rounds=12 install_count=1 confirm_count=1 rollback_count=0 max_wait_bursts=2 mean_wait_bursts=1.500 final_previous_config_present=false
remote_free_service_runtime_apply_confirm_sample_summary windows=3 samples=8 policy_drains_min=12 policy_drains_max=12 policy_drains_mean=12.000 drain_rounds_min=12 drain_rounds_max=12 drain_rounds_mean=12.000 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
time:   [57.853 us 57.887 us 57.928 us]
```

With JSON enabled, the runtime apply-confirm parser assertion passed:

```text
json_rows=2 text_rows=2 benchmark=remote_free_service_runtime_apply_confirm submitted=768 released_bytes=3145728
```

The JSON rows preserved typed fields for 768 submitted blocks, 768 drained
blocks, 3,145,728 released bytes, one confirm, zero rollbacks, and
`final_previous_config_present=false`. The JSON-enabled timing run measured:

```text
time:   [57.706 us 57.752 us 57.798 us]
```

With JSON enabled, the exact validated local dirty-buffer group parser
assertion passed:

```text
json_rows=2 text_rows=2 benchmark=validated_collection submitted=2048 released_bytes=9437440
```

The JSON rows preserved typed fields for 2048 submitted blocks, 2048 drained
blocks, 9,437,440 released bytes, three registered owners, 46 reports needing
retune, and `final_previous_config_present=false`. The JSON-enabled timing
run measured:

```text
time:   [195.46 us 195.69 us 195.93 us]
```

The unfiltered inventory path produced one JSON row for every text sample row:

```text
text_rows=60 json_rows=60
```

The first clippy run found two helper-style issues: `map(...).unwrap_or(false)`
on the environment variable result, and `format!` appended to an existing
`String` for control-character escapes. Both were fixed with `is_ok_and` and
`write!`. The final clippy run passed with warnings denied. Workspace tests
passed after the fix.

## Interpretation

The postulate survived.

The benchmark target now has optional JSON sample lines without changing
default output. Focused filters still emit only the selected sample and sample
summary, and the JSON rows expose counters as typed fields that future scripts
can compare without parsing the human text row.

The output helper remains benchmark scoped, so production crates gain no JSON
dependency or runtime path.

## Next Question

Can the JSON sample rows be consumed by a small repository script that compares
two benchmark runs and reports counter drift before timing deltas are trusted?
