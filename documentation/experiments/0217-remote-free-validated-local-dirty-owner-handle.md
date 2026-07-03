# Experiment 0217: Remote-Free Validated Local Dirty Owner Handle

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0209-remote-free-validated-local-dirty-owner-handle.md`

The postulate said that the owner registry can issue a small validated local
dirty-owner handle so callers avoid manually passing owner limits while
preserving the bounded local dirty-buffer group behavior measured in
Experiment 0216.

## Change

Added:

- `RemoteFreeServiceRuntimeValidatedDirtyOwner`;
- `RemoteFreeServiceRuntimeDirtyOwnerLocalBuffers::validate_owner`;
- `RemoteFreeServiceRuntimeDirtyOwnerLocalBuffers::mark_validated_dirty`;
- `RemoteFreeServiceRuntimeDirtyOwnerLocalBuffers::validated_local_marker`;
- `RemoteFreeServiceRuntimeRetuneOwners::validate_local_dirty_owner`;
- `remote_free_service_runtime_dirty_local_buffer_group_validated_collection_sequence`.

The registry method validates the requested owner ID against the current owner
count and returns a compact copyable handle. The local dirty-buffer group can
then mark or borrow a marker from that handle without each call site passing
an owner limit. Missing owner IDs still return
`OwnerOutOfRange` before local buffer allocation.

## Commands

```text
cargo fmt --all --check
cargo test -p locus-alloc remote_free::dirty -- --nocapture
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo bench -p locus-alloc --bench remote_free_service_telemetry remote_free_service_runtime_dirty_local_buffer_group
cargo bench -p locus-alloc --bench remote_free_service_telemetry remote_free_service_runtime_dirty_local_buffer_group
rg -n "$(printf '\342\200\224')" documentation crates || true
git diff --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Results

The focused validation passed. The dirty focused test run reported 29 tests
passing. The benchmark target compiled successfully. The no-em-dash check
returned no matches. `git diff --check` passed. Workspace clippy passed with
warnings denied. The workspace test run passed, including 191 `locus_alloc`
unit tests, one `locus_alloc` integration test, 13 `locus_core` unit tests,
34 `locus_observe` unit tests, six `locus_sys` unit tests, two
`locus_topology` unit tests, 59 `locus_validate` unit tests, and three
`locus_alloc` doctests.

The new tests covered:

- validating an owner handle from the registry;
- rejecting a missing owner handle from the registry;
- marking a validated owner without rechecking owner limits;
- borrowing a validated local marker without passing owner limits;
- preserving duplicate local mark counters for accepted handles.

Validated local buffer group collection sample:

```text
remote_free_service_runtime_dirty_local_buffer_group_validated_collection_sample owners=3 windows=8 stable_windows=2 max_mutations=2 rollback_validation_bytes=8193 submitted_count=2048 drained_count=2048 released_bytes=9437440 policy_drains=12 drain_rounds=36 registered_owners=3 service_window_observations=8 observed_reports=64 reports_needing_retune=46 max_pending_over_target=192 max_queued_bytes_over_budget=786432 queue_backpressure_reports=12 hold_decisions=3 apply_decisions=2 confirmed_decisions=1 rollback_decisions=1 mutation_limit_decisions=1 runtime_install_count=2 runtime_confirm_count=1 runtime_rollback_count=1 runtime_no_change_outcomes=4 missing_owner_checks=1 max_wait_bursts=8 mean_wait_bursts=3.312 final_queue_capacity=256 final_previous_config_present=false final_guard_pending_candidate=none final_guard_applied_mutations=2 final_guard_confirmed_mutations=1 final_guard_rollbacks=1
remote_free_service_runtime_dirty_local_buffer_group_validated_collection_sample_summary owners=3 windows=8 samples=8 policy_drains_min=12 policy_drains_max=12 policy_drains_mean=12.000 drain_rounds_min=36 drain_rounds_max=36 drain_rounds_mean=36.000 reports_needing_retune_min=46 reports_needing_retune_max=46 reports_needing_retune_mean=46.000 apply_decisions_min=2 apply_decisions_max=2 apply_decisions_mean=2.000 confirmed_decisions_min=1 confirmed_decisions_max=1 confirmed_decisions_mean=1.000 rollback_decisions_min=1 rollback_decisions_max=1 rollback_decisions_mean=1.000 mutation_limit_decisions_min=1 mutation_limit_decisions_max=1 mutation_limit_decisions_mean=1.000 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=3.312 mean_wait_max=3.312 mean_wait_mean=3.312
```

First timing run:

| Case | Time range |
| --- | ---: |
| `remote_free_service_runtime_dirty_local_buffer_group_bounded_collection_sequence` | 203.40 to 205.16 us |
| `remote_free_service_runtime_dirty_local_buffer_group_validated_collection_sequence` | 197.83 to 198.27 us |

Second timing run:

| Case | Time range |
| --- | ---: |
| `remote_free_service_runtime_dirty_local_buffer_group_bounded_collection_sequence` | 200.07 to 201.61 us |
| `remote_free_service_runtime_dirty_local_buffer_group_validated_collection_sequence` | 201.58 to 208.39 us |

## Interpretation

The correctness part of the postulate survived.

The registry rejected missing owner IDs before local buffer allocation.
Accepted handles preserved local duplicate counters, flush counters, tracked
collection semantics, and the full service-window decision sequence.

The strict performance part needs more data. The first run favored the
validated handle path, but the second run showed the validated path slightly
overlapping and then exceeding the bounded path with many high outliers. Treat
the validated handle as the cleaner production API for avoiding manual owner
limit plumbing, but do not claim it is faster than the bounded path.

## Next Question

Can the service-window benchmark factor shared local dirty-buffer group
collection assertions so manual, integrated, bounded, and validated paths are
less noisy and easier to compare?
