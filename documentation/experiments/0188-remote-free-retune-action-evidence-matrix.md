# Experiment 0188: Remote-Free Retune Action Evidence Matrix

Date: 2026-07-03

## Postulate

[Postulate 0180](../postulates/0180-remote-free-retune-action-evidence-matrix.md)
claimed that Locus should encode the validated
`RemoteFreeQueuedByteRetuneAction` evidence as a named matrix before adding
adaptive remote-free policy behavior.

## Change

Added `queued_byte_retune_action_matches_validated_surface_matrix` as a
dedicated `locus-alloc` integration test.

The test rebuilds drift reports from public queued-byte APIs:

- `RemoteFreeQueuedByteDrainConfig::from_item_shape`;
- `RemoteFreeQueuedByteDrainConfig::from_item_sizes`;
- `RemoteFreeQueuedByteDrainConfig::from_grouped_item_shape`;
- `RemoteFreeQueuedByteDriftReport::from_observation`.

The matrix names the validated surfaces from Experiments 0184 through 0187:

- uniform capacity backpressure;
- uniform capacity retained-window drift;
- uniform queued-byte policy clean window;
- mixed-size retained-window drift;
- mixed-size queued-byte policy clean window;
- runtime owner-loop clean window;
- KV end-drain retained-window drift;
- KV queued-byte policy clean window;
- request end-drain retained-window drift;
- request queued-byte policy clean window.

This is a regression matrix for the diagnostic action mapping. It does not
replace real allocation benchmarks; it preserves the already measured
benchmarks as a compact test before adaptive policy work.

## Validation

Commands:

```bash
cargo fmt --all
cargo test -p locus-alloc --test remote_free_retune_action_matrix queued_byte_retune_action_matches_validated_surface_matrix -- --nocapture
cargo test -p locus-alloc remote_free::drift
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Results:

- The first focused matrix test failed because the synthetic mixed-size
  sequence did not match the benchmark's 64-item, 655,360-byte retained
  window. The fixture was corrected to use the benchmark's eight-size pattern:
  4096, 4096, 8192, 4096, 16,384, 4096, 32,768, and 8192 bytes.
- Focused matrix integration test passed: 1 passed, 0 failed.
- Focused drift tests passed: 7 passed, 0 failed.
- Format check passed.
- Clippy passed with `-D warnings`.
- Workspace tests passed: 226 unit tests, 1 integration test, and 3
  `locus_alloc` doctests passed.

## Matrix Coverage

The validated actions are:

| Surface | Observation | Expected action |
| --- | --- | --- |
| Uniform capacity 64 | queue backpressure only | `increase_queue_capacity` |
| Uniform capacity 128 | backpressure plus retained-window drift | `increase_queue_capacity_and_drain_earlier` |
| Uniform capacity 256 | retained-window drift without backpressure | `drain_earlier` |
| Uniform policy capacity 256 | clean 64-item, 262,144-byte window | `keep_config` |
| Mixed-size capacity 128 | backpressure plus retained-window drift | `increase_queue_capacity_and_drain_earlier` |
| Mixed-size capacity 256 | retained-window drift without backpressure | `drain_earlier` |
| Mixed-size policy capacity 256 | clean 64-item, 655,360-byte window | `keep_config` |
| Owner-loop policy | clean 64-item, 655,360-byte window | `keep_config` |
| KV end-drain | retained-window drift without backpressure | `drain_earlier` |
| KV policy | clean 64-block, 262,144-byte window | `keep_config` |
| Request end-drain | retained-window drift without backpressure | `drain_earlier` |
| Request policy | clean 8-request, 262,144-byte window | `keep_config` |

## Interpretation

The postulate survived.

The matrix keeps the action mapping tied to measured workload surfaces without
duplicating benchmark control loops in every future test. The failed first run
was useful: it caught a size-shape mismatch that would have made the matrix
less faithful to the mixed-size benchmark.

Future adaptive policy work can now cite a focused regression test for the
current action semantics, but any adaptive mutation of queue capacity or drain
cadence still needs a real allocation benchmark for the affected workload.

## Next Step

Define the first adaptive-policy postulate using service-level telemetry as an
observation source, then benchmark it against the fixed queued-byte policy.
