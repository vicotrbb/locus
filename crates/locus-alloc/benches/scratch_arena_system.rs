#![allow(missing_docs)]

use std::{alloc::System, mem::MaybeUninit, sync::mpsc::sync_channel, thread};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[global_allocator]
static GLOBAL: System = System;

enum ProducerCommand {
    Run(usize),
    Stop,
}

enum HandoffMessage {
    Block(Vec<u8>),
    EndBatch,
    Stop,
}

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

fn system_vec_producer_consumer_handoff_cycle(c: &mut Criterion) {
    c.bench_function("system_vec_producer_consumer_handoff_256x4k", |bench| {
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

fn system_vec_persistent_worker_handoff_cycle(c: &mut Criterion) {
    c.bench_function("system_vec_persistent_worker_handoff_256x4k", |bench| {
        let (command_sender, command_receiver) = sync_channel::<ProducerCommand>(1);
        let (handoff_sender, handoff_receiver) = sync_channel::<HandoffMessage>(32);
        let (done_sender, done_receiver) = sync_channel::<usize>(1);

        let producer = thread::spawn(move || {
            while let Ok(command) = command_receiver.recv() {
                match command {
                    ProducerCommand::Run(blocks) => {
                        for _ in 0..blocks {
                            let mut block = vec![0_u8; 4096];
                            black_box(block.as_mut_ptr());
                            handoff_sender
                                .send(HandoffMessage::Block(block))
                                .expect("send block");
                        }
                        handoff_sender
                            .send(HandoffMessage::EndBatch)
                            .expect("send end batch");
                    }
                    ProducerCommand::Stop => {
                        handoff_sender
                            .send(HandoffMessage::Stop)
                            .expect("send stop");
                        break;
                    }
                }
            }
        });

        let consumer = thread::spawn(move || {
            let mut blocks = 0_usize;
            while let Ok(message) = handoff_receiver.recv() {
                match message {
                    HandoffMessage::Block(mut block) => {
                        black_box(block.as_mut_ptr());
                        blocks += 1;
                    }
                    HandoffMessage::EndBatch => {
                        done_sender.send(blocks).expect("send done");
                        blocks = 0;
                    }
                    HandoffMessage::Stop => break,
                }
            }
        });

        bench.iter(|| {
            command_sender
                .send(ProducerCommand::Run(256))
                .expect("send run");
            black_box(done_receiver.recv().expect("receive done"));
        });

        command_sender
            .send(ProducerCommand::Stop)
            .expect("send stop");
        producer.join().expect("producer thread");
        consumer.join().expect("consumer thread");
    });
}

criterion_group!(
    benches,
    system_vec_allocation_cycle,
    system_vec_uninit_capacity_allocation_cycle,
    system_kv_vec_allocation_cycle,
    system_kv_vec_uninit_capacity_allocation_cycle,
    system_vec_producer_consumer_handoff_cycle,
    system_vec_persistent_worker_handoff_cycle
);
criterion_main!(benches);
