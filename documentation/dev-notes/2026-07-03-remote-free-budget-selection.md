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

Experiment 0187 wired `retune_action` into the request remote-free benchmark.
End-drain reported `drain_earlier` against the 8-request, 262,144-byte target,
while both max-wait-2 and queued-byte request policies reported `keep_config`
with real `RequestScratchPool::close_request` release.

Experiment 0188 added a focused retune-action evidence matrix covering the
validated uniform, mixed-size, owner-loop, KV, and request surfaces from
Experiments 0184 through 0187. Treat it as a regression tripwire for diagnostic
action semantics, not as a replacement for real allocation benchmarks.

Experiment 0189 added `RemoteFreeServiceRetuneSummary` as service-level
telemetry across multiple owner loops. A four-owner fixed-policy benchmark
reported 32 `keep_config` reports, while a service with one end-drain owner
reported six `drain_earlier` reports, 192 pending items over target, and
786,432 queued bytes over budget without mutating policy.

Experiment 0190 added `RemoteFreeServiceRetuneCandidate::from_summary` as the
first non-mutating planner over service telemetry. The fixed-policy service
selected `keep_config`; the one-owner end-drain service selected
`drain_earlier` while preserving the same measured owner-loop counters.

Experiment 0191 added an explicit `planner_candidate_drain_earlier` service
benchmark. It restored the fixed queued-byte window after the one-owner
end-drain baseline: zero reports needing retune, zero retained-window drift,
32 `keep_config` reports, max wait 2 bursts, and mean wait 1.500 bursts.

Experiment 0192 added the combined backpressure plus retained-window service
case. The capacity-128 end-drain owner selected
`increase_queue_capacity_and_drain_earlier` with four combined-action reports,
four queue-backpressure reports, max pending over target 64, and max queued
bytes over budget 262,144. The explicit capacity-plus-policy candidate
restored zero drift, 32 `keep_config` reports, max wait 2 bursts, and mean
wait 1.500 bursts.

Experiment 0193 added `RemoteFreeServiceRetuneDryRunPlanner` as a non-mutating
planner across service windows. A six-window sequence over real owner-loop
cases recorded one stable `drain_earlier` would-apply window, one stable
`increase_queue_capacity_and_drain_earlier` would-apply window, then reset to
`keep_config` with final streak 0 and no final would-apply candidate after a
clean window.

The service telemetry benchmark harness was then split into a small Criterion
entrypoint plus static-service and dry-run support modules under
`crates/locus-alloc/benches/remote_free_service/` so future service-window
experiments do not keep growing one large benchmark file.

Experiment 0194 added an oscillating dry-run sequence. Alternating
`drain_earlier` and `increase_queue_capacity_and_drain_earlier` service
windows produced zero would-apply windows for both candidates while preserving
6144 submitted blocks, 6144 drained blocks, 25,165,824 released bytes, max
wait 8 bursts, and mean wait 1.875 bursts.

Experiment 0195 added `RemoteFreeServiceRetuneGuard` as the first guarded
planning layer after dry-run telemetry. The confirming benchmark emitted two
apply decisions, confirmed both with clean candidate windows, and preserved
7168 submitted blocks, 7168 drained blocks, and 29,360,128 released bytes. The
rollback benchmark emitted one apply decision, rejected a non-clean validation
window with one rollback, and preserved 4096 submitted blocks, 4096 drained
blocks, and 16,777,216 released bytes.

Experiment 0196 measured the guarded mutation-limit path. After two confirmed
candidate applications, a third stable `drain_earlier` candidate emitted one
`mutation_limit_reached` decision, zero additional apply decisions, and no
pending validation candidate while preserving 9216 submitted blocks, 9216
drained blocks, and 37,748,736 released bytes.

Experiment 0197 added `RemoteFreeServiceRetunePolicyApplicator` as a narrow
bridge from guarded decisions to validated queued-byte drain configs. The
guarded service benchmark now routes pending candidates through the applicator
before running the real allocation service cases. Confirming, rollback, and
mutation-limit counters stayed unchanged, including 37,748,736 released bytes
in the mutation-limit sequence.

