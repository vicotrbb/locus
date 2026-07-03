# Experiment 0220: Remote-Free Service-Window Filtered Sample Printing

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0212-remote-free-service-window-filtered-sample-printing.md`

The postulate said that the service-window benchmark harness could make its
sample printing aware of the Criterion benchmark filter while preserving
unfiltered sample output and benchmark execution.

## Change

Added a service-window sample filter helper that reads Criterion filter tokens
from process arguments and compares them with:

- the service-window sample label;
- the benchmark name derived from the sample label by replacing `_sample` or
  `_sample_summary` with `_sequence`.

The change only gates service-window sample and sample-summary printing.
Benchmark registration, benchmark names, descriptor registration, and
allocation assertions are unchanged.

## Commands

```text
cargo fmt --all --check
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo bench -p locus-alloc --bench remote_free_service_telemetry remote_free_service_runtime_dirty_local_buffer_group_validated_collection_sequence 2>&1 | rg "remote_free_service_runtime_(service_window|window_collection|dirty_window|dirty_enqueue|dirty_local)"
cargo bench -p locus-alloc --bench remote_free_service_telemetry remote_free_service_runtime_dirty_local_buffer_group 2>&1 | rg "remote_free_service_runtime_dirty_local_buffer_group.*_sample"
find target/criterion -path '*remote_free_service_runtime_dirty_local_buffer_group*' -name estimates.json -print
cargo bench -p locus-alloc --bench remote_free_service_telemetry -- --list 2>&1 | rg "remote_free_service_runtime_(service_window|window_collection|dirty_window|dirty_enqueue|dirty_local).*_sample" | wc -l
rg -n "$(printf '\342\200\224')" documentation crates || true
git diff --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Results

The benchmark target compiled successfully. The no-em-dash check returned no
matches. `git diff --check` passed. Workspace clippy passed with warnings
denied. The workspace test run passed, including 191 `locus_alloc` unit tests,
one `locus_alloc` integration test, 13 `locus_core` unit tests, 34
`locus_observe` unit tests, six `locus_sys` unit tests, two `locus_topology`
unit tests, 59 `locus_validate` unit tests, and three `locus_alloc` doctests.

The exact validated filter printed only the matching service-window sample and
sample summary before the matching benchmark measurement:

```text
remote_free_service_runtime_dirty_local_buffer_group_validated_collection_sample owners=3 windows=8 stable_windows=2 max_mutations=2 rollback_validation_bytes=8193 submitted_count=2048 drained_count=2048 released_bytes=9437440 policy_drains=12 drain_rounds=36 registered_owners=3 service_window_observations=8 observed_reports=64 reports_needing_retune=46 max_pending_over_target=192 max_queued_bytes_over_budget=786432 queue_backpressure_reports=12 hold_decisions=3 apply_decisions=2 confirmed_decisions=1 rollback_decisions=1 mutation_limit_decisions=1 runtime_install_count=2 runtime_confirm_count=1 runtime_rollback_count=1 runtime_no_change_outcomes=4 missing_owner_checks=1 max_wait_bursts=8 mean_wait_bursts=3.312 final_queue_capacity=256 final_previous_config_present=false final_guard_pending_candidate=none final_guard_applied_mutations=2 final_guard_confirmed_mutations=1 final_guard_rollbacks=1
remote_free_service_runtime_dirty_local_buffer_group_validated_collection_sample_summary owners=3 windows=8 samples=8 policy_drains_min=12 policy_drains_max=12 policy_drains_mean=12.000 drain_rounds_min=36 drain_rounds_max=36 drain_rounds_mean=36.000 reports_needing_retune_min=46 reports_needing_retune_max=46 reports_needing_retune_mean=46.000 apply_decisions_min=2 apply_decisions_max=2 apply_decisions_mean=2.000 confirmed_decisions_min=1 confirmed_decisions_max=1 confirmed_decisions_mean=1.000 rollback_decisions_min=1 rollback_decisions_max=1 rollback_decisions_mean=1.000 mutation_limit_decisions_min=1 mutation_limit_decisions_max=1 mutation_limit_decisions_mean=1.000 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=3.312 mean_wait_max=3.312 mean_wait_mean=3.312
```

The broad local group filter printed all four local dirty-buffer group sample
and sample-summary pairs, which is expected because all four benchmark names
match the broad filter. Each pair preserved the same 2048 submitted blocks,
2048 drained blocks, 9,437,440 released bytes, 12 policy drains, 36 drain
rounds, 46 reports needing retune, two apply decisions, one confirm, one
rollback, one mutation-limit decision, max wait 8 bursts, and mean wait 3.312
bursts.

The no-filter `--list` check printed 24 service-window sample lines, matching
12 service-window cases times sample plus summary.

Latest Criterion mean confidence intervals after the broad local group run:

| Case | Time range |
| --- | ---: |
| `remote_free_service_runtime_dirty_local_buffer_group_collection_sequence` | 196.05 to 196.33 us |
| `remote_free_service_runtime_dirty_local_buffer_group_integrated_collection_sequence` | 197.79 to 198.76 us |
| `remote_free_service_runtime_dirty_local_buffer_group_bounded_collection_sequence` | 199.83 to 201.41 us |
| `remote_free_service_runtime_dirty_local_buffer_group_validated_collection_sequence` | 197.43 to 198.37 us |

## Interpretation

The postulate survived for service-window sample printing.

Exact service-window filters now suppress unrelated service-window sample
labels, broad filters still print all matching service-window samples, and an
unfiltered registration path still prints the full service-window sample set.
The measured allocation counters stayed unchanged.

This is intentionally scoped to `runtime_service_window_harness.rs`. Other
remote-free service telemetry harnesses still print samples before Criterion
filtering and need a shared filter helper before the whole benchmark target has
clean focused output.

## Next Question

Can the remote-free service telemetry benchmark share one filter-aware sample
printing helper across all harness modules so focused runs suppress unrelated
sample blocks target-wide?
