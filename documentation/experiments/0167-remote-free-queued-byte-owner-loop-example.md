# Experiment 0167: Remote-Free Queued-Byte Owner Loop Example

Date: 2026-07-03

## Postulate

[Postulate 0159](../postulates/0159-remote-free-queued-byte-owner-loop-example.md)
claimed that a small owner-loop example can show how to use the queued-byte
remote-free policy candidate without hiding allocator-specific release logic.

## Change

Added `crates/locus-alloc/examples/remote_free_queued_byte_owner_loop.rs`.

The example derives a retained-byte budget from runtime-shaped inputs:

```text
request_concurrency=4
remote_free_blocks_per_request=16
representative_block_bytes=10240
queued_byte_budget=655360
```

It then configures:

```text
RemoteFreeDrainPolicy::with_max_queued_bytes(655360)
```

The owner loop keeps release ownership explicit:

- producers submit real `Vec` allocations to `RemoteFreeQueue`;
- the owner records submits in `RemoteFreeDrainController`;
- the owner checks `should_drain_queue` at burst control points;
- allocator-specific release remains inside the `drain_batch` closure.

## Validation

Host commands:

```bash
cargo run -q -p locus-alloc --example remote_free_queued_byte_owner_loop
cargo test -p locus-alloc
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Docker command:

```bash
docker run --rm -v "$PWD":/work -w /work rust:1.96 sh -lc '/usr/local/cargo/bin/cargo run -q -p locus-alloc --example remote_free_queued_byte_owner_loop'
```

Results:

- Host example: passed.
- Docker example: passed.
- `cargo test -p locus-alloc`: passed, 77 tests plus 1 doc test.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace`: passed, 191 tests plus doc tests.

Host and Docker example output:

```text
remote_free_queued_byte_owner_loop=started
queue_capacity=256
drain_batch_limit=64
request_concurrency=4
remote_free_blocks_per_request=16
representative_block_bytes=10240
queued_byte_budget=655360
remote_free_queued_byte_owner_loop=complete blocks=256 submitted_count=256 drained_count=256 pending_count=0 full_count=0 forced_drains=0 policy_drains=4 drain_rounds=4 max_pending_count=64 max_queued_bytes=655360 released_bytes=2621440 max_wait_bursts=2 mean_wait_bursts=1.500
```

## Interpretation

The postulate survived.

The example reproduces the important queued-byte policy counters from
experiment 0166 while keeping the ownership boundary explicit. It does not need
queue internals, and it does not move allocator-specific release logic into the
controller.

The budget calculation is intentionally simple and auditable:

```text
4 requests * 16 remote-free blocks/request * 10,240 bytes/block = 655,360 bytes
```

That gives the same retained-byte envelope as the measured
`max_queued640kib` benchmark case:

- peak queued bytes: 655,360;
- max pending count: 64;
- policy drains: 4;
- max wait: 2 bursts;
- mean wait: 1.500 bursts;
- `full_count=0`.

This is integration evidence, not a new timing result. The best-results note
was not changed.

## Next Step

Use the example shape to add a request-scratch or KV-block runtime integration
benchmark that derives a queued-byte budget from real handle sizes and request
concurrency instead of representative `Vec` sizes.