Experiment 0198 added `RemoteFreeOwnerRuntime` as the first owner-side wrapper
that can install typed application plans and roll back to the previous config
at empty queue/controller boundaries. A real allocation benchmark installed
capacity 256, rolled back to capacity 128, and preserved 768 submitted blocks,
768 drained blocks, 3,145,728 released bytes, 12 policy drains, one install,
one rollback, max wait 2 bursts, and mean wait 1.500 bursts.

Experiment 0199 added `RemoteFreeOwnerRuntime::confirm` so successful
validation windows can clear rollback state at empty owner boundaries. A real
allocation benchmark installed capacity 256, confirmed it, and preserved 768
submitted blocks, 768 drained blocks, 3,145,728 released bytes, 12 policy
drains, one install, one confirm, zero rollbacks, final capacity 256, and no
remaining rollback config.

Experiment 0200 connected guarded service decisions to
`RemoteFreeOwnerRuntime` operations in one measured sequence. Across nine real
owner-runtime allocation windows, controlled service summaries produced two
runtime installs, one confirm, one rollback, one mutation-limit decision, five
runtime no-change outcomes, 2304 submitted blocks, 2304 drained blocks,
9,437,184 released bytes, final queue capacity 128, and no rollback config.

Experiment 0201 replaced controlled summary shapes for the guarded
apply-confirm path with telemetry collected directly from
`RemoteFreeOwnerRuntime`. Three real owner-runtime windows produced one hold,
one apply, one confirm, 768 submitted blocks, 768 drained blocks, 3,145,728
released bytes, 12 runtime-collected reports needing retune before apply, a
clean validation window after apply, final capacity 256, and no rollback
config.

Experiment 0202 extended runtime-collected guarded telemetry to a three-owner
mutation-limit sequence. One service guard consumed runtime reports from three
`RemoteFreeOwnerRuntime` instances, allowed two apply-confirm mutations, then
returned one `mutation_limit_reached` decision for the third stable owner
candidate while preserving 2048 submitted blocks, 2048 drained blocks,
8,388,608 released bytes, two installs, two confirms, zero rollbacks, and four
runtime no-change outcomes.

Experiment 0203 added runtime-collected rollback from a real validation
failure. After two 4096-byte capacity-128 owner windows produced a stable
`increase_queue_capacity_and_drain_earlier` apply, the validation window
allocated 8193-byte blocks and recorded their true retained bytes. Runtime
telemetry reported retained-byte drift, the guard rolled back, and the runtime
restored capacity 128 while preserving 768 submitted blocks, 768 drained
blocks, 4,194,560 released bytes, one install, one rollback, zero confirms,
and one no-change outcome.

Experiment 0204 added `RemoteFreeServiceRuntimeRetuneCoordinator` as the first
reusable API boundary for guarded owner-runtime retune orchestration. The
coordinator owns the service guard and typed applicator, applies decisions to
targeted owner runtimes, and preserved the measured coordinator sequence: 2048
submitted blocks, 2048 drained blocks, 9,437,440 released bytes, two installs,
one confirm, one rollback, one mutation-limit decision, and four runtime
no-change outcomes.

Experiment 0205 added `RemoteFreeServiceRuntimeRetuneOwners` and
`RemoteFreeServiceRuntimeOwnerId` as the first reusable owner registry around
the coordinator. A real allocation benchmark registered three owners, routed
runtime-collected summaries by owner ID, preserved the same 2048 submitted
blocks, 2048 drained blocks, 9,437,440 released bytes, two installs, one
confirm, one rollback, one mutation-limit decision, four no-change outcomes,
and reported one missing-owner check.

Experiment 0206 added `RemoteFreeServiceRuntimeWindowObservation`,
`RemoteFreeServiceRuntimeWindowStats`, and
`RemoteFreeServiceRuntimeRetuneOwners::observe_service_window` as the first
reusable service-window runner over registered owner runtimes. A real
allocation benchmark processed eight routed observations, preserved the same
2048 submitted blocks, 2048 drained blocks, 9,437,440 released bytes, two
installs, one confirm, one rollback, one mutation-limit decision, four
no-change outcomes, and one missing-owner check while reporting drift and
decision counters through the returned window stats.

