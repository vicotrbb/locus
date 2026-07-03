# Experiment 0202: Remote-Free Runtime-Collected Multi-Owner Mutation Limit

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0194-remote-free-runtime-collected-multi-owner-mutation-limit.md`

The postulate said that a service-wide `RemoteFreeServiceRetuneGuard` can
enforce its mutation limit across multiple `RemoteFreeOwnerRuntime` instances
using only runtime-collected drift reports, while preserving real allocation
and release counters for every owner window.

## Change

Added `remote_free_service_runtime_collected_multi_owner_mutation_limit` to the
remote-free service telemetry benchmark suite.

The benchmark keeps one service guard and runs three owner runtimes:

1. owner 1 starts with an empty initial drain policy, emits two drifting
   runtime-collected windows, installs the guarded `drain_earlier` candidate,
   then confirms after a clean runtime-collected validation window;
2. owner 2 repeats the same runtime-collected apply-confirm path;
3. owner 3 emits two drifting runtime-collected windows, but the service guard
   reports `mutation_limit_reached` because the two-mutation budget is already
   exhausted.

Each owner summary is built from `RemoteFreeOwnerRuntime::drift_report`. The
runtime no-change path is exercised for hold and mutation-limit decisions.

## Commands

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo bench -p locus-alloc --bench remote_free_service_telemetry -- --sample-size 10 --warm-up-time 1 --measurement-time 1
```

## Results

All validation commands passed. The workspace test run reported 266 unit
tests, 1 integration test, and 3 `locus_alloc` doctests passing. Additional
benchmark and doctest targets with zero tests also completed successfully.

Runtime-collected multi-owner mutation-limit sample:

```text
remote_free_service_runtime_collected_multi_owner_mutation_limit_sample owners=3 windows=8 stable_windows=2 max_mutations=2 submitted_count=2048 drained_count=2048 released_bytes=8388608 policy_drains=8 drain_rounds=32 observed_reports=64 reports_needing_retune=36 max_pending_over_target=192 max_queued_bytes_over_budget=786432 queue_backpressure_reports=0 hold_decisions=3 apply_decisions=2 confirmed_decisions=2 mutation_limit_decisions=1 runtime_install_count=2 runtime_confirm_count=2 runtime_rollback_count=0 runtime_no_change_decisions=4 max_wait_bursts=8 mean_wait_bursts=3.750 final_queue_capacity=256 final_previous_config_present=false final_guard_pending_candidate=none final_guard_applied_mutations=2 final_guard_confirmed_mutations=2 final_guard_rollbacks=0
remote_free_service_runtime_collected_multi_owner_mutation_limit_sample_summary owners=3 windows=8 samples=8 policy_drains_min=8 policy_drains_max=8 policy_drains_mean=8.000 drain_rounds_min=32 drain_rounds_max=32 drain_rounds_mean=32.000 reports_needing_retune_min=36 reports_needing_retune_max=36 reports_needing_retune_mean=36.000 apply_decisions_min=2 apply_decisions_max=2 apply_decisions_mean=2.000 confirmed_decisions_min=2 confirmed_decisions_max=2 confirmed_decisions_mean=2.000 mutation_limit_decisions_min=1 mutation_limit_decisions_max=1 mutation_limit_decisions_mean=1.000 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=3.750 mean_wait_max=3.750 mean_wait_mean=3.750
```

Short-run timing:

| Case | Time range |
| --- | ---: |
| `remote_free_service_runtime_collected_multi_owner_mutation_limit` | 163.80 to 164.32 us |

## Interpretation

The postulate survived this test and benchmark pass.

Runtime-collected drift reports from three owner runtimes drove one
service-wide guard. The first two owners each produced a stable
`drain_earlier` apply and a clean confirmation. The third owner produced the
same stable runtime-collected drift, but the guard returned
`mutation_limit_reached` and the runtime path recorded a no-change outcome.

The benchmark preserved real allocation counters across all owner windows:
2048 submitted blocks, 2048 drained blocks, 8,388,608 released bytes, 8 policy
drains, 32 drain rounds, max wait 8 bursts, and mean wait 3.750 bursts. Runtime
operations matched guard decisions: two installs, two confirms, zero
rollbacks, and four no-change outcomes.

This proves the runtime-collected mutation-limit path across multiple owner
runtimes. It still does not prove runtime-collected rollback after a failed
validation window.

## Next Question

Drive rollback from runtime-collected telemetry without manufacturing an
invalid validation summary.
