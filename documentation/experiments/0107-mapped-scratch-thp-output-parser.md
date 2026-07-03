# Experiment 0107: Mapped Scratch THP Output Parser

Date: 2026-07-02

## Postulate

[Postulate 0099: Mapped Scratch THP Output Parser](../postulates/0099-mapped-scratch-thp-output-parser.md)

## Change

Added a parser for `mapped_scratch_thp` output in `locus-alloc`.

The parser extracts:

- probe run status from `mapped_scratch_thp=<started|unsupported-platform>`;
- advice mode from `mode=<hugepage|no_hugepage>`;
- advice status from `thp_advice=<ok|error>`;
- touched page count after successful advice;
- optional numeric `kernel_page_kb`;
- THP observation status and stable reason from `thp_observed=<yes|no|unknown> reason=<token>`.

The parser accepts unsupported platform output and advice-error output without requiring touch or observation lines.

## Commands

```text
cargo fmt --all
cargo test -p locus-alloc
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
docker run --rm -v "$PWD":/work -w /work rust:1.96 cargo test -p locus-alloc
```

## Results

Host focused tests passed:

```text
test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Workspace tests passed after the final refactor. Final clippy passed with `-D warnings`.

The first clippy run rejected the parser because the public function exceeded the crate line limit:

```text
error: this function has too many lines (101/100)
```

Refactoring the parser into a private fields-and-finish struct fixed the line count. The next clippy run rejected an internal `Option<Option<usize>>` for `kernel_page_kb`; replacing it with an explicit seen flag and optional numeric value fixed that issue. A final clippy run caught one unnecessary semicolon, which was removed.

Docker focused tests passed:

```text
test result: ok. 53 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The Docker run included Linux-only mapped scratch THP advice coverage and the new parser coverage.

## Conclusion

The postulate survived. `locus-alloc` can now parse the THP probe output while keeping `thp_advice=ok` separate from `thp_observed=yes`.

This gives future validation gates a stable way to consume transparent huge page evidence without inferring adoption from accepted advisory calls.
