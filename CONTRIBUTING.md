# Contributing to Locus

Locus is research-driven: the crate only ships code whose behavior
and performance claims are backed by the record in `documentation/`.
Read `documentation/README.md` first.

## The experiment methodology

- Postulate first. Before writing code or benchmarks, add a numbered
  postulate in `documentation/postulates/`: a falsifiable prediction,
  including the direction and rough size of the expected effect.
- Two runs minimum. Benchmarks are Criterion medians over two full
  runs; when the two medians disagree by more than 5 percent, run a
  third and report all three. Rerun the whole binary, not one
  benchmark, so every workload of that binary gets the extra median.
- Falsified results ship unedited. If the experiment kills the
  postulate, the experiment document says so and stays in the record
  with its data. Falsified designs are deleted from the build and
  mapped in a graveyard dev-note with recovery hashes, never from
  git history.
- The record is immutable. Never edit, rename, or move an existing
  postulate, experiment, evaluation, or dev-note; append-only
  addenda inside evaluation documents are the one sanctioned
  exception, clearly dated and below the frozen tables. Raw logs are
  copied verbatim into `documentation/evaluations/logs/`.
- Performance claims cite their evidence. Every number in README,
  rustdoc, or an ADR links the experiment or evaluation that
  produced it. Trace-workload margins must use the touch-parity
  audited values (see the LOCUS-EVAL v1 audit addendum).

## Gates

Every commit must pass:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CI runs these on Linux and macOS plus `cargo deny check` (licenses
and advisories). Changes that could affect the validated hot paths
must also rerun `locus_eval_locus` (see `scripts/run-eval.sh` for
the flags) and stay within 5 percent of the LOCUS-EVAL v1
locus_mailbox medians.

## Unsafe code policy

The crate is `deny(unsafe_code)` except the `locus::sys` module
(ADR 0002). Any new unsafe block goes in `sys`, carries a SAFETY
comment stating the invariant that makes it sound, and keeps the
public API safe and owned. PRs adding unsafe code anywhere else are
rejected.

## Scope

No process-wide allocator replacement (ADR 0001), and no new tuning
parameters on the remote-free path: experiment 0357's design
deliberately has none. Ideas that need new evidence start as
postulates, not as code.
