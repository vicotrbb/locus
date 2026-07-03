# Experiment 0201: Remote-Free Runtime-Collected Guarded Confirm

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0193-remote-free-runtime-collected-guarded-confirm.md`

The postulate said that a guarded remote-free retune confirm path can be driven
by telemetry collected directly from `RemoteFreeOwnerRuntime` windows, without
controlled synthetic service-summary shapes, while preserving real allocation
and release counters.

## Change

Added two owner-runtime telemetry APIs:

- `RemoteFreeOwnerRuntime::new_with_drain_policy`, which keeps the queued-byte
  config as the diagnostic target while using a caller-supplied initial drain
  policy;
- `RemoteFreeOwnerRuntime::drift_report`, which builds
  `RemoteFreeQueuedByteDriftReport` directly from runtime status.

Added `remote_free_service_runtime_collected_guarded_confirm` to the
remote-free service telemetry benchmark suite.

The benchmark runs three real owner-runtime allocation windows:

1. an initial end-drain runtime window using a queued-byte diagnostic config
   and empty drain policy, producing a `drain_earlier` hold decision;
2. a second end-drain runtime window, producing a stable `drain_earlier` apply
   decision that installs the config through `RemoteFreeOwnerRuntime`;
3. a queued-byte-policy validation window, producing a clean runtime-collected
   summary that confirms the installed config.

Every summary is collected from runtime status reports inside the owner window.
No controlled synthetic service-summary shapes are used in this benchmark.

## Commands

```text
cargo fmt --all --check
cargo test -p locus-alloc remote_free::runtime -- --nocapture
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo bench -p locus-alloc --bench remote_free_service_telemetry -- --sample-size 10 --warm-up-time 1 --measurement-time 1
```

## Results

All validation commands passed. The focused runtime test run reported 14
runtime tests passing. The workspace test run reported 266 unit tests, 1
integration test, and 3 `locus_alloc` doctests passing. Additional benchmark
and doctest targets with zero tests also completed successfully.

Runtime-collected guarded confirm sample:

```text
remote_free_service_runtime_collected_guarded_confirm_sample windows=3 stable_windows=2 max_mutations=2 submitted_count=768 drained_count=768 released_bytes=3145728 policy_drains=4 drain_rounds=12 observed_reports=24 reports_needing_retune=12 max_pending_over_target=192 max_queued_bytes_over_budget=786432 queue_backpressure_reports=0 hold_decisions=1 apply_decisions=1 confirmed_decisions=1 runtime_install_count=1 runtime_confirm_count=1 runtime_rollback_count=0 runtime_no_change_decisions=1 max_wait_bursts=8 mean_wait_bursts=3.500 final_queue_capacity=256 final_previous_config_present=false final_guard_pending_candidate=none final_guard_applied_mutations=1 final_guard_confirmed_mutations=1 final_guard_rollbacks=0
remote_free_service_runtime_collected_guarded_confirm_sample_summary windows=3 samples=8 policy_drains_min=4 policy_drains_max=4 policy_drains_mean=4.000 drain_rounds_min=12 drain_rounds_max=12 drain_rounds_mean=12.000 reports_needing_retune_min=12 reports_needing_retune_max=12 reports_needing_retune_mean=12.000 apply_decisions_min=1 apply_decisions_max=1 apply_decisions_mean=1.000 confirmed_decisions_min=1 confirmed_decisions_max=1 confirmed_decisions_mean=1.000 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=3.500 mean_wait_max=3.500 mean_wait_mean=3.500
```

Short-run timing:

| Case | Time range | Criterion note |
| --- | ---: | --- |
| `remote_free_service_runtime_collected_guarded_confirm` | 60.621 to 60.892 us | Change within noise threshold; 1 outlier among 10 measurements |

## Interpretation

The postulate survived this test and benchmark pass.

Runtime-collected status reports drove the guarded sequence from hold to apply
to confirm. The first two owner windows used an empty drain policy against the
queued-byte diagnostic config and produced 12 reports needing retune, max
pending drift of 192 items, and max queued-byte drift of 786,432 bytes. After
the guarded `drain_earlier` apply rebuilt the runtime with the config's
queued-byte policy, the validation window produced a clean confirmation.

The benchmark preserved real allocation counters: 768 submitted blocks, 768
drained blocks, 3,145,728 released bytes, 4 policy drains, 12 drain rounds, max
wait 8 bursts, and mean wait 3.500 bursts. The final runtime config remained
at queue capacity 256, and confirmation cleared rollback state.

This replaces controlled summary shapes for the guarded apply-confirm path.
It does not yet prove runtime-collected rollback, mutation-limit, or
multi-owner orchestration behavior.

## Next Question

Drive rollback and mutation-limit paths from runtime-collected telemetry, then
lift the runtime-collected guarded path to multi-owner orchestration.
