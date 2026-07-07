# Ferrite Is Now a KvBlockPool Consumer

Date: 2026-07-06

## What happened

Ferrite (a separate sibling repo, `../ferrite`) added an opt-in KV-cache
storage backend for its scalar inference session that embeds
`locus_alloc::KvBlockPool` as a single-owner, mapped, LIFO-reuse block
allocator, replacing per-token `Vec<f32>` heap allocations with fixed-size
pool blocks (`tokens_per_block` positions per block, one block list per
`(layer, K|V)`). This is real proof-of-consumer usage, not a synthetic
benchmark: `KvBlockPool::new_mapped(NodeId(0), block_size, capacity,
KvReuseOrder::Lifo)` is constructed per inference session, `allocate` is
called lazily on block boundaries, `block_mut` is used for both reads and
writes (reinterpreted as `f32` via `bytemuck`, relying on mmap page
alignment), and `free` runs on `truncate`, handing blocks back for LIFO reuse
within that same session. The pool is owned by the session (via Ferrite's
`LocusKvStore`, itself owned by `ScalarLlamaSession`), not shared or reused
across sessions: on session drop the whole pool — every block in it, freed or
not — is released together (its `MappedRegion` unmaps), rather than being
freed block-by-block for a later session to reuse.

Ferrite consumed the `locus-alloc` crate as-is via a path dependency. No
Locus code was changed for this work; the two observations below are backlog
for Locus, not requests folded into this note as changes.

Ferrite-side references, for anyone who wants to see the consumer code:

- `crates/ferrite-inference/src/scalar/kv_store/locus.rs` (the `LocusKvStore`
  wrapper)
- `crates/ferrite-inference/src/scalar/kv_store.rs` (the `KvCacheStore` seam)
- `documentation/adr/0010-locus-kv-block-backend.md` (the Ferrite-side ADR)
- `documentation/dev-notes/2026-07-06-locus-kv-backend.md` and
  `documentation/benchmarks/2026-07-06-locus-kv-backend.md` (Ferrite-side
  evidence)

## API friction observed (backlog, not fixed here)

1. **No immutable or typed block accessor.** `KvBlockPool` exposes block
   bytes only through `block_mut(&mut self, handle) -> Result<&mut [u8],
   KvBlockPoolError>`. There is no `block(&self, handle) -> &[u8]` (or
   similarly typed) read-only accessor. A consumer that only wants to *read*
   a block — the common case for attention's K-score and V-output passes —
   still has to take `&mut self` and do its own `bytemuck` cast to get a
   typed view. This forces every read to look like a write in the borrow
   checker's eyes, which in turn is why a single-block-at-a-time borrow
   discipline (one `block_mut` call, use it, drop it, move to the next
   block) is required rather than holding several block views live at once.
   That discipline turned out to be a fine fit for Ferrite's sequential
   attention access pattern, but it is a real constraint a future consumer
   with a different access pattern would hit immediately.

2. **`KvBlockTable` has no indexed handle access.** `KvBlockTable` (see
   `crates/locus-alloc/src/kv/block.rs`) supports appending and releasing
   blocks (`append_tokens`-style growth, `release_all`) but does not expose a
   way to look up "the handle for logical position `t`" or "the handle at
   table index `i`". Ferrite needed exactly a position -> block lookup (to
   find which block holds token position `t`, then the byte offset within
   it), and `KvBlockTable` in its current form doesn't provide it, so the
   consumer hand-rolled a thin per-`(layer, K|V)` ordered `Vec<KvBlockHandle>`
   instead (`LocusKvStore::key_blocks` / `value_blocks` in
   `kv_store/locus.rs`), with `position / tokens_per_block` as the index.
   `KvBlockTable` was evaluated first per the design's stated preference, but
   without indexed access it didn't fit; if `KvBlockTable` gained indexed
   handle access this hand-rolled table could likely be replaced with it
   directly.

Neither observation blocked the integration; both are noted here as
candidate ergonomic improvements for a future Locus API pass.
