# Postulate 0082: Mapped Scratch Lock Output Parser

Date: 2026-07-02

## Statement

The mapped scratch lock probe should have a focused parser for its stable page-lock status lines.

## Rationale

`mapped_scratch_lock` now prints machine-readable `page_lock=<status>` and `page_unlock=<status>` lines. Automation should be able to consume those lines without parsing free-form error details.

Parsing the stable status lines is the next step toward treating page-lock readiness as a validation signal for future pinned host staging buffers.

## Experiment

Add parser types and functions in `locus-alloc` for:

- `page_lock=<ok|error>`;
- `page_unlock=<ok|error>`;
- full multiline `mapped_scratch_lock` output.

The multiline parser should require a page-lock line and allow the page-unlock line to be absent when locking fails.

## Expected Result

The parser should accept the current Docker success output, reject malformed or duplicate status lines, and ignore free-form detail lines.
