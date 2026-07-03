# Experiment 0178: Remote-Free Queued-Byte Drift Report

Date: 2026-07-03

## Postulate

[Postulate 0170](../postulates/0170-remote-free-queued-byte-drift-report.md)
claimed that a small typed drift report should compare a queued-byte drain
config with live remote-free queue and controller observations before Locus
attempts adaptive remote-free policy mutation.

## Change

Added `RemoteFreeQueuedByteDriftReport` in a new focused
`remote_free::drift` module.

The report compares:

- configured target pending items;
- configured queued-byte budget;
- observed pending items;
- observed queued bytes;
- observed queue `full_count`.

It exposes saturating over-target counters for pending items and queued bytes,
plus boolean helpers for pending drift, byte drift, queue backpressure, and
whether the config needs review.

The mixed-size remote-free policy benchmark now exercises the report on the
real queued-byte config sample path. The timed Criterion benchmark path keeps
the original `TraceStats` shape, while sample output records separate drift
fields. This keeps diagnostic evidence visible without intentionally changing
the measured policy body.

Updated the queued-byte budget selection note with a drift diagnostics section.

## Validation

Commands:

```bash
cargo fmt --all
cargo test -p locus-alloc queued_byte_drift_report
cargo test -p locus-alloc remote_free_queued_byte_drain_config
cargo bench -p locus-alloc --bench remote_free_mixed_size_policy -- --sample-size 10 --warm-up-time 1 --measurement-time 1
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Results:

- Focused drift tests passed: 4 passed, 0 failed.
- Focused queued-byte drain config tests passed: 15 passed, 0 failed.
- Workspace tests passed: 223 unit tests and 3 `locus_alloc` doctests passed.
- Clippy passed with `-D warnings` after replacing a boolean-to-integer helper
  with `u64::from(value)`.
- The short mixed-size Criterion run passed.

## Mixed-Size Drift Results

Final short-run sample summaries:

```text
remote_free_mixed_size_policy_sample_summary=end_drain blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 drift_configured=0 samples=8 full_min=0 full_max=0 full_mean=0.000 forced_drains_min=0 forced_drains_max=0 forced_drains_mean=0.000 policy_drains_min=0 policy_drains_max=0 policy_drains_mean=0.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=256 max_pending_max=256 max_pending_mean=256.000 max_queued_bytes_min=2621440 max_queued_bytes_max=2621440 max_queued_bytes_mean=2621440 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=4.500 mean_wait_max=4.500 mean_wait_mean=4.500 max_pending_over_target_min=0 max_pending_over_target_max=0 max_pending_over_target_mean=0.000 max_queued_bytes_over_budget_min=0 max_queued_bytes_over_budget_max=0 max_queued_bytes_over_budget_mean=0 queue_backpressure_observed_min=0 queue_backpressure_observed_max=0 queue_backpressure_observed_mean=0.000
remote_free_mixed_size_policy_sample_summary=max_wait2 blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 drift_configured=0 samples=8 full_min=0 full_max=0 full_mean=0.000 forced_drains_min=0 forced_drains_max=0 forced_drains_mean=0.000 policy_drains_min=4 policy_drains_max=4 policy_drains_mean=4.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=655360 max_queued_bytes_max=655360 max_queued_bytes_mean=655360 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500 max_pending_over_target_min=0 max_pending_over_target_max=0 max_pending_over_target_mean=0.000 max_queued_bytes_over_budget_min=0 max_queued_bytes_over_budget_max=0 max_queued_bytes_over_budget_mean=0 queue_backpressure_observed_min=0 queue_backpressure_observed_max=0 queue_backpressure_observed_mean=0.000
remote_free_mixed_size_policy_sample_summary=max_queued640kib blocks=256 bursts=8 burst_blocks=32 capacity=256 batch_limit=64 drift_configured=1 samples=8 full_min=0 full_max=0 full_mean=0.000 forced_drains_min=0 forced_drains_max=0 forced_drains_mean=0.000 policy_drains_min=4 policy_drains_max=4 policy_drains_mean=4.000 drain_rounds_min=4 drain_rounds_max=4 drain_rounds_mean=4.000 max_pending_min=64 max_pending_max=64 max_pending_mean=64.000 max_queued_bytes_min=655360 max_queued_bytes_max=655360 max_queued_bytes_mean=655360 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500 max_pending_over_target_min=0 max_pending_over_target_max=0 max_pending_over_target_mean=0.000 max_queued_bytes_over_budget_min=0 max_queued_bytes_over_budget_max=0 max_queued_bytes_over_budget_mean=0 queue_backpressure_observed_min=0 queue_backpressure_observed_max=0 queue_backpressure_observed_mean=0.000
```

The configured queued-byte case reported:

- `drift_configured=1`;
- `max_pending_over_target=0`;
- `max_queued_bytes_over_budget=0`;
- `queue_backpressure_observed=0`;
- `full_count=0`;
- max pending 64;
- max queued bytes 655,360;
- max wait 2 bursts;
- mean wait 1.500 bursts.

Final short-run Criterion timings:

| Benchmark | Timing | Criterion note |
| --- | ---: | --- |
| `remote_free_mixed_size_trace_capacity256_batch64_end_drain` | 42.544 us to 42.811 us | Change within noise threshold |
| `remote_free_mixed_size_trace_capacity256_batch64_max_wait2` | 36.830 us to 37.497 us | Performance has improved |
| `remote_free_mixed_size_trace_capacity256_batch64_max_queued640kib` | 37.313 us to 38.426 us | Change within noise threshold |

## Negative Results And Fixes

An initial benchmark version stored drift counters directly in `TraceStats`.
That made the timed benchmark body carry diagnostic fields and the first short
Criterion run reported small regressions across all rows. The benchmark was
then split so timed iterations return the original `TraceStats`, while sample
paths return separate `TraceDriftStats`.

Clippy also rejected a hand-written boolean-to-integer helper. Replacing it
with `u64::from(value)` kept the benchmark output explicit and warning-free.

## Interpretation

The postulate survived.

The runtime now has a typed, allocation-free diagnostic that can detect
pending-item drift, queued-byte drift, and queue backpressure from the same
owner-loop observations used by the drain policy. The mixed-size allocation
sample confirms that the known-good queued-byte config has no drift on the real
allocation path.

This is not a new best-result claim. The value is adaptive-policy readiness:
future work can consume a typed drift report before deciding whether to change
queue capacity, drain cadence, drain batch size, or queued-byte budget.

## Next Step

Use `RemoteFreeQueuedByteDriftReport` in a deliberately mis-sized benchmark
matrix so the adaptive-policy inputs are tested against positive drift cases,
not only the known-good zero-drift config.
