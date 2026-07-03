# Experiment 0197: Remote-Free Guarded Policy Application

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0189-remote-free-guarded-policy-application.md`

The postulate said that a production-facing remote-free retune application API
should only translate guarded `apply` decisions into validated queue and
drain-policy configs. All other guarded decisions should be observable
no-change outcomes, and telemetry should not be able to mutate live policy
directly.

## Change

Added `RemoteFreeServiceRetunePolicyApplicator` as a narrow bridge from
`RemoteFreeServiceRetuneGuardDecision` to validated
`RemoteFreeQueuedByteDrainConfig` values.

The applicator:

- rejects queue-capacity growth factors of zero or one;
- returns no-change application plans for collect, hold, confirm, rollback,
  and mutation-limit decisions;
- maps `drain_earlier` apply decisions to the current queued-byte drain
  config;
- maps `increase_queue_capacity` and
  `increase_queue_capacity_and_drain_earlier` apply decisions to checked
  larger queue capacity while preserving drain batch limit, target pending
  items, and queued-byte budget;
- rejects non-actionable apply candidates;
- rejects queue-capacity growth overflow.

The guarded service benchmark now validates pending candidates through the
applicator before selecting the explicit real-allocation service case. The
application benchmark helper lives in
`crates/locus-alloc/benches/remote_free_service/application_harness.rs` so the
guarded harness stays focused on guard sequencing.

## Commands

```text
cargo fmt --all --check
cargo test -p locus-alloc remote_free:: -- --nocapture
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo bench -p locus-alloc --bench remote_free_service_telemetry -- --sample-size 10 --warm-up-time 1 --measurement-time 1
cargo bench -p locus-alloc --bench remote_free_service_telemetry -- --sample-size 10 --warm-up-time 1 --measurement-time 1
```

## Results

All validation commands passed. The focused remote-free test run reported 78
remote-free tests passing. The workspace test run reported 252 unit tests, 1
integration test, and 3 `locus_alloc` doctests passing.

The final benchmark rerun kept the guarded allocation counters unchanged after
the applicator was wired into candidate validation.

Guarded confirming sequence:

```text
remote_free_service_guarded_confirming_sample windows=7 stable_windows=2 max_mutations=2 submitted_count=7168 drained_count=7168 released_bytes=29360128 policy_drains=96 observed_reports=224 reports_needing_retune=24 max_pending_over_target=192 max_queued_bytes_over_budget=786432 queue_backpressure_reports=8 hold_decisions=3 apply_decisions=2 confirmed_decisions=2 rollback_decisions=0 mutation_limit_decisions=0 drain_earlier_apply_decisions=1 combined_apply_decisions=1 max_wait_bursts=8 mean_wait_bursts=1.821 final_pending_candidate=none final_applied_mutations=2 final_confirmed_mutations=2 final_rollbacks=0
```

Guarded rollback sequence:

```text
remote_free_service_guarded_rollback_sample windows=4 stable_windows=2 max_mutations=2 submitted_count=4096 drained_count=4096 released_bytes=16777216 policy_drains=52 observed_reports=128 reports_needing_retune=18 max_pending_over_target=192 max_queued_bytes_over_budget=786432 queue_backpressure_reports=0 hold_decisions=2 apply_decisions=1 confirmed_decisions=0 rollback_decisions=1 mutation_limit_decisions=0 drain_earlier_apply_decisions=1 combined_apply_decisions=0 max_wait_bursts=8 mean_wait_bursts=2.062 final_pending_candidate=none final_applied_mutations=1 final_confirmed_mutations=0 final_rollbacks=1
```

Guarded mutation-limit sequence:

```text
remote_free_service_guarded_mutation_limit_sample windows=9 stable_windows=2 max_mutations=2 submitted_count=9216 drained_count=9216 released_bytes=37748736 policy_drains=120 observed_reports=288 reports_needing_retune=36 max_pending_over_target=192 max_queued_bytes_over_budget=786432 queue_backpressure_reports=8 hold_decisions=4 apply_decisions=2 confirmed_decisions=2 rollback_decisions=0 mutation_limit_decisions=1 drain_earlier_apply_decisions=1 combined_apply_decisions=1 max_wait_bursts=8 mean_wait_bursts=1.916 final_pending_candidate=none final_applied_mutations=2 final_confirmed_mutations=2 final_rollbacks=0
```

Short-run timing ranges from the final rerun:

| Case | Time range | Criterion note |
| --- | ---: | --- |
| `remote_free_service_telemetry_guarded_confirming` | 598.95 to 604.00 us | Change within noise threshold |
| `remote_free_service_telemetry_guarded_rollback` | 341.91 to 342.81 us | No change in performance detected; 2 outliers |
| `remote_free_service_telemetry_guarded_mutation_limit` | 767.12 to 771.66 us | No change in performance detected; 2 outliers |

One earlier run after the harness split flagged the mutation-limit case as a
3.0158 percent regression with the same allocation counters. A repeat run did
not reproduce the regression, so treat that signal as measurement noise unless
it reappears in a longer run.

## Interpretation

The postulate survived this benchmark and test run.

The new applicator is a narrow bridge from guard output to validated configs.
It does not observe telemetry, does not own guard state, and cannot mutate live
policy by itself. Only guarded `apply` decisions can produce an applied config,
and that config is rebuilt through `RemoteFreeQueuedByteDrainConfig` validation
with checked capacity growth.

The guarded benchmark still exercised real `Vec<u8>` allocation and owner-side
remote-free release. The confirming, rollback, and mutation-limit counters
remained identical to the previous measured guard paths after application
planning was inserted.

## Next Question

Add a small owner runtime wrapper that can install an applied config and keep
the previous config available for rollback, then benchmark rollback with real
queue reconstruction boundaries.
