# Experiment 0211: Remote-Free Local Dirty Flush Cadence

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0203-remote-free-local-dirty-flush-cadence.md`

The postulate said that flushing a local dirty-owner buffer once per worker
burst can preserve tracked dirty-owner correctness while keeping most of the
enqueue-side synchronization reduction from local buffering.

## Change

Added:

- a shared submit-loop event hook for successful enqueue and burst completion;
- `RuntimeLocalDirtyFlushStats` in the benchmark harness;
- `remote_free_service_runtime_dirty_local_burst_collection_sequence`.

The burst-flush path still records local dirty marks only after successful
`try_enqueue` calls. It flushes the local buffer into the shared
`RemoteFreeServiceRuntimeDirtyOwnerTracker` after each worker burst, then
collects through `collect_tracked_dirty_service_window`.

The benchmark asserts that each owner window performs 8 non-empty flushes,
flushes 8 unique owner marks, records 1 newly pending tracker mark, and
deduplicates 248 local duplicate marks. Later burst flushes increment the
tracker generation for an already pending owner instead of adding duplicate
pending owners.

## Commands

```text
cargo fmt --all --check
cargo test -p locus-alloc remote_free::dirty -- --nocapture
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo bench -p locus-alloc --bench remote_free_service_telemetry remote_free_service_runtime_dirty_local_burst_collection_sequence -- --sample-size 10 --warm-up-time 0.1 --measurement-time 0.1
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

Runtime local burst collection sample:

```text
remote_free_service_runtime_dirty_local_burst_collection_sample owners=3 windows=8 stable_windows=2 max_mutations=2 rollback_validation_bytes=8193 submitted_count=2048 drained_count=2048 released_bytes=9437440 policy_drains=12 drain_rounds=36 registered_owners=3 service_window_observations=8 observed_reports=64 reports_needing_retune=46 max_pending_over_target=192 max_queued_bytes_over_budget=786432 queue_backpressure_reports=12 hold_decisions=3 apply_decisions=2 confirmed_decisions=1 rollback_decisions=1 mutation_limit_decisions=1 runtime_install_count=2 runtime_confirm_count=1 runtime_rollback_count=1 runtime_no_change_outcomes=4 missing_owner_checks=1 max_wait_bursts=8 mean_wait_bursts=3.312 final_queue_capacity=256 final_previous_config_present=false final_guard_pending_candidate=none final_guard_applied_mutations=2 final_guard_confirmed_mutations=1 final_guard_rollbacks=1
remote_free_service_runtime_dirty_local_burst_collection_sample_summary owners=3 windows=8 samples=8 policy_drains_min=12 policy_drains_max=12 policy_drains_mean=12.000 drain_rounds_min=36 drain_rounds_max=36 drain_rounds_mean=36.000 reports_needing_retune_min=46 reports_needing_retune_max=46 reports_needing_retune_mean=46.000 apply_decisions_min=2 apply_decisions_max=2 apply_decisions_mean=2.000 confirmed_decisions_min=1 confirmed_decisions_max=1 confirmed_decisions_mean=1.000 rollback_decisions_min=1 rollback_decisions_max=1 rollback_decisions_mean=1.000 mutation_limit_decisions_min=1 mutation_limit_decisions_max=1 mutation_limit_decisions_mean=1.000 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=3.312 mean_wait_max=3.312 mean_wait_mean=3.312
```

Short-run timing:

| Case | Time range |
| --- | ---: |
| `remote_free_service_runtime_dirty_local_collection_sequence` | 199.29 to 200.68 us |
| `remote_free_service_runtime_dirty_enqueue_collection_sequence` | 201.32 to 203.72 us |
| `remote_free_service_runtime_dirty_local_burst_collection_sequence` | 204.32 to 207.50 us |

## Interpretation

The correctness part of the postulate survived, but the performance part did
not survive this workload shape.

Burst flushing preserved the real allocation counters: 2048 submitted blocks,
2048 drained blocks, 9,437,440 released bytes, 12 policy drains, 36 drain
rounds, max wait 8 bursts, and mean wait 3.312 bursts.

Burst flushing also preserved the guarded service counters: two apply
decisions, one confirm, one rollback, one mutation-limit decision, four
runtime no-change outcomes, eight service-window observations, and one
missing-owner check.

However, burst flushing was slower than both before-collection local flushing
and direct dirty-enqueue tracker marking in the sequential short benchmark.
For the current 8-burst, 32-block owner-window shape, paying 8 tracker flushes
per owner window is not the best candidate.

Treat the postulate as failed for this workload shape. Keep
before-collection local flushing as the measured default for worker-owned
enqueue loops until a lower-latency flush trigger is justified by a workload
that needs earlier service visibility.

## Next Question

Can a pressure-triggered or service-demand-triggered flush provide earlier
dirty-owner visibility without paying the fixed cost of one tracker flush per
burst?
