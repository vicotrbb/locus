# Locus

NUMA-aware memory pooling primitives for CPU LLM inference serving.

Locus provides a KV-block pool with owner-drained remote frees:
workers free whole request chunks into per-worker lock-free
mailboxes, and the pool owner drains them off the allocation hot
path. The design was developed postulate-first, and every performance
claim below cites the experiment or evaluation in
[`documentation/`](documentation/README.md) that produced it.

## Validated result

On LOCUS-EVAL v1, a frozen four-workload suite of deterministic
serving-shaped KV traces on Apple Silicon
([`documentation/evaluations/0001-locus-eval-v1.md`](documentation/evaluations/0001-locus-eval-v1.md)),
the mailbox design ranks first against jemalloc, mimalloc, and system
malloc on every workload. The honest margins are the audited,
touch-parity ones:

- 1.6x over mimalloc on steady decode, 2.7x on burst cancellation,
  and 2.3x on long-tail decode, at touch parity (the suite's audit
  addendum; the raw untouched-trace tables show 2.3x/3.9x/4.6x, but
  the audit found those margins overstated by a touch asymmetry, so
  quote the audited ones).
- When write bandwidth dominates (every block fully written, the
  churn-touch workload), the margin compresses to about 1.15x over
  system malloc: allocator choice matters far less once KV writes
  are counted.
- The trade shipped with the win: under a full burst cancellation
  the pool transiently holds up to 1.5x the theoretical peak block
  count, because the owner drains once per step (same evaluation,
  quality table and burst-storm verdict).

Two designs this replaced are kept in the record: a shared bounded
remote-free queue loses to every malloc baseline once producers are
concurrent (experiments 0351 to 0355), and chunk-preserving pool
recycling was falsified outright (0356).

## Design

- `locus_alloc::kv::KvBlockPool`: fixed-size KV blocks, generation-validated
  handles, LIFO reuse order for cache warmth (experiment 0358),
  optional mapped-region backing with a Linux `mbind` path (0359).
- `locus_alloc::remote_free::ChunkMailbox`: per-worker lock-free mailboxes;
  workers push whole request chunks, the owner drains all mailboxes.
  No capacity, batch, or retune parameters exist by design (0357).
- `locus_alloc::sys`: the single module allowed unsafe code (ADR 0002);
  the rest of the crate is `deny(unsafe_code)`.
- `locus_alloc::topology`: NUMA topology types; Linux sysfs discovery
  behind the `numa` feature.

Locus is not a global allocator and does not replace malloc
(ADR 0001). It is a domain pool a serving engine embeds; see
[`crates/locus-alloc/examples/serving_engine.rs`](crates/locus-alloc/examples/serving_engine.rs)
for the intended pattern (per-request chunk free, per-worker mailbox,
owner drain):

```sh
cargo run -p locus-alloc --example serving_engine
```

## Usage

```toml
[dependencies]
locus-alloc = "0.1"
```

## Repository layout

- `crates/locus-alloc`: the published crate.
- `crates/locus-observe`, `crates/locus-validate`: unpublished
  workspace members holding Linux NUMA evidence readers and
  validation gates used by the research harnesses.
- `documentation/`: the complete immutable research record
  (postulates, experiments, evaluations, ADRs, raw benchmark logs).
  Start with [its README](documentation/README.md).

## Reproducing the evaluation

```sh
scripts/run-eval.sh
```

runs the full LOCUS-EVAL suite with the exact Criterion flags used
for the recorded results and captures host info alongside the logs.

## Development gates

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for the experiment
methodology; results land in the record before code lands in the
crate.

## License

Apache-2.0
