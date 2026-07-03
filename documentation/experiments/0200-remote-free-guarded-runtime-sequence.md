# Experiment 0200: Remote-Free Guarded Runtime Sequence

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0192-remote-free-guarded-runtime-sequence.md`

The postulate said that guarded remote-free retune decisions can drive
`RemoteFreeOwnerRuntime` install, confirm, rollback, and mutation-limit
outcomes in one measured sequence while preserving real owner-side allocation
and release counters.

## Change

Added `remote_free_service_guarded_runtime_sequence` to the remote-free service
telemetry benchmark suite.

The benchmark keeps the guarded-service logic and the owner runtime connected
through the typed applicator:

- each guard step runs one real `RemoteFreeOwnerRuntime` allocation window;
- controlled service-summary shapes drive `RemoteFreeServiceRetuneGuard`;
- apply decisions pass through `RemoteFreeServiceRetunePolicyApplicator`;
- runtime installs apply the typed config plan;
- clean validation windows confirm the runtime config;
- failed validation windows roll back the runtime config;
- mutation-limit decisions are translated into runtime no-change outcomes.

The sequence covers one confirmed `drain_earlier` apply, one rolled-back
`increase_queue_capacity_and_drain_earlier` apply, and one mutation-limit
decision after the mutation budget is exhausted.

The service summaries are controlled diagnostic shapes, not telemetry collected
directly from the runtime windows. Every guard window is still paired with a
real runtime allocation and release window.

## Commands

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo bench -p locus-alloc --bench remote_free_service_telemetry -- --sample-size 10 --warm-up-time 1 --measurement-time 1
```

## Results

All validation commands passed. The workspace test run reported 265 unit tests,
1 integration test, and 3 `locus_alloc` doctests passing. Additional benchmark
and doctest targets with zero tests also completed successfully.

Guarded runtime sample:

```text
remote_free_service_guarded_runtime_sample windows=9 stable_windows=2 max_mutations=2 submitted_count=2304 drained_count=2304 released_bytes=9437184 policy_drains=36 drain_rounds=36 observed_reports=9 reports_needing_retune=7 max_pending_over_target=32 max_queued_bytes_over_budget=262144 queue_backpressure_reports=2 hold_decisions=4 apply_decisions=2 confirmed_decisions=1 rollback_decisions=1 mutation_limit_decisions=1 runtime_install_count=2 runtime_confirm_count=1 runtime_rollback_count=1 runtime_no_change_decisions=5 drain_earlier_apply_decisions=1 combined_apply_decisions=1 max_wait_bursts=2 mean_wait_bursts=1.500 final_queue_capacity=128 final_previous_config_present=false final_guard_pending_candidate=none final_guard_applied_mutations=2 final_guard_confirmed_mutations=1 final_guard_rollbacks=1
remote_free_service_guarded_runtime_sample_summary windows=9 samples=8 policy_drains_min=36 policy_drains_max=36 policy_drains_mean=36.000 drain_rounds_min=36 drain_rounds_max=36 drain_rounds_mean=36.000 reports_needing_retune_min=7 reports_needing_retune_max=7 reports_needing_retune_mean=7.000 apply_decisions_min=2 apply_decisions_max=2 apply_decisions_mean=2.000 confirmed_decisions_min=1 confirmed_decisions_max=1 confirmed_decisions_mean=1.000 rollback_decisions_min=1 rollback_decisions_max=1 rollback_decisions_mean=1.000 mutation_limit_decisions_min=1 mutation_limit_decisions_max=1 mutation_limit_decisions_mean=1.000 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
```

Short-run timing:

| Case | Time range | Criterion note |
| --- | ---: | --- |
| `remote_free_service_guarded_runtime_sequence` | 178.59 to 179.40 us | Performance improved by 3.4475 to 4.2331 percent |

## Interpretation

The postulate survived this test and benchmark pass.

The guarded runtime sequence translated decisions into the expected runtime
operations: two installs, one confirm, one rollback, and five no-change
outcomes. The guard recorded two applied mutations, one confirmed mutation, one
rollback, one mutation-limit decision, and no pending validation candidate at
the end.

The real runtime windows preserved the allocation counters: 2304 submitted
blocks, 2304 drained blocks, 9,437,184 released bytes, 36 policy drains, 36
drain rounds, max wait 2 bursts, and mean wait 1.500 bursts. The final runtime
state returned to queue capacity 128 with no previous rollback config.

This gives Locus a measured bridge from service guard decisions to owner
runtime operations. It does not yet prove live retune safety from telemetry
collected inside the same runtime windows.

## Next Question

Replace the controlled service-summary shapes with telemetry collected directly
from runtime owner windows, then lift the same decision path to multi-owner
runtime orchestration.