Experiment 0207 added
`RemoteFreeServiceRuntimeRetuneOwners::collect_service_window` as the first
borrow-scoped collection helper over registered owner runtimes. A real
allocation benchmark collected eight owner summaries through short mutable
owner borrows, preserved the same 2048 submitted blocks, 2048 drained blocks,
9,437,440 released bytes, two installs, one confirm, one rollback, one
mutation-limit decision, four no-change outcomes, and one missing-owner check.

Experiment 0208 added `RemoteFreeServiceRuntimeDirtyOwners` and
`RemoteFreeServiceRuntimeRetuneOwners::collect_dirty_service_window` as the
first dirty-owner selection helper over registered owner runtimes. A real
allocation benchmark marked only active owners dirty, deduplicated repeated
marks, collected eight dirty owner windows, preserved the same 2048 submitted
blocks, 2048 drained blocks, 9,437,440 released bytes, two installs, one
confirm, one rollback, one mutation-limit decision, four no-change outcomes,
and one missing-owner check.

Experiment 0209 added `RemoteFreeServiceRuntimeDirtyOwnerTracker` and
`RemoteFreeServiceRuntimeDirtySink` as the first enqueue-side dirty-owner mark
path. A real allocation benchmark marked owners dirty from successful
`try_enqueue` calls, collected from tracker snapshots, preserved newer marks
across snapshot clearing, and preserved the same 2048 submitted blocks, 2048
drained blocks, 9,437,440 released bytes, two installs, one confirm, one
rollback, one mutation-limit decision, four no-change outcomes, and one
missing-owner check.

Experiment 0210 added `RemoteFreeServiceRuntimeDirtyOwnerLocalBuffer` as a
per-worker batching path before the shared dirty-owner tracker. The real
allocation service-window sequence preserved 2048 submitted blocks, 2048
drained blocks, 9,437,440 released bytes, 12 policy drains, 36 drain rounds,
46 reports needing retune, two apply decisions, one confirm, one rollback, and
one mutation-limit decision. The local buffer measured 198.83 to 199.35 us in
the short run, while the direct dirty-enqueue tracker path measured 209.44 to
212.01 us in the same sequential benchmark session. Treat Vec-only local
buffers as the current measured candidate for worker-owned enqueue loops.

Experiment 0211 tested end-of-burst local dirty-buffer flushing. The path
preserved 2048 submitted blocks, 2048 drained blocks, 9,437,440 released
bytes, 12 policy drains, 36 drain rounds, 46 reports needing retune, two apply
decisions, one confirm, one rollback, and one mutation-limit decision. It did
not preserve the performance benefit in the short run: before-collection local
flushing measured 199.29 to 200.68 us, direct dirty-enqueue tracker marking
measured 201.32 to 203.72 us, and burst flushing measured 204.32 to 207.50 us.
Treat fixed per-burst flushing as rejected for the current owner-window shape.

Experiment 0212 tested local dirty-buffer flushing at the configured
`TARGET_PENDING_BLOCKS` retained-item window. The path preserved 2048
submitted blocks, 2048 drained blocks, 9,437,440 released bytes, 12 policy
drains, 36 drain rounds, 46 reports needing retune, two apply decisions, one
confirm, one rollback, and one mutation-limit decision. It provided earlier
tracker visibility than before-collection local flushing, but did not earn a
default performance slot: before-collection local flushing measured 197.29 to
198.02 us, threshold flushing measured 200.36 to 205.87 us, and direct
dirty-enqueue tracker marking measured 203.45 to 205.38 us. Treat threshold
flushing as a correctness-validated visibility option, not the measured
default for this owner-window shape.

Experiment 0213 tested a longer-lived local dirty-buffer lifecycle with one
shared tracker and per-owner local buffers reused across the service-window
sequence. The path preserved 2048 submitted blocks, 2048 drained blocks,
9,437,440 released bytes, 12 policy drains, 36 drain rounds, 46 reports
needing retune, two apply decisions, one confirm, one rollback, and one
mutation-limit decision. It also asserted 8 non-empty local flushes, 8 owner
flush observations, 8 newly pending tracker marks, 2040 duplicate local marks,
and retained local buffer capacity after flush. Reused local buffers measured
195.12 to 196.07 us, fresh before-collection local buffers measured 198.76 to
201.50 us, and direct dirty-enqueue tracker marking measured 205.31 to 206.05
us. Treat long-lived worker-local buffers with service-demand flushing as the
current measured candidate.

