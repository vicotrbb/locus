# Experiment 0104: Linux Transparent Huge Page Advice

Date: 2026-07-02

## Postulate

[Postulate 0096: Linux Transparent Huge Page Advice](../postulates/0096-linux-transparent-huge-page-advice.md)

## Change

Added Linux-only transparent huge page advice to `locus-sys`:

- `LinuxTransparentHugePageAdvice`;
- `LinuxTransparentHugePageAdviceError`;
- `advise_region_transparent_huge_pages`.

The helper applies `MADV_HUGEPAGE` or `MADV_NOHUGEPAGE` to an owned `MappedRegion`. This is advisory only. It does not prove huge page promotion.

## Commands

```text
cargo test -p locus-sys
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-sys
```

## Results

Host `cargo test -p locus-sys` passed on the non-Linux host path:

```text
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Clippy passed with `-D warnings`.

Docker Linux `cargo test -p locus-sys` passed and exercised both transparent huge page advice modes:

```text
running 18 tests
test linux::tests::applies_transparent_huge_page_advice ... ok
test linux::tests::reports_transparent_huge_page_advice_errors ... ok
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Conclusion

The postulate survives. `locus-sys` now has a narrow safe wrapper for Linux transparent huge page advice on owned mapped regions.

This is only the control primitive. A later allocator-level experiment should expose opt-in THP hints on `MappedScratchArena` and pair the hint with observability from `numa_maps` or `smaps` before making any huge page adoption claim.
