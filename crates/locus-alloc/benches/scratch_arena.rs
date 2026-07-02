#![allow(missing_docs)]

use std::alloc::Layout;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use locus_alloc::{RequestScratch, RequestScratchPool, ScratchArena};
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

criterion_group!(
    benches,
    scratch_arena_reset_cycle,
    vec_allocation_cycle,
    request_scratch_cycle,
    request_vec_allocation_cycle,
    request_scratch_pool_cycle
);
criterion_main!(benches);
