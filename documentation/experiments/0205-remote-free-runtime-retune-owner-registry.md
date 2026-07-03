# Experiment 0205: Remote-Free Runtime Retune Owner Registry

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0197-remote-free-runtime-retune-owner-registry.md`

The postulate said that a small owner registry around
`RemoteFreeServiceRuntimeRetuneCoordinator` can address multiple
`RemoteFreeOwnerRuntime` instances by stable owner IDs while preserving one
shared service-level mutation budget.

## Change

Added a reusable owner registry API:

- `RemoteFreeServiceRuntimeOwnerId`;
- `RemoteFreeServiceRuntimeRetuneOwners`;
- `RemoteFreeServiceRuntimeRetuneOwnerError`.

The registry owns one `RemoteFreeServiceRuntimeRetuneCoordinator`, owns a
collection of owner runtimes, returns stable owner IDs when runtimes are
registered, routes summaries by owner ID through the shared coordinator, and
reports missing owner IDs explicitly.

Added `remote_free_service_runtime_registry_sequence` to the remote-free
service telemetry benchmark suite. The benchmark registers three owner
runtimes and addresses them by owner ID:

1. one owner applies and confirms;
2. one owner applies and rolls back after retained-byte drift;
3. one owner reaches the shared service mutation limit.

## Commands

```text
cargo fmt --all --check
cargo test -p locus-alloc remote_free::coordinator -- --nocapture
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo bench -p locus-alloc --bench remote_free_service_telemetry -- --sample-size 10 --warm-up-time 1 --measurement-time 1
```

## Results

All validation commands passed. The focused coordinator test run reported 4
coordinator tests passing. The workspace test run reported 270 unit tests, 1
integration test, and 3 `locus_alloc` doctests passing. Additional benchmark
and doctest targets with zero tests also completed successfully.

Runtime registry sample:

```text
remote_free_service_runtime_registry_sample owners=3 windows=8 stable_windows=2 max_mutations=2 rollback_validation_bytes=8193 submitted_count=2048 drained_count=2048 released_bytes=9437440 policy_drains=12 drain_rounds=36 registered_owners=3 observed_reports=64 reports_needing_retune=46 max_pending_over_target=192 max_queued_bytes_over_budget=786432 queue_backpressure_reports=12 hold_decisions=3 apply_decisions=2 confirmed_decisions=1 rollback_decisions=1 mutation_limit_decisions=1 runtime_install_count=2 runtime_confirm_count=1 runtime_rollback_count=1 runtime_no_change_decisions=4 missing_owner_checks=1 max_wait_bursts=8 mean_wait_bursts=3.312 final_queue_capacity=256 final_previous_config_present=false final_guard_pending_candidate=none final_guard_applied_mutations=2 final_guard_confirmed_mutations=1 final_guard_rollbacks=1
remote_free_service_runtime_registry_sample_summary owners=3 windows=8 samples=8 policy_drains_min=12 policy_drains_max=12 policy_drains_mean=12.000 drain_rounds_min=36 drain_rounds_max=36 drain_rounds_mean=36.000 reports_needing_retune_min=46 reports_needing_retune_max=46 reports_needing_retune_mean=46.000 apply_decisions_min=2 apply_decisions_max=2 apply_decisions_mean=2.000 confirmed_decisions_min=1 confirmed_decisions_max=1 confirmed_decisions_mean=1.000 rollback_decisions_min=1 rollback_decisions_max=1 rollback_decisions_mean=1.000 mutation_limit_decisions_min=1 mutation_limit_decisions_max=1 mutation_limit_decisions_mean=1.000 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=3.312 mean_wait_max=3.312 mean_wait_mean=3.312
```

Short-run timing:

| Case | Time range |
| --- | ---: |
| `remote_free_service_runtime_registry_sequence` | 199.90 to 201.10 us |

## Interpretation

The postulate survived this test and benchmark pass.

The registry addressed three owner runtimes by stable IDs while preserving one
shared coordinator and mutation budget. The measured sequence produced two
installs, one confirm, one rollback, one mutation-limit decision, four
no-change outcomes, and one explicit missing-owner check.

The benchmark preserved real allocation counters: 2048 submitted blocks, 2048
drained blocks, 9,437,440 released bytes, 12 policy drains, 36 drain rounds,
max wait 8 bursts, and mean wait 3.312 bursts.

This creates the first reusable multi-owner runtime retune orchestration
surface in Locus.

## Next Question

Connect registered owner runtimes to a higher-level service window runner that
collects summaries and chooses owner IDs without benchmark-local sequencing.
