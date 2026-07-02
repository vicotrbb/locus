#![allow(missing_docs)]

use std::{alloc::Layout, mem::MaybeUninit};

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use locus_alloc::{
    KvBlockPool, KvBlockTable, KvSequenceId, MappedScratchArena, RequestScratch,
    RequestScratchPool, ScratchArena,
};
use locus_core::{NodeId, RequestHome, RequestId};

fn scratch_arena_reset_cycle(c: &mut Criterion) {
    c.bench_function("scratch_arena_reset_cycle_64x256b", |bench| {
        let mut arena = ScratchArena::new(NodeId(0), 32 * 1024).expect("arena");
        let layout = Layout::from_size_align(256, 64).expect("layout");

        bench.iter(|| {
            arena.reset();

            for _ in 0..64 {
                let allocation = arena.alloc_bytes(layout).expect("allocation");
                black_box(allocation.as_mut_ptr());
            }

            black_box(arena.stats());
        });
    });
}

fn vec_allocation_cycle(c: &mut Criterion) {
    c.bench_function("vec_allocation_cycle_64x256b", |bench| {
        bench.iter(|| {
            let mut buffers = Vec::with_capacity(64);

            for _ in 0..64 {
                let mut allocation = vec![0_u8; 256];
                black_box(allocation.as_mut_ptr());
                buffers.push(allocation);
            }

            black_box(buffers.len());
        });
    });
}

fn vec_uninit_capacity_allocation_cycle(c: &mut Criterion) {
    c.bench_function("vec_uninit_capacity_allocation_cycle_64x256b", |bench| {
        bench.iter(|| {
            let mut buffers = Vec::with_capacity(64);

            for _ in 0..64 {
                let mut allocation = Vec::<MaybeUninit<u8>>::with_capacity(256);
                black_box(allocation.as_mut_ptr());
                buffers.push(allocation);
            }

            black_box(buffers.len());
        });
    });
}

fn mapped_scratch_arena_reset_cycle(c: &mut Criterion) {
    c.bench_function("mapped_scratch_arena_reset_cycle_64x256b", |bench| {
        let mut arena = MappedScratchArena::new(NodeId(0), 32 * 1024).expect("arena");
        let layout = Layout::from_size_align(256, 64).expect("layout");

        bench.iter(|| {
            arena.reset();

            for _ in 0..64 {
                let allocation = arena.alloc_bytes(layout).expect("allocation");
                black_box(allocation.as_mut_ptr());
            }

            black_box(arena.stats());
        });
    });
}

fn mapped_scratch_write_touch_1mib(c: &mut Criterion) {
    c.bench_function("mapped_scratch_write_touch_1mib", |bench| {
        bench.iter_batched(
            || MappedScratchArena::new(NodeId(0), 1024 * 1024).expect("arena"),
            |mut arena| black_box(arena.write_touch_pages().expect("touch pages")),
            BatchSize::SmallInput,
        );
    });
}

fn vec_write_touch_1mib(c: &mut Criterion) {
    c.bench_function("vec_write_touch_1mib", |bench| {
        bench.iter_batched(
            || vec![0_u8; 1024 * 1024],
            |mut allocation| {
                let page_size = 4096;
                for offset in (0..allocation.len()).step_by(page_size) {
                    allocation[offset] = allocation[offset].wrapping_add(1);
                }
                black_box(allocation);
            },
            BatchSize::SmallInput,
        );
    });
}

fn request_scratch_cycle(c: &mut Criterion) {
    c.bench_function("request_scratch_cycle_16x64x256b", |bench| {
        let homes = (0..16)
            .map(|request| RequestHome {
                request_id: RequestId(request),
                node: Some(NodeId((request % 2) as u32)),
                reason: "bench",
            })
            .collect::<Vec<_>>();
        let layout = Layout::from_size_align(256, 64).expect("layout");

        bench.iter(|| {
            let mut scratch = RequestScratch::new();

            for home in &homes {
                scratch.open_request(home, 32 * 1024).expect("open request");
                for _ in 0..64 {
                    let allocation = scratch
                        .alloc_bytes(home.request_id, layout)
                        .expect("allocation");
                    black_box(allocation.as_mut_ptr());
                }
                black_box(
                    scratch
                        .close_request(home.request_id)
                        .expect("close request"),
                );
            }
        });
    });
}

