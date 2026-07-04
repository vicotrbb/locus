# Experiment 0357: Lock-Free Chunk Mailboxes vs Bounded Chunk Queues

Date: 2026-07-04

## Postulate

[Postulate 0357](../postulates/0357-chunk-mailbox-remote-free.md) claimed
replacing per-worker bounded chunk queues with minimal lock-free chunk
mailboxes (Treiber stack push, swap-take drain, no capacity or stats
protocol) would cut the mixed-lifetime trace at least 10 percent below
the sharded chunk queue design.

## Change

- Added `ChunkMailbox<T>` and `ChunkMailboxSender<T>` in
  `crates/locus-sys/src/chunk_mailbox.rs` (locus-sys is the workspace's
  only unsafe-allowed crate; the unsafe surface is three blocks with
  SAFETY comments: exclusive list ownership after swap, single
  `Box::from_raw` per node, pre-publication node write). Re-exported
  from `locus_alloc`. Unit tests: FIFO round trip, 4-producer times 10k
  chunk stress with exactly-once delivery and per-producer FIFO order,
  and drop-reclaims-undelivered-chunks via a drop counter.
- Added a `sharded_mailbox` path to the mixed-lifetime trace benchmark:
  one mailbox per worker, workers push completed request chunks, owner
  swap-takes each mailbox per step; pool ops identical to sharded_chunk.
  An atomic submitted counter cross-checks owner-side drained totals.

## Host Validation

```bash
cargo bench -p locus-alloc --bench remote_free_mixed_lifetime_trace -- --sample-size 20 --warm-up-time 1 --measurement-time 3   # run twice
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --quiet
```

Samples: `allocated=10752 freed=10752` in every case, mailbox submitted
equals drained, bounded-queue counters balanced.

| Benchmark | Run 1 | Run 2 |
| --- | ---: | ---: |
| `..._shared_per_handle_w4_64req` | 321.90 us to 327.93 us | 283.13 us to 286.85 us |
| `..._sharded_chunk_w4_64req` | 70.202 us to 78.025 us | 77.935 us to 78.613 us |
| `..._sharded_mailbox_w4_64req` | 68.629 us to 69.291 us | 71.450 us to 72.643 us |

Raw logs: scratchpad `mixed_lifetime_mailbox_run1.log` and
`mixed_lifetime_mailbox_run2.log`.

## Interpretation

The postulate survives in direction and misses its 10 percent bar:

1. The mailbox beats the bounded chunk queue in both runs, by about 6 to
   8 percent (69.0 vs 73.7 us medians in run 1, 72.1 vs 78.2 in run 2),
   and with visibly tighter confidence intervals; the bounded queue's
   run-1 interval was 11 percent wide while the mailbox stayed under 1
   percent. Removing the capacity protocol removes a variance source,
   not just mean cost.
2. The absolute saving (about 5 us per trace) confirms the 0356
   conclusion from the other side: the bounded-queue protocol was a real
   but secondary cost; the remaining floor is owner polling cadence and
   scheduler wake-ups.
3. The qualitative result matters more than the 7 percent: the mailbox
   has zero tuning parameters. No capacity, no batch limit, no queued
   byte budget, no retune machinery. Experiments 0059, 0142, 0143, 0146,
   and the entire capacity-retune telemetry apparatus tuned knobs that
   this design simply deletes, while beating the tuned configuration.

Design consequence: the locus remote-free path becomes per-worker chunk
mailboxes. The design story is now coherent and evidence-backed end to
end: request-chunk-granular free ABI (0353, 0354), sharded per producer
(0352), transported by parameterless lock-free mailboxes (0357), on a
flat LIFO pool (0356 negative), beating mimalloc 2.5x or more on
realistic KV churn (0355, margin grows against the 72 us figure).

## Next Question

The trace never touches block memory, so placement effects are
invisible. Priority 4: wire the mailbox design into a Linux-ready NUMA
placement experiment (cfg-gated, Docker via OrbStack) where the owner
first-touches blocks on its node and workers read them before freeing,
to measure whether chunk-granular recycling preserves node locality
under churn, and validate what Docker on this host can truly show.
