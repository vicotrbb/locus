# Postulate 0111: THP Fault Sample Report Parser

Date: 2026-07-03

## Statement

Mapped scratch THP fault sample validation output should have a combined typed report parser that consumes the availability gate and comparison line together.

## Rationale

The validation command now emits two stable lines:

- `mapped_scratch_thp_fault_sample_validation_gate=...`;
- `mapped_scratch_thp_fault_sample_comparison=...`.

Each line has a typed parser, but report aggregation should not need to parse both independently and then remember the consistency rules between them. A focused report parser can preserve the conservative meaning of the output while keeping comparison-specific code out of the large validation root module.

The report is still benchmark interpretation infrastructure. It should not claim transparent huge page adoption, timing superiority, or GPU-near placement.

## Experiment

Add a dedicated `locus-validate` module that:

- parses both fault sample validation lines from multiline output;
- returns one typed report containing the gate verdict and comparison output;
- accepts ready gate plus available comparison;
- accepts unavailable gate plus unavailable fault-counter comparison;
- accepts the defensive ready gate plus comparison-unavailable output that the current command can produce from inconsistent hand-built values;
- rejects missing, duplicate, malformed, or inconsistent combined output.

## Expected Result

Focused tests should cover ready, unavailable, defensive, parser-error, and inconsistent report cases. Workspace validation, clippy, Docker validation, and documentation hygiene should remain clean.
