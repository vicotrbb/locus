# Postulate 0158: Remote-Free Queued-Byte Policy

Date: 2026-07-03

## Claim

A queued-byte remote-free drain threshold can match the retained-memory bound
of the current max-wait-2 mixed-size policy without relying on logical burst
age.

## Rationale

The mixed-size trace repeatedly showed that max-wait-2 reduces peak retained
remote-free bytes from 2,621,440 to 655,360 while keeping `full_count=0`.
That is strong evidence for owner-side draining, but the trigger is age-based.
Age is useful when scheduler turns are meaningful, yet memory pressure is more
directly expressed as queued bytes.

The production `RemoteFreeDrainPolicy` already supports a queued-byte budget.
The next benchmark should test it directly against the existing end-drain and
max-wait-2 policies on the same mixed-size workload.

## Experiment

Add a mixed-size benchmark case using:

```text
RemoteFreeDrainPolicy::with_max_queued_bytes(655360)
```

Keep the workload unchanged:

- 256 real `Vec` allocations;
- eight bursts of 32 blocks;
- existing mixed-size allocation pattern;
- capacity 256 and batch limit 64.

Compare repeated summaries and Criterion timings against end-drain and
max-wait-2.

## Falsification

The postulate is weakened if the queued-byte threshold fails to keep peak
queued bytes near 655,360, adds producer backpressure, increases max wait above
2 bursts, or compiles only by changing the mixed-size workload.

## Expected Value

If the postulate survives, retained-byte policy can become the next remote-free
runtime candidate because it binds memory pressure directly instead of relying
only on scheduler age.
