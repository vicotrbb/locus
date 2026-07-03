# Experiment 0145: Remote-Free Drain Policy Model

Date: 2026-07-03

## Postulate

[Postulate 0137](../postulates/0137-remote-free-drain-policy-model.md) claimed that the remote-free owner drain decision should be represented as a small pure policy model before it is wired into queue draining or scheduler code.

## Change

Added reusable remote-free drain policy types to `locus-alloc`:

- `RemoteFreeDrainObservation`, which carries pending item count, queued bytes, and oldest pending age;
- `RemoteFreeDrainPolicy`, which carries optional thresholds for pending count, queued bytes, and pending age;
- `RemoteFreeDrainDecision`, which returns drain or defer;
- `RemoteFreeDrainReason`, which reports whether queued bytes, pending age, or pending count triggered draining.

The model is pure and does not change `RemoteFreeQueue` behavior. It is re-exported from the crate root.

## Validation

Host commands:

```bash
cargo test -p locus-alloc remote_free_drain_policy
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Docker command:

```bash
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-alloc remote_free_drain_policy
```

Results:

- `cargo test -p locus-alloc remote_free_drain_policy`: passed, 5 focused tests.
- `cargo test --workspace`: passed, 158 tests plus doc tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- Docker focused policy tests: passed, 5 tests.

## Test Coverage

The focused tests cover:

- deferring when no thresholds are configured;
- deferring when there is no pending work, even if byte and age signals are high;
- triggering on queued bytes;
- triggering on oldest pending age;
- triggering on pending item count;
- deterministic reason priority when multiple thresholds are reached.

## Interpretation

The postulate survived.

The policy that experiment 0144 encoded inside the benchmark can now be represented with public reusable types:

- queued byte threshold for retained-memory pressure;
- pending age threshold for release latency;
- pending count threshold for queue occupancy.

This keeps the current queue primitive unchanged while giving future runtime loops a tested decision model.

## Next Step

Wire `RemoteFreeDrainPolicy` into the mixed-size benchmark so the benchmark uses the production policy model instead of its local `DrainPolicy` enum. That will prove the model expresses the previously measured queued-byte policy without duplicating logic.