Experiment 0214 moved the reused local-buffer lifecycle into
`RemoteFreeServiceRuntimeDirtyOwnerLocalBuffers`. The helper preserved 2048
submitted blocks, 2048 drained blocks, 9,437,440 released bytes, 12 policy
drains, 36 drain rounds, 46 reports needing retune, two apply decisions, one
confirm, one rollback, and one mutation-limit decision. The helper added
grow-only indexed buffers for sparse owner IDs and a hot-path
`RemoteFreeServiceRuntimeDirtyOwnerLocalMarker`. The strict performance claim
did not fully survive: the benchmark-only reused path measured 196.61 to
197.42 us, the helper path measured 197.23 to 198.17 us, and direct
dirty-enqueue tracker marking measured 202.16 to 202.52 us. Treat the helper
as the production API for the measured lifecycle, but use `local_marker` in
hot enqueue loops and keep hand-rolled overhead checks in future benchmarks.

Experiment 0215 integrated one local owner-buffer flush with tracked dirty
service-window collection in
`RemoteFreeServiceRuntimeRetuneOwners::collect_local_dirty_service_window`.
The path preserved 2048 submitted blocks, 2048 drained blocks, 9,437,440
released bytes, 12 policy drains, 36 drain rounds, 46 reports needing retune,
two apply decisions, one confirm, one rollback, and one mutation-limit
decision. The first benchmark attempt exposed a `usize::MAX` missing-owner
capacity overflow in the benchmark harness, so the integrated method now
rejects missing owners before resizing local buffer storage. The manual local
buffer group path measured 196.69 to 197.23 us, while the integrated path
measured 195.74 to 196.82 us. Treat integrated collection as the preferred
production service path for the measured local-buffer lifecycle.

Experiment 0216 added bounded direct marking through
`RemoteFreeServiceRuntimeDirtyOwnerLocalBuffers::try_mark_dirty` and
`try_local_marker`. Rejected owner IDs return
`RemoteFreeServiceRuntimeDirtyOwnerLocalBufferGroupError::OwnerOutOfRange`
before local buffer growth, including the `usize::MAX` case that previously
exposed vector growth risk. The bounded service-window path preserved 2048
submitted blocks, 2048 drained blocks, 9,437,440 released bytes, 12 policy
drains, 36 drain rounds, 46 reports needing retune, two apply decisions, one
confirm, one rollback, and one mutation-limit decision. The bounded path
measured 196.85 to 197.30 us in the first run and 196.78 to 197.18 us in the
second run, while the manual and integrated baselines were noisier in the same
sessions. Treat bounded marking as the default when a caller has the current
owner count or the owner ID may be stale or externally supplied.

Experiment 0217 added `RemoteFreeServiceRuntimeValidatedDirtyOwner` and
`RemoteFreeServiceRuntimeRetuneOwners::validate_local_dirty_owner` so callers
can obtain a registry-validated handle instead of passing owner limits
manually. The validated path preserved 2048 submitted blocks, 2048 drained
blocks, 9,437,440 released bytes, 12 policy drains, 36 drain rounds, 46
reports needing retune, two apply decisions, one confirm, one rollback, and
one mutation-limit decision. The first timing run favored the validated path:
bounded measured 203.40 to 205.16 us and validated measured 197.83 to 198.27
us. The second timing run was mixed: bounded measured 200.07 to 201.61 us and
validated measured 201.58 to 208.39 us with high outliers. Treat the validated
handle as the cleaner production API for avoiding manual owner-limit plumbing,
but do not claim it is faster than the bounded path.

Experiment 0218 factored manual, integrated, bounded, and validated local
dirty-buffer group benchmark collection paths into
`runtime_local_dirty_group_harness.rs`. The refactor preserved 2048 submitted
blocks, 2048 drained blocks, 9,437,440 released bytes, 12 policy drains, 36
drain rounds, 46 reports needing retune, two apply decisions, one confirm, one
rollback, and one mutation-limit decision across all four group modes. The
main service-window harness shrank from 1503 lines to 1233 lines, with a
210-line helper owning shared duplicate-mark, flush, capacity reuse, tracker,
and missing-owner assertions. Treat the helper as the local dirty-buffer group
benchmark extension point.

