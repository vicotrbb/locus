# Postulate 0235: Remote-Free Service Telemetry Summary Host Output

Date: 2026-07-03

## Claim

The remote-free service telemetry summary validator can print capture host
metadata in its one-line success output without making host metadata mandatory
for older evidence bundles.

## Rationale

Experiment 0242 stored optional host metadata in `collection-summary.json`.
The validator still prints only summary path, manifest path, collection mode,
run id, and output count in its success line. Printing host status in the same
line makes capture context visible in logs and CI output without requiring
callers to open JSON by hand.

Older schema v1 summaries should continue to validate. For those bundles, the
success line can report `host_present=false` instead of failing or inventing
host values.

## Test

Add host-aware formatting to the summary validator success line:

- new summaries with host metadata print `host_present=true`, `host_os`,
  `host_arch`, and `host_hostname`;
- summaries without host metadata print `host_present=false`;
- missing host metadata remains valid for artifact checks and stability
  recomputation.

Run the validator against both an older no-host bundle and the real
host-bearing bundle from Experiment 0242.

## Expected Outcome

The postulate survives if focused tests cover both output forms, the real
host-bearing bundle prints `host_present=true host_os=macos host_arch=aarch64`,
and the older no-host bundle still validates with `host_present=false`.
