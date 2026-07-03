# Postulate 0166: Remote-Free Budget Selection Note

Date: 2026-07-03

## Claim

A focused runtime configuration note should capture how to select queued-byte
remote-free thresholds from workload shape inputs using the validated budget
helper APIs.

## Rationale

Experiments 0171, 0172, and 0173 proved that `RemoteFreeQueuedByteBudget` can
derive retained-byte thresholds for:

- grouped uniform shapes in the owner-loop example;
- uniform item-count and item-size shapes in KV and request benchmarks;
- heterogeneous item-size traces in the mixed-size benchmark.

The evidence is now spread across helper experiments, benchmark experiments,
and the best-results note. Runtime users need one short note that says which
API to use for each workload shape, which counters proved the selection, and
which limits still prevent this from becoming a production default.

## Experiment

Add a dev note that documents:

- when to use queued-byte remote-free policy;
- how to derive budgets for grouped, uniform, and heterogeneous retained work;
- the measured thresholds and counters from mixed-size, KV, request, and
  owner-loop paths;
- current guardrails for queue capacity, batch size, and release-latency
  validation;
- open questions before this becomes a default runtime policy.

## Falsification

The postulate is weakened if the note overstates benchmark evidence, omits the
validation limits, fails to map shape inputs to `RemoteFreeQueuedByteBudget`
APIs, or conflicts with the recorded benchmark counters.

## Expected Value

If the postulate survives, future runtime configuration work can start from a
single evidence-backed note instead of rediscovering the queued-byte policy
rules from scattered benchmark logs.
