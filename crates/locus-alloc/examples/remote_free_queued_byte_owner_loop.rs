#![allow(missing_docs)]

use std::thread;

use locus_alloc::{
    RemoteFreeDrainController, RemoteFreeQueue, RemoteFreeQueuedByteDrainConfig,
    RemoteFreeTryEnqueueErrorKind,
};

const QUEUE_CAPACITY: usize = 256;
const DRAIN_BATCH_LIMIT: usize = 64;
const REQUEST_CONCURRENCY: u64 = 4;
const REMOTE_FREE_BLOCKS_PER_REQUEST: u64 = 16;
const REPRESENTATIVE_BLOCK_BYTES: u64 = 10 * 1024;
const TRACE_BURSTS: u64 = 8;
const TRACE_BURST_BLOCKS: u64 = 32;
const TRACE_BLOCKS: u64 = TRACE_BURSTS * TRACE_BURST_BLOCKS;
const TRACE_SIZES: [usize; 8] = [4096, 4096, 8192, 4096, 16384, 4096, 32768, 8192];
const TRACE_SIZES_U64: [u64; 8] = [4096, 4096, 8192, 4096, 16384, 4096, 32768, 8192];

#[derive(Debug)]
struct RemoteFreeBlock {
    submit_burst: u64,
    allocation: Vec<u8>,
}

#[derive(Debug, Default)]
struct OwnerLoopStats {
    submitted_count: u64,
    drained_count: u64,
    full_count: u64,
    forced_drains: u64,
    policy_drains: u64,
    drain_rounds: u64,
    max_pending_count: u64,
    max_queued_bytes: u64,
    released_bytes: u64,
    max_wait_bursts: u64,
    total_wait_bursts: u64,
}

impl OwnerLoopStats {
    fn mean_wait_milli(&self) -> u64 {
        if self.drained_count == 0 {
            return 0;
        }

        self.total_wait_bursts.saturating_mul(1000) / self.drained_count
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RemoteFreeQueuedByteDrainConfig::from_grouped_item_shape(
        QUEUE_CAPACITY,
        DRAIN_BATCH_LIMIT,
        REQUEST_CONCURRENCY,
        REMOTE_FREE_BLOCKS_PER_REQUEST,
        REPRESENTATIVE_BLOCK_BYTES,
    )?;
    let queued_byte_budget = config.queued_byte_budget();
    let policy = config.drain_policy();
    let mut controller = RemoteFreeDrainController::new(policy);
    let mut queue = config.queue::<RemoteFreeBlock>()?;
    let sink = queue.sink();
    let mut stats = OwnerLoopStats::default();
    let mut size_index = 0_usize;

    println!("remote_free_queued_byte_owner_loop=started");
    println!("queue_capacity={}", config.queue_capacity());
    println!("drain_batch_limit={}", config.drain_batch_limit());
    println!("request_concurrency={REQUEST_CONCURRENCY}");
    println!("remote_free_blocks_per_request={REMOTE_FREE_BLOCKS_PER_REQUEST}");
    println!("representative_block_bytes={REPRESENTATIVE_BLOCK_BYTES}");
    println!("target_pending_items={}", config.target_pending_items());
    println!("queued_byte_budget={}", queued_byte_budget.bytes());

    for burst in 0..TRACE_BURSTS {
        for _ in 0..TRACE_BURST_BLOCKS {
            let size = TRACE_SIZES[size_index % TRACE_SIZES.len()];
            let queued_bytes = TRACE_SIZES_U64[size_index % TRACE_SIZES_U64.len()];
            size_index = size_index.saturating_add(1);
            let mut block = RemoteFreeBlock {
                submit_burst: burst,
                allocation: vec![0_u8; size],
            };

            loop {
                match sink.try_enqueue(block) {
                    Ok(()) => {
                        controller.record_submit(burst, queued_bytes);
                        stats.submitted_count = stats.submitted_count.saturating_add(1);
                        stats.max_pending_count =
                            stats.max_pending_count.max(controller.pending_count());
                        stats.max_queued_bytes =
                            stats.max_queued_bytes.max(controller.queued_bytes());
                        break;
                    }
                    Err(error) if error.kind() == RemoteFreeTryEnqueueErrorKind::Full => {
                        stats.full_count = stats.full_count.saturating_add(1);
                        block = error.into_item();
                        if drain_owner_batch(&mut queue, burst, &mut controller, &mut stats) == 0 {
                            thread::yield_now();
                        } else {
                            stats.forced_drains = stats.forced_drains.saturating_add(1);
                        }
                    }
                    Err(error) => return Err(Box::new(error)),
                }
            }
        }

        let completed_bursts = burst.saturating_add(1);
        if controller.should_drain_queue(&queue, completed_bursts)? {
            let drained =
                drain_owner_batch(&mut queue, completed_bursts, &mut controller, &mut stats);
            if drained > 0 {
                stats.policy_drains = stats.policy_drains.saturating_add(1);
            }
        }
    }

    while stats.drained_count < stats.submitted_count {
        if drain_owner_batch(&mut queue, TRACE_BURSTS, &mut controller, &mut stats) == 0 {
            thread::yield_now();
        }
    }

    let queue_stats = queue.stats();
    println!(
        "remote_free_queued_byte_owner_loop=complete blocks={TRACE_BLOCKS} submitted_count={} drained_count={} pending_count={} full_count={} forced_drains={} policy_drains={} drain_rounds={} max_pending_count={} max_queued_bytes={} released_bytes={} max_wait_bursts={} mean_wait_bursts={}",
        stats.submitted_count,
        stats.drained_count,
        queue_stats.pending_count,
        stats.full_count,
        stats.forced_drains,
        stats.policy_drains,
        stats.drain_rounds,
        stats.max_pending_count,
        stats.max_queued_bytes,
        stats.released_bytes,
        stats.max_wait_bursts,
        format_milli(stats.mean_wait_milli())
    );

    Ok(())
}

fn drain_owner_batch(
    queue: &mut RemoteFreeQueue<RemoteFreeBlock>,
    current_burst: u64,
    controller: &mut RemoteFreeDrainController,
    stats: &mut OwnerLoopStats,
) -> usize {
    let drained = queue.drain_batch(|mut block| {
        let released_bytes =
            u64::try_from(block.allocation.len()).expect("allocation length fits u64");
        let tracked = controller
            .record_drain(released_bytes)
            .expect("tracked remote-free drain");
        assert_eq!(tracked.submit_turn, block.submit_burst);
        assert_eq!(tracked.released_bytes, released_bytes);

        let wait_bursts = current_burst.saturating_sub(block.submit_burst);
        stats.drained_count = stats.drained_count.saturating_add(1);
        stats.released_bytes = stats.released_bytes.saturating_add(released_bytes);
        stats.max_wait_bursts = stats.max_wait_bursts.max(wait_bursts);
        stats.total_wait_bursts = stats.total_wait_bursts.saturating_add(wait_bursts);
        std::hint::black_box(block.allocation.as_mut_ptr());
    });

    if drained.drained > 0 {
        stats.drain_rounds = stats.drain_rounds.saturating_add(1);
    }

    drained.drained
}

fn format_milli(value: u64) -> String {
    format!("{}.{:03}", value / 1000, value % 1000)
}
