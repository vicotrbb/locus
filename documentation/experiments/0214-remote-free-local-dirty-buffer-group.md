# Experiment 0214: Remote-Free Local Dirty Buffer Group

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0206-remote-free-local-dirty-buffer-group.md`

The postulate said that a production-facing local dirty-buffer group can own
the shared dirty-owner tracker and reusable per-owner local buffers without
adding measurable overhead to the reused-buffer service-window path.

## Change

Added:

- `RemoteFreeServiceRuntimeDirtyOwnerLocalBuffers`;
- `RemoteFreeServiceRuntimeDirtyOwnerLocalMarker`;
- `remote_free_service_runtime_dirty_local_buffer_group_collection_sequence`;
- grow-only indexed local-buffer allocation for sparse owner IDs.

The helper owns one `RemoteFreeServiceRuntimeDirtyOwnerTracker` and a reusable
local buffer vector indexed by `RemoteFreeServiceRuntimeOwnerId`. It exposes a
hot-path local marker so enqueue loops can borrow one owner's local buffer once
and mark repeated successful enqueue attempts without indexing the buffer
group on every mark.

The focused tests cover one-owner-at-a-time flushing, retained local buffer
capacity after flush, sparse owner IDs, marker borrowing, and tracker
generation preservation after snapshot clearing.

## Commands

```text
cargo fmt --all --check
cargo test -p locus-alloc remote_free::dirty -- --nocapture
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo bench -p locus-alloc --bench remote_free_service_telemetry remote_free_service_runtime_dirty_local_buffer_group_collection_sequence -- --sample-size 10 --warm-up-time 0.1 --measurement-time 0.1
cargo bench -p locus-alloc --bench remote_free_service_telemetry remote_free_service_runtime_dirty_local_reused_collection_sequence -- --sample-size 10 --warm-up-time 0.1 --measurement-time 0.1
cargo bench -p locus-alloc --bench remote_free_service_telemetry remote_free_service_runtime_dirty_enqueue_collection_sequence -- --sample-size 10 --warm-up-time 0.1 --measurement-time 0.1
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Results

All validation commands passed. The focused run reported 16 dirty tests
passing. The workspace test run reported 178 `locus_alloc` unit tests, 1
integration test, 3 `locus_alloc` doctests, and the remaining workspace unit
and doctest targets passing. The benchmark target compiled successfully, and
the three short-run Criterion benchmarks completed successfully.

Runtime local buffer group collection sample:

```text
remote_free_service_runtime_dirty_local_buffer_group_collection_sample owners=3 windows=8 stable_windows=2 max_mutations=2 rollback_validation_bytes=8193 submitted_count=2048 drained_count=2048 released_bytes=9437440 policy_drains=12 drain_rounds=36 registered_owners=3 service_window_observations=8 observed_reports=64 reports_needing_retune=46 max_pending_over_target=192 max_queued_bytes_over_budget=786432 queue_backpressure_reports=12 hold_decisions=3 apply_decisions=2 confirmed_decisions=1 rollback_decisions=1 mutation_limit_decisions=1 runtime_install_count=2 runtime_confirm_count=1 runtime_rollback_count=1 runtime_no_change_outcomes=4 missing_owner_checks=1 max_wait_bursts=8 mean_wait_bursts=3.312 final_queue_capacity=256 final_previous_config_present=false final_guard_pending_candidate=none final_guard_applied_mutations=2 final_guard_confirmed_mutations=1 final_guard_rollbacks=1
remote_free_service_runtime_dirty_local_buffer_group_collection_sample_summary owners=3 windows=8 samples=8 policy_drains_min=12 policy_drains_max=12 policy_drains_mean=12.000 drain_rounds_min=36 drain_rounds_max=36 drain_rounds_mean=36.000 reports_needing_retune_min=46 reports_needing_retune_max=46 reports_needing_retune_mean=46.000 apply_decisions_min=2 apply_decisions_max=2 apply_decisions_mean=2.000 confirmed_decisions_min=1 confirmed_decisions_max=1 confirmed_decisions_mean=1.000 rollback_decisions_min=1 rollback_decisions_max=1 rollback_decisions_mean=1.000 mutation_limit_decisions_min=1 mutation_limit_decisions_max=1 mutation_limit_decisions_mean=1.000 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=3.312 mean_wait_max=3.312 mean_wait_mean=3.312
```

Short-run timing:

| Case | Time range |
| --- | ---: |
| `remote_free_service_runtime_dirty_local_reused_collection_sequence` | 196.61 to 197.42 us |
| `remote_free_service_runtime_dirty_local_buffer_group_collection_sequence` | 197.23 to 198.17 us |
| `remote_free_service_runtime_dirty_enqueue_collection_sequence` | 202.16 to 202.52 us |

## Interpretation

The correctness part of the postulate survived, but the strict performance
part needs refinement.

The helper preserved the real allocation counters: 2048 submitted blocks, 2048
drained blocks, 9,437,440 released bytes, 12 policy drains, 36 drain rounds,
max wait 8 bursts, and mean wait 3.312 bursts.

The helper also preserved the guarded service counters: two apply decisions,
one confirm, one rollback, one mutation-limit decision, four runtime
no-change outcomes, eight service-window observations, and one missing-owner
check.

The first helper implementation indexed the buffer group for every successful
enqueue mark and cloned the tracker during owner flush. The final
implementation added a borrowed local marker for the hot enqueue loop and
flushes owner buffers without cloning the tracker handle. That still measured
slightly slower than the benchmark-only reused-buffer path in the same
session, but it remained faster than direct dirty-enqueue tracker marking.

Treat `RemoteFreeServiceRuntimeDirtyOwnerLocalBuffers` as the production API
for the measured local-buffer lifecycle, but do not claim it has zero overhead
against hand-rolled buffer borrowing. Use `local_marker` for hot enqueue
paths. Avoid calling group-level `mark_dirty` inside tight successful-enqueue
loops unless a workload benchmark accepts the extra indexing.

## Next Question

Can tracked dirty collection be integrated with the local buffer group so the
service path avoids repeated caller-side tracker, local buffer, and collection
plumbing without adding measurable overhead?
