# Postulate 0227: Remote-Free Service Telemetry Rollup Bundle Table

Date: 2026-07-03

## Claim

The remote-free service telemetry rollup artifact can include a compact
per-bundle table with summary paths, run ids, validation status, and timing
range counts while staying small enough for release checks.

## Rationale

Experiment 0234 made the evidence root self-describing at the aggregate level,
but a failing release check would still need to rescan the directory to locate
the exact bundle that drifted or lost an artifact. A compact bundle table should
preserve the aggregate checks and expose enough detail for dashboards to point
at the specific bundle without storing full validation logs.

## Test

Extend the rollup artifact writer so `collection-summary-rollup.json` includes
a sorted `bundles` array. Each entry should record:

- relative `summary` path;
- `run_id` when the summary can be parsed;
- validation `status`;
- `timing_ranges` for valid bundles.

Validate it with focused tests and with the real
`target/locus-evidence/remote-free-service-summary-json` evidence root.

## Expected Outcome

The postulate survives if the real rollup artifact includes one bundle row for
the `apply-confirm-summary-1783084007-13676` run, reports `valid` status,
records one timing range, and stays below 512 bytes for the current single
bundle evidence root.
