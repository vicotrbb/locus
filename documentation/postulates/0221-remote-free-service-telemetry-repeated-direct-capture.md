# Postulate 0221: Remote-Free Service Telemetry Repeated Direct Capture

Date: 2026-07-03

## Claim

The direct remote-free service telemetry collector can run one selected
Criterion benchmark repeatedly, assign deterministic labels to each run, and
preserve the same manifest-backed validation contract used by saved-output and
explicit direct-capture modes.

## Rationale

The direct collector already removes manual benchmark-output capture for
baseline and candidate runs, but repeated stability checks still require users
to repeat the same benchmark filter and hand-name each run. That manual step is
small for two runs and error-prone for larger cohorts.

Generating labels from one validated base label should make small repeated
cohorts easier to collect while keeping the durable evidence shape unchanged:
one output file per run, one `manifest.txt`, and one
`validation-summary.txt`.

## Test

Add an opt-in repeated direct-capture mode to
`remote_free_service_telemetry_collect`:

```text
--bench --repeat <count> <evidence-root> <base-label> <benchmark-filter> [-- <criterion-arg> ...]
```

The mode should:

- reject counts below two;
- generate labels as `<base-label>-01`, `<base-label>-02`, and so on;
- validate generated labels through the manifest label rules;
- run the same benchmark filter once per generated label;
- write the same manifest-backed evidence bundle as the existing collector.

Validate with a real short Criterion run of
`remote_free_service_runtime_apply_confirm`, not with handcrafted output only.

## Expected Outcome

The postulate survives if the repeated direct collector creates a real evidence
directory with three captured outputs, a manifest, and a stability summary that
accepts all counter-stable runs into one timing range.
