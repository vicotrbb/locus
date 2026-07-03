# Experiment 0203: Remote-Free Runtime-Collected Rollback On Byte Drift

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0195-remote-free-runtime-collected-rollback-byte-drift.md`

The postulate said that `RemoteFreeServiceRetuneGuard` can roll back a
runtime-applied remote-free candidate using only runtime-collected telemetry
when the validation workload reveals retained-byte drift after apply.

## Change

Added variable retained-byte owner windows to the runtime benchmark harness so
validation can allocate real blocks with a different retained byte size and
record the true byte size in `RemoteFreeOwnerRuntime` accounting.

Added `remote_free_service_runtime_collected_rollback` to the remote-free
service telemetry benchmark suite.

The benchmark runs three owner-runtime windows:

1. a capacity-128 owner runtime starts with an empty initial drain policy and
   records a runtime-collected combined drift summary from 4096-byte blocks;
2. a second 4096-byte window produces a stable
   `increase_queue_capacity_and_drain_earlier` apply decision, installing
   queue capacity 256 and retaining rollback state;
3. a validation window allocates 8193-byte blocks, records those true retained
   bytes, reports retained-byte drift through
   `RemoteFreeOwnerRuntime::drift_report`, and triggers runtime rollback to
   queue capacity 128.

No synthetic validation summaries are used.

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

Runtime-collected rollback sample:

```text
remote_free_service_runtime_collected_rollback_sample windows=3 stable_windows=2 max_mutations=2 validation_bytes=8193 submitted_count=768 drained_count=768 released_bytes=4194560 policy_drains=8 drain_rounds=16 observed_reports=24 reports_needing_retune=22 max_pending_over_target=64 max_queued_bytes_over_budget=262144 queue_backpressure_reports=12 hold_decisions=1 apply_decisions=1 rollback_decisions=1 runtime_install_count=1 runtime_confirm_count=0 runtime_rollback_count=1 runtime_no_change_decisions=1 max_wait_bursts=4 mean_wait_bursts=2.333 final_queue_capacity=128 final_previous_config_present=false final_guard_pending_candidate=none final_guard_applied_mutations=1 final_guard_confirmed_mutations=0 final_guard_rollbacks=1
remote_free_service_runtime_collected_rollback_sample_summary windows=3 samples=8 policy_drains_min=8 policy_drains_max=8 policy_drains_mean=8.000 drain_rounds_min=16 drain_rounds_max=16 drain_rounds_mean=16.000 reports_needing_retune_min=22 reports_needing_retune_max=22 reports_needing_retune_mean=22.000 apply_decisions_min=1 apply_decisions_max=1 apply_decisions_mean=1.000 rollback_decisions_min=1 rollback_decisions_max=1 rollback_decisions_mean=1.000 max_wait_min=4 max_wait_max=4 max_wait_mean=4.000 mean_wait_min=2.333 mean_wait_max=2.333 mean_wait_mean=2.333
```

Short-run timing:

| Case | Time range |
| --- | ---: |
| `remote_free_service_runtime_collected_rollback` | 88.271 to 88.852 us |

## Interpretation

The postulate survived this test and benchmark pass.

Runtime-collected reports from the first two windows produced the combined
capacity-plus-drain apply candidate. The validation window used real 8193-byte
allocations and recorded 8193 retained bytes per submitted block. That exposed
the old 4096-byte budget as stale, so the guard returned rollback and the
runtime restored queue capacity 128 at an empty boundary.

The benchmark preserved real allocation counters: 768 submitted blocks, 768
drained blocks, 4,194,560 released bytes, 8 policy drains, 16 drain rounds,
max wait 4 bursts, and mean wait 2.333 bursts. Runtime operations matched
guard decisions: one install, one rollback, zero confirms, and one no-change
outcome.

This completes runtime-collected evidence for guarded apply, confirm,
rollback, no-change, and mutation-limit outcomes.

## Next Question

Move the benchmark orchestration patterns into a reusable multi-owner runtime
orchestration API.
