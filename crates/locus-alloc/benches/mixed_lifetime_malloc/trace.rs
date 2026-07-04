use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{sync_channel, SyncSender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use criterion::{black_box, Criterion};

pub const BLOCK_SIZE: usize = 4096;
pub const REQUESTS: usize = 64;
pub const ARRIVALS_PER_STEP: usize = 4;
pub const PREFILL_BLOCKS: usize = 16;
pub const CANCEL_DECODE_STEPS: usize = 8;
pub const WORKERS: usize = 4;
pub const BLOCKS_PER_TRACE: usize = 2688;

enum WorkerCommand {
    Free(Vec<Vec<u8>>),
    Stop,
}

struct Request {
    blocks: Vec<Vec<u8>>,
    decode_steps_left: usize,
}

fn decode_steps(request_index: usize) -> usize {
    if request_index % 4 == 0 {
        CANCEL_DECODE_STEPS
    } else {
        16 + (request_index % 3) * 16
    }
}

struct WorkerPool {
    commands: Vec<SyncSender<WorkerCommand>>,
    workers: Vec<JoinHandle<()>>,
}

impl WorkerPool {
    fn spawn(freed_blocks: &Arc<AtomicUsize>) -> Self {
        let mut commands = Vec::with_capacity(WORKERS);
        let mut workers = Vec::with_capacity(WORKERS);

        for _ in 0..WORKERS {
            let (command_sender, command_receiver) = sync_channel::<WorkerCommand>(REQUESTS);
            let freed = Arc::clone(freed_blocks);
            commands.push(command_sender);
            workers.push(thread::spawn(move || {
                while let Ok(WorkerCommand::Free(blocks)) = command_receiver.recv() {
                    let count = blocks.len();
                    drop(blocks);
                    freed.fetch_add(count, Ordering::Release);
                }
            }));
        }

        Self { commands, workers }
    }

    fn dispatch(&self, worker: usize, blocks: Vec<Vec<u8>>) {
        self.commands[worker]
            .send(WorkerCommand::Free(blocks))
            .expect("worker accepts free command");
    }

    fn stop(self) {
        for command in &self.commands {
            command
                .send(WorkerCommand::Stop)
                .expect("worker accepts stop command");
        }
        for worker in self.workers {
            worker.join().expect("worker thread joins");
        }
    }
}

fn allocate_block() -> Vec<u8> {
    let mut block = Vec::with_capacity(BLOCK_SIZE);
    block.push(1_u8);
    black_box(block.as_mut_ptr());
    block
}

fn run_trace(worker_pool: &WorkerPool, freed_blocks: &AtomicUsize) -> usize {
    let baseline = freed_blocks.load(Ordering::Acquire);
    let mut active: Vec<Request> = Vec::with_capacity(REQUESTS);
    let mut arrived = 0_usize;
    let mut allocated_blocks = 0_usize;
    let mut next_worker = 0_usize;

    while arrived < REQUESTS || !active.is_empty() {
        let arriving = ARRIVALS_PER_STEP.min(REQUESTS - arrived);
        for _ in 0..arriving {
            let mut blocks = Vec::with_capacity(PREFILL_BLOCKS + decode_steps(arrived));
            for _ in 0..PREFILL_BLOCKS {
                blocks.push(allocate_block());
            }
            allocated_blocks += PREFILL_BLOCKS;
            active.push(Request {
                blocks,
                decode_steps_left: decode_steps(arrived),
            });
            arrived += 1;
        }

        let mut index = 0;
        while index < active.len() {
            if active[index].decode_steps_left == 0 {
                let request = active.swap_remove(index);
                worker_pool.dispatch(next_worker, request.blocks);
                next_worker = (next_worker + 1) % WORKERS;
                continue;
            }
            active[index].blocks.push(allocate_block());
            allocated_blocks += 1;
            active[index].decode_steps_left -= 1;
            index += 1;
        }
    }

    while freed_blocks.load(Ordering::Acquire) - baseline < allocated_blocks {
        thread::yield_now();
    }

    allocated_blocks
}

pub fn bench_mixed_lifetime_malloc(c: &mut Criterion, allocator_name: &str) {
    let freed_blocks = Arc::new(AtomicUsize::new(0));
    let worker_pool = WorkerPool::spawn(&freed_blocks);

    let mut sample_allocated = 0_usize;
    for _ in 0..4 {
        sample_allocated += run_trace(&worker_pool, &freed_blocks);
    }
    assert_eq!(sample_allocated, 4 * BLOCKS_PER_TRACE);
    assert_eq!(freed_blocks.load(Ordering::Acquire), sample_allocated);
    println!(
        "mixed_lifetime_malloc_sample allocator={allocator_name} traces=4 \
         allocated={sample_allocated} freed={}",
        freed_blocks.load(Ordering::Acquire),
    );

    let name = format!("kv_mixed_lifetime_{allocator_name}_w{WORKERS}_{REQUESTS}req");
    c.bench_function(&name, |b| {
        b.iter(|| {
            let allocated = run_trace(&worker_pool, &freed_blocks);
            assert_eq!(allocated, BLOCKS_PER_TRACE);
        });
    });

    worker_pool.stop();
}
