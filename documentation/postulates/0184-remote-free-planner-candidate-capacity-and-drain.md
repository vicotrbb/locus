# Postulate 0184: Remote-Free Planner Candidate Capacity And Drain

Date: 2026-07-03

## Claim

The `increase_queue_capacity_and_drain_earlier` service candidate selected by
`RemoteFreeServiceRetuneCandidate` should restore the fixed queued-byte
service window when one owner shows both queue backpressure and retained-window
drift.

## Rationale

Experiment 0191 validated the simpler `drain_earlier` candidate for an
end-drain owner without queue backpressure. The remaining stronger service
signal is a mixed case where one owner both fills its queue and exceeds the
retained-byte window. That case should be measured before any adaptive policy
mutation is introduced.

## Experiment

Extend `remote_free_service_telemetry` with:

- a `one_capacity128_end_drain_owner` baseline where one owner uses capacity
  128 and end-drain behavior;
- a `planner_candidate_capacity_and_drain_earlier` case that applies the
  planner candidate explicitly with a larger queue and queued-byte drains.

Both cases should keep the same service workload:

- four real owner loops;
- 256 `Vec<u8>` blocks per owner;
- 4096 bytes per block;
- eight bursts of 32 blocks per owner;
- drain batch limit 64;
- 64-block, 262,144-byte queued-byte target.

## Falsification

The postulate fails if the capacity-128 baseline does not produce the combined
candidate, if the explicit candidate does not restore zero drift and 32
`keep_config` reports, if release accounting changes, or if the candidate is
applied as telemetry-driven runtime mutation rather than as a static benchmark
case.

## Expected Value

If the postulate survives, Locus will have measured both planner-selected
service candidates, `drain_earlier` and
`increase_queue_capacity_and_drain_earlier`, before introducing adaptive
remote-free policy mutation.
