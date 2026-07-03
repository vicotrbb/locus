# Postulate 0160: KV Remote-Free Queued-Byte Policy

Date: 2026-07-03

## Claim

A queued-byte remote-free drain threshold can match the max-wait-2 KV
remote-free policy counters while using a budget derived directly from real KV
block size and target pending block count.

## Rationale

Experiment 0166 showed that a queued-byte policy can match the age-based
max-wait-2 policy on a mixed-size `Vec` trace. Experiment 0167 showed a
runtime-shaped owner-loop example, but it still used representative `Vec`
sizes.

The KV remote-free policy benchmark already uses real `KvBlockHandle`s and
frees them inside the owner `drain_batch` closure. It is the right next place
to test whether a byte budget derived from block size can replace burst-age as
the policy trigger.

## Experiment

Add a KV remote-free policy benchmark case using:

```text
RemoteFreeDrainPolicy::with_max_queued_bytes(64 * 4096)
```

Keep the workload unchanged:

- 256 real KV block handles;
- eight bursts of 32 handles;
- block size 4096 bytes;
- queue capacity 256;
- drain batch limit 64.

Compare repeated sample counters and Criterion timings against end-drain and
max-wait-2.

## Falsification

The postulate is weakened if the queued-byte policy fails to cap peak queued KV
bytes at 262,144, increases max wait above 2 bursts, adds queue backpressure,
or requires moving `KvBlockPool::free` out of the owner drain closure.

## Expected Value

If the postulate survives, the queued-byte remote-free policy has evidence on a
real KV handle path, not only on generic `Vec` traces.
