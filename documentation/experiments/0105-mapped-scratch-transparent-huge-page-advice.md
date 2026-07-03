# Experiment 0105: Mapped Scratch Transparent Huge Page Advice

Date: 2026-07-02

## Postulate

[Postulate 0097: Mapped Scratch Transparent Huge Page Advice](../postulates/0097-mapped-scratch-transparent-huge-page-advice.md)

## Change

Added allocator-facing transparent huge page advice to `MappedScratchArena`:

- `MappedScratchHugePageAdvice`;
- `MappedScratchArena::advise_transparent_huge_pages`;
- `MappedScratchAllocError::LinuxTransparentHugePageAdvice`.

The API is Linux-only and maps to the `locus-sys` `madvise` wrapper. It keeps huge page hints tied to an arena object while preserving the narrow system boundary.

## Commands

```text
cargo test -p locus-alloc
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-alloc
```

## Results

Host `cargo test -p locus-alloc` passed on the non-Linux host path:

```text
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Clippy passed with `-D warnings`.

Docker Linux `cargo test -p locus-alloc` passed and exercised the new arena advice method:

```text
running 49 tests
test tests::mapped_scratch_arena_applies_transparent_huge_page_advice ... ok
test result: ok. 49 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Conclusion

The postulate survives. `MappedScratchArena` now has an opt-in Linux transparent huge page hint API without exposing syscall details to allocator users.

This still does not prove huge page promotion. The next step should be a probe that applies the hint, write-touches the arena, and inspects `numa_maps` or `smaps` evidence such as `kernelpagesize_kB` before making any huge page adoption claim.
