# Postulate 0165: Remote-Free Heterogeneous Budget Helper

Date: 2026-07-03

## Claim

`RemoteFreeQueuedByteBudget` should support heterogeneous retained-work shapes
by deriving a budget from an iterator of item sizes.

## Rationale

Experiment 0172 validated `RemoteFreeQueuedByteBudget` in the uniform KV and
request benchmark paths. The mixed-size remote-free benchmark still keeps an
explicit `TRACE_TARGET_QUEUED_BYTES` constant because its queued-byte threshold
represents two bursts of variable-size allocation records.

That path should not be forced through the uniform item-shape API. A separate
heterogeneous constructor can express the benchmark's actual retained work:

- every pending item contributes its own retained byte size;
- empty retained-work sequences are invalid;
- zero-sized retained items are invalid for a drain threshold;
- byte summation must be checked for overflow.

## Experiment

Add a `RemoteFreeQueuedByteBudget::from_item_sizes` constructor that accepts an
iterator of `u64` item sizes, sums the retained bytes with checked addition,
and returns the existing typed non-zero budget.

Wire the mixed-size queued-byte benchmark case through this constructor by
passing the two-burst retained allocation sizes that make up the 655,360-byte
threshold.

## Falsification

The postulate is weakened if the helper obscures the benchmark trace shape,
allows empty or zero-sized retained work, misses sum overflow, or changes the
mixed-size queued-byte benchmark counters.

## Expected Value

If the postulate survives, queued-byte budget derivation will cover both
uniform domain allocators and heterogeneous allocation traces with explicit,
checked APIs.
