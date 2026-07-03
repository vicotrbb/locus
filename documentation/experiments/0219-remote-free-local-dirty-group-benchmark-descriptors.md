# Experiment 0219: Remote-Free Local Dirty Group Benchmark Descriptors

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0211-remote-free-local-dirty-group-benchmark-descriptors.md`

The postulate said that the local dirty-buffer group service-window benchmarks
could be registered from a descriptor table while preserving Criterion
benchmark names, sample labels, allocation counters, and validation behavior.

## Change

Replaced four local dirty-buffer group Criterion wrapper functions with
`LOCAL_DIRTY_GROUP_BENCHMARKS`, a descriptor table containing:

- runner mode;
- sample label;
- sample summary label;
- Criterion benchmark name.

Added
`benchmark_runtime_dirty_local_buffer_group_collection_sequences` as the single
registration function for all four local dirty-buffer group cases.

## Commands

```text
cargo fmt --all
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo bench -p locus-alloc --bench remote_free_service_telemetry remote_free_service_runtime_dirty_local_buffer_group
cargo bench -p locus-alloc --bench remote_free_service_telemetry remote_free_service_runtime_dirty_local_buffer_group_validated_collection_sequence
git show HEAD:crates/locus-alloc/benches/remote_free_service/runtime_service_window_harness.rs | wc -l
wc -l crates/locus-alloc/benches/remote_free_service/runtime_service_window_harness.rs
rg -o "remote_free_service_runtime_dirty_local_buffer_group[a-z_]*collection_sequence" crates/locus-alloc/benches/remote_free_service/runtime_service_window_harness.rs | sort
rg -n "$(printf '\342\200\224')" documentation crates || true
git diff --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Results

The benchmark target compiled successfully after descriptor registration. The
no-em-dash check returned no matches. `git diff --check` passed. Workspace
clippy passed with warnings denied. The workspace test run passed, including
191 `locus_alloc` unit tests, one `locus_alloc` integration test, 13
`locus_core` unit tests, 34 `locus_observe` unit tests, six `locus_sys` unit
tests, two `locus_topology` unit tests, 59 `locus_validate` unit tests, and
three `locus_alloc` doctests.

The local dirty group Criterion names remained present:

```text
remote_free_service_runtime_dirty_local_buffer_group_bounded_collection_sequence
remote_free_service_runtime_dirty_local_buffer_group_collection_sequence
remote_free_service_runtime_dirty_local_buffer_group_integrated_collection_sequence
remote_free_service_runtime_dirty_local_buffer_group_validated_collection_sequence
```

The main service-window harness moved from 1246 lines to 1204 lines.

All four descriptor-registered local dirty-buffer group sample outputs
preserved the same service-window counters:

```text
submitted_count=2048 drained_count=2048 released_bytes=9437440 policy_drains=12 drain_rounds=36 registered_owners=3 service_window_observations=8 observed_reports=64 reports_needing_retune=46 max_pending_over_target=192 max_queued_bytes_over_budget=786432 queue_backpressure_reports=12 hold_decisions=3 apply_decisions=2 confirmed_decisions=1 rollback_decisions=1 mutation_limit_decisions=1 runtime_install_count=2 runtime_confirm_count=1 runtime_rollback_count=1 runtime_no_change_outcomes=4 missing_owner_checks=1 max_wait_bursts=8 mean_wait_bursts=3.312 final_queue_capacity=256 final_previous_config_present=false final_guard_pending_candidate=none final_guard_applied_mutations=2 final_guard_confirmed_mutations=1 final_guard_rollbacks=1
```

Initial descriptor timing run:

| Case | Time range | Criterion note |
| --- | ---: | --- |
| `remote_free_service_runtime_dirty_local_buffer_group_collection_sequence` | 197.75 to 198.11 us | no change detected |
| `remote_free_service_runtime_dirty_local_buffer_group_integrated_collection_sequence` | 196.60 to 197.66 us | improved |
| `remote_free_service_runtime_dirty_local_buffer_group_bounded_collection_sequence` | 199.22 to 200.37 us | change within noise threshold |
| `remote_free_service_runtime_dirty_local_buffer_group_validated_collection_sequence` | 221.04 to 239.78 us | regressed |

Focused validated rerun:

| Case | Time range | Criterion note |
| --- | ---: | --- |
| `remote_free_service_runtime_dirty_local_buffer_group_validated_collection_sequence` | 197.82 to 199.02 us | improved against the noisy prior sample |

## Interpretation

The descriptor-registration postulate survived for correctness and benchmark
identity.

The focused filter still ran all four original benchmark names during the
group run, and the exact validated filter ran the validated name separately.
The allocation and service-window counters stayed unchanged. The first
validated timing block reported a large regression, but the exact rerun
returned to the normal 198 us range without a code change. Treat that as
session noise, not descriptor overhead.

Do not claim a performance gain from descriptor registration. The value is
maintainability: future local dirty-buffer group modes now need one descriptor
instead of one wrapper function plus one Criterion entrypoint call.

## Next Question

Can service-window benchmark sample printing become filter-aware so a focused
Criterion run does not print unrelated sample blocks?
