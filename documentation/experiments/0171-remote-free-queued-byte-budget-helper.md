# Experiment 0171: Remote-Free Queued-Byte Budget Helper

Date: 2026-07-03

## Postulate

[Postulate 0163](../postulates/0163-remote-free-queued-byte-budget-helper.md)
claimed that a small typed helper for deriving remote-free queued-byte budgets
will reduce duplicated sizing arithmetic without weakening the explicit
owner-drained runtime boundary.

## Change

Added `RemoteFreeQueuedByteBudget` to `locus-alloc`.

The helper:

- stores a non-zero queued-byte budget;
- derives budgets from pending item count and bytes per item;
- derives budgets from grouped item shapes such as requests, blocks per
  request, and bytes per block;
- reports zero and overflow failures through
  `RemoteFreeQueuedByteBudgetError`;
- exposes `bytes()` and `as_non_zero_u64()` for policy composition;
- provides `into_policy()` for simple queued-byte-only policies.

The queued-byte owner-loop example now uses
`RemoteFreeQueuedByteBudget::from_grouped_item_shape` instead of local
checked multiplication. The owner still releases real `Vec` allocations inside
the `RemoteFreeQueue::drain_batch` closure.

## Validation

Commands:

```bash
cargo fmt --all
cargo test -p locus-alloc remote_free_queued_byte_budget
cargo run -q -p locus-alloc --example remote_free_queued_byte_owner_loop
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo bench -p locus-alloc --bench remote_free_mixed_size_policy --no-run
cargo bench -p locus-alloc --bench kv_remote_free_policy --no-run
cargo bench -p locus-alloc --bench request_remote_free_policy --no-run
cargo bench -p locus-alloc --bench remote_free_mixed_size_policy -- --sample-size 10 --warm-up-time 1 --measurement-time 1
```

Results:

- Focused helper tests passed: 9 passed, 0 failed.
- Workspace tests passed, including doctests: 200 unit tests passed across the
  workspace and 2 `locus_alloc` doctests passed.
- Clippy passed with `-D warnings`.
- The three remote-free policy benchmarks built in optimized bench mode.
- The owner-loop example completed with:
  - `queued_byte_budget=655360`;
  - `submitted_count=256`;
  - `drained_count=256`;
  - `full_count=0`;
  - `policy_drains=4`;
  - `max_pending_count=64`;
  - `max_queued_bytes=655360`;
  - `released_bytes=2621440`;
  - `max_wait_bursts=2`;
  - `mean_wait_bursts=1.500`.
- The short mixed-size Criterion run completed with the same counter behavior
  for `max_wait2` and `max_queued640kib`:
  - both had `full_count=0`;
  - both had `policy_drains=4`;
  - both had `drain_rounds=4`;
  - both had `max_pending_count=64`;
  - both had `max_queued_bytes=655360`;
  - both had `max_wait_bursts=2`;
  - both had `mean_wait_bursts=1.500`.
- Short-run timing estimates were:
  - end-drain: 41.959 us to 42.484 us;
  - max-wait-2: 38.640 us to 39.099 us;
  - max queued 640 KiB: 39.136 us to 39.392 us.

## Interpretation

The postulate survived.

The helper removed local budget arithmetic from the example while preserving
the real allocation path and the previously observed queued-byte counters. The
timing run is not a new best-result claim because it used a short Criterion
configuration and the helper does not change the benchmark policy path.

The useful result is API validation: queued-byte policy configuration can now
be derived from retained-byte shape inputs with centralized zero and overflow
handling.

## Next Step

Use the helper in later domain-specific configuration surfaces for KV-cache,
request-affine arenas, and GPU-near staging buffers when those call sites need
to derive retained-byte thresholds from runtime shape inputs.
