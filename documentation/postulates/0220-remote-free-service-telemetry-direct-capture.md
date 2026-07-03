# Postulate 0220: Remote-Free Service Telemetry Direct Capture

Date: 2026-07-03

## Claim

The remote-free service telemetry collector can run selected Criterion
benchmarks directly, capture their output into an evidence directory, and then
reuse the same manifest-backed validation path used for saved outputs.

## Rationale

Experiment 0227 made saved-output evidence bundles reproducible, but it still
required benchmark output files to exist before collection. Direct capture
should remove that manual step while keeping the validation boundary intact:
captured benchmark output must be written to files, listed in `manifest.txt`,
and checked through the existing counter-gated stability report.

## Prediction

Two direct captures of `remote_free_service_runtime_apply_confirm` with JSON
telemetry enabled should create:

- two copied benchmark output files named from labels;
- `manifest.txt`;
- `validation-summary.txt`;
- a `stable` timing stability report with one timing range.

The generated output files should contain remote-free service telemetry JSON
rows and Criterion timing intervals.

## Falsification

The postulate fails if direct capture:

- skips writing captured benchmark output files;
- skips the manifest and validation summary;
- omits JSON telemetry rows needed by the counter gate;
- produces a summary through a different path than saved-output collection;
- hides a failing Criterion command.

## Validation Plan

Extend `remote_free_service_telemetry_collect` with an opt-in `--bench` mode.
Run the real `remote_free_service_runtime_apply_confirm` benchmark twice with
small Criterion timing parameters, persist both outputs, verify the manifest
and validation summary, and run the existing Rust validation gates.
