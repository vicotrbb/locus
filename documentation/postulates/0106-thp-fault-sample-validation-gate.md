# Postulate 0106: THP Fault Sample Validation Gate

Date: 2026-07-03

## Statement

Locus should provide a validation gate for mapped scratch THP benchmark fault sample logs so saved benchmark output can be checked with one stable verdict line.

## Rationale

The THP benchmark now emits process fault sample lines, and `locus-alloc` can parse them. The next useful step is a validation-layer command that turns a saved benchmark log into a stable verdict for automation and experiment notes.

The gate should not claim huge page adoption or timing superiority. It should only validate that fault sample evidence is complete and usable.

## Experiment

Add a `locus-validate` gate that:

- parses benchmark output with the typed fault sample parser;
- reports `ready` when `default`, `hugepage`, and `no_hugepage` samples are present and available;
- reports `unavailable` when the complete sample set exists but one or more samples could not read process fault counters;
- rejects malformed, duplicate, or incomplete sample output through the parser error path;
- exposes a file-based example for saved benchmark logs.

## Expected Result

Focused tests should cover ready, unavailable, and parse-error outcomes. The example should print one stable verdict line for a valid saved benchmark log.
