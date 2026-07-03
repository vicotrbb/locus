# Experiment 0170: Queued-Byte Policy Evidence Summary

Date: 2026-07-03

## Postulate

[Postulate 0162](../postulates/0162-queued-byte-policy-evidence-summary.md)
claimed that the best-results note should record queued-byte remote-free policy
evidence as a cross-domain validation result, not only as scattered
per-benchmark experiments.

## Change

Updated `documentation/dev-notes/2026-07-03-best-benchmark-results.md` with a
new best validation row for `RemoteFreeDrainPolicy::with_max_queued_bytes`.

The row links the three domain experiments that proved the result:

- [Experiment 0166](0166-remote-free-queued-byte-policy.md), mixed-size
  allocation trace;
- [Experiment 0168](0168-kv-remote-free-queued-byte-policy.md), real KV block
  handles;
- [Experiment 0169](0169-request-remote-free-queued-byte-policy.md),
  request-affine arena returns.

The current interpretation was refined to say that queued-byte policy evidence
is now cross-domain and counter-based.

## Validation

Commands:

```bash
rg -n "Queued-byte remote-free policy|cross-domain|with_max_queued_bytes" documentation/dev-notes/2026-07-03-best-benchmark-results.md
rg -n $'\342\200\224|\342\200\223' documentation/dev-notes/2026-07-03-best-benchmark-results.md documentation/postulates/0162-queued-byte-policy-evidence-summary.md documentation/experiments/0170-queued-byte-policy-evidence-summary.md
LC_ALL=C rg -n "[^[:ascii:]]" documentation/dev-notes/2026-07-03-best-benchmark-results.md documentation/postulates/0162-queued-byte-policy-evidence-summary.md documentation/experiments/0170-queued-byte-policy-evidence-summary.md
git diff --check
```

Results:

- Best-results note contains the queued-byte validation row and interpretation
  bullet.
- Dash hygiene check passed with no matches.
- ASCII hygiene check passed with no matches.
- `git diff --check` passed.

## Interpretation

The postulate survived.

The best-results note still preserves timing bests separately from validation
evidence. The queued-byte policy is now captured as a cross-domain retained-byte
policy candidate:

- mixed-size trace: 655,360 peak queued bytes, max wait 2 bursts,
  `full_count=0`;
- KV handles: 262,144 peak queued bytes, max wait 2 bursts, `full_count=0`;
- request arenas: 262,144 peak queued bytes, max wait 2 bursts,
  `full_count=0`.

This is not a new throughput claim. It is a stronger summary of the current
runtime-policy evidence.

## Next Step

Use the queued-byte policy evidence to design a small runtime configuration
surface or helper that derives retained-byte budgets from request concurrency,
KV block size, arena size, and target release latency.
