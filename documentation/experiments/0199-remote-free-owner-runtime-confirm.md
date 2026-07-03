# Experiment 0199: Remote-Free Owner Runtime Confirm

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0191-remote-free-owner-runtime-confirm.md`

The postulate said that a remote-free owner runtime needs an explicit confirm
operation after a guarded candidate validates cleanly. Confirmation should
clear rollback state only at an empty owner boundary and should not rebuild or
change the active queue and controller config.

## Change

Added `RemoteFreeOwnerRuntime::confirm` and
`RemoteFreeOwnerRuntimeConfirmOutcome`.

The confirm operation:

- verifies the same empty owner boundary used by install and rollback;
- clears the previous rollback config;
- preserves the active config and queue sizing;
- returns a no-op outcome when there is no previous config;
- rejects confirmation while queue or controller work is pending.

Extended `remote_free_service_telemetry` with
`remote_free_service_runtime_apply_confirm`, a real allocation benchmark that
runs:

1. one owner window at queue capacity 128;
2. an install to queue capacity 256 from a guarded combined candidate plan;
3. one clean owner window at queue capacity 256;
4. runtime confirmation;
5. one more owner window at queue capacity 256.

Each owner window allocates real `Vec<u8>` blocks and releases them through
owner-side runtime drains.

## Commands

```text
cargo fmt --all --check
cargo test -p locus-alloc remote_free:: -- --nocapture
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo bench -p locus-alloc --bench remote_free_service_telemetry -- --sample-size 10 --warm-up-time 1 --measurement-time 1
```

## Results

All validation commands passed. The focused remote-free test run reported 91
remote-free tests passing. The workspace test run reported 265 unit tests, 1
integration test, and 3 `locus_alloc` doctests passing.

Runtime apply-confirm sequence:

```text
remote_free_service_runtime_apply_confirm_sample windows=3 initial_queue_capacity=128 installed_queue_capacity=256 final_queue_capacity=256 submitted_count=768 drained_count=768 released_bytes=3145728 policy_drains=12 drain_rounds=12 install_count=1 confirm_count=1 rollback_count=0 max_wait_bursts=2 mean_wait_bursts=1.500 final_previous_config_present=false
remote_free_service_runtime_apply_confirm_sample_summary windows=3 samples=8 policy_drains_min=12 policy_drains_max=12 policy_drains_mean=12.000 drain_rounds_min=12 drain_rounds_max=12 drain_rounds_mean=12.000 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
```

The existing runtime apply-rollback sequence still preserved its counters:

```text
remote_free_service_runtime_apply_rollback_sample windows=3 initial_queue_capacity=128 installed_queue_capacity=256 final_queue_capacity=128 submitted_count=768 drained_count=768 released_bytes=3145728 policy_drains=12 drain_rounds=12 install_count=1 rollback_count=1 max_wait_bursts=2 mean_wait_bursts=1.500 final_previous_config_present=false
```

Short-run timing ranges:

| Case | Time range | Criterion note |
| --- | ---: | --- |
| `remote_free_service_runtime_apply_confirm` | 58.605 to 58.861 us | No change in performance detected; 2 low mild outliers |
| `remote_free_service_runtime_apply_rollback` | 58.982 to 59.234 us | No change in performance detected; 2 mild outliers |

## Interpretation

The postulate survived this test and benchmark pass.

Focused tests showed that confirmation clears rollback state at an empty
boundary, leaves active config and queue capacity unchanged, no-ops when no
rollback config exists, and rejects pending work. The benchmark then confirmed
the operation on the real allocation path: 768 submitted blocks, 768 drained
blocks, 3,145,728 released bytes, 12 policy drains, one install, one confirm,
zero rollbacks, final queue capacity 256, and no remaining rollback config.

This completes the owner runtime's install, confirm, and rollback primitives.
The next integration can wire guarded decisions into this runtime path without
leaving stale rollback state after successful validation.

## Next Question

Connect guarded apply, confirm, rollback, and mutation-limit decisions to
`RemoteFreeOwnerRuntime` in one service benchmark sequence.
