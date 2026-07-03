# Experiment 0155: Mapped Scratch THP Smaps Fallback

Date: 2026-07-03

## Postulate

[Postulate 0147](../postulates/0147-mapped-scratch-thp-smaps-fallback.md)
claimed that the mapped scratch THP probe should fall back to
`/proc/self/smaps` `KernelPageSize` evidence when `/proc/self/numa_maps` is
unavailable or does not contain the target mapping.

## Change

Added `smaps` mapping evidence to `locus-observe`:

- `SmapsEntry`;
- `parse_smaps`;
- `read_smaps`;
- `read_self_smaps`;
- `smaps_entry_for_address`.

Updated `mapped_scratch_thp` and `live_mapped_scratch_thp_validation_gate` so
they try `numa_maps` first, then fall back to `smaps` before emitting one final
`kernel_page_kb` and `thp_observed` verdict.

## Validation

Host commands:

```bash
cargo test -p locus-observe smaps
cargo test -p locus-alloc mapped_scratch_thp
cargo test -p locus-validate mapped_scratch_thp_gate
cargo run -p locus-alloc --example mapped_scratch_thp
cargo run -p locus-validate --example live_mapped_scratch_thp_validation_gate
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Host results:

- `cargo test -p locus-observe smaps`: passed, 4 focused tests.
- `cargo test -p locus-alloc mapped_scratch_thp`: passed, 11 focused tests.
- `cargo test -p locus-validate mapped_scratch_thp_gate`: passed, 10
  focused tests.
- Host `mapped_scratch_thp`: reported `mapped_scratch_thp=unsupported-platform`
  on macOS.
- Host `live_mapped_scratch_thp_validation_gate`: reported
  `mapped_scratch_thp=unsupported-platform` and
  `mapped_scratch_thp_validation_gate=unavailable reason=unsupported_platform`
  on macOS.
- `cargo test --workspace`: passed, 171 tests plus doc tests.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.

Docker commands:

```bash
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-alloc --example mapped_scratch_thp
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-alloc --example mapped_scratch_thp -- no_hugepage
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-validate --example live_mapped_scratch_thp_validation_gate
```

Docker `hugepage` result summary:

```text
mapped_scratch_thp=started mode=hugepage
base_page_kb=4
thp_advice=ok mode=hugepage
touched=1025
numa_maps=unavailable
smaps=available entries=25
smaps_match=containing_range
kernel_page_kb=4
thp_observed=no reason=base_page_size
```

Docker `no_hugepage` result summary:

```text
mapped_scratch_thp=started mode=no_hugepage
base_page_kb=4
thp_advice=ok mode=no_hugepage
touched=1025
numa_maps=unavailable
smaps=available entries=25
smaps_match=containing_range
kernel_page_kb=4
thp_observed=no reason=base_page_size
```

Docker live validation gate result summary:

```text
mapped_scratch_thp=started mode=hugepage
base_page_kb=4
thp_advice=ok mode=hugepage
touched=1025
numa_maps=unavailable
smaps=available entries=25
smaps_match=containing_range
kernel_page_kb=4
thp_observed=no reason=base_page_size
mapped_scratch_thp_validation_gate=not_ready reason=base_page_size
```

## Interpretation

The postulate survived.

In Docker, `numa_maps` was unavailable, but `smaps` was readable and matched the
mapped scratch arena range. The probe now reports a concrete kernel page size
and a `base_page_size` verdict instead of ending with
`numa_maps_unavailable`. The live validation gate consumes the same evidence
shape and reports `not_ready reason=base_page_size`.

This does not prove that THP materialized. It proves the opposite for this run:
the kernel accepted the advice, pages were touched, and the matched mapping
still reported 4 KiB kernel pages. That is useful because the readiness failure
is now based on kernel page-size evidence rather than missing observability.

## Next Step

Run the mapped scratch THP validation gate in a Linux environment with THP
configured to `always` or `madvise`, then compare gate output before and after
the fallback with the same allocation size and touch pattern.
