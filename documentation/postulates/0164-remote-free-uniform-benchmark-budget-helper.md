# Postulate 0164: Remote-Free Uniform Benchmark Budget Helper

Date: 2026-07-03

## Claim

The uniform queued-byte benchmark cases should derive their retained-byte
thresholds through `RemoteFreeQueuedByteBudget` instead of local constants.

## Rationale

Experiment 0171 added a typed helper for deriving queued-byte policy budgets
from retained item shape inputs. The owner-loop example now uses that helper,
but the KV and request remote-free benchmarks still compute uniform retained
byte thresholds with benchmark-local multiplication:

- KV: target pending blocks times block size;
- request arenas: target pending requests times arena capacity.

Those are exactly the uniform item shapes the helper is meant to cover. Moving
the benchmark policy construction onto the helper should validate the API in
real measured paths without changing the owner release logic or benchmark
workloads.

## Experiment

Update the KV and request remote-free queued-byte policy cases to:

- construct `RemoteFreeQueuedByteBudget` from item count and bytes per item;
- build the queued-byte drain policy with `into_policy()`;
- keep the existing benchmark names, counters, queue sizes, batch limits, and
  release closures unchanged.

Do not change the mixed-size benchmark in this experiment. Its threshold comes
from a heterogeneous trace pattern and should get a separate helper only after
we define a precise API for variable-size retained work.

## Falsification

The postulate is weakened if either benchmark counter output changes, if the
helper makes policy setup less clear, if real owner-side release paths are
altered, or if validation no longer builds and runs the affected benchmarks.

## Expected Value

If the postulate survives, `RemoteFreeQueuedByteBudget` is proven in two
uniform measured domains: real KV block handles and request-affine arena
returns. That gives later runtime configuration code a tested path for deriving
retained-byte thresholds from domain sizes.
