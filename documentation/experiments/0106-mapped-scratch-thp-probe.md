# Experiment 0106: Mapped Scratch THP Probe

Date: 2026-07-02

## Postulate

[Postulate 0098: Mapped Scratch THP Probe](../postulates/0098-mapped-scratch-thp-probe.md)

## Change

Added `mapped_scratch_thp` to `locus-alloc`.

On Linux, the probe:

- accepts optional `hugepage` or `no_hugepage` mode;
- creates a 4 MiB `MappedScratchArena`;
- prints mapping identity and base page size;
- applies the selected THP advice;
- write-touches the arena;
- reads `/proc/self/numa_maps`;
- reports `kernel_page_kb` and `thp_observed` when the mapping evidence is available.

On non-Linux targets it prints `mapped_scratch_thp=unsupported-platform`.

## Commands

```text
cargo run -p locus-alloc --example mapped_scratch_thp
cargo test -p locus-alloc
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-alloc --example mapped_scratch_thp
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo run -p locus-alloc --example mapped_scratch_thp -- no_hugepage
```

## Results

Host probe output:

```text
mapped_scratch_thp=unsupported-platform
```

Host focused tests passed:

```text
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Clippy passed with `-D warnings`.

The first Docker compile attempt failed because `MappedScratchHugePageAdvice` was imported inside `main` while the `ThpMode` implementation needed it at module scope:

```text
error[E0412]: cannot find type `MappedScratchHugePageAdvice` in this scope
error[E0433]: failed to resolve: use of undeclared type `MappedScratchHugePageAdvice`
```

Moving the import to a Linux-gated module-scope item fixed the example.

Docker `hugepage` probe output:

```text
mapped_scratch_thp=started mode=hugepage
mapping_start=0xffff8753f000
mapping_len=4198399
base_page_kb=4
thp_advice=ok mode=hugepage
touched=1025
numa_maps=unavailable
thp_observed=unknown reason=numa_maps_unavailable
```

Docker `no_hugepage` probe output:

```text
mapped_scratch_thp=started mode=no_hugepage
mapping_start=0xffffa618f000
mapping_len=4198399
base_page_kb=4
thp_advice=ok mode=no_hugepage
touched=1025
numa_maps=unavailable
thp_observed=unknown reason=numa_maps_unavailable
```

## Conclusion

The postulate survives for the probe behavior. The kernel accepted both THP advice modes in Docker, and the probe clearly refused to claim huge page adoption because `numa_maps` evidence was unavailable.

This creates the evidence path needed for future Linux host runs. A host with readable `numa_maps` can now show whether the advised mapping still uses the base page size or reports a larger `kernelpagesize_kB`.
