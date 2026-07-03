# Experiment 0174: Remote-Free Budget Selection Note

Date: 2026-07-03

## Postulate

[Postulate 0166](../postulates/0166-remote-free-budget-selection-note.md)
claimed that a focused runtime configuration note should capture how to select
queued-byte remote-free thresholds from workload shape inputs using the
validated budget helper APIs.

## Change

Added `documentation/dev-notes/2026-07-03-remote-free-budget-selection.md`.

The note records:

- when queued-byte remote-free policy is appropriate;
- how to map grouped, uniform, heterogeneous, and already-validated byte
  shapes to `RemoteFreeQueuedByteBudget` APIs;
- measured thresholds and matching counters for owner-loop, mixed-size, KV,
  and request paths;
- guardrails around queue capacity, batch size, byte accounting, and production
  default claims;
- open questions for future runtime configuration work.

## Validation

Commands:

```bash
for f in documentation/experiments/0166-remote-free-queued-byte-policy.md documentation/experiments/0167-remote-free-queued-byte-owner-loop-example.md documentation/experiments/0168-kv-remote-free-queued-byte-policy.md documentation/experiments/0169-request-remote-free-queued-byte-policy.md documentation/experiments/0171-remote-free-queued-byte-budget-helper.md documentation/experiments/0172-remote-free-uniform-benchmark-budget-helper.md documentation/experiments/0173-remote-free-heterogeneous-budget-helper.md; do test -f "$f" || exit 1; done
rg -n $'\342\200\224|\342\200\223' documentation/dev-notes/2026-07-03-remote-free-budget-selection.md documentation/postulates/0166-remote-free-budget-selection-note.md documentation/experiments/0174-remote-free-budget-selection-note.md
LC_ALL=C rg -n "[^[:ascii:]]" documentation/dev-notes/2026-07-03-remote-free-budget-selection.md documentation/postulates/0166-remote-free-budget-selection-note.md documentation/experiments/0174-remote-free-budget-selection-note.md
git diff --check
```

Results:

- Every cited experiment file exists.
- Dash hygiene check passed with no matches.
- ASCII hygiene check passed with no matches.
- `git diff --check` passed.

## Interpretation

The postulate survived.

The new note gives future runtime configuration work one evidence-backed place
to start. It preserves the current limits:

- queued-byte thresholds are validated as counter behavior, not as production
  defaults;
- queue capacity and batch size still require workload-specific validation;
- retained-byte policy requires owner-side submit and drain byte accounting;
- heterogeneous traces should use actual retained item sizes unless an average
  is separately validated.

No new performance claim was added.

## Next Step

Use the selection note to guide a small runtime configuration type only after
we decide whether queue capacity, drain batch size, and queued-byte budget
should be validated together.
