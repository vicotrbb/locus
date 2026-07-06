#![allow(missing_docs)]

use std::collections::VecDeque;

use criterion::{criterion_group, criterion_main, Criterion};
use locus::NodeId;
use locus::{KvBlockHandle, KvBlockPool, KvReuseOrder};

const BLOCK_SIZE: usize = 4096;
const POOL_BLOCKS: usize = 16384;
const CHUNK_BLOCKS: usize = 16;
const LIVE_CHUNKS: usize = 256;
const STEPS_PER_CYCLE: usize = 64;

fn allocate_chunk(pool: &mut KvBlockPool, touch: bool, fill: u8) -> Vec<KvBlockHandle> {
    let mut chunk = Vec::with_capacity(CHUNK_BLOCKS);
    for _ in 0..CHUNK_BLOCKS {
        let handle = pool.allocate().expect("pool has free blocks");
        if touch {
            pool.block_mut(handle)
                .expect("live handle resolves")
                .fill(fill);
        }
        chunk.push(handle);
    }
    chunk
}

fn free_chunk(pool: &mut KvBlockPool, chunk: Vec<KvBlockHandle>) {
    for handle in chunk {
        pool.free(handle).expect("live handle frees");
    }
}

fn run_cycle(pool: &mut KvBlockPool, live: &mut VecDeque<Vec<KvBlockHandle>>, touch: bool) {
    for step in 0..STEPS_PER_CYCLE {
        let oldest = live.pop_front().expect("live window is primed");
        free_chunk(pool, oldest);
        #[allow(clippy::cast_possible_truncation)]
        let fill = step as u8;
        live.push_back(allocate_chunk(pool, touch, fill));
    }
}

fn bench_kv_reuse_order_locality(c: &mut Criterion) {
    let cases = [
        ("lifo", KvReuseOrder::Lifo, true, false),
        ("fifo", KvReuseOrder::Fifo, true, false),
        ("lifo_mapped", KvReuseOrder::Lifo, true, true),
        ("lifo", KvReuseOrder::Lifo, false, false),
        ("fifo", KvReuseOrder::Fifo, false, false),
        ("lifo_mapped", KvReuseOrder::Lifo, false, true),
    ];

    for (order_name, order, touch, mapped) in cases {
        let mut pool = if mapped {
            KvBlockPool::new_mapped(NodeId(0), BLOCK_SIZE, POOL_BLOCKS, order)
                .expect("mapped pool configuration")
        } else {
            KvBlockPool::new_with_reuse_order(NodeId(0), BLOCK_SIZE, POOL_BLOCKS, order)
                .expect("pool configuration")
        };
        let mut live: VecDeque<Vec<KvBlockHandle>> = VecDeque::with_capacity(LIVE_CHUNKS);
        for _ in 0..LIVE_CHUNKS {
            live.push_back(allocate_chunk(&mut pool, touch, 0));
        }

        for _ in 0..4 {
            run_cycle(&mut pool, &mut live, touch);
        }
        let stats = pool.stats();
        println!(
            "reuse_order_sample order={order_name} touch={touch} allocated={} free={} \
             allocation_count={} free_count={}",
            stats.allocated, stats.free, stats.allocation_count, stats.free_count,
        );
        assert_eq!(stats.allocated, LIVE_CHUNKS * CHUNK_BLOCKS);
        assert_eq!(stats.free, POOL_BLOCKS - LIVE_CHUNKS * CHUNK_BLOCKS);

        let touch_name = if touch { "touch4k" } else { "notouch" };
        let name =
            format!("kv_reuse_order_{order_name}_{touch_name}_{STEPS_PER_CYCLE}x{CHUNK_BLOCKS}blk");
        c.bench_function(&name, |b| {
            b.iter(|| {
                run_cycle(&mut pool, &mut live, touch);
            });
        });

        while let Some(chunk) = live.pop_front() {
            free_chunk(&mut pool, chunk);
        }
        assert_eq!(pool.stats().allocated, 0);
    }
}

criterion_group!(benches, bench_kv_reuse_order_locality);
criterion_main!(benches);