Experiment 0219 replaced the four local dirty-buffer group Criterion wrappers
with the `LOCAL_DIRTY_GROUP_BENCHMARKS` descriptor table. The descriptor path
preserved the four existing benchmark names and the same 2048 submitted
blocks, 2048 drained blocks, 9,437,440 released bytes, 12 policy drains, 36
drain rounds, 46 reports needing retune, two apply decisions, one confirm, one
rollback, and one mutation-limit decision. The main service-window harness
shrank from 1246 lines to 1204 lines. A first validated timing block reported
221.04 to 239.78 us, but an exact validated rerun returned to 197.82 to
199.02 us without code changes. Treat descriptor registration as a
maintainability improvement, not a performance claim.

Experiment 0220 made `runtime_service_window_harness.rs` sample printing aware
of Criterion filter tokens. An exact validated local dirty group filter printed
only the validated service-window sample and summary while preserving 2048
submitted blocks, 2048 drained blocks, 9,437,440 released bytes, 12 policy
drains, 36 drain rounds, 46 reports needing retune, two apply decisions, one
confirm, one rollback, one mutation-limit decision, max wait 8 bursts, and
mean wait 3.312 bursts. A broad local group filter still printed all four
local group sample pairs, and a no-filter `--list` check printed 24
service-window sample lines. Treat the helper as service-window scoped until
the other telemetry harnesses share the same filter logic.

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
8. Aggregate owner-loop reports with `RemoteFreeServiceRetuneSummary` when a
   service-level decision needs to distinguish isolated owner drift from
   service-wide clean policy.
9. Use `RemoteFreeServiceRetuneCandidate::from_summary` to select the next
   benchmark candidate from telemetry. Do not apply the candidate directly to
   live policy.
10. Benchmark the selected candidate as an explicit static case before
    introducing any adaptive mutation.
11. Benchmark every planner-selected combined candidate as an explicit static
    case before any dry-run or live adaptive policy.
12. Use `RemoteFreeServiceRetuneDryRunPlanner` to require repeated actionable
    service windows before considering live mutation. Treat `would_apply` as
    evidence for the next guarded benchmark, not as permission to mutate by
    itself.
13. Use `RemoteFreeServiceRetuneGuard` when translating stable dry-run signals
    into explicit apply, confirm, rollback, or mutation-limit decisions. The
    guard still does not mutate policy directly.
14. Treat `mutation_limit_reached` as a hold decision for live policy and
    record it in service telemetry before allowing any later retune window to
    apply.
15. Use `RemoteFreeServiceRetunePolicyApplicator` to translate guarded apply
    decisions into validated configs. Do not let callers apply raw telemetry
    candidates directly.
16. Install or roll back configs through `RemoteFreeOwnerRuntime` only at empty
    owner boundaries. Do not migrate pending remote-free work across queue
    reconstruction without a separate measured design.
17. Confirm successfully validated runtime configs at an empty owner boundary
    so stale rollback configs cannot be applied after acceptance.
18. Use the guarded runtime sequence to verify that guard decisions translate
    into runtime install, confirm, rollback, and no-change outcomes before
    attempting live orchestration.
19. Prefer `RemoteFreeOwnerRuntime::drift_report` when guarded decisions can be
    driven by runtime-collected owner telemetry. Use controlled summaries only
    for paths that do not yet have a measured runtime-collected equivalent.
20. Treat the service guard as the owner-spanning mutation budget when multiple
    runtimes report drift. A later drifting owner must hold when the service
    mutation budget is exhausted.
21. Validate applied configs against the current retained-byte workload shape.
    If block or arena size changes after apply, runtime-collected byte drift
    should roll back the candidate rather than confirming stale sizing.
22. Use `RemoteFreeServiceRuntimeRetuneCoordinator` as the reusable service
    boundary for translating runtime-collected summaries into owner runtime
    operations. Keep one coordinator per service-level mutation budget.
23. Register owner runtimes with `RemoteFreeServiceRuntimeRetuneOwners` when
    multiple owners share one service mutation budget. Route summaries by
    `RemoteFreeServiceRuntimeOwnerId` instead of passing ad hoc runtime
    references through each call site.
