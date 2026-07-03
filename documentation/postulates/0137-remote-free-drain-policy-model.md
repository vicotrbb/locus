# Postulate 0137: Remote-Free Drain Policy Model

Date: 2026-07-03

## Claim

The remote-free owner drain decision should be represented as a small pure policy model before it is wired into queue draining or scheduler code.

## Rationale

Experiment 0144 showed that a latency-bounded policy reduced peak queued bytes from 2,621,440 to 655,360 without increasing `full_count`. That benchmark encoded the policy directly in the trace harness. Keeping policy logic inside benchmark code makes it harder to reuse, test, or connect to future request schedulers.

A reusable policy model should take only the observable inputs needed for a drain decision:

- pending item count;
- queued bytes;
- oldest pending age in logical turns.

It should return a deterministic drain or defer decision, with a reason when draining is requested.

## Experiment

Add public remote-free drain policy types to `locus-alloc`:

- an observation type for pending items, queued bytes, and oldest pending age;
- a policy type with optional thresholds for pending items, queued bytes, and age;
- a decision type that reports whether to drain and why.

The model should be pure and covered by focused unit tests. It should not change existing `RemoteFreeQueue` behavior.

## Falsification

The postulate is weakened if the model makes the queue API harder to use, changes existing queue semantics, or cannot express the policy that survived experiment 0144.

## Expected Value

If the postulate survives, the next step can wire this policy into a benchmark or runtime owner loop without duplicating ad hoc threshold logic.
