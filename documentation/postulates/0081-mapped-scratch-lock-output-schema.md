# Postulate 0081: Mapped Scratch Lock Output Schema

Date: 2026-07-02

## Statement

The mapped scratch lock probe should keep machine-readable status tokens separate from free-form error details.

## Rationale

The initial probe prints `page_lock=ok` on success, but a failure would print `page_lock=error <error text>`. That mixes a stable status token with free-form display text on the same line.

Keeping the status line as `page_lock=<status>` and printing error details on a separate line makes future parsing stricter and less fragile.

## Experiment

Update `mapped_scratch_lock` to print:

- `page_lock=ok` or `page_lock=error`;
- `page_lock_error=<error text>` only on lock failure;
- `page_unlock=ok` or `page_unlock=error`;
- `page_unlock_error=<error text>` only on unlock failure.

Run the probe in Docker and verify the success output remains stable.

## Expected Result

The current Docker run should still print `page_lock=ok` and `page_unlock=ok`. Failure output should have stable status lines with free-form details separated.
