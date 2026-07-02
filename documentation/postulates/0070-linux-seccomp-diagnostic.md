# Postulate 0070: Linux Seccomp Diagnostic

Date: 2026-07-02

## Statement

The Linux memory-policy probes should report the current process seccomp mode so a denied `mbind` attempt can be interpreted more precisely.

## Rationale

Docker commonly runs with a seccomp filter enabled. If `mbind` returns `EPERM`, knowing that `Seccomp: 2` and one or more seccomp filters are active makes the failure more actionable than `permission_denied` alone.

This diagnostic does not prove that seccomp caused the denial, but it records relevant execution context for the validation gate.

## Experiment

Add a small parser for the `NoNewPrivs`, `Seccomp`, and `Seccomp_filters` fields in `/proc/self/status`. Print one stable diagnostic line from the memory-policy examples:

```text
seccomp=<mode> seccomp_filters=<count-or-unavailable> no_new_privs=<value-or-unavailable>
```

## Expected Result

The probes should keep their existing readiness and placement verdict lines. In Docker, the new diagnostic should report `seccomp=filter`, helping explain why the current environment cannot prove placement.
