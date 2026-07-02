#![allow(missing_docs)]

use std::alloc::Layout;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use locus_alloc::ScratchArena;
use locus_core::NodeId;

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

criterion_group!(benches, scratch_arena_reset_cycle, vec_allocation_cycle);
criterion_main!(benches);
