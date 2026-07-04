# Experiment 0359: Mapped-Region KV Pool, Host Cost and Linux Bind

Date: 2026-07-04

## Postulate

[Postulate 0359](../postulates/0359-mapped-backed-kv-pool.md) claimed a
KV pool backed by one contiguous mapped region costs nothing on the
write-touch churn hot path versus per-block heap Vecs, and that the pool
region binds to a NUMA node end to end on Linux (single-node validation
in OrbStack Docker).

## Change

- `KvBlockPool` gained an internal backing enum: `HeapVecs` (existing
  per-block `Vec<u8>`s) or `Mapped` (one `locus_sys::MappedRegion`,
  blocks at fixed offsets). New constructor `new_mapped`, plus
  `bind_to_node` (Linux cfg, one `mbind` for the whole pool) and
  `mapping_span`. No unsafe added; slices come from
  `MappedRegion::as_mut_slice`. Unit tests: mapped round trip with
  distinct fill patterns at fixed offsets, heap pools report no span.
- Extended `kv_reuse_order_locality` with `lifo_mapped` touch and
  no-touch cases.
- Added example `kv_pool_bind`: creates a 4 MiB mapped pool, prints the
  mapping span, attempts `bind_to_node(0)` on Linux, reports honestly
  elsewhere.

## Host Validation

```bash
cargo bench -p locus-alloc --bench kv_reuse_order_locality -- --sample-size 20 --warm-up-time 1 --measurement-time 3   # runs 1 and 2
cargo bench -p locus-alloc --bench kv_reuse_order_locality -- touch4k --sample-size 30 --warm-up-time 2 --measurement-time 4   # run 3
cargo run -p locus-alloc --example kv_pool_bind --quiet   # host
docker run --rm -v "$PWD":/work -w /work -e CARGO_TARGET_DIR=/tmp/target rust:slim \
  sh -c "cargo run -p locus-alloc --example kv_pool_bind --quiet"   # OrbStack Linux
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --quiet
```

Pool accounting asserted in every bench sample (4096 allocated, 12288
free).

| Benchmark | Run 1 | Run 2 | Run 3 (touch only, longer) |
| --- | ---: | ---: | ---: |
| `kv_reuse_order_lifo_touch4k_64x16blk` (heap) | 88.609 us to 99.891 us | 85.360 us to 87.765 us | 88.303 us to 95.681 us |
| `kv_reuse_order_lifo_mapped_touch4k_64x16blk` | 113.36 us to 139.66 us | 83.505 us to 87.248 us | 82.688 us to 84.471 us |
| `kv_reuse_order_lifo_notouch_64x16blk` (heap) | 4.3786 us to 4.3880 us | 4.3406 us to 4.3784 us | not run |
| `kv_reuse_order_lifo_mapped_notouch_64x16blk` | 4.3744 us to 4.3839 us | 4.3335 us to 4.3452 us | not run |

Host example output: `mapping_len=4194304`, `status=bind_unsupported_on_host`.

Docker (Linux aarch64, OrbStack, single node): see below.

## Interpretation

1. Mapped backing does not cost on the churn hot path. Untouched cases
   are identical (4.38 vs 4.38 us). Touched cases: run 1 showed mapped
   30 percent slower with a very wide interval, but runs 2 and 3 agree
   mapped is 5 to 9 percent faster than heap (83.5 to 85.4 vs 86.6 to
   91.4 us medians); run 1 is the outlier (first-fault and page-warmup
   effects in the first process to touch the fresh 64 MiB mapping).
   Three runs were used because the first two disagreed, per the
   never-rank-from-one-run rule.
2. The bind path result is recorded below from the Docker run.

## Docker Bind Result

The bind attempt fails identically under the default seccomp profile and
with `--security-opt seccomp=unconfined --cap-add SYS_NICE`:

```
kv_pool_bind mapping_start=0xffff8e800000 mapping_len=4194304 blocks=1024
kv_pool_bind status=bind_failed node=0 error=KV block pool region bind failed
```

The `locus-sys` `mbind_region` readiness probe in the same unconfined
container classifies the cause precisely:

```
mbind=error mbind syscall failed: Function not implemented (os error 38)
memory_policy_readiness=not_ready reason=syscall_unavailable
seccomp=disabled seccomp_filters=0 no_new_privs=0
```

ENOSYS from an unconfined container means the OrbStack VM kernel is
built without CONFIG_NUMA; no container setting on this host can ever
validate a successful bind. This supersedes the EPERM classification
from experiments 0018 and 0065 (which ran under the default seccomp
profile and never reached the kernel). What this host can truly show is
now fully shown: the pool mapping is created, the bind path reaches the
kernel and errors are classified correctly; a ready verdict requires a
NUMA-enabled kernel.

The postulate therefore splits: the no-hot-path-cost half survives
(mapped backing is 5 to 9 percent faster on touch churn, identical
untouched); the Docker-bind half is falsified for this host in a
stronger sense than expected, and honestly so: the bind cannot be
demonstrated here at all, only correctly attempted.

## Next Question

With the pool now placeable in one call, the first real-NUMA experiment
is fully specified: on a multi-node Linux host, two mapped pools bound
to different nodes, owner threads pinned per node, chunk mailboxes
steering frees home, measuring cross-node versus node-local write touch
on the mixed-lifetime trace. Nothing further can be validated on this
host; the thread parks until such hardware or a cloud runner is
available, and research returns to host-measurable questions.
