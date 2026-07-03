# Experiment 0208: Remote-Free Dirty Owner Window Collection

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0200-remote-free-dirty-owner-window-collection.md`

The postulate said that an ordered dirty-owner set can avoid scanning every
registered remote-free owner on each service window while preserving the same
guarded retune behavior as explicit owner collection.

## Change

Added `RemoteFreeServiceRuntimeDirtyOwners` and
`RemoteFreeServiceRuntimeRetuneOwners::collect_dirty_service_window`.

The dirty-owner set records owner IDs in first-marked order, deduplicates
repeated marks, exposes pending dirty owner IDs, and clears only after a
successful collection window. If an owner is missing, summary collection fails,
or service-window routing fails, dirty marks remain for caller inspection or
retry.

Extended the service-window benchmark harness with
`remote_free_service_runtime_dirty_window_collection_sequence`. It runs the
same real allocation sequence as Experiments 0206 and 0207, but each measured
window marks the active owner dirty and collects through
`collect_dirty_service_window`.

## Commands

```text
cargo fmt --all --check
cargo test -p locus-alloc remote_free::dirty_window -- --nocapture
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo bench -p locus-alloc --bench remote_free_service_telemetry remote_free_service_runtime_dirty_window_collection_sequence -- --sample-size 10 --warm-up-time 0.1 --measurement-time 0.1
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Results

All validation commands passed. The focused dirty-window test run reported 4
dirty-window tests passing. The workspace test run reported 280 unit tests, 1
integration test, and 3 `locus_alloc` doctests passing. Additional doctest
targets with zero tests also completed successfully. The benchmark target
compiled successfully, and the short-run Criterion benchmark completed
successfully.

Runtime dirty-window collection sample:

```text
remote_free_service_runtime_dirty_window_collection_sample owners=3 windows=8 stable_windows=2 max_mutations=2 rollback_validation_bytes=8193 submitted_count=2048 drained_count=2048 released_bytes=9437440 policy_drains=12 drain_rounds=36 registered_owners=3 service_window_observations=8 observed_reports=64 reports_needing_retune=46 max_pending_over_target=192 max_queued_bytes_over_budget=786432 queue_backpressure_reports=12 hold_decisions=3 apply_decisions=2 confirmed_decisions=1 rollback_decisions=1 mutation_limit_decisions=1 runtime_install_count=2 runtime_confirm_count=1 runtime_rollback_count=1 runtime_no_change_outcomes=4 missing_owner_checks=1 max_wait_bursts=8 mean_wait_bursts=3.312 final_queue_capacity=256 final_previous_config_present=false final_guard_pending_candidate=none final_guard_applied_mutations=2 final_guard_confirmed_mutations=1 final_guard_rollbacks=1
remote_free_service_runtime_dirty_window_collection_sample_summary owners=3 windows=8 samples=8 policy_drains_min=12 policy_drains_max=12 policy_drains_mean=12.000 drain_rounds_min=36 drain_rounds_max=36 drain_rounds_mean=36.000 reports_needing_retune_min=46 reports_needing_retune_max=46 reports_needing_retune_mean=46.000 apply_decisions_min=2 apply_decisions_max=2 apply_decisions_mean=2.000 confirmed_decisions_min=1 confirmed_decisions_max=1 confirmed_decisions_mean=1.000 rollback_decisions_min=1 rollback_decisions_max=1 rollback_decisions_mean=1.000 mutation_limit_decisions_min=1 mutation_limit_decisions_max=1 mutation_limit_decisions_mean=1.000 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=3.312 mean_wait_max=3.312 mean_wait_mean=3.312
```

Short-run timing:

| Case | Time range |
| --- | ---: |
| `remote_free_service_runtime_dirty_window_collection_sequence` | 194.30 to 194.85 us |

## Interpretation

The postulate survived the focused test and benchmark pass.

The dirty-owner helper preserved the full collection allocation counters: 2048
submitted blocks, 2048 drained blocks, 9,437,440 released bytes, 12 policy
drains, 36 drain rounds, max wait 8 bursts, and mean wait 3.312 bursts.

The helper also preserved guarded orchestration counters: two installs, one
confirm, one rollback, one mutation-limit decision, four no-change outcomes,
eight service-window observations, and one explicit missing-owner check.

Focused tests verified the dirty-owner invariants that the benchmark does not
fully expose: repeated marks are deduplicated, successful collection clears
marks, missing-owner errors preserve marks, and collector errors preserve
marks.

## Next Question

Where should dirty-owner marks be emitted in a live runtime: directly from
remote enqueue handles, from owner control-loop drain observations, or from a
separate service scheduler event queue?
