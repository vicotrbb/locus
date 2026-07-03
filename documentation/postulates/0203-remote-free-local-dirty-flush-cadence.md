# Postulate 0203: Remote-Free Local Dirty Flush Cadence

Date: 2026-07-03

## Claim

Flushing a local dirty-owner buffer once per worker burst can preserve tracked
dirty-owner correctness while keeping most of the enqueue-side synchronization
reduction from local buffering.

## Rationale

Experiment 0210 measured local dirty-owner buffering when the worker flushed
once immediately before service-window collection. That minimized shared
tracker touches in the benchmark, but a live service might need dirty-owner
visibility earlier than the end of a full owner window.

End-of-burst flushing is the next candidate because it bounds dirty visibility
by the owner-loop control interval while still deduplicating repeated owner
marks inside each burst.

## Experiment

Add a real allocation benchmark path that:

- records local dirty marks only after successful enqueue attempts;
- flushes the local buffer into the shared tracker after each worker burst;
- preserves tracker generation safety when repeated burst flushes touch an
  owner already pending in the tracker;
- collects through the same tracked dirty service-window path;
- compares the burst-flush path against the current before-collection local
  flush path and the direct dirty-enqueue tracker path.

## Falsification

The postulate fails if burst flushing changes allocation counters, clears a
newer tracker generation incorrectly, marks failed enqueue attempts, or costs
enough that it no longer preserves the practical benefit of local buffering
against direct tracker marking.

## Expected Value

If the postulate survives, Locus will have measured guidance for choosing
between lowest-overhead before-collection flushing and lower-latency burst
flushing in worker-owned remote-free submit loops.
