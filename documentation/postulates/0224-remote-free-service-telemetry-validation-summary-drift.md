# Postulate 0224: Remote-Free Service Telemetry Validation Summary Drift

Date: 2026-07-03

## Claim

The remote-free service telemetry summary validator can compare the saved
`validation-summary.txt` artifact with a freshly computed manifest-backed
stability report and report drift when the saved summary no longer matches the
evidence bundle.

## Rationale

Experiment 0231 verified that `collection-summary.json` can index artifact
byte counts and route validation through the manifest. That proves the indexed
files are present and sized as expected, but it does not prove that the saved
`validation-summary.txt` content still matches the captured outputs listed in
the manifest.

An exact saved-summary comparison turns the bundle into a stronger audit unit:
the validator can prove the index, the manifest-backed evidence, and the saved
human-readable summary are still synchronized.

## Test

Extend `remote_free_service_telemetry_summary_validate` so it:

- resolves the `validation_summary` artifact from `collection-summary.json`;
- reads the saved validation summary;
- recomputes the manifest-backed stability report;
- prints a matched line when the saved summary equals the recomputed report;
- returns a drift error when the saved and recomputed summaries differ.

Validate the matching path against the real repeated direct-capture bundle
from Experiment 0230 and add focused tests for matched and drifted summary
text.

## Expected Outcome

The postulate survives if the real bundle validates with an explicit
`validation_summary=matched` report and the focused drift test rejects a
modified saved summary.
