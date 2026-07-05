//! LOCUS-EVAL v1 runner for global-allocator baselines. Blocks are
//! real 4 KiB heap allocations freed natively on worker threads; the
//! trace shapes come from `workloads.rs` and must match the locus
//! runner exactly.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{sync_channel, SyncSender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use criterion::{black_box, Criterion};

use super::workloads::{
    Workload, BLOCK_SIZE, CHURN_CHUNK_BLOCKS, CHURN_LIVE_CHUNKS, CHURN_STEPS, PREFILL_BLOCKS,
    REQUESTS, TRACE_WORKLOADS, WORKERS,
};

enum WorkerCommand {
    Free(Vec<Vec<u8>>),
    Stop,
}

struct Request {
    blocks: Vec<Vec<u8>>,
    decode_steps_left: usize,
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

fn allocate_touched_block(fill: u8) -> Vec<u8> {
    let mut block = Vec::with_capacity(BLOCK_SIZE);
    block.resize(BLOCK_SIZE, fill);
    black_box(block.as_mut_ptr());
    block
}

struct TraceOutcome {
    allocated: usize,
    peak_outstanding: usize,
}

fn run_trace(
    workload: Workload,
    worker_pool: &WorkerPool,
    freed_blocks: &AtomicUsize,
) -> TraceOutcome {
    let baseline = freed_blocks.load(Ordering::Acquire);
    let mut active: Vec<Request> = Vec::with_capacity(REQUESTS);
    let mut arrived = 0_usize;
    let mut allocated_blocks = 0_usize;
    let mut peak_outstanding = 0_usize;
    let mut next_worker = 0_usize;

    while arrived < REQUESTS || !active.is_empty() {
        let arriving = workload.arrivals_per_step().min(REQUESTS - arrived);
        for _ in 0..arriving {
            let mut blocks = Vec::with_capacity(PREFILL_BLOCKS + workload.decode_steps(arrived));
            for _ in 0..PREFILL_BLOCKS {
                blocks.push(allocate_block());
            }
            allocated_blocks += PREFILL_BLOCKS;
            active.push(Request {
                blocks,
                decode_steps_left: workload.decode_steps(arrived),
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

        let freed = freed_blocks.load(Ordering::Acquire) - baseline;
        peak_outstanding = peak_outstanding.max(allocated_blocks - freed.min(allocated_blocks));
    }

    while freed_blocks.load(Ordering::Acquire) - baseline < allocated_blocks {
        thread::yield_now();
    }

    TraceOutcome {
        allocated: allocated_blocks,
        peak_outstanding,
    }
}

fn run_churn(
    worker_pool: &WorkerPool,
    freed_blocks: &AtomicUsize,
    live: &mut VecDeque<Vec<Vec<u8>>>,
    fill: u8,
) -> TraceOutcome {
    let baseline = freed_blocks.load(Ordering::Acquire);
    let mut allocated_blocks = 0_usize;
    let mut dispatched = 0_usize;
    let mut peak_in_flight = 0_usize;
    let mut next_worker = 0_usize;

    for _ in 0..CHURN_STEPS {
        let oldest = live.pop_front().expect("steady-state chunk available");
        dispatched += oldest.len();
        worker_pool.dispatch(next_worker, oldest);
        next_worker = (next_worker + 1) % WORKERS;

        let mut chunk = Vec::with_capacity(CHURN_CHUNK_BLOCKS);
        for _ in 0..CHURN_CHUNK_BLOCKS {
            chunk.push(allocate_touched_block(fill));
        }
        allocated_blocks += CHURN_CHUNK_BLOCKS;
        live.push_back(chunk);

        let freed = freed_blocks.load(Ordering::Acquire) - baseline;
        peak_in_flight = peak_in_flight.max(dispatched - freed.min(dispatched));
    }

    while freed_blocks.load(Ordering::Acquire) - baseline < dispatched {
        thread::yield_now();
    }

    TraceOutcome {
        allocated: allocated_blocks,
        peak_outstanding: CHURN_LIVE_CHUNKS * CHURN_CHUNK_BLOCKS + peak_in_flight,
    }
}

fn bench_workload(
    c: &mut Criterion,
    workload: Workload,
    allocator_name: &str,
    worker_pool: &WorkerPool,
    freed_blocks: &Arc<AtomicUsize>,
) {
    let mut live: VecDeque<Vec<Vec<u8>>> = VecDeque::new();
    if workload == Workload::ChurnTouch {
        for _ in 0..CHURN_LIVE_CHUNKS {
            let mut chunk = Vec::with_capacity(CHURN_CHUNK_BLOCKS);
            for _ in 0..CHURN_CHUNK_BLOCKS {
                chunk.push(allocate_touched_block(0xA5));
            }
            live.push_back(chunk);
        }
    }

    let mut peak_outstanding = 0_usize;
    let mut total_allocated = 0_usize;
    for round in 0..4_u8 {
        let outcome = if workload == Workload::ChurnTouch {
            run_churn(worker_pool, freed_blocks, &mut live, round)
        } else {
            run_trace(workload, worker_pool, freed_blocks)
        };
        assert_eq!(outcome.allocated, workload.blocks_per_trace());
        total_allocated += outcome.allocated;
        peak_outstanding = peak_outstanding.max(outcome.peak_outstanding);
    }
    println!(
        "locus_eval_sample workload={} contender={} traces=4 allocated={} freed={} \
         peak_outstanding={} theoretical_peak={}",
        workload.name(),
        allocator_name,
        total_allocated,
        total_allocated,
        peak_outstanding,
        workload.theoretical_peak_live(),
    );

    let name = format!(
        "locus_eval_{}_{}_w{WORKERS}",
        workload.name(),
        allocator_name
    );
    c.bench_function(&name, |b| {
        b.iter(|| {
            let outcome = if workload == Workload::ChurnTouch {
                run_churn(worker_pool, freed_blocks, &mut live, 0x5A)
            } else {
                run_trace(workload, worker_pool, freed_blocks)
            };
            assert_eq!(outcome.allocated, workload.blocks_per_trace());
        });
    });
}

pub fn bench_locus_eval_malloc(c: &mut Criterion, allocator_name: &str) {
    let freed_blocks = Arc::new(AtomicUsize::new(0));
    let worker_pool = WorkerPool::spawn(&freed_blocks);

    for workload in TRACE_WORKLOADS {
        bench_workload(c, workload, allocator_name, &worker_pool, &freed_blocks);
    }
    bench_workload(
        c,
        Workload::ChurnTouch,
        allocator_name,
        &worker_pool,
        &freed_blocks,
    );

    worker_pool.stop();
}
