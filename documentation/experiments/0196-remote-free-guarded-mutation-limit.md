# Experiment 0196: Remote-Free Guarded Mutation Limit

Date: 2026-07-03

## Postulate

Postulate:
`documentation/postulates/0188-remote-free-guarded-mutation-limit.md`

The postulate said that a guarded service retune planner should enforce its
mutation limit after confirmed candidate applications. Once the configured
apply budget is used, a new stable actionable candidate should emit
`mutation_limit_reached` instead of another apply decision.

## Change

Extended the guarded remote-free service benchmark with
`remote_free_service_telemetry_guarded_mutation_limit`.

The sequence uses real owner-loop service windows:

- one clean fixed-policy window;
- two end-drain windows that stabilize `drain_earlier`;
- one explicit `drain_earlier` candidate window that confirms cleanly;
- two capacity-128 end-drain windows that stabilize
  `increase_queue_capacity_and_drain_earlier`;
- one explicit combined candidate window that confirms cleanly;
- two more end-drain windows that stabilize `drain_earlier` after the mutation
  budget is spent.

The guard is configured with `stable_windows=2` and `max_mutations=2`. The
final stable actionable candidate should therefore return
`mutation_limit_reached` and leave no pending validation candidate.

## Commands

```text
cargo fmt --all --check
cargo bench -p locus-alloc --bench remote_free_service_telemetry -- --sample-size 10 --warm-up-time 1 --measurement-time 1
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Results

All validation commands passed. The workspace test run reported 245 unit tests,
1 integration test, and 3 `locus_alloc` doctests passing.

Mutation-limit sequence:

```text
remote_free_service_guarded_mutation_limit_sample windows=9 stable_windows=2 max_mutations=2 submitted_count=9216 drained_count=9216 released_bytes=37748736 policy_drains=120 observed_reports=288 reports_needing_retune=36 max_pending_over_target=192 max_queued_bytes_over_budget=786432 queue_backpressure_reports=8 hold_decisions=4 apply_decisions=2 confirmed_decisions=2 rollback_decisions=0 mutation_limit_decisions=1 drain_earlier_apply_decisions=1 combined_apply_decisions=1 max_wait_bursts=8 mean_wait_bursts=1.916 final_pending_candidate=none final_applied_mutations=2 final_confirmed_mutations=2 final_rollbacks=0
remote_free_service_guarded_mutation_limit_sample_summary windows=9 stable_windows=2 max_mutations=2 samples=8 reports_needing_retune_min=36 reports_needing_retune_max=36 reports_needing_retune_mean=36.000 max_pending_over_target_min=192 max_pending_over_target_max=192 max_pending_over_target_mean=192.000 max_queued_bytes_over_budget_min=786432 max_queued_bytes_over_budget_max=786432 max_queued_bytes_over_budget_mean=786432 queue_backpressure_reports_min=8 queue_backpressure_reports_max=8 queue_backpressure_reports_mean=8.000 apply_decisions_min=2 apply_decisions_max=2 apply_decisions_mean=2.000 confirmed_decisions_min=2 confirmed_decisions_max=2 confirmed_decisions_mean=2.000 rollback_decisions_min=0 rollback_decisions_max=0 rollback_decisions_mean=0.000 max_wait_min=8 max_wait_max=8 max_wait_mean=8.000 mean_wait_min=1.916 mean_wait_max=1.916 mean_wait_mean=1.916 final_pending_candidate=none final_applied_mutations=2 final_confirmed_mutations=2 final_rollbacks=0
```

Short-run timing range:

| Case | Time range | Criterion note |
| --- | ---: | --- |
| `remote_free_service_telemetry_guarded_mutation_limit` | 750.96 to 755.29 us | No change in performance detected; 1 low mild outlier |

## Interpretation

The postulate survived this benchmark.

After two confirmed candidate applications, the guard rejected the next stable
actionable `drain_earlier` candidate with one `mutation_limit_reached`
decision. It emitted no third apply decision, left no pending validation
candidate, and preserved the real service-window allocation counters: 9216
submitted blocks, 9216 drained blocks, and 37,748,736 released bytes.

This closes the measured guarded decision surface for apply, confirm,
rollback, and mutation-limit behavior. The guard still only returns decisions;
production policy mutation should go through a narrow API that records the
candidate, applied policy parameters, confirmation outcome, and mutation-limit
telemetry.

## Next Question

Define the smallest production-facing policy application API that can consume
guard decisions without giving telemetry direct mutation authority.
