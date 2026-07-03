# Postulate 0217: Remote-Free Service Telemetry Timing Stability

Date: 2026-07-03

## Claim

Repeated remote-free service telemetry benchmark outputs can be summarized into
a small stability report by using the JSON counter comparison as the admission
gate for timing evidence.

## Rationale

Experiment 0224 showed that pairwise timing deltas are misleading when the
underlying telemetry counters drift. A repeated-run report should preserve that
ordering across more than two outputs:

1. compare each candidate output with the baseline JSON telemetry rows;
2. accept a candidate timing only when its counters match the baseline;
3. discard counter-drifted candidates from timing ranges while still reporting
   them;
4. compute timing ranges from the baseline timing and accepted candidate
   timings only.

This should make saved Criterion output review less manual without hiding
counter drift.

## Prediction

For the saved `remote_free_service_runtime_apply_confirm` outputs:

- baseline `apply-confirm-a.txt` plus candidate `apply-confirm-b.txt` should
  produce one accepted candidate and one timing range;
- adding controlled drift output `apply-confirm-drift.txt` should keep the same
  accepted timing range and report the drifted output as discarded;
- an all-drift candidate set should emit no timing ranges.

## Falsification

The postulate fails if the stability report:

- includes a timing estimate from a counter-drifted output;
- hides which candidate output was discarded;
- cannot reject malformed or missing timing intervals for a counter-stable
  candidate;
- makes the pairwise comparison path harder to use.

## Validation Plan

Add a `locus-validate` stability module that reuses the existing combined
sample and timing comparator. Cover accepted runs, mixed accepted and discarded
runs, all-drift runs, duplicate labels, and stable candidates with missing
timings. Validate against the saved real benchmark outputs under
`target/locus-evidence/remote-free-service-sample-compare/`.
