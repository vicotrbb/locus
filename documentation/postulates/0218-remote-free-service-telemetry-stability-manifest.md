# Postulate 0218: Remote-Free Service Telemetry Stability Manifest

Date: 2026-07-03

## Claim

A small line-oriented manifest can make repeated remote-free service telemetry
stability checks reproducible without weakening the existing counter gate.

## Rationale

Experiment 0225 added repeated-run timing stability over saved benchmark
outputs, but the command still required long positional file lists. That is
easy to mistype and does not preserve run labels separately from local file
paths.

A manifest with one baseline entry and one or more candidate entries should
make saved-output review easier to repeat while preserving the existing rule:
only counter-stable candidate outputs can contribute timing estimates to the
range.

## Proposed Format

The manifest is plain text:

```text
# role label path
baseline apply-confirm-a target/locus-evidence/remote-free-service-sample-compare/apply-confirm-a.txt
candidate apply-confirm-b target/locus-evidence/remote-free-service-sample-compare/apply-confirm-b.txt
candidate apply-confirm-drift target/locus-evidence/remote-free-service-sample-compare/apply-confirm-drift.txt
```

Blank lines and `#` comments are ignored. Labels and paths are whitespace
separated, so labels should be short stable tokens and paths should not contain
spaces.

## Prediction

Parsing the real apply-confirm manifest should produce the same stability
summary as the positional three-file command from Experiment 0225:

- status `mixed`;
- two candidate runs;
- one accepted run;
- one discarded run;
- one timing range from 56,595,000 ps to 56,867,000 ps.

## Falsification

The postulate fails if the manifest path:

- accepts two baselines;
- accepts duplicate run labels;
- accepts a manifest with no candidate entries;
- changes the accepted timing range compared with the positional command;
- requires filesystem behavior inside the pure stability summarizer.

## Validation Plan

Add a manifest parser in `locus-validate` with focused tests for comments,
blank lines, duplicate labels, duplicate baselines, unknown roles, missing
baselines, missing candidates, and malformed rows. Extend the example command
with `--manifest <path>` and verify it against the saved real benchmark
outputs.
