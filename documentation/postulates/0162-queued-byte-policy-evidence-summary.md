# Postulate 0162: Queued-Byte Policy Evidence Summary

Date: 2026-07-03

## Claim

The best-results note should record queued-byte remote-free policy evidence as
a cross-domain validation result, not only as scattered per-benchmark
experiments.

## Rationale

Experiments 0166, 0168, and 0169 showed the same policy pattern across three
paths:

- mixed-size allocation traces;
- real KV block handles;
- request-affine arena returns.

In each path, a queued-byte threshold matched the current max-wait-2 counters
while expressing the policy in retained bytes instead of scheduler burst age.
This is not a new throughput best, but it is important runtime-policy evidence.
It should be easy to find from the best-results note alongside THP validation
and controller behavior preservation.

## Experiment

Update `documentation/dev-notes/2026-07-03-best-benchmark-results.md` to add a
best validation row for queued-byte remote-free policy evidence and refine the
current interpretation bullets accordingly.

## Falsification

The postulate is weakened if the best-results note presents the queued-byte
policy as a new timing best, omits that evidence is counter-based, or fails to
link to the domain experiments that prove the result.

## Expected Value

If the postulate survives, future allocator design work can find the strongest
queued-byte policy evidence quickly and use it to choose the next runtime
configuration or helper work.