fn request_vec_allocation_cycle(c: &mut Criterion) {
    c.bench_function("request_vec_allocation_cycle_16x64x256b", |bench| {
        bench.iter(|| {
            let mut request_buffers = Vec::with_capacity(16);

            for _ in 0..16 {
                let mut buffers = Vec::with_capacity(64);
                for _ in 0..64 {
                    let mut allocation = vec![0_u8; 256];
                    black_box(allocation.as_mut_ptr());
                    buffers.push(allocation);
                }
                request_buffers.push(buffers);
            }

            black_box(request_buffers.len());
        });
    });
}

fn request_scratch_pool_cycle(c: &mut Criterion) {
    c.bench_function("request_scratch_pool_cycle_16x64x256b", |bench| {
        let homes = (0..16)
            .map(|request| RequestHome {
                request_id: RequestId(request),
                node: Some(NodeId((request % 2) as u32)),
                reason: "bench",
            })
            .collect::<Vec<_>>();
        let layout = Layout::from_size_align(256, 64).expect("layout");
        let mut pool = RequestScratchPool::new();

        bench.iter(|| {
            for home in &homes {
                pool.open_request(home, 32 * 1024).expect("open request");
                for _ in 0..64 {
                    let allocation = pool
                        .alloc_bytes(home.request_id, layout)
                        .expect("allocation");
                    black_box(allocation.as_mut_ptr());
                }
                black_box(pool.close_request(home.request_id).expect("close request"));
            }
            black_box(pool.pool_stats());
        });
    });
}

fn kv_block_pool_cycle(c: &mut Criterion) {
    c.bench_function("kv_block_pool_cycle_256x4k", |bench| {
        let mut pool = KvBlockPool::new(NodeId(0), 4096, 256).expect("pool");

        bench.iter(|| {
            let mut handles = Vec::with_capacity(256);
            for _ in 0..256 {
                let handle = pool.allocate().expect("block");
                black_box(pool.block_mut(handle).expect("block").as_mut_ptr());
                handles.push(handle);
            }

            for handle in handles {
                pool.free(handle).expect("free block");
            }

            black_box(pool.stats());
        });
    });
}

fn kv_vec_allocation_cycle(c: &mut Criterion) {
    c.bench_function("kv_vec_allocation_cycle_256x4k", |bench| {
        bench.iter(|| {
            let mut blocks = Vec::with_capacity(256);
            for _ in 0..256 {
                let mut block = vec![0_u8; 4096];
                black_box(block.as_mut_ptr());
                blocks.push(block);
            }
            black_box(blocks.len());
        });
    });
}

fn kv_vec_uninit_capacity_allocation_cycle(c: &mut Criterion) {
    c.bench_function("kv_vec_uninit_capacity_allocation_cycle_256x4k", |bench| {
        bench.iter(|| {
            let mut blocks = Vec::with_capacity(256);
            for _ in 0..256 {
                let mut block = Vec::<MaybeUninit<u8>>::with_capacity(4096);
                black_box(block.as_mut_ptr());
                blocks.push(block);
            }
            black_box(blocks.len());
        });
    });
}

fn kv_block_table_append_release_cycle(c: &mut Criterion) {
    c.bench_function("kv_block_table_append_release_128x16tokens", |bench| {
        let mut pool = KvBlockPool::new(NodeId(0), 4096, 128).expect("pool");

        bench.iter(|| {
            let mut table = KvBlockTable::new(KvSequenceId(1), 16).expect("table");
            for _ in 0..128 {
                table.append_tokens(&mut pool, 16).expect("append tokens");
            }
            black_box(table.stats());
            table.release_all(&mut pool).expect("release table");
            black_box(pool.stats());
        });
    });
}

fn kv_vec_table_allocation_cycle(c: &mut Criterion) {
    c.bench_function("kv_vec_table_allocation_128x4k", |bench| {
        bench.iter(|| {
            let mut table = Vec::with_capacity(128);
            for _ in 0..128 {
                let mut block = vec![0_u8; 4096];
                black_box(block.as_mut_ptr());
                table.push(block);
            }
            black_box(table.len());
        });
    });
}

criterion_group!(
    benches,
    scratch_arena_reset_cycle,
    vec_allocation_cycle,
    vec_uninit_capacity_allocation_cycle,
    mapped_scratch_arena_reset_cycle,
    mapped_scratch_write_touch_1mib,
    vec_write_touch_1mib,
    request_scratch_cycle,
    request_vec_allocation_cycle,
    request_scratch_pool_cycle,
    kv_block_pool_cycle,
    kv_vec_allocation_cycle,
    kv_vec_uninit_capacity_allocation_cycle,
    kv_block_table_append_release_cycle,
    kv_vec_table_allocation_cycle
);
criterion_main!(benches);
