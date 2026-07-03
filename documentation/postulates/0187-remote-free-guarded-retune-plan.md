# Postulate 0187: Remote-Free Guarded Retune Plan

Date: 2026-07-03

## Claim

A guarded service retune planner should turn dry-run stability into explicit
non-automatic plan decisions: apply an actionable candidate only after the
configured stable-window count, confirm it only after a clean follow-up
service window, roll it back if the follow-up window still needs retuning, and
stop applying candidates after the configured mutation limit.

## Rationale

Experiments 0193 and 0194 showed that `RemoteFreeServiceRetuneDryRunPlanner`
can distinguish stable actionable candidates from oscillating candidates. The
next step toward live adaptation is a guard layer that records when mutation
would be attempted and when it would be confirmed or rolled back. The guard
must still leave actual policy mutation to the caller so benchmark code can
exercise and validate the candidate before production policy changes exist.

## Experiment

Add a public guarded planner over `RemoteFreeServiceRetuneSummary` that:

- uses the existing dry-run planner for stable candidate detection;
- returns an `apply` decision only for actionable candidates;
- limits total apply decisions with a configured mutation cap;
- marks one applied candidate as pending validation;
- returns `confirmed` after the next clean service window;
- returns `rollback` after the next non-clean service window;
- exposes stable labels and counters for benchmark output.

Extend the service telemetry benchmark with a guarded adaptive sequence that
uses real owner-loop cases:

- clean fixed-policy window;
- two end-drain windows that stabilize `drain_earlier`;
- one explicit `drain_earlier` candidate window that should confirm cleanly;
- two capacity-128 end-drain windows that stabilize the combined candidate;
- one explicit combined candidate window that should confirm cleanly.

Add a second guarded sequence that deliberately validates `drain_earlier`
against another end-drain window so the guard must return `rollback` with real
allocation and release accounting still intact.

## Falsification

The postulate fails if a candidate is applied before the stability threshold,
if oscillating or clean windows produce an apply decision, if confirmation does
not require a clean follow-up window, if a failed follow-up window is not
rolled back, if the mutation cap is exceeded, or if the adaptive benchmark
changes real submitted, drained, or released byte counters.

## Expected Value

If the postulate survives, Locus will have a measured guard layer between
dry-run telemetry and future live remote-free policy mutation.
