# Postulate 0062: Memory Policy Readiness Output Parser

Date: 2026-07-02

## Statement

Validation automation should be able to extract Linux memory-policy readiness from full `mbind_region` output.

## Rationale

The system probe prints the raw `mbind` result, the final memory-policy readiness line, and the page-touch count. Scripts should consume the final typed readiness verdict without duplicating multiline scanning and token parsing.

A `locus-sys` output parser keeps the readiness-line selection rule explicit and rejects duplicate or malformed final readiness lines.

## Experiment

Add an output parser that scans multiline `mbind_region` output for `memory_policy_readiness=<status> reason=<reason>`, returns `LinuxNumaPolicyReadiness`, and rejects missing, duplicate, or malformed readiness lines.

## Expected Result

The parser should pass focused Linux tests. Host workspace validation, Docker Linux tests, and the Docker `mbind_region` probe should pass with the existing permission-denied readiness output.
