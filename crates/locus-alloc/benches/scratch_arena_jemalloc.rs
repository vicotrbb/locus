#![allow(missing_docs)]

use std::{mem::MaybeUninit, sync::mpsc::sync_channel, thread};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tikv_jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn jemalloc_vec_allocation_cycle(c: &mut Criterion) {
    c.bench_function("jemalloc_vec_allocation_cycle_64x256b", |bench| {
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

fn jemalloc_vec_uninit_capacity_allocation_cycle(c: &mut Criterion) {
    c.bench_function(
        "jemalloc_vec_uninit_capacity_allocation_cycle_64x256b",
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

fn jemalloc_kv_vec_allocation_cycle(c: &mut Criterion) {
    c.bench_function("jemalloc_kv_vec_allocation_cycle_256x4k", |bench| {
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

fn jemalloc_kv_vec_uninit_capacity_allocation_cycle(c: &mut Criterion) {
    c.bench_function(
        "jemalloc_kv_vec_uninit_capacity_allocation_cycle_256x4k",
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

fn jemalloc_vec_producer_consumer_handoff_cycle(c: &mut Criterion) {
    c.bench_function("jemalloc_vec_producer_consumer_handoff_256x4k", |bench| {
        bench.iter(|| {
            thread::scope(|scope| {
                let (sender, receiver) = sync_channel::<Vec<u8>>(32);

                let consumer = scope.spawn(move || {
                    let mut blocks = 0_usize;
                    while let Ok(mut block) = receiver.recv() {
                        black_box(block.as_mut_ptr());
                        blocks += 1;
                    }
                    black_box(blocks);
                });

                let producer = scope.spawn(move || {
                    for _ in 0..256 {
                        let mut block = vec![0_u8; 4096];
                        black_box(block.as_mut_ptr());
                        sender.send(block).expect("send block");
                    }
                });

                producer.join().expect("producer thread");
                consumer.join().expect("consumer thread");
            });
        });
    });
}

criterion_group!(
    benches,
    jemalloc_vec_allocation_cycle,
    jemalloc_vec_uninit_capacity_allocation_cycle,
    jemalloc_kv_vec_allocation_cycle,
    jemalloc_kv_vec_uninit_capacity_allocation_cycle,
    jemalloc_vec_producer_consumer_handoff_cycle
);
criterion_main!(benches);
