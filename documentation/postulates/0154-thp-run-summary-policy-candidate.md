# Postulate 0154: THP Run Summary Policy Candidate

Date: 2026-07-03

## Claim

The repeated THP benchmark evidence summary should emit an explicit
policy-candidate verdict so benchmark timing ranges cannot be mistaken for
allocator policy support when page-size evidence is mixed, unavailable, or still
shows base pages.

## Rationale

The run summary already exposes the evidence needed for a human to make this
distinction, but a compact machine-readable report should make the safe
interpretation obvious. A summary line that always begins with
`mapped_scratch_thp_benchmark_evidence_runs=ready` can be read as stronger than
it is, even though that field only means the report lines parsed.

The conservative policy boundary should require all reports to be ready, the
page-size cohort to be consistent, and every report to show observed hugepage
adoption before a repeated run can be considered a policy candidate.

## Experiment

Add a typed policy-candidate reason to the repeated THP report summary:

- `ready`;
- `unavailable_reports`;
- `mixed_page_evidence`;
- `no_hugepage_adoption`.

The summary should print both `policy_candidate=<bool>` and
`policy_candidate_reason=<reason>`.

## Falsification

The postulate is weakened if the verdict hides useful benchmark evidence,
classifies base-page Docker runs as policy candidates, or makes mixed page-size
cohorts look comparable.

## Expected Value

If the postulate survives, repeated THP benchmark summaries remain useful for
timing analysis while preventing premature hugepage-aware allocator policy
changes.
