# Postulate 0112: THP Fault Sample Report Example

Date: 2026-07-03

## Statement

`locus-validate` should provide a small example command that parses saved mapped scratch THP fault sample validation output as one coherent report.

## Rationale

The library now exposes `parse_mapped_scratch_thp_fault_sample_report_output`, which validates the relationship between the fault sample gate and comparison line. Downstream automation can call the library directly, but users working with saved benchmark artifacts should also have a command-line path that exercises the same report parser.

The command should normalize only the stable report lines. It should not make a THP adoption claim or performance claim.

## Experiment

Add an example that:

- reads a saved two-line fault sample validation report;
- parses it with `parse_mapped_scratch_thp_fault_sample_report_output`;
- prints the normalized gate verdict line;
- prints the normalized comparison line.

Document the command in the README near the existing captured-output validation commands.

## Expected Result

The example should compile, normalize a ready sample report, and reject malformed or inconsistent reports through the typed parser. Focused tests, workspace validation, clippy, Docker validation, and documentation hygiene should remain clean.
