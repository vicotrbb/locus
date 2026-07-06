# 2026-07-05: Code graveyard (LOCUS-OSS Phase 1)

Code deleted from the build because the research record falsified or
obsoleted it. Nothing here is lost: every entry records the last
commit that contains the code in full, so `git show <hash>:<path>`
recovers any of it. The experiment documents that killed each entry
remain unedited in `documentation/experiments/`.

Last commit containing all code listed below:
`75272f157836b9516ca169d81de532fd80cb2961`.

## Deleted

| Module / file | What it was | Killed by |
| --- | --- | --- |
| `crates/locus-alloc/src/remote_free/budget.rs` | Queued-byte budget for the bounded remote-free queue | 0357 (mailbox deletes the capacity protocol; apparatus from 0059/0142/0143/0146) |
| `crates/locus-alloc/src/remote_free/config.rs` | Queued-byte drain configuration | 0357 (same) |
| `crates/locus-alloc/src/remote_free/controller.rs` | Drain controller/tracker/policy/decision machinery | 0357 (same) |
| `crates/locus-alloc/src/remote_free/drift.rs` | Queued-byte drift reports and retune hints | 0357 (same) |
| `crates/locus-alloc/src/remote_free/telemetry.rs` | Retune action counts and service retune summaries | 0357; part of the 0340-0350 rollup thread |
| `crates/locus-alloc/src/remote_free/application.rs` | Service retune policy applicator | 0340-0350 rollup thread, dead-ended; superseded by 0357 |
| `crates/locus-alloc/src/remote_free/coordinator.rs` | Service runtime retune coordinator and owner registry | same |
| `crates/locus-alloc/src/remote_free/dirty_buffer.rs` | Dirty-owner local buffers and flush stats | same |
| `crates/locus-alloc/src/remote_free/dirty_window.rs` | Dirty-owner trackers, snapshots, dirty sinks | same |
| `crates/locus-alloc/src/remote_free/guard.rs` | Service retune guard | same |
| `crates/locus-alloc/src/remote_free/planner.rs` | Retune dry-run planner | same |
| `crates/locus-alloc/src/remote_free/runtime.rs` | Per-owner retune apply/confirm/rollback runtime | same |
| `crates/locus-alloc/src/remote_free/service_window.rs` | Service window observations and stats | same |
| `crates/locus-alloc/benches/remote_free_backpressure.rs` | Bounded-queue backpressure microbench | 0357 |
| `crates/locus-alloc/benches/remote_free_capacity_retune.rs` | Capacity retune microbench (0142 thread) | 0357 |
| `crates/locus-alloc/benches/remote_free_mixed_size_capacity_retune.rs` | Mixed-size capacity retune microbench | 0357 |
| `crates/locus-alloc/benches/remote_free_drift_matrix.rs` | Drift/retune matrix bench | 0357 |
| `crates/locus-alloc/benches/remote_free_mixed_size_policy.rs` | Mixed-size drain policy bench | 0357 |
| `crates/locus-alloc/benches/kv_remote_free_policy.rs` | KV batch-size policy bench (0057/0059 thread) | 0357 |
| `crates/locus-alloc/benches/request_remote_free_policy.rs` | Request-return policy bench (0058 thread) | 0357 |
| `crates/locus-alloc/benches/remote_free_service_telemetry.rs` + `benches/remote_free_service/` (13 harnesses) | Service telemetry/rollup bench harnesses | 0340-0350 rollup thread |
| `crates/locus-alloc/tests/remote_free_retune_action_matrix.rs` | Retune action matrix test | 0357 |
| `crates/locus-alloc/examples/remote_free_queued_byte_owner_loop.rs` | Queued-byte owner loop example | 0357 |
| `crates/locus-validate/src/remote_free_service_collection_summary.rs` | Service telemetry collection summary validator | 0340-0350 rollup thread |
| `crates/locus-validate/src/remote_free_service_sample_compare.rs` | Service sample comparison validator | same |
| `crates/locus-validate/src/remote_free_service_timing_stability.rs` | Service timing stability validator | same |
| `crates/locus-validate/examples/remote_free_service_sample_compare.rs`, `remote_free_service_telemetry_collect.rs`, `remote_free_service_telemetry_summary_validate.rs` | Examples driving the above validators | same |

## Explicitly kept

- `RemoteFreeQueue`/`RemoteFreeSink` and their stats/error types in
  `crates/locus-alloc/src/remote_free.rs`: still the A/B comparison
  arm in `remote_free_mixed_lifetime_trace` and the locus-shared
  contender in LOCUS-EVAL v1. The retune apparatus built on top died;
  the primitive did not.
- `ChunkMailbox`/`ChunkMailboxSender` (0357, adopted) and
  `KvBlockPool`/`kv_block.rs` (0356's chunk-preserving change was
  falsified and already reverted; nothing to delete).
- Benches `locus_eval_*`, `remote_free_mixed_lifetime_trace`,
  `remote_free_chunk_publish`, `remote_free_concurrent`,
  `remote_free_sharded`, `kv_reuse_order_locality`.
- `mixed_lifetime_{jemalloc,mimalloc,system}` benches and their shared
  `mixed_lifetime_malloc/trace.rs`: direct backing for experiment
  0355, which 0357 cites as live evidence. Kept, not superseded, since
  LOCUS-EVAL v1 measures a related but distinct trace set.
- `scratch_arena*` benches: separate scratch-arena workstream, out of
  scope for this graveyard.

## Undecided

None. Every candidate resolved to delete or keep after checking
callers across the workspace.
