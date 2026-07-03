# Postulate 0185: Remote-Free Dry-Run Service Planner

Date: 2026-07-03

## Claim

A dry-run service planner over `RemoteFreeServiceRetuneSummary` should record
candidate changes across consecutive service windows and expose a stable
would-apply candidate without mutating queue capacity, drain policy, or
queued-byte budgets.

## Rationale

Experiments 0191 and 0192 validated the two planner-selected static candidates:
`drain_earlier` and `increase_queue_capacity_and_drain_earlier`. The next
adaptive step should still be non-mutating. It should prove that service
telemetry can be converted into a stable plan signal over time before any live
policy mutation is attempted.

## Experiment

Add a small public dry-run planner type that:

- observes service retune summaries as discrete windows;
- records the current candidate from each window;
- tracks how many consecutive windows selected the same candidate;
- exposes `would_apply_candidate` only after the same actionable candidate is
  selected for the configured stability window;
- resets the streak on `keep_config`, `collect_telemetry`, candidate changes,
  or budget-review candidates.

Extend `remote_free_service_telemetry` with a dry-run sequence benchmark that
uses the existing real owner-loop service cases:

- clean fixed-policy window;
- end-drain window selecting `drain_earlier`;
- repeated end-drain window reaching dry-run stability;
- capacity-128 end-drain window selecting the combined candidate;
- repeated capacity-128 end-drain window reaching dry-run stability;
- clean fixed-policy window resetting the dry-run state.

## Falsification

The postulate fails if the dry-run planner applies mutations, if it reports a
would-apply candidate from a single unstable window, if clean telemetry fails to
reset the dry-run plan, if candidate changes fail to reset the stability
streak, or if the benchmark sequence changes the previously measured allocation
and release counters for the underlying service cases.

## Expected Value

If the postulate survives, Locus will have a measured non-mutating bridge
between service-level telemetry and future adaptive remote-free policy logic.
