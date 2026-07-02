#![allow(missing_docs)]

use std::{alloc::System, mem::MaybeUninit};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[global_allocator]
static GLOBAL: System = System;

fn system_vec_allocation_cycle(c: &mut Criterion) {
    c.bench_function("system_vec_allocation_cycle_64x256b", |bench| {
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

fn system_vec_uninit_capacity_allocation_cycle(c: &mut Criterion) {
    c.bench_function(
        "system_vec_uninit_capacity_allocation_cycle_64x256b",
        |bench| {
            bench.iter(|| {
                let mut buffers = Vec::with_capacity(64);

                for _ in 0..64 {
                    let mut allocation = Vec::<MaybeUninit<u8>>::with_capacity(256);
                    black_box(allocation.as_mut_ptr());
                    buffers.push(allocation);
                }

                black_box(buffers.len());
            });
        },
    );
}

fn system_kv_vec_allocation_cycle(c: &mut Criterion) {
    c.bench_function("system_kv_vec_allocation_cycle_256x4k", |bench| {
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

fn system_kv_vec_uninit_capacity_allocation_cycle(c: &mut Criterion) {
    c.bench_function(
        "system_kv_vec_uninit_capacity_allocation_cycle_256x4k",
        |bench| {
            bench.iter(|| {
                let mut blocks = Vec::with_capacity(256);
                for _ in 0..256 {
                    let mut block = Vec::<MaybeUninit<u8>>::with_capacity(4096);
                    black_box(block.as_mut_ptr());
                    blocks.push(block);
                }
                black_box(blocks.len());
            });
        },
    );
}

criterion_group!(
    benches,
    system_vec_allocation_cycle,
    system_vec_uninit_capacity_allocation_cycle,
    system_kv_vec_allocation_cycle,
    system_kv_vec_uninit_capacity_allocation_cycle
);
criterion_main!(benches);