24. Use `RemoteFreeServiceRuntimeWindowObservation` and
    `observe_service_window` when a service loop needs reusable drift,
    decision, and runtime-outcome counters for routed owner summaries.
25. Use `collect_service_window` when the service loop owns registered owner
    runtimes and needs to collect summaries through short mutable owner borrows
    before routing them through the shared service-window runner.
26. Use `RemoteFreeServiceRuntimeDirtyOwners` and
    `collect_dirty_service_window` when a service loop can mark owners with new
    remote-free activity and wants to avoid scanning every registered owner on
    each service window.
27. Use `RemoteFreeServiceRuntimeDirtyOwnerTracker` and
    `RemoteFreeServiceRuntimeDirtySink` when remote enqueue handles should mark
    owners dirty directly after successful enqueue attempts. Collect from
    tracker snapshots so newer marks are not cleared by an older successful
    service window.
28. Use `RemoteFreeServiceRuntimeDirtyOwnerLocalBuffer` when a worker-owned
    enqueue loop can batch repeated owner marks locally before flushing unique
    owner IDs into the shared tracker. Keep the local buffer compact and flush
    before tracked service-window collection.
29. Prefer before-collection local dirty-buffer flushing for the current
    measured worker-owned enqueue shape. Do not use fixed end-of-burst
    flushing unless a workload-specific benchmark shows the earlier visibility
    is worth the extra tracker flush cost.
30. Use retained-window threshold flushing only when earlier dirty-owner
    visibility is required and same-session allocation benchmarks show that
    the added tracker flush cadence is acceptable. It is not the current
    default over before-collection local flushing.
31. Keep worker-local dirty buffers alive across service windows when the
    worker can own that lifecycle. Flush them on service demand before tracked
    dirty collection, and retain their capacity after each flush.
32. Use `RemoteFreeServiceRuntimeDirtyOwnerLocalBuffers` as the production
    owner for the shared tracker plus reusable local dirty buffers. Use
    `local_marker` for hot enqueue loops so successful enqueue marks do not
    index the buffer group on every item.
33. Prefer `collect_local_dirty_service_window` when a service loop needs to
    flush one local dirty owner buffer and collect the shared dirty tracker.
    The method validates owner registration before local buffer resizing,
    preserves tracked dirty generation semantics, and returns both flush and
    service-window stats.
34. Use `try_mark_dirty` or `try_local_marker` when a local dirty-buffer group
    caller has a current owner count or the owner ID is not already trusted.
    The bounded methods reject out-of-range owner IDs before vector growth.
    Keep `local_marker` for the tightest hot path after owner IDs have already
    been validated.
35. Prefer `validate_local_dirty_owner` when a service loop already owns the
    runtime owner registry. Use the returned
    `RemoteFreeServiceRuntimeValidatedDirtyOwner` with
    `mark_validated_dirty` or `validated_local_marker` so call sites avoid
    manual owner-limit plumbing.
36. Add future local dirty-buffer group benchmark variants through
    `runtime_local_dirty_group_harness.rs` so duplicate mark, flush, capacity
    reuse, tracker-empty, and missing-owner assertions stay shared.
37. Register future local dirty-buffer group Criterion cases in
    `LOCAL_DIRTY_GROUP_BENCHMARKS` so benchmark name, sample label, and mode
    stay in one descriptor.
38. Keep service-window sample printing tied to Criterion filter tokens so
    exact focused runs do not print unrelated service-window sample labels.

## Guardrails

- Do not select queue capacity alone as the latency policy. Larger capacity can
  reduce producer backpressure while hiding release latency.
- Do not use queued-byte thresholds without byte accounting in the owner-side
  controller. The owner must call `record_submit` and `record_drain` with the
  retained and released byte sizes for each remote-free item.
- Do not infer a production default from the current thresholds. The current
  evidence is counter validation from microbenchmarks and examples.
- Do not use the retune-action evidence matrix as proof that an adaptive policy
  is safe. Adaptive changes still need workload-specific allocation benchmarks.
- Do not let service-level telemetry mutate policy directly. It is an
  observation source until a concrete adaptive candidate survives benchmarks.
- Do not let the candidate planner mutate policy directly. It selects the next
  benchmark case, not a production action.
