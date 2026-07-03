# Experiment 0198: Remote-Free Owner Runtime Rollback

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0190-remote-free-owner-runtime-rollback.md`

The postulate said that a remote-free owner runtime can safely install guarded
policy application configs and preserve rollback state if it rebuilds queue
and controller state only at empty owner boundaries.

## Change

Added `RemoteFreeOwnerRuntime<T>` as a small owner-side wrapper around:

- `RemoteFreeQueuedByteDrainConfig`;
- `RemoteFreeQueue<T>`;
- `RemoteFreeDrainController`;
- one previous config for rollback.

The runtime exposes fresh sinks for the current queue generation, records
submits in controller accounting, drains batches while recording released byte
sizes, applies `RemoteFreeServiceRetunePolicyApplication` plans, and rolls back
to the previous config. Install and rollback both reject non-empty queue or
controller state.

Added runtime result and error types:

- `RemoteFreeOwnerRuntimeApplyOutcome`;
- `RemoteFreeOwnerRuntimeRollbackOutcome`;
- `RemoteFreeOwnerRuntimeError`.

Extended `remote_free_service_telemetry` with
`remote_free_service_runtime_apply_rollback`, a real allocation benchmark that
runs:

1. one owner window at queue capacity 128;
2. an install to queue capacity 256 from a guarded combined candidate plan;
3. one owner window at queue capacity 256;
4. rollback to queue capacity 128;
5. one owner window at queue capacity 128.

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

All validation commands passed. The focused remote-free test run reported 88
remote-free tests passing. The workspace test run reported 262 unit tests, 1
integration test, and 3 `locus_alloc` doctests passing.

Runtime apply/rollback sequence:

```text
remote_free_service_runtime_apply_rollback_sample windows=3 initial_queue_capacity=128 installed_queue_capacity=256 final_queue_capacity=128 submitted_count=768 drained_count=768 released_bytes=3145728 policy_drains=12 drain_rounds=12 install_count=1 rollback_count=1 max_wait_bursts=2 mean_wait_bursts=1.500 final_previous_config_present=false
remote_free_service_runtime_apply_rollback_sample_summary windows=3 samples=8 policy_drains_min=12 policy_drains_max=12 policy_drains_mean=12.000 drain_rounds_min=12 drain_rounds_max=12 drain_rounds_mean=12.000 max_wait_min=2 max_wait_max=2 max_wait_mean=2.000 mean_wait_min=1.500 mean_wait_max=1.500 mean_wait_mean=1.500
```

Short-run timing range:

| Case | Time range | Criterion note |
| --- | ---: | --- |
| `remote_free_service_runtime_apply_rollback` | 59.467 to 60.234 us | No change in performance detected; 2 high outliers |

## Interpretation

The postulate survived this test and benchmark pass.

The owner runtime refused non-empty install and rollback in focused tests,
rebuilt queue and controller state at empty boundaries, disconnected old sinks
after a rebuild, and restored the previous config on rollback. The benchmark
then exercised install and rollback through real allocation and release
counters: 768 submitted blocks, 768 drained blocks, 3,145,728 released bytes,
12 policy drains, one install, one rollback, final queue capacity 128, and no
remaining rollback config.

This does not migrate pending work across queue generations. That remains
intentionally out of scope until a separate postulate proves it is necessary
and safe.

## Next Question

Connect `RemoteFreeOwnerRuntime` to the guarded service sequence so apply,
confirm, rollback, and mutation-limit decisions can be measured through a
single runtime-owned path.
