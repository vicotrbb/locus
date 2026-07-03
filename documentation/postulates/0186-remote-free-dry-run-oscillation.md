# Postulate 0186: Remote-Free Dry-Run Oscillation

Date: 2026-07-03

## Claim

The dry-run service planner should reject oscillating actionable candidates:
with a two-window stability requirement, alternating `drain_earlier` and
`increase_queue_capacity_and_drain_earlier` service windows should never expose
a would-apply candidate.

## Rationale

Experiment 0193 showed that repeated matching actionable windows produce a
non-mutating would-apply signal and that a clean window resets the planner. The
remaining safety case before live mutation is candidate oscillation. A planner
that reacts to alternating service shapes would risk changing policy in
response to unstable workload evidence.

## Experiment

Extend the dry-run service benchmark with an oscillating sequence over the same
real owner-loop cases:

- clean fixed-policy window;
- end-drain window selecting `drain_earlier`;
- capacity-128 end-drain window selecting the combined candidate;
- end-drain window selecting `drain_earlier`;
- capacity-128 end-drain window selecting the combined candidate;
- clean fixed-policy window.

The sequence should use the existing two-window stability requirement and
should preserve the same real allocation and release checks as the stable
dry-run sequence.

## Falsification

The postulate fails if any oscillating actionable window exposes a would-apply
candidate, if the final clean window does not reset the planner to
`keep_config`, or if the sequence changes the expected submitted, drained, or
released byte counters.

## Expected Value

If the postulate survives, Locus will have dry-run evidence that the service
planner does not convert unstable alternating telemetry into an adaptive policy
signal.
