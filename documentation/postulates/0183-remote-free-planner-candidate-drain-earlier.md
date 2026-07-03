# Postulate 0183: Remote-Free Planner Candidate Drain Earlier

Date: 2026-07-03

## Claim

The `drain_earlier` service candidate selected by
`RemoteFreeServiceRetuneCandidate` should restore the fixed queued-byte
service window when benchmarked as an explicit candidate case, without adding
adaptive runtime mutation.

## Rationale

Experiment 0190 selected `drain_earlier` for a four-owner service where one
owner used end-drain behavior. The candidate still needs a direct benchmark
case before Locus treats it as evidence for adaptive policy. The benchmark
should apply the candidate explicitly as a configured scenario, not as live
policy mutation driven by telemetry.

## Experiment

Extend `remote_free_service_telemetry` with a
`planner_candidate_drain_earlier` case.

The new case should:

- keep the same four real owner loops;
- keep 256 `Vec<u8>` remote-free blocks per owner;
- use the same 64-block, 262,144-byte queued-byte target;
- apply queued-byte policy drains to the owner that drifted in the baseline;
- report `retune_candidate=keep_config` after the candidate is applied;
- restore max wait 2 bursts, mean wait 1.500 bursts, zero retained-window
  drift, and 32 `keep_config` reports.

## Falsification

The postulate fails if the explicit candidate does not restore the fixed
queued-byte window, if it changes release accounting, if it requires runtime
mutation from telemetry, or if it cannot be compared directly against the
one-owner end-drain baseline.

## Expected Value

If the postulate survives, Locus has a measured transition from service-level
telemetry to an explicit benchmark candidate, while still keeping production
policy static.
