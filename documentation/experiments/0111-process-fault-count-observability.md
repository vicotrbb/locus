# Experiment 0111: Process Fault Count Observability

Date: 2026-07-03

## Postulate

[Postulate 0103: Process Fault Count Observability](../postulates/0103-process-fault-count-observability.md)

## Change

Added process fault counter observability to `locus-observe`.

The change includes:

- `ProcessFaultCounts`;
- `ProcessFaultDelta`;
- `parse_process_stat_fault_counts`;
- `read_process_fault_counts`;
- `read_self_process_fault_counts`;
- `process_fault_counts` example.

The parser extracts `minflt`, `cminflt`, `majflt`, and `cmajflt` from `/proc/<pid>/stat`. It finds the final command-name closing parenthesis before tokenizing the numeric tail, so command names containing spaces or parentheses do not corrupt the field offsets.

## Commands

```sh
cargo fmt --all
cargo test -p locus-observe
cargo run -p locus-observe --example process_fault_counts
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-observe
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-observe --example process_fault_counts
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
rg "$(printf '\342\200\224')" README.md documentation crates Cargo.toml Cargo.lock
```

## Results

`cargo test -p locus-observe` passed:

- `locus-observe`: 30 unit tests passed.

Host example output:

```text
process_faults=unavailable
```

The host run is macOS, so `/proc/self/stat` is not present.

Docker `cargo test -p locus-observe` passed:

- `locus-observe`: 30 unit tests passed.

Docker example output:

```text
process_faults=available minor_faults=16856 child_minor_faults=36013 major_faults=9 child_major_faults=327
```

Full workspace validation passed:

- `cargo test --workspace`: 127 unit tests passed across workspace crates, plus doc tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed.
- em dash scan: no matches.

## Conclusion

The postulate survived. `locus-observe` now has reusable process fault counter parsing, explicit-path and self readers, fixture coverage for command names with spaces and parentheses, signed deltas for before-and-after samples, and a live Linux example.

These counters are not page-placement proof. They are supporting evidence for first-touch, THP, and warmup benchmark interpretation.

## Next Questions

- Should the mapped scratch THP benchmark sample fault counters before and after each measured allocation path?
- Should Locus add `perf_event_open` page-fault counters later to separate process-wide noise from benchmark-scoped events?
