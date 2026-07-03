# Experiment 0212: Remote-Free Local Dirty Threshold Flush

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0204-remote-free-local-dirty-threshold-flush.md`

The postulate said that flushing a local dirty-owner buffer after the worker
accumulates the configured target pending item window can provide earlier
dirty-owner visibility than before-collection flushing while avoiding the
fixed cost of flushing once per burst.

## Change

Added:

- a threshold-flush owner-window helper for the benchmark harness;
- `remote_free_service_runtime_dirty_local_threshold_collection_sequence`;
- strict threshold flush assertions in the service-window benchmark.

The threshold path uses the normal runtime sink and records local dirty marks
only after successful `try_enqueue` calls. It flushes the local buffer into the
shared `RemoteFreeServiceRuntimeDirtyOwnerTracker` when local successful marks
reach `TARGET_PENDING_BLOCKS`, then flushes any remaining marks before
tracked dirty service-window collection.

For the current 2048-submit benchmark shape with a 64-item target pending
window, each owner window performs 32 non-empty local flushes, records 32 owner
flush observations, records 1 newly pending tracker mark, and deduplicates
2016 local duplicate marks.

## Commands

```text
cargo fmt --all --check
cargo test -p locus-alloc remote_free::dirty -- --nocapture
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo bench -p locus-alloc --bench remote_free_service_telemetry remote_free_service_runtime_dirty_local_threshold_collection_sequence -- --sample-size 10 --warm-up-time 0.1 --measurement-time 0.1
cargo bench -p locus-alloc --bench remote_free_service_telemetry remote_free_service_runtime_dirty_enqueue_collection_sequence -- --sample-size 10 --warm-up-time 0.1 --measurement-time 0.1
cargo bench -p locus-alloc --bench remote_free_service_telemetry remote_free_service_runtime_dirty_local_collection_sequence -- --sample-size 10 --warm-up-time 0.1 --measurement-time 0.1
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Results

All validation commands passed. The focused run reported 12 dirty tests
passing. The workspace test run reported 174 `locus_alloc` unit tests, 1
integration test, 3 `locus_alloc` doctests, and the remaining workspace unit
and doctest targets passing. The benchmark target compiled successfully, and
the three short-run Criterion benchmarks completed successfully.

Runtime local threshold collection sample:

```text
remote_free_service_runtime_dirty_local_threshold_collection_sample owners=3 windows=8 stable_windows=2 max_mutations=2 rollback_validation_bytes=8193 submitted_count=2048 drained_count=2048 released_bytes=9437440 policy_drains=12 drain_rounds=36 registered_owners=3 service_window_observations=8 observed_reports=64 reports_needing_retune=46 max_pending_over_target=192 max_queued_bytes_over_budget=786432 queue_backpressure_reports=12 hold_decisions=3 apply_decisions=2 confirmed_decisions=1 rollback_decisions=1 mutation_limit_decisions=1 runtime_install_count=2 runtime_confirm_count=1 runtime_rollback_count=1 runtime_no_change_outcomes=4 missing_owner_checks=1 max_wait_bursts=8 mean_wait_bursts=3.312 final_queue_capacity=256 final_previous_config_present=false final_guard_pending_candidate=none final_guard_applied_mutations=2 final_guard_confirmed_mutations=1 final_guard_rollbacks=1
remote_free_service_runtime_dirty_local_threshold_collection_sample_summary owners=3 windows=8 samples=8 policy_drains_min=12 policy_drains_max=12 policy_drains_mean=12.000 drain_rounds_min=36 drain_rounds_max=36 drain_rounds_mean=36.000 reports_needing_retune_min=46 reports_needing_retune_max=46 reports_needing_retune_mean=46.000 apply_decisions_min=2 apply_decisions_max=2 apply_decisions_mean=2.000 confirmed_decisions_min=1 confirmed_decisions_max=1 confirmed_decisions_mean=1.000 rollback_decisions_min=1 rollback_decisions_max=1 rollback_decisions_mean=1.000 mutation_limit_decisions_min=1 mutation_limit_decisions_max=1 mutation_limit_decisions_mean=1.000 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=3.312 mean_wait_max=3.312 mean_wait_mean=3.312
```

Short-run timing:

| Case | Time range |
| --- | ---: |
| `remote_free_service_runtime_dirty_local_collection_sequence` | 197.29 to 198.02 us |
| `remote_free_service_runtime_dirty_enqueue_collection_sequence` | 203.45 to 205.38 us |
| `remote_free_service_runtime_dirty_local_threshold_collection_sequence` | 200.36 to 205.87 us |

## Interpretation

The correctness part of the postulate survived, but the performance part did
not clearly survive this workload shape.

Threshold flushing preserved the real allocation counters: 2048 submitted
blocks, 2048 drained blocks, 9,437,440 released bytes, 12 policy drains, 36
drain rounds, max wait 8 bursts, and mean wait 3.312 bursts.

Threshold flushing also preserved the guarded service counters: two apply
decisions, one confirm, one rollback, one mutation-limit decision, four
runtime no-change outcomes, eight service-window observations, and one
missing-owner check.

However, threshold flushing was slower than before-collection local flushing
and did not produce a clean win over direct dirty-enqueue tracker marking in
the sequential short benchmark. Its range overlapped the direct tracker path
and had a slightly worse high end.

Treat the postulate as useful for correctness and earlier visibility, but not
as a default performance improvement for this owner-window shape. Keep
before-collection local flushing as the measured default when service
visibility can wait until collection. Revisit threshold flushing only for a
workload where earlier dirty-owner visibility is required and the added flush
cadence survives a same-session allocation benchmark.

## Next Question

Can a service-demand-triggered local dirty-buffer flush fire only when the
collector is about to wait on dirty-owner visibility, instead of using a fixed
burst or retained-window cadence?
