# Postulate 0190: Remote-Free Owner Runtime Rollback

Date: 2026-07-03

## Claim

A remote-free owner runtime can safely install guarded policy application
configs and preserve rollback state if it rebuilds queue and controller state
only at empty owner boundaries. Pending remote-free work should not be moved
across queue reconstruction.

## Rationale

Experiment 0197 added a typed applicator that translates guarded apply
decisions into validated configs. The next risk is live ownership state. A
bounded remote-free queue owns a receiver and may have producer sinks outside
the owner. Rebuilding queue capacity while work is pending can lose accounting
or hide disconnected old sinks. A small owner runtime should make the safe
boundary explicit: install and rollback only when queue pending count and
controller pending count are both zero.

## Experiment

Add an owner runtime wrapper that owns:

- the current `RemoteFreeQueuedByteDrainConfig`;
- a `RemoteFreeQueue<T>`;
- a `RemoteFreeDrainController`;
- one previous config for rollback.

The wrapper should:

- create queue and controller from the validated config;
- expose fresh sinks from the current queue;
- install applied configs only at empty boundaries;
- keep the previous config after install;
- roll back to the previous config only at empty boundaries;
- reject install or rollback when queued or tracked work is pending.

Benchmark a real allocation sequence that runs one owner window, installs a
larger capacity config, runs a second owner window, rolls back, and runs a
third owner window. The benchmark must allocate real `Vec<u8>` blocks, release
them through owner-side drains, and record submitted, drained, released-byte,
policy-drain, install, rollback, and wait counters.

## Falsification

The postulate fails if install or rollback succeeds while queue or controller
state is non-empty, if rollback loses the previous config, if queue sizing or
policy does not match the installed config, or if the benchmark allocation
counters fail to match the expected real release path.

## Expected Value

If the postulate survives, Locus will have a measured owner-side boundary for
turning guarded application plans into live queue/controller state without
pretending pending work can be safely migrated.
