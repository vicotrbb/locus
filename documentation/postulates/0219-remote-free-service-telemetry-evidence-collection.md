# Postulate 0219: Remote-Free Service Telemetry Evidence Collection

Date: 2026-07-03

## Claim

Saved remote-free service telemetry outputs can be collected into one evidence
directory with copied outputs, a manifest, and a validation summary from a
single command without weakening the counter-gated timing review.

## Rationale

Experiment 0226 made repeated-run validation reproducible from a manifest, but
the user still has to create the evidence directory, copy outputs, write the
manifest, and run validation as separate steps. A collector command should
remove that manual assembly while keeping validation behavior identical to
manifest mode.

The collector should not run Criterion itself yet. It should collect already
saved benchmark outputs and produce a deterministic review bundle that can be
shared, archived, or rerun.

## Prediction

For the real `remote_free_service_runtime_apply_confirm` saved outputs, a
collector command with labels `apply-confirm-a`, `apply-confirm-b`, and
`apply-confirm-drift` should create:

- one run directory under the chosen evidence root;
- copied output files named from stable labels;
- `manifest.txt` with one baseline and two candidate rows;
- `validation-summary.txt` with the same `mixed` stability report and
  272,000 ps timing spread observed in Experiments 0225 and 0226.

## Falsification

The postulate fails if the collector:

- changes the stability result compared with manifest mode;
- writes path labels that cannot be parsed by the manifest parser;
- allows label path traversal;
- omits the validation summary;
- requires the pure stability summarizer to know about filesystem paths.

## Validation Plan

Add a manifest formatter with unit tests for stable output and rejected
invalid labels. Add a `remote_free_service_telemetry_collect` example that
copies labeled saved outputs, writes `manifest.txt`, runs the existing
stability report, writes `validation-summary.txt`, and prints a compact
collection summary. Validate it against the saved real apply-confirm outputs.
