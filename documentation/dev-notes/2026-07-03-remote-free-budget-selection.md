# Remote-Free Queued-Byte Budget Selection

Date: 2026-07-03

This note records the current evidence-backed way to choose queued-byte
remote-free drain thresholds in Locus.

The policy is a runtime guard for retained remote-free bytes. It is not a
replacement for queue capacity, batch size, or release-latency measurement.

## Current Rule

Use a queued-byte drain threshold when the runtime can estimate bytes retained
by pending remote-free work and wants an owner drain trigger that is independent
of scheduler turn age.

The threshold should represent the largest retained remote-free byte window
that the owner is allowed to defer before draining. In current experiments,
that window was chosen to match the existing max-wait-2 counter behavior.

## Shape To API Mapping

| Workload shape | Budget derivation | Helper API | Validated path |
| --- | --- | --- | --- |
| Grouped uniform work | groups times items per group times bytes per item | `RemoteFreeQueuedByteBudget::from_grouped_item_shape` | queued-byte owner-loop example |
| Uniform retained items | pending item count times bytes per item | `RemoteFreeQueuedByteBudget::from_item_shape` | KV blocks and request arenas |
| Heterogeneous retained items | checked sum of retained item sizes | `RemoteFreeQueuedByteBudget::from_item_sizes` | mixed-size allocation trace |
| Already validated bytes | non-zero retained-byte budget | `RemoteFreeQueuedByteBudget::new` or `try_new` | low-level policy composition |

Use `budget.into_policy()` for a queued-byte-only policy. Use
`budget.as_non_zero_u64()` with `RemoteFreeDrainPolicy::with_max_queued_bytes`
when composing the queued-byte guard with pending-age or pending-count
thresholds.

## Config Helper

Use `RemoteFreeQueuedByteDrainConfig` when the runtime can express a target
pending item window and wants queue capacity, drain batch size, and queued-byte
budget validated together.

The config currently supports:

- grouped retained item shapes through
  `RemoteFreeQueuedByteDrainConfig::from_grouped_item_shape`;
- uniform retained item shapes through
  `RemoteFreeQueuedByteDrainConfig::from_item_shape`;
- heterogeneous retained item sizes through
  `RemoteFreeQueuedByteDrainConfig::from_item_sizes`.

It rejects:

- zero queue capacity;
- zero drain batch limit;
- zero target pending item windows;
- queue capacity below the target pending item window;
- drain batch limits below the target pending item window;
- retained-byte budget derivation failures.

The config exposes `drain_policy()` for `RemoteFreeDrainController` and
`queue::<T>()` for building a `RemoteFreeQueue<T>` with the validated queue
sizing. Allocator-specific release behavior remains outside the config in the
owner's `drain_batch` closure.

## Drift Diagnostics

Use `RemoteFreeQueuedByteDriftReport` when a runtime has a queued-byte drain
config and wants to compare it with live owner-loop observations before
retuning policy.

The report compares:

- target pending items against observed pending items;
- queued-byte budget against observed queued bytes;
- queue `full_count` against zero backpressure.

The report is diagnostic only. It does not mutate the drain policy. Treat
non-zero pending over-target, queued bytes over-budget, or queue backpressure
as evidence that the config needs review, a larger queue, a different drain
cadence, or more workload-specific measurement.

`RemoteFreeQueuedByteDriftReport::retune_hint()` classifies the first
diagnostic response:

| Hint | Meaning |
| --- | --- |
| `keep_config` | no drift signal was observed |
| `increase_queue_capacity` | queue backpressure was the only signal |
| `review_drain_cadence` | pending items exceeded the target window |
| `review_queued_byte_budget` | queued bytes exceeded the budget |
| `review_multiple_signals` | more than one drift signal was observed |

The hint is still diagnostic. Benchmark the candidate change before changing
production policy.

`RemoteFreeQueuedByteDriftReport::retune_action()` recommends the first action
to benchmark:

| Action | Meaning |
| --- | --- |
| `keep_config` | no action needed |
| `increase_queue_capacity` | add producer slack while preserving the configured retained window |
| `drain_earlier` | move owner drains earlier to restore the retained item and byte window |
| `review_queued_byte_budget` | recheck workload size shape or byte budget before changing cadence |
| `increase_queue_capacity_and_drain_earlier` | add producer slack and preserve the retained window with earlier owner drains |

The action is also diagnostic and non-mutating. It should select the next
benchmark candidate, not change production policy by itself.

Experiment 0181 tested `increase_queue_capacity` as a pure capacity action on
the 256-block remote-free trace. Capacity 256 removed `full_count`, but also
raised max pending items from 64 to 256, retained queued bytes from 262,144 to
1,048,576, max wait from 2 to 8 bursts, and mean wait from 1.500 to 4.500
bursts. Treat capacity increases as backpressure fixes that need separate
latency and retained-byte validation.

Experiment 0182 tested the same larger-capacity cases with queued-byte policy
drains enabled. Capacity 128 and capacity 256 both kept `full_count=0` while
preserving the 64-item, 262,144-byte, max wait 2 burst, and mean wait 1.500
burst window. Treat earlier owner-side drains as the first adaptive action
when retained-memory and release-latency targets must remain fixed.

