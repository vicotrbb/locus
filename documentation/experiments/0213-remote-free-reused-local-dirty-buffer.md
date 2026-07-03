# Experiment 0213: Remote-Free Reused Local Dirty Buffer

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0205-remote-free-reused-local-dirty-buffer.md`

The postulate said that keeping worker-local dirty-owner buffers alive across
service windows can preserve before-collection flush correctness while
avoiding per-window buffer allocation churn in the measured runtime
service-window sequence.

## Change

Added:

- `RemoteFreeServiceRuntimeDirtyOwnerLocalBuffer::capacity`;
- `remote_free_service_runtime_dirty_local_reused_collection_sequence`;
- a benchmark-only reused local dirty state with one shared dirty-owner tracker
  and per-owner local buffers across the whole service-window sequence.

The reused-buffer path marks local dirty owners only after successful
`try_enqueue` calls. It flushes the relevant local buffer into the shared
`RemoteFreeServiceRuntimeDirtyOwnerTracker` immediately before tracked dirty
service-window collection. After each flush, the benchmark asserts that the
local buffer retains capacity for reuse.

Across the full sequence, the benchmark asserts 8 non-empty local flushes, 8
owner flush observations, 8 newly pending tracker marks, and 2040 duplicate
local marks. The shared tracker is empty after the final collection.

## Commands

```text
cargo fmt --all --check
cargo test -p locus-alloc remote_free::dirty -- --nocapture
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo bench -p locus-alloc --bench remote_free_service_telemetry remote_free_service_runtime_dirty_local_reused_collection_sequence -- --sample-size 10 --warm-up-time 0.1 --measurement-time 0.1
cargo bench -p locus-alloc --bench remote_free_service_telemetry remote_free_service_runtime_dirty_local_collection_sequence -- --sample-size 10 --warm-up-time 0.1 --measurement-time 0.1
cargo bench -p locus-alloc --bench remote_free_service_telemetry remote_free_service_runtime_dirty_enqueue_collection_sequence -- --sample-size 10 --warm-up-time 0.1 --measurement-time 0.1
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Results

All validation commands passed. The focused run reported 12 dirty tests
passing. The workspace test run reported 174 `locus_alloc` unit tests, 1
integration test, 3 `locus_alloc` doctests, and the remaining workspace unit
and doctest targets passing. The benchmark target compiled successfully, and
the three short-run Criterion benchmarks completed successfully.

Runtime local reused collection sample:

```text
remote_free_service_runtime_dirty_local_reused_collection_sample owners=3 windows=8 stable_windows=2 max_mutations=2 rollback_validation_bytes=8193 submitted_count=2048 drained_count=2048 released_bytes=9437440 policy_drains=12 drain_rounds=36 registered_owners=3 service_window_observations=8 observed_reports=64 reports_needing_retune=46 max_pending_over_target=192 max_queued_bytes_over_budget=786432 queue_backpressure_reports=12 hold_decisions=3 apply_decisions=2 confirmed_decisions=1 rollback_decisions=1 mutation_limit_decisions=1 runtime_install_count=2 runtime_confirm_count=1 runtime_rollback_count=1 runtime_no_change_outcomes=4 missing_owner_checks=1 max_wait_bursts=8 mean_wait_bursts=3.312 final_queue_capacity=256 final_previous_config_present=false final_guard_pending_candidate=none final_guard_applied_mutations=2 final_guard_confirmed_mutations=1 final_guard_rollbacks=1
remote_free_service_runtime_dirty_local_reused_collection_sample_summary owners=3 windows=8 samples=8 policy_drains_min=12 policy_drains_max=12 policy_drains_mean=12.000 drain_rounds_min=36 drain_rounds_max=36 drain_rounds_mean=36.000 reports_needing_retune_min=46 reports_needing_retune_max=46 reports_needing_retune_mean=46.000 apply_decisions_min=2 apply_decisions_max=2 apply_decisions_mean=2.000 confirmed_decisions_min=1 confirmed_decisions_max=1 confirmed_decisions_mean=1.000 rollback_decisions_min=1 rollback_decisions_max=1 rollback_decisions_mean=1.000 mutation_limit_decisions_min=1 mutation_limit_decisions_max=1 mutation_limit_decisions_mean=1.000 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=3.312 mean_wait_max=3.312 mean_wait_mean=3.312
```

Short-run timing:

| Case | Time range |
| --- | ---: |
| `remote_free_service_runtime_dirty_local_reused_collection_sequence` | 195.12 to 196.07 us |
| `remote_free_service_runtime_dirty_local_collection_sequence` | 198.76 to 201.50 us |
| `remote_free_service_runtime_dirty_enqueue_collection_sequence` | 205.31 to 206.05 us |

## Interpretation

The postulate survived this workload shape.

Reused local dirty buffers preserved the real allocation counters: 2048
submitted blocks, 2048 drained blocks, 9,437,440 released bytes, 12 policy
drains, 36 drain rounds, max wait 8 bursts, and mean wait 3.312 bursts.

Reused local dirty buffers also preserved the guarded service counters: two
apply decisions, one confirm, one rollback, one mutation-limit decision, four
runtime no-change outcomes, eight service-window observations, and one
missing-owner check.

The benchmark kept one shared tracker for the whole service-window sequence
and retained per-owner local buffer capacity after each flush. That matched
the intended production worker lifecycle more closely than recreating a fresh
buffer per collected owner window.

Treat long-lived worker-local dirty buffers with service-demand flushing as
the current measured candidate for worker-owned enqueue loops. Do not infer
that earlier flush cadence is needed from this result. The win came from local
buffer lifecycle and capacity reuse while keeping one tracker flush per
collected owner window.

## Next Question

Can a production-facing helper own the shared dirty tracker and worker-local
buffers without exposing callers to per-window buffer lifecycle details?
