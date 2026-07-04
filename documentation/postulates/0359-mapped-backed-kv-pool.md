# Postulate 0359: A Mapped-Region KV Pool Is NUMA-Placeable at No Hot-Path Cost

Date: 2026-07-04

## Statement

The current `KvBlockPool` backs blocks with individually heap-allocated
`Vec<u8>`s, which makes NUMA placement impossible: there is no single
region to bind. Backing the pool with one contiguous page-aligned mapped
region (blocks as fixed offsets) should cost nothing on the write-touch
churn hot path, and may even help through contiguity and TLB behavior,
while making the whole pool bindable with one `mbind` call. On Linux
(OrbStack Docker on this host, single node), binding the pool region to
node 0 should succeed and report a ready verdict end to end, proving the
same binary is placement-ready for a real multi-node host.

## Rationale

Experiments 0354 to 0358 fixed the transport and reuse policy; the NUMA
design (per-node pools, chunk steering) now needs a pool whose memory
can actually be placed. Standard practice for serving engines is a
single large slab per device or node; the open question here is whether
moving from per-block heap Vecs to one mapped region changes host churn
performance at all (allocation is index recycling either way, but block
writes hit one contiguous 64 MiB region instead of 16384 scattered heap
chunks). Contiguity could cut TLB pressure or increase cache conflict
misses; measuring decides. The Docker leg validates only what one node
can truly show: that the bind path (mmap, mbind, verdict plumbing) works
for the pool region.

## Experiment

1. Add a mapped backing to `KvBlockPool`: internal enum over the
   existing per-block Vecs and a single `locus_sys::MappedRegion`
   (blocks as offsets), constructor `new_mapped`, `block_mut` serving
   slices from either backing, plus `bind_to_node` (Linux) and
   `mapping_span` accessors for the mapped variant. Unit tests cover
   mapped round trip, block isolation at offsets, and capacity math.
2. Extend `kv_reuse_order_locality` with `lifo_mapped` touch and
   no-touch cases (same churn as 0358) to compare against heap-backed
   `lifo` on the same runs, twice.
3. Add example `kv_pool_bind` that creates a mapped pool, binds it to
   node 0, and prints a readiness line; run it on the host (expect
   unsupported) and in OrbStack Docker (expect ready or a specific
   kernel refusal, recorded honestly).

## Expected Result

lifo_mapped within noise of lifo (or faster) on the touch churn, and a
successful single-node bind in Docker. If mapped backing is measurably
slower on the host, NUMA placement has a hot-path price that must be
weighed explicitly; if the Docker bind fails, the placement design needs
a different syscall path before real-NUMA validation.
