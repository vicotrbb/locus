# Postulate 0152: THP Timing Parser Edge Cases

Date: 2026-07-03

## Claim

The THP benchmark evidence report should reject malformed or ambiguous
Criterion timing evidence instead of producing a compact report with misleading
timing fields.

## Rationale

The report is intended to summarize real allocator benchmark output. A permissive
timing parser would make the report look authoritative even when a benchmark
case was missing, duplicated, or printed with an unsupported unit.

Focused parser tests can harden the evidence boundary without changing allocator
behavior or adding larger report aggregation logic.

## Experiment

Add tests that verify the timing parser:

- normalizes `ps`, `ns`, ASCII `us`, `ms`, and `s` to picoseconds;
- rejects missing mapped scratch THP timing blocks;
- rejects duplicate timing blocks for the same benchmark case;
- rejects unknown timing units.

## Falsification

The postulate is weakened if these tests require broad parser rewrites, depend
on synthetic output that conflicts with observed Criterion logs, or make valid
real benchmark output harder to parse.

## Expected Value

If the postulate survives, the compact THP evidence report has a stronger guard
against invalid memory-allocation benchmark conclusions.
