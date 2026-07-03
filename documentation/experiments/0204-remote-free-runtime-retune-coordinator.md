# Experiment 0204: Remote-Free Runtime Retune Coordinator

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0196-remote-free-runtime-retune-coordinator.md`

The postulate said that a small reusable runtime retune coordinator can replace
benchmark-local guarded orchestration by owning the service guard and policy
applicator, then applying guard decisions to targeted
`RemoteFreeOwnerRuntime` instances.

## Change

Added `RemoteFreeServiceRuntimeRetuneCoordinator` as a public service-level
coordination API.

The coordinator:

- owns one `RemoteFreeServiceRetuneGuard`;
- owns one `RemoteFreeServiceRetunePolicyApplicator`;
- observes a `RemoteFreeServiceRetuneSummary` for a targeted owner runtime;
- applies runtime no-change outcomes for hold and mutation-limit decisions;
- installs configs for apply decisions through the typed applicator;
- confirms clean validation windows;
- rolls back failed validation windows;
- keeps mutation budget accounting service-wide across owner runtimes.

Added `RemoteFreeServiceRuntimeRetuneOutcome` and
`RemoteFreeServiceRuntimeRetuneError` so callers can inspect guard decisions
and runtime operation outcomes without reimplementing the branch logic.

Added `remote_free_service_runtime_coordinator_sequence` to the remote-free
service telemetry benchmark suite. The benchmark uses real runtime-collected
owner windows to cover:

1. one owner that applies and confirms;
2. one owner that applies and rolls back after retained-byte drift;
3. one owner that reaches the shared service mutation limit.

## Commands

```text
cargo fmt --all --check
cargo test -p locus-alloc remote_free::coordinator -- --nocapture
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo bench -p locus-alloc --bench remote_free_service_telemetry -- --sample-size 10 --warm-up-time 1 --measurement-time 1
```

## Results

All validation commands passed. The focused coordinator test run reported 2
coordinator tests passing. The workspace test run reported 268 unit tests, 1
integration test, and 3 `locus_alloc` doctests passing. Additional benchmark
and doctest targets with zero tests also completed successfully.

Runtime coordinator sample:

```text
remote_free_service_runtime_coordinator_sample owners=3 windows=8 stable_windows=2 max_mutations=2 rollback_validation_bytes=8193 submitted_count=2048 drained_count=2048 released_bytes=9437440 policy_drains=12 drain_rounds=36 observed_reports=64 reports_needing_retune=46 max_pending_over_target=192 max_queued_bytes_over_budget=786432 queue_backpressure_reports=12 hold_decisions=3 apply_decisions=2 confirmed_decisions=1 rollback_decisions=1 mutation_limit_decisions=1 runtime_install_count=2 runtime_confirm_count=1 runtime_rollback_count=1 runtime_no_change_decisions=4 max_wait_bursts=8 mean_wait_bursts=3.312 final_queue_capacity=256 final_previous_config_present=false final_guard_pending_candidate=none final_guard_applied_mutations=2 final_guard_confirmed_mutations=1 final_guard_rollbacks=1
remote_free_service_runtime_coordinator_sample_summary owners=3 windows=8 samples=8 policy_drains_min=12 policy_drains_max=12 policy_drains_mean=12.000 drain_rounds_min=36 drain_rounds_max=36 drain_rounds_mean=36.000 reports_needing_retune_min=46 reports_needing_retune_max=46 reports_needing_retune_mean=46.000 apply_decisions_min=2 apply_decisions_max=2 apply_decisions_mean=2.000 confirmed_decisions_min=1 confirmed_decisions_max=1 confirmed_decisions_mean=1.000 rollback_decisions_min=1 rollback_decisions_max=1 rollback_decisions_mean=1.000 mutation_limit_decisions_min=1 mutation_limit_decisions_max=1 mutation_limit_decisions_mean=1.000 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=3.312 mean_wait_max=3.312 mean_wait_mean=3.312
```

Short-run timing:

| Case | Time range | Criterion note |
| --- | ---: | --- |
| `remote_free_service_runtime_coordinator_sequence` | 194.95 to 196.07 us | 1 high mild outlier among 10 measurements |

## Interpretation

The postulate survived this test and benchmark pass.

The reusable coordinator produced the same guarded runtime behavior that had
previously lived in benchmark-local branching: two installs, one confirm, one
rollback, one mutation-limit decision, and four runtime no-change outcomes.
The service guard stayed shared across the three owners, so the third drifting
owner was mutation-limited after the first two apply decisions.

The benchmark preserved real allocation counters: 2048 submitted blocks, 2048
drained blocks, 9,437,440 released bytes, 12 policy drains, 36 drain rounds,
max wait 8 bursts, and mean wait 3.312 bursts.

This establishes the first reusable API boundary for service-level
runtime-collected remote-free retune orchestration.

## Next Question

Use the coordinator from a reusable multi-owner runtime collection instead of
passing owner runtimes one at a time from benchmark code.
