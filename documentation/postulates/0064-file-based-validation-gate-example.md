# Postulate 0064: File-Based Validation Gate Example

Date: 2026-07-02

## Statement

The combined placement validation gate should be runnable from captured probe output files.

## Rationale

`locus-validate` now provides a typed helper, but validation operators still need a simple command-line path that can consume outputs captured from the three existing probes. Reading files keeps the example portable across shell scripts, CI jobs, and future permitted-host validation runs without introducing process orchestration or external dependencies.

## Experiment

Add a Linux-only `placement_validation_gate` example to `locus-validate`. The example should read three positional file paths:

1. memory-policy output;
2. placement-readiness output;
3. placement-proof output.

It should print `placement_validation_gate=<status> reason=<reason>`.

## Expected Result

The example should compile under workspace checks and classify current Docker probe output captures as `not_ready reason=memory_policy_not_ready`.
