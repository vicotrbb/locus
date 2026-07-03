# Experiment 0216: Remote-Free Bounded Local Dirty Buffer Group Marking

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0208-remote-free-bounded-local-dirty-buffer-group-marking.md`

The postulate said that a bounded local dirty-buffer group marking API can
reject invalid or extremely sparse owner IDs before vector growth while
preserving the measured local dirty-buffer lifecycle for validated service
loops.

## Change

Added:

- `RemoteFreeServiceRuntimeDirtyOwnerLocalBufferGroupError`;
- `RemoteFreeServiceRuntimeDirtyOwnerLocalBuffers::try_mark_dirty`;
- `RemoteFreeServiceRuntimeDirtyOwnerLocalBuffers::try_local_marker`;
- public error accessors for the rejected owner ID and owner limit;
- `remote_free_service_runtime_dirty_local_buffer_group_bounded_collection_sequence`.

The bounded methods accept an owner ID and an exclusive owner-index limit. If
the owner index is greater than or equal to that limit, they return
`OwnerOutOfRange` before allocating or resizing local buffer storage. The
existing `mark_dirty` and `local_marker` methods remain available for trusted
hot paths where owner IDs have already been validated by the registry.

## Commands

```text
cargo fmt --all --check
rg -n "$(printf '\342\200\224')" documentation crates || true
git diff --check
cargo test -p locus-alloc remote_free::dirty -- --nocapture
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo bench -p locus-alloc --bench remote_free_service_telemetry remote_free_service_runtime_dirty_local_buffer_group
cargo bench -p locus-alloc --bench remote_free_service_telemetry remote_free_service_runtime_dirty_local_buffer_group
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Results

The focused validation passed. The dirty focused test run reported 25 tests
passing. The benchmark target compiled successfully. The final workspace test
run reported 187 `locus_alloc` unit tests, 1 integration test, 3
`locus_alloc` doctests, and the remaining workspace unit and doctest targets
passing. Clippy passed for all workspace targets with warnings denied.

The new tests covered:

- rejecting `usize::MAX` in `try_mark_dirty` without local buffer allocation;
- rejecting a finite out-of-range owner in `try_local_marker` without local
  buffer allocation;
- accepted bounded one-shot marking preserving duplicate counts;
- accepted bounded marker borrowing preserving duplicate counts;
- public error accessors and display text.

Bounded local buffer group collection sample:

```text
remote_free_service_runtime_dirty_local_buffer_group_bounded_collection_sample owners=3 windows=8 stable_windows=2 max_mutations=2 rollback_validation_bytes=8193 submitted_count=2048 drained_count=2048 released_bytes=9437440 policy_drains=12 drain_rounds=36 registered_owners=3 service_window_observations=8 observed_reports=64 reports_needing_retune=46 max_pending_over_target=192 max_queued_bytes_over_budget=786432 queue_backpressure_reports=12 hold_decisions=3 apply_decisions=2 confirmed_decisions=1 rollback_decisions=1 mutation_limit_decisions=1 runtime_install_count=2 runtime_confirm_count=1 runtime_rollback_count=1 runtime_no_change_outcomes=4 missing_owner_checks=1 max_wait_bursts=8 mean_wait_bursts=3.312 final_queue_capacity=256 final_previous_config_present=false final_guard_pending_candidate=none final_guard_applied_mutations=2 final_guard_confirmed_mutations=1 final_guard_rollbacks=1
remote_free_service_runtime_dirty_local_buffer_group_bounded_collection_sample_summary owners=3 windows=8 samples=8 policy_drains_min=12 policy_drains_max=12 policy_drains_mean=12.000 drain_rounds_min=36 drain_rounds_max=36 drain_rounds_mean=36.000 reports_needing_retune_min=46 reports_needing_retune_max=46 reports_needing_retune_mean=46.000 apply_decisions_min=2 apply_decisions_max=2 apply_decisions_mean=2.000 confirmed_decisions_min=1 confirmed_decisions_max=1 confirmed_decisions_mean=1.000 rollback_decisions_min=1 rollback_decisions_max=1 rollback_decisions_mean=1.000 mutation_limit_decisions_min=1 mutation_limit_decisions_max=1 mutation_limit_decisions_mean=1.000 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=3.312 mean_wait_max=3.312 mean_wait_mean=3.312
```

First timing run:

| Case | Time range |
| --- | ---: |
| `remote_free_service_runtime_dirty_local_buffer_group_collection_sequence` | 209.58 to 220.43 us |
| `remote_free_service_runtime_dirty_local_buffer_group_integrated_collection_sequence` | 199.19 to 203.94 us |
| `remote_free_service_runtime_dirty_local_buffer_group_bounded_collection_sequence` | 196.85 to 197.30 us |

Second timing run:

| Case | Time range |
| --- | ---: |
| `remote_free_service_runtime_dirty_local_buffer_group_collection_sequence` | 200.27 to 201.48 us |
| `remote_free_service_runtime_dirty_local_buffer_group_integrated_collection_sequence` | 201.08 to 202.18 us |
| `remote_free_service_runtime_dirty_local_buffer_group_bounded_collection_sequence` | 196.78 to 197.18 us |

## Interpretation

The correctness part of the postulate survived.

Rejected owner IDs did not grow the local buffer group. Accepted owner IDs
preserved local duplicate counters, flush counters, tracked collection
semantics, and the full service-window decision sequence.

The benchmark runs were noisy for the manual and integrated group baselines,
but the bounded path stayed near 197 us across both runs while preserving the
same counters. The bounded marker check happens once before the enqueue loop,
not on every successful enqueue, so it did not add measurable overhead in this
allocation sequence.

Treat `try_mark_dirty` and `try_local_marker` as the default choice when the
caller has a current owner count available or the owner ID may be stale,
external, or otherwise untrusted. Keep `local_marker` for the tightest trusted
hot path after owner IDs have already been validated.

## Next Question

Can owner registration expose a small reusable validated local dirty marker
handle so call sites do not need to pass owner limits manually?
