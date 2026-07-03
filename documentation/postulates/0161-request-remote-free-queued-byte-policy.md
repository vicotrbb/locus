# Postulate 0161: Request Remote-Free Queued-Byte Policy

Date: 2026-07-03

## Claim

A queued-byte remote-free drain threshold can match the max-wait-2 request
remote-free policy counters while using a budget derived from arena capacity
and target pending request count.

## Rationale

Experiments 0166 and 0168 showed that queued-byte thresholds can match
age-based max-wait-2 policy counters on generic mixed-size allocations and real
KV block handles. The remaining request-scratch policy benchmark still tests
only end-drain and max-wait-2.

Request scratch returns retain whole request arenas until the owner closes
them. A queued-byte threshold derived from arena capacity is therefore a direct
way to express retained-memory budget without relying on scheduler burst age.

## Experiment

Add a request remote-free policy benchmark case using:

```text
RemoteFreeDrainPolicy::with_max_queued_bytes(8 * 32768)
```

Keep the workload unchanged:

- 16 request arenas;
- four bursts of four request IDs;
- 32 KiB arena capacity;
- 64 allocations of 256 bytes per request;
- queue capacity 16;
- drain batch limit 8.

Compare repeated sample counters and Criterion timings against end-drain and
max-wait-2.

## Falsification

The postulate is weakened if the queued-byte policy fails to cap peak queued
arena bytes at 262,144, increases max wait above 2 bursts, adds queue
backpressure, or requires moving `RequestScratchPool::close_request` out of
the owner drain closure.

## Expected Value

If the postulate survives, the queued-byte remote-free policy has evidence on
request-affine arena returns in addition to KV handles and mixed allocation
traces.