- Do not let dry-run planner output mutate policy directly. It records stable
  would-apply candidates across windows and still needs a guarded adaptive
  benchmark before live policy changes.
- Do not treat oscillating actionable candidates as stable evidence. Experiment
  0194 showed that alternating `drain_earlier` and combined candidates should
  keep `would_apply` at none.
- Do not apply a guarded candidate without feeding the next service window back
  to the guard for confirmation or rollback.
- Do not ignore guarded mutation-limit decisions. They mean the service should
  hold its current policy and record telemetry instead of applying another
  candidate.
- Do not let service telemetry or raw candidates mutate live policy directly.
  Guard decisions must pass through the typed policy applicator.
- Do not rebuild a live owner queue while queue or controller pending work is
  non-empty. Runtime install and rollback are empty-boundary operations.
- Do not keep rollback state after a validation window has confirmed the active
  runtime config.
- Do not treat the controlled-summary guarded runtime benchmark as a full live
  retune proof. Runtime-collected telemetry still needs separate measurement.
- Do not treat the runtime initial-policy override as a production shortcut.
  It is a measured way to compare runtime telemetry against a diagnostic config
  while the applied config still restores the config's queued-byte policy.
- Do not reset the service guard per owner when enforcing a service-wide
  mutation budget. The mutation limit is only meaningful across owners when the
  guard state is shared.
- Do not confirm an applied config after validation reports retained-byte
  drift. Roll back and remeasure the workload shape before deriving a new
  queued-byte budget.
- Do not reimplement guard, applicator, and runtime operation branching in each
  caller. Use the coordinator once runtime-collected summaries are available.
- Do not address multi-owner runtime retune by raw vector indexes in caller
  code. Register owners and route through stable owner IDs so missing owners
  are reported explicitly.
- Do not duplicate service-window drift and decision counter aggregation at
  each caller. Use the window runner stats once summaries are already routed by
  owner ID.
- Do not hold mutable owner runtime borrows across service coordinator
  application, confirmation, or rollback. Use the collection helper when
  collection and routing happen in the same service loop.
- Do not clear dirty-owner marks before a service window has completed
  successfully. Keep marks available for retry or inspection when collection
  or routing fails.
- Do not mark an owner dirty before an enqueue attempt has succeeded. Failed
  full or disconnected enqueue attempts should preserve queue accounting
  without scheduling unnecessary service-window collection.
- Do not clear all tracker marks after collecting one snapshot. Clear only the
  captured owner generations so marks that arrive during collection remain
  visible.
- Do not use a tree-backed local dirty-owner set for tiny per-worker buffers
  without benchmark evidence. The measured path favors compact Vec-only
  deduplication for the current owner-window shape.
- Do not collect a tracked dirty service window before local dirty buffers have
  been flushed into the shared tracker.
- Do not assume earlier local-buffer flush cadence is better. Experiment 0211
  showed fixed end-of-burst flushing was slower than both before-collection
  local flushing and direct dirty-enqueue tracker marking for the current real
  allocation service-window shape.
- Do not promote retained-window threshold flushing from correctness alone.
  Experiment 0212 preserved counters, but before-collection local flushing was
  still faster for the current real allocation service-window shape.
- Do not recreate worker-local dirty buffers per service window when a worker
  can retain them across windows. Experiment 0213 showed capacity reuse was
  faster than fresh before-collection local flushing for the current real
  allocation service-window shape.
- Do not assume the local buffer group is free compared with hand-rolled owner
  buffer borrows. Experiment 0214 showed the helper remained faster than
  direct tracker marking, but was slightly slower than the benchmark-only
  reused-buffer path.
- Do not flush vector-indexed local dirty buffers for unvalidated owner IDs.
  Experiment 0215 showed that an extreme missing ID can request impossible
  local buffer growth. Validate through the owner registry before integrated
  collection, and add a bounded or fallible direct marking path before using
  externally supplied owner IDs with the local buffer group.
- Do not call unbounded local dirty-buffer group marking for stale, external,
  or otherwise untrusted owner IDs. Experiment 0216 added bounded direct
  marking for those paths while retaining unbounded markers only for already
  validated hot loops.
