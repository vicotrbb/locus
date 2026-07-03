# Experiment 0215: Remote-Free Local Dirty Buffer Group Collection

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0207-remote-free-local-dirty-buffer-group-collection.md`

The postulate said that integrating local-buffer flush and tracked dirty
collection into one production-facing owner-registry method can remove
caller-side collection plumbing without changing correctness and without
measurable overhead versus the existing local dirty-buffer group path.

## Change

Added:

- `RemoteFreeServiceRuntimeLocalDirtyWindowStats`;
- `RemoteFreeServiceRuntimeRetuneOwners::collect_local_dirty_service_window`;
- `remote_free_service_runtime_dirty_local_buffer_group_integrated_collection_sequence`.

The integrated method accepts a mutable
`RemoteFreeServiceRuntimeDirtyOwnerLocalBuffers` and one owner ID. It rejects a
missing owner before touching the local buffer group, flushes that owner's
local buffer into the group's shared tracker, collects through the existing
tracked dirty service-window path, and returns both window stats and local
flush stats.

The first benchmark attempt exposed a real edge case: using `usize::MAX` as a
missing owner ID with a vector-indexed local buffer group can request an
impossible buffer resize. The integrated method now checks owner registration
first so invalid owner IDs return `MissingOwner` without local buffer
allocation.

## Commands

```text
cargo fmt --all --check
rg -n "—" documentation crates || true
git diff --check
cargo test -p locus-alloc remote_free::dirty -- --nocapture
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo bench -p locus-alloc --bench remote_free_service_telemetry remote_free_service_runtime_dirty_local_buffer_group
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Results

The focused validation passed after the missing-owner guard was added. The
dirty focused test run reported 21 tests passing. The benchmark target
compiled successfully. The first benchmark run failed after measuring the
existing local buffer group path because the integrated missing-owner harness
premarked `usize::MAX` and triggered `capacity overflow`. The corrected run
completed successfully.

The full workspace validation passed after the benchmark harness refactor. The
workspace test run reported 183 `locus_alloc` unit tests, 1 integration test,
3 `locus_alloc` doctests, and the remaining workspace unit and doctest targets
passing. Clippy passed for all workspace targets with warnings denied.

Existing local buffer group collection sample:

```text
remote_free_service_runtime_dirty_local_buffer_group_collection_sample owners=3 windows=8 stable_windows=2 max_mutations=2 rollback_validation_bytes=8193 submitted_count=2048 drained_count=2048 released_bytes=9437440 policy_drains=12 drain_rounds=36 registered_owners=3 service_window_observations=8 observed_reports=64 reports_needing_retune=46 max_pending_over_target=192 max_queued_bytes_over_budget=786432 queue_backpressure_reports=12 hold_decisions=3 apply_decisions=2 confirmed_decisions=1 rollback_decisions=1 mutation_limit_decisions=1 runtime_install_count=2 runtime_confirm_count=1 runtime_rollback_count=1 runtime_no_change_outcomes=4 missing_owner_checks=1 max_wait_bursts=8 mean_wait_bursts=3.312 final_queue_capacity=256 final_previous_config_present=false final_guard_pending_candidate=none final_guard_applied_mutations=2 final_guard_confirmed_mutations=1 final_guard_rollbacks=1
```

Integrated local buffer group collection sample:

```text
remote_free_service_runtime_dirty_local_buffer_group_integrated_collection_sample owners=3 windows=8 stable_windows=2 max_mutations=2 rollback_validation_bytes=8193 submitted_count=2048 drained_count=2048 released_bytes=9437440 policy_drains=12 drain_rounds=36 registered_owners=3 service_window_observations=8 observed_reports=64 reports_needing_retune=46 max_pending_over_target=192 max_queued_bytes_over_budget=786432 queue_backpressure_reports=12 hold_decisions=3 apply_decisions=2 confirmed_decisions=1 rollback_decisions=1 mutation_limit_decisions=1 runtime_install_count=2 runtime_confirm_count=1 runtime_rollback_count=1 runtime_no_change_outcomes=4 missing_owner_checks=1 max_wait_bursts=8 mean_wait_bursts=3.312 final_queue_capacity=256 final_previous_config_present=false final_guard_pending_candidate=none final_guard_applied_mutations=2 final_guard_confirmed_mutations=1 final_guard_rollbacks=1
```

Corrected-run timing:

| Case | Time range |
| --- | ---: |
| `remote_free_service_runtime_dirty_local_buffer_group_collection_sequence` | 196.69 to 197.23 us |
| `remote_free_service_runtime_dirty_local_buffer_group_integrated_collection_sequence` | 195.74 to 196.82 us |

## Interpretation

The postulate survived for this allocation sequence.

The integrated method preserved the real allocation counters: 2048 submitted
blocks, 2048 drained blocks, 9,437,440 released bytes, 12 policy drains, 36
drain rounds, max wait 8 bursts, and mean wait 3.312 bursts.

The integrated method also preserved the guarded service counters: two apply
decisions, one confirm, one rollback, one mutation-limit decision, four
runtime no-change outcomes, eight service-window observations, and one
missing-owner check.

The benchmark measured the integrated path slightly faster than the manual
local buffer group path in this run. The likely cause is less caller-side
tracker plumbing and assertion work in the benchmark path, not a new allocator
primitive. Treat the result as evidence that the integrated API does not add
measurable overhead for this workload, not as proof that it is intrinsically
faster.

The missing-owner overflow discovery is the stronger correctness result. Any
API that combines owner validation with vector-indexed local buffer access
must validate registration before resizing the local buffer group for an owner
ID that might not have come from the registry.

## Next Question

Can the local dirty-buffer group provide a bounded or fallible path for direct
local marking so invalid or extremely sparse owner IDs cannot trigger large
buffer growth outside the integrated collection method?
