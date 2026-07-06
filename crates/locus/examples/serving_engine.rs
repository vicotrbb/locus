//! The serving-engine embedding pattern for Locus.
//!
//! One owner thread owns the `KvBlockPool`. Each worker thread gets its own
//! `ChunkMailboxSender` and frees a finished request's blocks as ONE chunk
//! push (the design validated by experiment 0357 and LOCUS-EVAL v1). The
//! owner drains every mailbox once per scheduling step and returns blocks
//! to the pool; the free path never contends on a shared queue.

use std::sync::mpsc::{channel, Sender};
use std::thread;

use locus::{ChunkMailbox, KvBlockHandle, KvBlockPool, NodeId};

const WORKERS: usize = 4;
const REQUESTS_PER_WORKER: usize = 8;
const BLOCKS_PER_REQUEST: usize = 16;
const BLOCK_SIZE: usize = 4096;

fn main() {
    let pool_blocks = WORKERS * REQUESTS_PER_WORKER * BLOCKS_PER_REQUEST;
    let mut pool = KvBlockPool::new(NodeId(0), BLOCK_SIZE, pool_blocks).expect("pool construction");

    // One mailbox per worker, owned by the pool owner; workers hold senders.
    let mailboxes: Vec<ChunkMailbox<Vec<KvBlockHandle>>> =
        (0..WORKERS).map(|_| ChunkMailbox::new()).collect();

    // The owner allocates each request's chunk up front (in a real engine,
    // per prefill/decode step) and hands it to a worker.
    let (done_sender, done_receiver) = channel::<()>();
    let mut workers = Vec::new();
    for mailbox in &mailboxes {
        let sender = mailbox.sender();
        let (chunk_sender, chunk_receiver) = channel::<Vec<KvBlockHandle>>();
        let done = Sender::clone(&done_sender);
        workers.push((
            chunk_sender,
            thread::spawn(move || {
                // Worker: run the request, then free its whole chunk with one
                // push. No per-block frees, no shared queue.
                while let Ok(chunk) = chunk_receiver.recv() {
                    sender.push(chunk);
                    done.send(()).expect("owner alive");
                }
            }),
        ));
    }

    let total_requests = WORKERS * REQUESTS_PER_WORKER;
    let mut completed = 0;
    for round in 0..REQUESTS_PER_WORKER {
        // Owner: allocate one request chunk per worker for this step.
        for (chunk_sender, _) in &workers {
            let chunk: Vec<KvBlockHandle> = (0..BLOCKS_PER_REQUEST)
                .map(|_| pool.allocate().expect("pool sized to the workload"))
                .collect();
            chunk_sender.send(chunk).expect("worker alive");
        }

        // Owner: drain every mailbox once per step and return blocks.
        for _ in 0..WORKERS {
            done_receiver.recv().expect("worker completion");
            completed += 1;
        }
        let mut freed = 0;
        for mailbox in &mailboxes {
            mailbox.take_all(|chunk| {
                for handle in chunk {
                    pool.free(handle).expect("drained handle frees");
                    freed += 1;
                }
            });
        }
        println!(
            "step={round} freed_blocks={freed} allocated={}",
            pool.stats().allocated
        );
    }

    for (chunk_sender, worker) in workers {
        drop(chunk_sender);
        worker.join().expect("worker join");
    }
    assert_eq!(completed, total_requests);
    let stats = pool.stats();
    assert_eq!(stats.allocation_count, stats.free_count);
    println!(
        "requests={completed} allocations={} frees={} high_water_mark={}",
        stats.allocation_count, stats.free_count, stats.high_water_mark
    );
}