- Do not claim the registry-validated local dirty handle is faster than the
  bounded owner-limit path. Experiment 0217 preserved counters but produced
  mixed timing evidence.
- Do not add another local dirty-buffer group service-window variant by
  duplicating collection assertions in the main service-window harness.
  Experiment 0218 moved those assertions into a dedicated helper module.
- Do not add another one-off Criterion wrapper for a local dirty-buffer group
  mode. Experiment 0219 made the descriptor table the registration surface.
- Do not treat a single Criterion regression block as proof when a same-code
  rerun contradicts it. Experiment 0219 recorded one noisy validated block and
  an exact rerun that returned to the normal range.
- Do not assume the whole service telemetry target has filter-clean output
  just because service-window samples are filtered. Experiment 0220 only gated
  `runtime_service_window_harness.rs`.
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
- `documentation/experiments/0187-request-remote-free-retune-action.md`
- `documentation/experiments/0188-remote-free-retune-action-evidence-matrix.md`
- `documentation/experiments/0189-remote-free-service-retune-telemetry.md`
- `documentation/experiments/0190-remote-free-service-retune-candidate-planner.md`
- `documentation/experiments/0191-remote-free-planner-candidate-drain-earlier.md`
- `documentation/experiments/0192-remote-free-planner-candidate-capacity-and-drain.md`
- `documentation/experiments/0193-remote-free-dry-run-service-planner.md`
- `documentation/experiments/0194-remote-free-dry-run-oscillation.md`
- `documentation/experiments/0195-remote-free-guarded-retune-plan.md`
- `documentation/experiments/0196-remote-free-guarded-mutation-limit.md`
- `documentation/experiments/0197-remote-free-guarded-policy-application.md`
- `documentation/experiments/0198-remote-free-owner-runtime-rollback.md`
- `documentation/experiments/0199-remote-free-owner-runtime-confirm.md`
- `documentation/experiments/0200-remote-free-guarded-runtime-sequence.md`
- `documentation/experiments/0201-remote-free-runtime-collected-guarded-confirm.md`
- `documentation/experiments/0202-remote-free-runtime-collected-multi-owner-mutation-limit.md`
- `documentation/experiments/0203-remote-free-runtime-collected-rollback-byte-drift.md`
- `documentation/experiments/0204-remote-free-runtime-retune-coordinator.md`
- `documentation/experiments/0205-remote-free-runtime-retune-owner-registry.md`
- `documentation/experiments/0206-remote-free-runtime-service-window-runner.md`
- `documentation/experiments/0207-remote-free-runtime-window-collection.md`
- `documentation/experiments/0208-remote-free-dirty-owner-window-collection.md`
- `documentation/experiments/0209-remote-free-enqueue-dirty-owner-marks.md`
- `documentation/experiments/0210-remote-free-local-dirty-mark-buffer.md`
- `documentation/experiments/0211-remote-free-local-dirty-flush-cadence.md`
- `documentation/experiments/0212-remote-free-local-dirty-threshold-flush.md`
- `documentation/experiments/0213-remote-free-reused-local-dirty-buffer.md`
- `documentation/experiments/0214-remote-free-local-dirty-buffer-group.md`
- `documentation/experiments/0215-remote-free-local-dirty-buffer-group-collection.md`
- `documentation/experiments/0216-remote-free-bounded-local-dirty-buffer-group-marking.md`
- `documentation/experiments/0217-remote-free-validated-local-dirty-owner-handle.md`
- `documentation/experiments/0218-remote-free-local-dirty-group-benchmark-helper.md`
- `documentation/experiments/0219-remote-free-local-dirty-group-benchmark-descriptors.md`
- `documentation/experiments/0220-remote-free-service-window-filtered-sample-printing.md`

## Open Questions

- Can the remote-free service telemetry benchmark share one filter-aware
  sample printing helper across all harness modules so focused runs suppress
  unrelated sample blocks target-wide?
- Which workload signal should set the retained item window in production:
  scheduler turn age, active request concurrency, KV cache pressure, or memory
  pressure from observability counters?
- Should queued-byte policy be combined with a max-age fallback for low-byte
  remote-free work that can still hold scarce handles or request IDs?
- How should the policy adapt when observed `full_count` rises even though the
  queued-byte budget is being respected?
