# Experiment 0175: Remote-Free Queued-Byte Drain Config

Date: 2026-07-03

## Postulate

[Postulate 0167](../postulates/0167-remote-free-queued-byte-drain-config.md)
claimed that a small queued-byte drain configuration type should validate queue
capacity, drain batch size, target pending item window, and retained-byte
budget together.

## Change

Added `RemoteFreeQueuedByteDrainConfig` to `locus-alloc`.

The config:

- validates non-zero queue capacity;
- validates non-zero drain batch limit;
- validates a non-zero target pending item window;
- rejects queue capacity below the target pending item window;
- rejects drain batch limits below the target pending item window;
- derives grouped retained-byte budgets through
  `RemoteFreeQueuedByteBudget`;
- exposes `drain_policy()` and `queue::<T>()` for the owner loop;
- keeps allocator-specific release behavior in the `drain_batch` closure.

The queued-byte owner-loop example now builds its queue and drain policy from
the config.

Updated `documentation/dev-notes/2026-07-03-remote-free-budget-selection.md`
with the new config helper and current open questions.

## Validation

Commands:

```bash
cargo fmt --all
cargo test -p locus-alloc remote_free_queued_byte_drain_config
cargo run -q -p locus-alloc --example remote_free_queued_byte_owner_loop
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
cargo bench -p locus-alloc --bench remote_free_mixed_size_policy --no-run
```

Results:

- Focused config tests passed: 8 passed, 0 failed.
- Owner-loop example passed.
- Workspace tests passed: 212 unit tests and 3 `locus_alloc` doctests passed.
- Clippy passed with `-D warnings`.
- Format check passed.
- Mixed-size remote-free benchmark built in optimized bench mode.

Owner-loop output:

```text
remote_free_queued_byte_owner_loop=started
queue_capacity=256
drain_batch_limit=64
request_concurrency=4
remote_free_blocks_per_request=16
representative_block_bytes=10240
target_pending_items=64
queued_byte_budget=655360
remote_free_queued_byte_owner_loop=complete blocks=256 submitted_count=256 drained_count=256 pending_count=0 full_count=0 forced_drains=0 policy_drains=4 drain_rounds=4 max_pending_count=64 max_queued_bytes=655360 released_bytes=2621440 max_wait_bursts=2 mean_wait_bursts=1.500
```

## Interpretation

The postulate survived.

The new config catches the sizing invariants captured in the budget-selection
note while preserving explicit release ownership. The owner-loop counters are
unchanged:

- queued-byte budget: 655,360 bytes;
- target pending items: 64;
- policy drains: 4;
- max pending count: 64;
- max queued bytes: 655,360;
- max wait: 2 bursts;
- mean wait: 1.500 bursts;
- `full_count=0`.

The config currently covers the grouped owner-loop shape. Uniform and
heterogeneous constructors should only be added when a call site needs queue
and batch validation for those shapes.

## Next Step

Evaluate whether KV and request remote-free benchmark setup should use the
config once their queue capacity and batch-limit choices need the same
validation at construction time.
