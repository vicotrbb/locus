# Experiment 0206: Remote-Free Runtime Service Window Runner

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0198-remote-free-runtime-service-window-runner.md`

The postulate said that a reusable service-window runner over registered
remote-free owner runtimes can collect routed owner summaries, aggregate drift
and decision counters, and preserve the same guarded apply, confirm, rollback,
and mutation-limit behavior as direct registry calls.

## Change

Added a reusable service-window runner API:

- `RemoteFreeServiceRuntimeWindowObservation`;
- `RemoteFreeServiceRuntimeWindowStats`;
- `RemoteFreeServiceRuntimeWindowError`;
- `RemoteFreeServiceRuntimeRetuneOwners::observe_service_window`.

The runner accepts routed owner observations, routes each observation through
the registered-owner coordinator, records observed report counts, drift maxima,
queue backpressure, guard decision counts, and runtime outcome counts, and
returns owner ID context on routing or retune errors.

Added `remote_free_service_runtime_service_window_sequence` to the remote-free
service telemetry benchmark suite. The benchmark uses real runtime-collected
owner windows for:

1. one owner that applies and confirms;
2. one owner that applies and rolls back after retained-byte drift;
3. one owner that reaches the shared service mutation limit.

## Commands

```text
cargo fmt --all --check
cargo test -p locus-alloc remote_free::service_window -- --nocapture
cargo bench -p locus-alloc --bench remote_free_service_telemetry --no-run
cargo bench -p locus-alloc --bench remote_free_service_telemetry remote_free_service_runtime_service_window_sequence -- --sample-size 10 --warm-up-time 0.1 --measurement-time 0.1
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Results

All validation commands passed. The focused service-window test run reported 3
service-window tests passing. The workspace test run reported 273 unit tests, 1
integration test, and 3 `locus_alloc` doctests passing. Additional doctest
targets with zero tests also completed successfully. The benchmark target
compiled successfully, and the short-run Criterion benchmark completed
successfully.

Runtime service-window sample:

```text
remote_free_service_runtime_service_window_sample owners=3 windows=8 stable_windows=2 max_mutations=2 rollback_validation_bytes=8193 submitted_count=2048 drained_count=2048 released_bytes=9437440 policy_drains=12 drain_rounds=36 registered_owners=3 service_window_observations=8 observed_reports=64 reports_needing_retune=46 max_pending_over_target=192 max_queued_bytes_over_budget=786432 queue_backpressure_reports=12 hold_decisions=3 apply_decisions=2 confirmed_decisions=1 rollback_decisions=1 mutation_limit_decisions=1 runtime_install_count=2 runtime_confirm_count=1 runtime_rollback_count=1 runtime_no_change_outcomes=4 missing_owner_checks=1 max_wait_bursts=8 mean_wait_bursts=3.312 final_queue_capacity=256 final_previous_config_present=false final_guard_pending_candidate=none final_guard_applied_mutations=2 final_guard_confirmed_mutations=1 final_guard_rollbacks=1
remote_free_service_runtime_service_window_sample_summary owners=3 windows=8 samples=8 policy_drains_min=12 policy_drains_max=12 policy_drains_mean=12.000 drain_rounds_min=36 drain_rounds_max=36 drain_rounds_mean=36.000 reports_needing_retune_min=46 reports_needing_retune_max=46 reports_needing_retune_mean=46.000 apply_decisions_min=2 apply_decisions_max=2 apply_decisions_mean=2.000 confirmed_decisions_min=1 confirmed_decisions_max=1 confirmed_decisions_mean=1.000 rollback_decisions_min=1 rollback_decisions_max=1 rollback_decisions_mean=1.000 mutation_limit_decisions_min=1 mutation_limit_decisions_max=1 mutation_limit_decisions_mean=1.000 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=3.312 mean_wait_max=3.312 mean_wait_mean=3.312
```

Short-run timing:

| Case | Time range |
| --- | ---: |
| `remote_free_service_runtime_service_window_sequence` | 192.91 to 194.64 us |

## Interpretation

The postulate survived this test and benchmark pass.

The service-window runner preserved the owner registry allocation counters:
2048 submitted blocks, 2048 drained blocks, 9,437,440 released bytes, 12 policy
drains, 36 drain rounds, max wait 8 bursts, and mean wait 3.312 bursts.

The runner also preserved the guarded orchestration counters: two installs,
one confirm, one rollback, one mutation-limit decision, four no-change
outcomes, eight routed observations, and one explicit missing-owner check.

This removes benchmark-local drift and decision aggregation from the runtime
retune path and gives service loops one reusable stats object to log.

## Next Question

How should a live service event loop collect owner summaries and produce
`RemoteFreeServiceRuntimeWindowObservation` values without holding unnecessary
mutable owner borrows?
