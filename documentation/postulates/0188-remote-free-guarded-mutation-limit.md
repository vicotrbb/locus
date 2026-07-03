# Postulate 0188: Remote-Free Guarded Mutation Limit

Date: 2026-07-03

## Claim

The guarded service retune planner should enforce its mutation limit after
confirmed candidate applications: once the configured apply budget is used, a
new stable actionable candidate should emit `mutation_limit_reached` instead
of another apply decision.

## Rationale

Experiment 0195 validated guarded apply, confirm, and rollback decisions. The
remaining guard path before a production-facing application API is the mutation
limit. Without a measured cap, a service could keep changing remote-free
policy in response to a long run of telemetry windows even when each local
decision appears stable.

## Experiment

Extend the guarded service benchmark with a mutation-limit sequence using real
owner-loop cases:

- clean fixed-policy window;
- two end-drain windows that stabilize `drain_earlier`;
- one explicit `drain_earlier` candidate window that confirms cleanly;
- two capacity-128 end-drain windows that stabilize the combined candidate;
- one explicit combined candidate window that confirms cleanly;
- two more end-drain windows that stabilize `drain_earlier` again.

With `max_mutations=2`, the final stable `drain_earlier` window should return
`mutation_limit_reached` and leave no pending candidate.

## Falsification

The postulate fails if the third stable actionable candidate returns `apply`,
if a pending validation candidate remains after the limit decision, if the
confirmed mutation count changes unexpectedly, or if real submitted, drained,
or released byte counters differ from the measured service windows.

## Expected Value

If the postulate survives, Locus will have measured all guarded decision paths
needed before designing a production-facing policy application API.
