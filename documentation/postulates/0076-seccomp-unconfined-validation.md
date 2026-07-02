# Postulate 0076: Seccomp Unconfined Validation

Date: 2026-07-02

## Statement

Running the Linux validation probes in Docker with seccomp disabled may change the memory-policy readiness result from `permission_denied` to `ready`.

## Rationale

The current Docker environment reports:

```text
seccomp=filter seccomp_filters=1 no_new_privs=0
memory_policy_readiness=not_ready reason=permission_denied
```

That shows a seccomp filter is active when `mbind` is denied. A seccomp-unconfined run can separate seccomp filtering from other possible Linux permission constraints.

## Experiment

Run the memory-policy probe and live validation gate with:

```sh
docker run --rm --security-opt seccomp=unconfined -v "$PWD":/work -w /work rust:1.96 ...
```

Compare the `seccomp`, `memory_policy_readiness`, and final gate lines to the default Docker run.

## Expected Result

If seccomp filtering is the only blocker, the unconfined run should report `seccomp=disabled` and `memory_policy_readiness=ready reason=ready`. If `mbind` still fails, another permission boundary remains.
