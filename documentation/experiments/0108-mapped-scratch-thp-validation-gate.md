# Experiment 0108: Mapped Scratch THP Validation Gate

Date: 2026-07-03

## Postulate

[Postulate 0100: Mapped Scratch THP Validation Gate](../postulates/0100-mapped-scratch-thp-validation-gate.md)

## Change

Added a mapped scratch THP validation gate to `locus-validate`.

The gate prints:

```text
mapped_scratch_thp_validation_gate=<ready|unavailable|not_ready> reason=<reason>
```

The evaluator reports `ready` only when the parsed probe used `hugepage` mode, advice succeeded, and `thp_observed=yes`. It reports `unavailable` for unsupported platforms or missing observation evidence, and `not_ready` for advice failure, `no_hugepage` mode, or observed base pages.

Added `mapped_scratch_thp_validation_gate`, a file-based example that reads captured `mapped_scratch_thp` output and prints the final gate line.

## Commands

```text
cargo fmt --all
cargo test -p locus-validate
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
rg "$(printf '\342\200\224')" README.md crates/locus-validate/src/lib.rs crates/locus-validate/examples/mapped_scratch_thp_validation_gate.rs documentation/postulates/0100-mapped-scratch-thp-validation-gate.md documentation/experiments/0108-mapped-scratch-thp-validation-gate.md
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-validate
printf '%s\n' 'mapped_scratch_thp=started mode=hugepage' 'thp_advice=ok mode=hugepage' 'touched=1025' 'kernel_page_kb=2048' 'thp_observed=yes reason=kernel_page_size' | cargo run -p locus-validate --example mapped_scratch_thp_validation_gate -- /dev/stdin
cargo run -p locus-alloc --example mapped_scratch_thp | cargo run -p locus-validate --example mapped_scratch_thp_validation_gate -- /dev/stdin
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-alloc --example mapped_scratch_thp | cargo run -p locus-validate --example mapped_scratch_thp_validation_gate -- /dev/stdin
```

## Results

Focused host tests passed:

```text
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Workspace tests passed.

Clippy passed with `-D warnings`. Rustfmt check and `git diff --check` passed. The touched-file em dash scan found no matches.

Docker focused tests passed:

```text
test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Synthetic ready output:

```text
mapped_scratch_thp_validation_gate=ready reason=ready
```

Host live pipeline output:

```text
mapped_scratch_thp_validation_gate=unavailable reason=unsupported_platform
```

Docker live probe classified by the host validator:

```text
mapped_scratch_thp_validation_gate=unavailable reason=observation_unavailable
```

The Docker result is conservative: the kernel accepted the advice, but the probe did not expose `numa_maps` page-size evidence.

## Conclusion

The postulate survived. Locus now has a stable validation gate for mapped scratch THP evidence, and the gate does not treat accepted advice as proof of huge page adoption.

The current Docker environment remains unavailable for THP adoption proof because `numa_maps` evidence is unavailable.
