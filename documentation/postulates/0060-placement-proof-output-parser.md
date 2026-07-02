# Postulate 0060: Placement Proof Output Parser

Date: 2026-07-02

## Statement

Validation automation should be able to extract the final placement proof verdict from full mapped scratch bind probe output.

## Rationale

The probe now prints multiple machine-readable lines. The final placement proof line is the authoritative placement verdict, but scripts should not need to manually scan and split output before using the typed `NumaPlacementProof` parser.

A small observe-layer output parser keeps the proof-line selection rule explicit, rejects duplicate final proof lines, and preserves typed errors for malformed proof lines.

## Experiment

Add an output parser that scans multiline probe output for `placement_proof=<status> reason=<reason>`, returns `NumaPlacementProof`, and rejects missing, duplicate, or malformed proof lines.

## Expected Result

The parser should pass focused tests. Workspace tests, clippy, and the Docker mapped scratch bind probe should pass with the existing unavailable placement proof output.
