# Postulate 0077: Memory Policy Syscall Unavailable

Date: 2026-07-02

## Statement

The Linux memory-policy readiness verdict should distinguish an unavailable `mbind` syscall from other syscall failures.

## Rationale

The seccomp-unconfined Docker run returned `Function not implemented` for `mbind`. Reporting that as `syscall_failed` is correct but too broad for validation planning.

A stable `syscall_unavailable` reason makes it clear that the running kernel or virtualization layer does not provide the syscall path needed for NUMA placement proof.

## Experiment

Add `LinuxNumaPolicyReadinessReason::SyscallUnavailable` for `ENOSYS`, serialize it as `syscall_unavailable`, and include it in readiness parser consistency rules.

Run the seccomp-unconfined Docker memory-policy probe to confirm it reports the more specific reason.

## Expected Result

The default Docker run should still report `permission_denied`. The seccomp-unconfined run should report `memory_policy_readiness=not_ready reason=syscall_unavailable`.
