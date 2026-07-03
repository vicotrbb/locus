# Experiment 0218: Remote-Free Local Dirty Group Benchmark Helper

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0210-remote-free-local-dirty-group-benchmark-helper.md`

The postulate said that the service-window benchmark could factor repeated
local dirty-buffer group collection assertions into a dedicated helper module
without changing measured allocation counters, dirty flush counters, or
missing-owner guard behavior.

## Change

Added `runtime_local_dirty_group_harness.rs` for the local dirty-buffer group
benchmark paths.

The helper owns:

- `RuntimeLocalDirtyGroupCollectionMode`;
- shared collection for manual, integrated, bounded, and validated group modes;
- per-window duplicate mark assertions;
- per-window flush counter assertions;
- local buffer capacity reuse assertions;
- tracker-empty assertions after successful collection;
- integrated, bounded, and validated missing-owner checks.

The main service-window harness now routes group modes through the helper
instead of carrying four repeated collection functions.

## Commands

```text
cargo fmt --all
cargo fmt --all --check
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo bench -p locus-alloc --bench remote_free_service_telemetry remote_free_service_runtime_dirty_local_buffer_group
git show HEAD:crates/locus-alloc/benches/remote_free_service/runtime_service_window_harness.rs | wc -l
wc -l crates/locus-alloc/benches/remote_free_service/runtime_service_window_harness.rs crates/locus-alloc/benches/remote_free_service/runtime_local_dirty_group_harness.rs
rg -n "$(printf '\342\200\224')" documentation crates || true
git diff --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Results

The benchmark target compiled successfully after the extraction. The
no-em-dash check returned no matches. `git diff --check` passed. Workspace
clippy passed with warnings denied after replacing a dynamic `expect` message
with an explicit panic closure and reducing `observe_window` back below the
line-count lint threshold. The workspace test run passed, including 191
`locus_alloc` unit tests, one `locus_alloc` integration test, 13 `locus_core`
unit tests, 34 `locus_observe` unit tests, six `locus_sys` unit tests, two
`locus_topology` unit tests, 59 `locus_validate` unit tests, and three
`locus_alloc` doctests.

The main service-window harness moved from 1503 lines to 1233 lines. The new
local dirty group helper is 210 lines, so the combined harness code for this
area is 1443 lines while the high-level router file is smaller and easier to
scan.

All four local dirty-buffer group sample outputs preserved the same
service-window counters:

```text
submitted_count=2048 drained_count=2048 released_bytes=9437440 policy_drains=12 drain_rounds=36 registered_owners=3 service_window_observations=8 observed_reports=64 reports_needing_retune=46 max_pending_over_target=192 max_queued_bytes_over_budget=786432 queue_backpressure_reports=12 hold_decisions=3 apply_decisions=2 confirmed_decisions=1 rollback_decisions=1 mutation_limit_decisions=1 runtime_install_count=2 runtime_confirm_count=1 runtime_rollback_count=1 runtime_no_change_outcomes=4 missing_owner_checks=1 max_wait_bursts=8 mean_wait_bursts=3.312 final_queue_capacity=256 final_previous_config_present=false final_guard_pending_candidate=none final_guard_applied_mutations=2 final_guard_confirmed_mutations=1 final_guard_rollbacks=1
```

Timing run after extraction:

| Case | Time range | Criterion note |
| --- | ---: | --- |
| `remote_free_service_runtime_dirty_local_buffer_group_collection_sequence` | 200.70 to 201.53 us | change within noise threshold |
| `remote_free_service_runtime_dirty_local_buffer_group_integrated_collection_sequence` | 200.51 to 201.88 us | no change detected |
| `remote_free_service_runtime_dirty_local_buffer_group_bounded_collection_sequence` | 200.13 to 200.91 us | no change detected |
| `remote_free_service_runtime_dirty_local_buffer_group_validated_collection_sequence` | 203.03 to 204.46 us | change within noise threshold |

## Interpretation

The postulate survived.

The refactor preserved real service-window allocation counters and the local
dirty-buffer group benchmark paths still compile as bench targets. The helper
made the main harness smaller and centralized the repeated local group
invariants. Treat the timing output as a regression check only; this refactor
is an organization change, not a new performance claim.

The direct manual group path still does not test missing-owner marking with
`usize::MAX`, because that would intentionally exercise the unbounded
vector-indexed API with an unsafe allocation shape. Integrated, bounded, and
validated missing-owner checks remain covered by the helper.

## Next Question

Can local dirty group benchmark registration become table-driven so adding a
new group mode requires one descriptor instead of another Criterion wrapper?
