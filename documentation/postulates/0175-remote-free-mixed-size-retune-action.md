# Postulate 0175: Remote-Free Mixed-Size Retune Action

Date: 2026-07-03

## Claim

The capacity-plus-queued-byte retune action that survived the uniform
remote-free trace should also survive a heterogeneous mixed-size allocation
trace.

## Rationale

Experiment 0182 showed that larger queue capacity plus earlier queued-byte
drains can remove producer backpressure while preserving the 64-item retained
window on uniform 4096-byte blocks. The result is not enough for inference
allocator policy because real host-side work can retain mixed allocation sizes.

The mixed-size trace has stronger retained-byte evidence: the known-good
queued-byte policy holds peak queued bytes to 655,360 while preserving
`full_count=0`, max pending 64, max wait 2 bursts, and mean wait 1.500 bursts.

## Experiment

Add a focused mixed-size capacity retune benchmark that tests:

- capacity 64 without queued-byte drains;
- capacity 128 without queued-byte drains;
- capacity 256 without queued-byte drains;
- capacity 128 with queued-byte drains;
- capacity 256 with queued-byte drains.

The benchmark must use real `Vec<u8>` allocations and the existing mixed-size
trace pattern from 4096 bytes to 32768 bytes. It must assert full queues,
forced drains, policy drains, drain rounds, max pending items, max queued bytes,
over-target drift, release wait, and retune hint.

## Falsification

The postulate is weakened if the policy cases exceed 64 pending items, exceed
655,360 queued bytes, show queue backpressure, increase max wait beyond 2
bursts, increase mean wait beyond 1.500 bursts, or require a synthetic counter
path that bypasses actual queueing and allocation.

## Expected Value

If the postulate survives, the adaptive retune rule becomes less trace-specific:
capacity can provide producer slack only when an owner-side queued-byte trigger
keeps heterogeneous retained memory and release latency inside the measured
window.
