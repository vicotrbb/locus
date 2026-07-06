# 2026-07-05: Evidence rescue (LOCUS-OSS Phase 0)

Before any reorganization work, all surviving raw benchmark logs were
copied verbatim from the session scratchpad directories into
`documentation/evaluations/logs/`. No log was regenerated, edited, or
reformatted; these are byte-for-byte copies of the Criterion output as
it was produced.

## Rescued (30 files)

LOCUS-EVAL v1 (backs `documentation/evaluations/0001-locus-eval-v1.md`
and the audit addendum):

- `locus_eval_locus_run{1,2,3}.log`
- `locus_eval_system_run{1,2,3}.log`
- `locus_eval_jemalloc_run{1,2}.log`
- `locus_eval_mimalloc_run{1,2}.log`
- `locus_eval_audit_touch_probe.log`

Experiment-era logs that still existed in scratchpads:

- `remote_free_concurrent_run{1,2}.log` (experiment 0351)
- `remote_free_sharded_run{1,2}.log` (experiment 0352)
- `remote_free_chunk_publish_run{1,2}.log` (experiment 0353)
- `remote_free_mixed_lifetime_run{1,2}.log` (experiment 0354)
- `mixed_lifetime_malloc_run{1,2}.log`,
  `mixed_lifetime_mailbox_run{1,2}.log`,
  `mixed_lifetime_chunk_pool_run{1,2}.log` (experiments 0355-0357)
- `kv_reuse_order_run{1,2}.log` (experiment 0358)
- `kv_reuse_mapped_run{1,2,3}.log` (experiment 0359)

## Lost

Raw logs for experiments 0001-0350 no longer exist in any scratchpad;
those sessions' temporary directories are gone. For that range, the
result tables embedded in the experiment documents under
`documentation/experiments/` are the only surviving record of the raw
numbers. Those documents were written at experiment time from the live
logs and are treated as authoritative; they must never be edited.

The validated thread that the published crate rests on (experiments
0351-0359 plus LOCUS-EVAL v1 and its audit addendum) is fully backed by
rescued raw logs.
