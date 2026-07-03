# Postulate 0182: Remote-Free Service Retune Candidate Planner

Date: 2026-07-03

## Claim

`RemoteFreeServiceRetuneSummary` should feed a non-mutating candidate planner
that recommends the next remote-free benchmark case without changing runtime
queue capacity, drain cadence, or queued-byte budgets.

## Rationale

Experiment 0189 gave Locus a service-level observation source. Adaptive policy
still needs a decision boundary between observation and mutation. A candidate
planner can make that boundary explicit by turning service telemetry into a
named benchmark candidate while keeping production behavior unchanged.

The planner should be conservative:

- no reports means collect more telemetry;
- no retune signals means keep the current config;
- queue backpressure alone means benchmark larger queue capacity;
- retained-window drift means benchmark earlier owner drains;
- byte-shape drift means review the queued-byte budget;
- combined backpressure and retained-window drift means benchmark capacity plus
  earlier drains.

## Experiment

Add a public `RemoteFreeServiceRetuneCandidate` with stable labels and a
`from_summary` planner.

Validate it with focused tests and wire it into the service telemetry
benchmark:

- the fixed-policy four-owner service should report `keep_config`;
- the service with one end-drain owner should report `drain_earlier`.

## Falsification

The postulate fails if the planner mutates policy, cannot distinguish empty
telemetry from a clean service, chooses a weaker candidate when stronger drift
signals are present, or changes the service telemetry benchmark counters.

## Expected Value

If the postulate survives, the next adaptive policy experiment can benchmark a
concrete candidate selected from service telemetry instead of embedding
decision logic inside benchmark code.