Experiment 0183 repeated that capacity-plus-policy action on the mixed-size
allocation trace. Capacity 128 and capacity 256 with queued-byte drains both
kept `full_count=0` while preserving the heterogeneous 64-item, 655,360-byte,
max wait 2 burst, and mean wait 1.500 burst window. Treat the action as valid
for the current uniform and heterogeneous traces, but still validate new trace
shapes before changing production policy.

Experiment 0184 moved the learned action mapping into
`RemoteFreeQueuedByteRetuneAction` and validated it against both uniform and
mixed-size capacity retune benchmarks. Capacity 128 without policy drains
reported `increase_queue_capacity_and_drain_earlier`, capacity 256 without
policy drains reported `drain_earlier`, and policy-drain cases reported
`keep_config`.

Experiment 0185 wired `retune_action` into the runtime-facing queued-byte
owner-loop example. The example kept the same real allocation counters and
printed `retune_action=keep_config` with zero pending drift, zero queued-byte
drift, and zero queue backpressure.

Experiment 0186 wired `retune_action` into the KV remote-free benchmark.
End-drain reported `drain_earlier` against the 64-block, 262,144-byte target,
while both max-wait-2 and queued-byte KV policies reported `keep_config` with
real `KvBlockHandle` release through `KvBlockPool::free`.

## Measured Thresholds

| Path | Shape inputs | Budget | Matched counters |
| --- | --- | ---: | --- |
| Owner-loop example | 4 active requests, 16 remote-free blocks per request, 10 KiB representative block bytes | 655,360 bytes | 64 max pending, 4 policy drains, max wait 2 bursts, mean wait 1.500 bursts, `full_count=0` |
| Mixed-size allocation trace | 2 retained bursts, 32 blocks per burst, repeated heterogeneous block sizes | 655,360 bytes | 64 max pending, 4 policy drains, max wait 2 bursts, mean wait 1.500 bursts, `full_count=0` |
| KV block handles | 64 target pending blocks, 4096 bytes per block | 262,144 bytes | 64 max pending, 4 policy drains, max wait 2 bursts, mean wait 1.500 bursts, `full_count=0` |
| Request arenas | 8 target pending requests, 32 KiB arena capacity | 262,144 bytes | 8 max pending, 2 policy drains, max wait 2 bursts, mean wait 1.500 bursts, `full_count=0` |

## Selection Procedure

1. Pick the release-latency target first, expressed in scheduler turns, bursts,
   or another owner-loop control interval.
2. Convert that target into the number of retained remote-free items that may
   queue before the owner must drain.
3. Convert retained items to bytes using one of the shape APIs above.
4. Keep queue capacity large enough to absorb the target retained item window
   without immediate producer backpressure.
5. Keep drain batch size large enough to clear the threshold window at each
   policy drain point.
6. Verify with a benchmark or example that `full_count`, max pending count,
   peak queued bytes, drain rounds, max wait, and mean wait match the intended
   behavior.
7. When a queued-byte config is available, record
   `RemoteFreeQueuedByteDriftReport` output at owner control points so drift
   from the configured window is visible before adding adaptive policy logic.

## Guardrails

- Do not select queue capacity alone as the latency policy. Larger capacity can
  reduce producer backpressure while hiding release latency.
- Do not use queued-byte thresholds without byte accounting in the owner-side
  controller. The owner must call `record_submit` and `record_drain` with the
  retained and released byte sizes for each remote-free item.
- Do not infer a production default from the current thresholds. The current
  evidence is counter validation from microbenchmarks and examples.
- Recheck thresholds when KV block size, request arena capacity, burst size,
  request concurrency, or batch size changes.
- For heterogeneous traces, derive the budget from actual retained item sizes
  instead of an average unless the average has separate validation.

## Evidence Sources

- `documentation/experiments/0166-remote-free-queued-byte-policy.md`
- `documentation/experiments/0167-remote-free-queued-byte-owner-loop-example.md`
- `documentation/experiments/0168-kv-remote-free-queued-byte-policy.md`
- `documentation/experiments/0169-request-remote-free-queued-byte-policy.md`
- `documentation/experiments/0171-remote-free-queued-byte-budget-helper.md`
- `documentation/experiments/0172-remote-free-uniform-benchmark-budget-helper.md`
- `documentation/experiments/0173-remote-free-heterogeneous-budget-helper.md`
- `documentation/experiments/0178-remote-free-queued-byte-drift-report.md`
- `documentation/experiments/0179-remote-free-positive-drift-matrix.md`
- `documentation/experiments/0180-remote-free-drift-retune-hint.md`
- `documentation/experiments/0181-remote-free-capacity-retune-action.md`
- `documentation/experiments/0182-remote-free-earlier-drain-retune-action.md`
- `documentation/experiments/0183-remote-free-mixed-size-retune-action.md`
- `documentation/experiments/0184-remote-free-retune-action-helper.md`
- `documentation/experiments/0185-remote-free-owner-loop-retune-action.md`
- `documentation/experiments/0186-kv-remote-free-retune-action.md`

## Open Questions

- Should request arenas or service-level telemetry be the next
  `retune_action` surface?
- Which workload signal should set the retained item window in production:
  scheduler turn age, active request concurrency, KV cache pressure, or memory
  pressure from observability counters?
- Should queued-byte policy be combined with a max-age fallback for low-byte
  remote-free work that can still hold scarce handles or request IDs?
- How should the policy adapt when observed `full_count` rises even though the
  queued-byte budget is being respected?
