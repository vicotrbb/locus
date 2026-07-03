# Experiment 0210: Remote-Free Local Dirty Mark Buffer

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0202-remote-free-local-dirty-mark-buffer.md`

The postulate said that a small per-worker dirty-owner buffer can batch
successful enqueue marks before touching the shared dirty-owner tracker,
preserving dirty-owner collection correctness while reducing enqueue-side
shared synchronization in the measured service-window path.

## Change

Added:

- `RemoteFreeServiceRuntimeDirtyOwnerLocalBuffer`;
- `RemoteFreeServiceRuntimeDirtyOwnerFlushStats`;
- `remote_free_service_runtime_dirty_local_collection_sequence`.

The local buffer records unique owner IDs in first-marked order with a compact
Vec, tracks duplicate local marks, and flushes unique owners into
`RemoteFreeServiceRuntimeDirtyOwnerTracker`. The flush returns owner count,
new tracker marks, and duplicate local marks, then clears the buffer.

The benchmark harness now has a submit-loop success hook. The local dirty path
uses the normal `RemoteFreeSink::try_enqueue` path and marks the local buffer
only after a successful enqueue. Full-queue retries do not mark the buffer.
After each real allocation owner window, the benchmark flushes the buffer into
the shared tracker and collects through
`collect_tracked_dirty_service_window`.

## Commands

```text
cargo fmt --all --check
cargo test -p locus-alloc remote_free::dirty -- --nocapture
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo bench -p locus-alloc --bench remote_free_service_telemetry remote_free_service_runtime_dirty_local_collection_sequence -- --sample-size 10 --warm-up-time 0.1 --measurement-time 0.1
cargo bench -p locus-alloc --bench remote_free_service_telemetry remote_free_service_runtime_dirty_enqueue_collection_sequence -- --sample-size 10 --warm-up-time 0.1 --measurement-time 0.1
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Results

All validation commands passed. The focused run reported 12 dirty tests
passing, including 3 local-buffer tests and the 9 tracked dirty-window tests
from Experiment 0209. The workspace test run reported 174 `locus_alloc` unit
tests, 1 integration test, 3 `locus_alloc` doctests, and the remaining
workspace unit and doctest targets passing. The benchmark target compiled
successfully, and both short-run Criterion benchmarks completed successfully.

Runtime local dirty collection sample:

```text
remote_free_service_runtime_dirty_local_collection_sample owners=3 windows=8 stable_windows=2 max_mutations=2 rollback_validation_bytes=8193 submitted_count=2048 drained_count=2048 released_bytes=9437440 policy_drains=12 drain_rounds=36 registered_owners=3 service_window_observations=8 observed_reports=64 reports_needing_retune=46 max_pending_over_target=192 max_queued_bytes_over_budget=786432 queue_backpressure_reports=12 hold_decisions=3 apply_decisions=2 confirmed_decisions=1 rollback_decisions=1 mutation_limit_decisions=1 runtime_install_count=2 runtime_confirm_count=1 runtime_rollback_count=1 runtime_no_change_outcomes=4 missing_owner_checks=1 max_wait_bursts=8 mean_wait_bursts=3.312 final_queue_capacity=256 final_previous_config_present=false final_guard_pending_candidate=none final_guard_applied_mutations=2 final_guard_confirmed_mutations=1 final_guard_rollbacks=1
remote_free_service_runtime_dirty_local_collection_sample_summary owners=3 windows=8 samples=8 policy_drains_min=12 policy_drains_max=12 policy_drains_mean=12.000 drain_rounds_min=36 drain_rounds_max=36 drain_rounds_mean=36.000 reports_needing_retune_min=46 reports_needing_retune_max=46 reports_needing_retune_mean=46.000 apply_decisions_min=2 apply_decisions_max=2 apply_decisions_mean=2.000 confirmed_decisions_min=1 confirmed_decisions_max=1 confirmed_decisions_mean=1.000 rollback_decisions_min=1 rollback_decisions_max=1 rollback_decisions_mean=1.000 mutation_limit_decisions_min=1 mutation_limit_decisions_max=1 mutation_limit_decisions_mean=1.000 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=3.312 mean_wait_max=3.312 mean_wait_mean=3.312
```

Short-run timing:

| Case | Time range |
| --- | ---: |
| `remote_free_service_runtime_dirty_local_collection_sequence` | 198.83 to 199.35 us |
| `remote_free_service_runtime_dirty_enqueue_collection_sequence` | 209.44 to 212.01 us |

The timing comparison used sequential Criterion runs in the same session after
the Vec-only local buffer was installed. An earlier tree-backed local-buffer
prototype was rejected because it did not improve the path consistently and
was a poor fit for the tiny per-worker dirty-owner set.

## Interpretation

The postulate survived the focused tests and the real allocation benchmark.

The local dirty-buffer path preserved the direct dirty-enqueue allocation
counters: 2048 submitted blocks, 2048 drained blocks, 9,437,440 released bytes,
12 policy drains, 36 drain rounds, max wait 8 bursts, and mean wait 3.312
bursts.

The local buffer also preserved the guarded service counters: two apply
decisions, one confirm, one rollback, one mutation-limit decision, four
runtime no-change outcomes, eight service-window observations, and one
missing-owner check.

The benchmark asserts that each 256-submit owner window records one unique
local owner mark and 255 duplicate local marks before flushing one tracker
mark. That means the measured path reduces tracker touches from one per
successful enqueue to one per owner window in this workload shape.

## Next Question

What flush cadence should a live service use for local dirty buffers: end of
worker burst, end of scheduler turn, queue-pressure threshold, or immediately
before service-window collection?
