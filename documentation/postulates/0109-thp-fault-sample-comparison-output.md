# Postulate 0109: THP Fault Sample Comparison Output

Date: 2026-07-03

## Statement

The mapped scratch THP fault sample validation command should print a stable comparison line when benchmark fault samples are usable.

## Rationale

`locus-alloc` now computes a typed comparison from parsed benchmark fault samples, but the validation command still prints only whether samples are present and available. Saved benchmark logs need a single command that reports both evidence availability and the computed minor-fault comparison.

The comparison line must remain separate from the validation gate. A ready fault sample gate means the counters are usable; it does not mean that `hugepage` advice improved performance or that transparent huge pages were adopted.

## Experiment

Extend `locus-validate` with a displayable mapped scratch THP fault sample comparison output line.

The line should report:

- `available reason=ready` plus the computed process minor-fault deltas when comparison is available;
- `unavailable reason=fault_counters_unavailable` when the gate reports unavailable process fault counters;
- `unavailable reason=comparison_unavailable` for defensive cases where samples are marked ready but cannot be compared.

Update the `mapped_scratch_thp_fault_sample_validation_gate` example to print the validation gate followed by the comparison line.

## Expected Result

Focused tests should verify the available and unavailable comparison lines. Workspace validation, clippy, Docker validation, and documentation hygiene should remain clean.
