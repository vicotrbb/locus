# Postulate 0204: Remote-Free Local Dirty Threshold Flush

Date: 2026-07-03

## Claim

Flushing a local dirty-owner buffer after the worker accumulates the configured
target pending item window can provide earlier dirty-owner visibility than
before-collection flushing while avoiding the fixed cost of flushing once per
burst.

## Rationale

Experiment 0211 rejected fixed end-of-burst local dirty flushing for the
current owner-window shape. It preserved correctness, but it paid 8 tracker
flushes per owner window and measured slower than direct tracker marking.

The queued-byte drain config already has a retained item window. Using that
window as the local dirty flush threshold should align dirty-owner visibility
with the same unit that bounds retained remote-free work. For the current
256-submit owner window and 64-item target, the worker should flush 4 times
instead of 8 burst flushes or 1 before-collection flush.

## Experiment

Add a real allocation benchmark path that:

- records local dirty marks only after successful enqueue attempts;
- flushes the local buffer into the shared tracker when local successful marks
  reach `TARGET_PENDING_BLOCKS`;
- flushes any remaining local marks before service-window collection;
- preserves tracker generation safety across repeated threshold flushes;
- compares the threshold path against before-collection local flushing and
  direct dirty-enqueue tracker marking.

## Falsification

The postulate fails if threshold flushing changes allocation counters, marks
failed enqueue attempts, loses tracker generations, or measures slower than
direct dirty-enqueue tracker marking in the same real allocation sequence.

## Expected Value

If the postulate survives, Locus will have a measured lower-latency dirty mark
cadence tied to the retained item window instead of an arbitrary burst cadence.
