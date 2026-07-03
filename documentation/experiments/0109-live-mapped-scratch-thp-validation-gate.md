# Experiment 0109: Live Mapped Scratch THP Validation Gate

Date: 2026-07-03

## Postulate

[Postulate 0101: Live Mapped Scratch THP Validation Gate](../postulates/0101-live-mapped-scratch-thp-validation-gate.md)

## Change

Added `live_mapped_scratch_thp_validation_gate` to `locus-validate`.

On Linux, the example:

- accepts optional `hugepage` or `no_hugepage` mode;
- creates a 4 MiB `MappedScratchArena`;
- applies the requested THP advice;
- write-touches the arena;
- reads `/proc/self/numa_maps`;
- emits stable mapped scratch THP probe lines;
- evaluates those lines with the mapped scratch THP validation gate;
- prints the final `mapped_scratch_thp_validation_gate=<status> reason=<reason>` line.

On non-Linux targets, it emits `mapped_scratch_thp=unsupported-platform` followed by the corresponding unavailable gate.

## Commands

```text
cargo fmt --all
cargo run -p locus-validate --example live_mapped_scratch_thp_validation_gate
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-validate --example live_mapped_scratch_thp_validation_gate
cargo test -p locus-validate
cargo test --workspace
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-validate
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
rg "$(printf '\342\200\224')" README.md crates/locus-validate/examples/live_mapped_scratch_thp_validation_gate.rs documentation/postulates/0101-live-mapped-scratch-thp-validation-gate.md documentation/experiments/0109-live-mapped-scratch-thp-validation-gate.md
```

## Results

Focused host tests passed:

```text
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Workspace tests passed.

Docker focused tests passed:

```text
test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Clippy passed with `-D warnings`.

Rustfmt check and `git diff --check` passed. The touched-file em dash scan found no matches.

Host output:

```text
mapped_scratch_thp=unsupported-platform
mapped_scratch_thp_validation_gate=unavailable reason=unsupported_platform
```

Docker output:

```text
mapped_scratch_thp=started mode=hugepage
mapping_start=0xffffa983f000
mapping_len=4198399
base_page_kb=4
thp_advice=ok mode=hugepage
touched=1025
numa_maps=unavailable
thp_observed=unknown reason=numa_maps_unavailable
mapped_scratch_thp_validation_gate=unavailable reason=observation_unavailable
```

## Conclusion

The postulate survived. Locus now has a one-command live validation path for mapped scratch THP evidence.

The Docker result confirms the conservative behavior: accepted THP advice is not reported as ready without live page-size evidence.
