# Experiment 0221: Remote-Free Service Telemetry Shared Sample Filter

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0213-remote-free-service-telemetry-shared-sample-filter.md`

The postulate said that the remote-free service telemetry benchmark target
could share one filter-aware sample printing helper across harness modules so
focused Criterion runs suppress unrelated sample blocks target-wide while
preserving benchmark registration and allocation counters.

## Change

Added `remote_free_service/sample_filter.rs` and included it from the
`remote_free_service_telemetry` benchmark target. The helper parses Criterion
filter tokens once per benchmark process with `OnceLock`, then compares each
sample label and benchmark label with the active filters.

Migrated every remote-free service telemetry sample printer to call the shared
helper before running its pre-benchmark sample or sample-summary collection.
Benchmark registration and Criterion benchmark labels stayed unchanged.

## Commands

```text
cargo fmt --all --check
git diff --check
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo bench -p locus-alloc --bench remote_free_service_telemetry -- --list 2>&1 | rg "^remote_free_service_.*sample" | wc -l
cargo bench -p locus-alloc --bench remote_free_service_telemetry remote_free_service_runtime_dirty_local_buffer_group_validated_collection_sequence 2>&1 | rg "(^remote_free_service_.*sample|time:)"
cargo bench -p locus-alloc --bench remote_free_service_telemetry remote_free_service_runtime_apply_confirm 2>&1 | rg "(^remote_free_service_.*sample|time:)"
rg -n "$(printf '\342\200\224')" documentation crates || true
```

Full workspace clippy and test commands were also run after documentation was
updated:

```text
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Results

The benchmark target compiled successfully. `cargo fmt --all --check`,
`git diff --check`, and the no-em-dash scan passed. Workspace clippy and
workspace tests passed after the documentation update.

The unfiltered inventory path preserved the full target sample set:

```text
cargo bench -p locus-alloc --bench remote_free_service_telemetry -- --list 2>&1 | rg "^remote_free_service_.*sample" | wc -l
      60
```

The exact service-window validated local dirty-buffer group filter printed
only its matching sample and sample summary, then measured the matching
benchmark:

```text
remote_free_service_runtime_dirty_local_buffer_group_validated_collection_sample owners=3 windows=8 stable_windows=2 max_mutations=2 rollback_validation_bytes=8193 submitted_count=2048 drained_count=2048 released_bytes=9437440 policy_drains=12 drain_rounds=36 registered_owners=3 service_window_observations=8 observed_reports=64 reports_needing_retune=46 max_pending_over_target=192 max_queued_bytes_over_budget=786432 queue_backpressure_reports=12 hold_decisions=3 apply_decisions=2 confirmed_decisions=1 rollback_decisions=1 mutation_limit_decisions=1 runtime_install_count=2 runtime_confirm_count=1 runtime_rollback_count=1 runtime_no_change_outcomes=4 missing_owner_checks=1 max_wait_bursts=8 mean_wait_bursts=3.312 final_queue_capacity=256 final_previous_config_present=false final_guard_pending_candidate=none final_guard_applied_mutations=2 final_guard_confirmed_mutations=1 final_guard_rollbacks=1
remote_free_service_runtime_dirty_local_buffer_group_validated_collection_sample_summary owners=3 windows=8 samples=8 policy_drains_min=12 policy_drains_max=12 policy_drains_mean=12.000 drain_rounds_min=36 drain_rounds_max=36 drain_rounds_mean=36.000 reports_needing_retune_min=46 reports_needing_retune_max=46 reports_needing_retune_mean=46.000 apply_decisions_min=2 apply_decisions_max=2 apply_decisions_mean=2.000 confirmed_decisions_min=1 confirmed_decisions_max=1 confirmed_decisions_mean=1.000 rollback_decisions_min=1 rollback_decisions_max=1 rollback_decisions_mean=1.000 mutation_limit_decisions_min=1 mutation_limit_decisions_max=1 mutation_limit_decisions_mean=1.000 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=3.312 mean_wait_max=3.312 mean_wait_mean=3.312
time:   [197.39 us 197.57 us 197.77 us]
```

The exact runtime application confirm filter printed only its matching sample
and sample summary, then measured the matching benchmark:

```text
remote_free_service_runtime_apply_confirm_sample windows=3 initial_queue_capacity=128 installed_queue_capacity=256 final_queue_capacity=256 submitted_count=768 drained_count=768 released_bytes=3145728 policy_drains=12 drain_rounds=12 install_count=1 confirm_count=1 rollback_count=0 max_wait_bursts=2 mean_wait_bursts=1.500 final_previous_config_present=false
remote_free_service_runtime_apply_confirm_sample_summary windows=3 samples=8 policy_drains_min=12 policy_drains_max=12 policy_drains_mean=12.000 drain_rounds_min=12 drain_rounds_max=12 drain_rounds_mean=12.000 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
time:   [56.476 us 56.556 us 56.660 us]
```

## Interpretation

The postulate survived.

The shared helper now gives the whole `remote_free_service_telemetry` target
filter-clean focused output without losing unfiltered sample visibility. The
sample counters stayed unchanged for the exact validated local dirty-buffer
group path and for the runtime apply-confirm path. The helper also avoids
repeated argument-token vector allocation by caching parsed Criterion filters
once per benchmark process.

## Next Question

Can remote-free service telemetry samples be emitted as optional
machine-readable JSON lines so future experiments can compare counters and
timings without relying on ad hoc text filters?
