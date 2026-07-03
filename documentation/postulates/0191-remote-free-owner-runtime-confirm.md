# Postulate 0191: Remote-Free Owner Runtime Confirm

Date: 2026-07-03

## Claim

A remote-free owner runtime needs an explicit confirm operation after a guarded
candidate validates cleanly. Confirmation should clear rollback state only at
an empty owner boundary and should not rebuild or change the active queue and
controller config.

## Rationale

Experiment 0198 added install and rollback for `RemoteFreeOwnerRuntime`.
Rollback state is intentionally retained after install so a failed validation
window can restore the previous config. A successful validation window needs a
separate owner operation that clears the previous config. Without it, a later
rollback could restore stale state after a candidate has already been
accepted.

## Experiment

Add a runtime confirm operation that:

- succeeds only when queue pending count, controller pending count, and
  controller queued bytes are zero;
- clears the previous rollback config;
- preserves the active config and queue sizing;
- is a no-op when no rollback config is present;
- rejects confirmation while pending work exists.

Extend the real allocation runtime benchmark with a sequence that installs a
larger queue config, confirms it after a clean owner window, installs the same
config again from the original config, then rolls it back after another owner
window. The benchmark should keep real `Vec<u8>` allocation and owner-side
release counters.

## Falsification

The postulate fails if confirm clears rollback state while work is pending, if
confirm changes the active config, if stale rollback state remains after
confirm, or if the benchmark real allocation counters change unexpectedly.

## Expected Value

If the postulate survives, `RemoteFreeOwnerRuntime` will have the complete
install, confirm, and rollback operations needed before wiring guarded service
decisions through one runtime-owned path.
