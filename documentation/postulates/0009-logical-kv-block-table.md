# Postulate 0009: Logical KV Block Table

Date: 2026-07-02

## Statement

A logical sequence-to-block table is the next foundation step after fixed-size KV block reuse because LLM KV cache growth is token driven rather than raw block driven.

## Rationale

Paged KV systems separate logical sequence growth from physical block storage. Locus should first model that separation safely: append tokens, allocate additional fixed blocks as needed, and release all sequence-owned blocks at completion.

## Experiment

Add a `KvBlockTable` that:

- tracks one sequence identifier;
- stores tokens per block and logical token length;
- appends tokens and allocates extra `KvBlockPool` blocks as needed;
- rolls back newly acquired blocks if an append fails;
- releases all blocks back to the pool at sequence completion;
- benchmarks append and release against a simple `Vec<u8>` block-table baseline.

## Expected Result

The table should pass correctness tests and provide a safe baseline for future work on prefix sharing, block lookup, and KV eviction behavior.
