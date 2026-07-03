# Postulate 0193: Remote-Free Runtime-Collected Guarded Confirm

Date: 2026-07-03

## Claim

A guarded remote-free retune confirm path can be driven by telemetry collected
directly from `RemoteFreeOwnerRuntime` windows, without controlled synthetic
service-summary shapes, while preserving real allocation and release counters.

## Rationale

Experiment 0200 connected guarded decisions to runtime install, confirm,
rollback, and no-change operations, but the service summaries were controlled
diagnostic shapes. The next evidence gap is whether runtime-collected drift
reports can drive at least the stable apply and clean confirmation path.

The runtime currently builds its controller from the queued-byte config, which
keeps the benchmark window clean. To produce a real drifting owner window, the
runtime needs a narrow construction path that keeps the diagnostic config but
uses a caller-supplied initial drain policy. Applying a guarded `drain_earlier`
candidate should then rebuild the runtime with the config's queued-byte policy
and allow the next runtime-collected window to confirm cleanly.

## Experiment

Add a measured guarded runtime confirmation sequence that:

- starts `RemoteFreeOwnerRuntime` with a queued-byte diagnostic config and an
  initial empty drain policy;
- records drift reports from runtime status at each owner control point;
- feeds each runtime-collected summary into `RemoteFreeServiceRetuneGuard`;
- applies the stable `drain_earlier` decision through
  `RemoteFreeServiceRetunePolicyApplicator`;
- installs the applied config through `RemoteFreeOwnerRuntime`;
- confirms the runtime only after a clean runtime-collected validation window;
- allocates real `Vec<u8>` blocks and releases them through owner runtime
  drains in every window.

## Falsification

The postulate fails if runtime-collected reports do not produce the expected
hold, apply, and confirm decisions; if the applied runtime does not switch from
end-drain behavior to queued-byte policy behavior; if stale rollback state
remains after confirmation; or if submitted, drained, released-byte, drain, or
wait counters diverge from the measured owner windows.

## Expected Value

If the postulate survives, Locus will have the first guarded retune path driven
by runtime-collected telemetry, narrowing the gap between controlled service
diagnostics and live owner runtime orchestration.
