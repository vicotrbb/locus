#![allow(missing_docs)]

use std::{num::NonZeroU64, sync::mpsc::sync_channel, thread};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use locus_alloc::{
    KvBlockHandle, KvBlockPool, RemoteFreeDrainObservation, RemoteFreeDrainPolicy,
    RemoteFreeDrainTracker, RemoteFreeQueue,
};
use locus_core::NodeId;

const BLOCK_SIZE: usize = 4096;
const BLOCKS: usize = 256;
const BURSTS: u64 = 8;
const BURST_BLOCKS: usize = 32;
const QUEUE_CAPACITY: usize = 256;
const BATCH_LIMIT: usize = 64;
const BLOCKS_U64: u64 = 256;
const BURST_BLOCKS_U64: u64 = 32;
const TOTAL_BYTES: u64 = BLOCKS_U64 * BLOCK_SIZE as u64;

enum RemoteCompletionCommand {
    Run(Vec<KvBlockHandle>),
    Stop,
}

#[derive(Debug, Clone, Copy)]
struct KvPolicyCase {
    label: &'static str,
    drain_policy: RemoteFreeDrainPolicy,
}

#[derive(Debug, Clone, Copy)]
struct KvPolicyStats {
    submitted_count: u64,
    drained_count: u64,
    full_count: u64,
    policy_drains: u64,
    drain_rounds: u64,
    max_pending_count: u64,
    queued_bytes: u64,
    max_queued_bytes: u64,
    released_bytes: u64,
    max_wait_bursts: u64,
    total_wait_bursts: u64,
}

impl KvPolicyCase {
    fn end_drain() -> Self {
        Self {
            label: "end_drain",
            drain_policy: RemoteFreeDrainPolicy::new(),
        }
    }

    fn max_wait2() -> Self {
        Self {
            label: "max_wait2",
            drain_policy: RemoteFreeDrainPolicy::new()
                .with_max_pending_age(NonZeroU64::new(2).expect("non-zero")),
        }
    }

    fn should_drain(self, observation: RemoteFreeDrainObservation) -> bool {
        self.drain_policy.should_drain(observation)
    }
}

impl KvPolicyStats {
    fn new() -> Self {
        Self {
            submitted_count: 0,
            drained_count: 0,
            full_count: 0,
            policy_drains: 0,
            drain_rounds: 0,
            max_pending_count: 0,
            queued_bytes: 0,
            max_queued_bytes: 0,
            released_bytes: 0,
            max_wait_bursts: 0,
            total_wait_bursts: 0,
        }
    }

    fn mean_wait_milli(self) -> u64 {
        if self.drained_count == 0 {
            return 0;
        }

        self.total_wait_bursts.saturating_mul(1000) / self.drained_count
    }
}

fn kv_remote_free_tracker_end_drain(c: &mut Criterion) {
    let case = KvPolicyCase::end_drain();
    print_kv_policy_sample(case);
    kv_remote_free_tracker_benchmark(
        c,
        "kv_remote_free_tracker_capacity256_batch64_end_drain_256x4k",
        case,
    );
}

fn kv_remote_free_tracker_max_wait2(c: &mut Criterion) {
    let case = KvPolicyCase::max_wait2();
    print_kv_policy_sample(case);
    kv_remote_free_tracker_benchmark(
        c,
        "kv_remote_free_tracker_capacity256_batch64_max_wait2_256x4k",
        case,
    );
}

fn kv_remote_free_tracker_benchmark(c: &mut Criterion, name: &'static str, case: KvPolicyCase) {
    c.bench_function(name, |bench| {
        let mut pool = KvBlockPool::new(NodeId(0), BLOCK_SIZE, BLOCKS).expect("pool");
        let mut queue = RemoteFreeQueue::new(QUEUE_CAPACITY, BATCH_LIMIT).expect("queue");
        let (command_sender, ack_receiver, remote_completion) = spawn_remote_completion(&queue);

        bench.iter(|| {
            let stats = run_kv_policy_iteration(
                &mut pool,
                &mut queue,
                &command_sender,
                &ack_receiver,
                case,
            );
            assert_eq!(stats.submitted_count, BLOCKS_U64);
            assert_eq!(stats.drained_count, BLOCKS_U64);
            assert_eq!(stats.queued_bytes, 0);
            assert_eq!(stats.released_bytes, TOTAL_BYTES);
            assert_eq!(pool.stats().allocated, 0);
            black_box(stats);
        });

        command_sender
            .send(RemoteCompletionCommand::Stop)
            .expect("send stop");
        remote_completion.join().expect("remote completion thread");
    });
}

fn print_kv_policy_sample(case: KvPolicyCase) {
    let mut pool = KvBlockPool::new(NodeId(0), BLOCK_SIZE, BLOCKS).expect("pool");
    let mut queue = RemoteFreeQueue::new(QUEUE_CAPACITY, BATCH_LIMIT).expect("queue");
    let (command_sender, ack_receiver, remote_completion) = spawn_remote_completion(&queue);

    let stats =
        run_kv_policy_iteration(&mut pool, &mut queue, &command_sender, &ack_receiver, case);
    let label = case.label;
    println!(
        "kv_remote_free_policy_sample={label} blocks={BLOCKS_U64} bursts={BURSTS} burst_blocks={BURST_BLOCKS_U64} capacity={QUEUE_CAPACITY} batch_limit={BATCH_LIMIT} submitted_count={} drained_count={} full_count={} policy_drains={} drain_rounds={} max_pending_count={} max_queued_bytes={} released_bytes={} max_wait_bursts={} mean_wait_bursts={}",
        stats.submitted_count,
        stats.drained_count,
        stats.full_count,
        stats.policy_drains,
        stats.drain_rounds,
        stats.max_pending_count,
        stats.max_queued_bytes,
        stats.released_bytes,
        stats.max_wait_bursts,
        format_milli(stats.mean_wait_milli())
    );

    assert_eq!(stats.submitted_count, BLOCKS_U64);
    assert_eq!(stats.drained_count, BLOCKS_U64);
    assert_eq!(stats.queued_bytes, 0);
    assert_eq!(stats.released_bytes, TOTAL_BYTES);
    assert_eq!(pool.stats().allocated, 0);

    command_sender
        .send(RemoteCompletionCommand::Stop)
        .expect("send stop");
    remote_completion.join().expect("remote completion thread");
}

fn spawn_remote_completion(
    queue: &RemoteFreeQueue<KvBlockHandle>,
) -> (
    std::sync::mpsc::SyncSender<RemoteCompletionCommand>,
    std::sync::mpsc::Receiver<usize>,
    thread::JoinHandle<()>,
) {
    let sink = queue.sink();
    let (command_sender, command_receiver) = sync_channel::<RemoteCompletionCommand>(1);
    let (ack_sender, ack_receiver) = sync_channel::<usize>(1);

    let remote_completion = thread::spawn(move || {
        while let Ok(command) = command_receiver.recv() {
            match command {
                RemoteCompletionCommand::Run(handles) => {
                    let submitted = handles.len();
                    for handle in handles {
                        sink.enqueue(handle).expect("enqueue handle");
                    }
                    ack_sender.send(submitted).expect("ack submitted");
                }
                RemoteCompletionCommand::Stop => break,
            }
        }
    });

    (command_sender, ack_receiver, remote_completion)
}

fn run_kv_policy_iteration(
    pool: &mut KvBlockPool,
    queue: &mut RemoteFreeQueue<KvBlockHandle>,
    command_sender: &std::sync::mpsc::SyncSender<RemoteCompletionCommand>,
    ack_receiver: &std::sync::mpsc::Receiver<usize>,
    case: KvPolicyCase,
) -> KvPolicyStats {
    let mut tracker = RemoteFreeDrainTracker::new();
    let mut stats = KvPolicyStats::new();

    for burst in 0..BURSTS {
        let mut handles = Vec::with_capacity(BURST_BLOCKS);
        for _ in 0..BURST_BLOCKS {
            let handle = pool.allocate().expect("block");
            black_box(pool.block_mut(handle).expect("block").as_mut_ptr());
            handles.push(handle);
        }

        command_sender
            .send(RemoteCompletionCommand::Run(handles))
            .expect("send handles");
        let submitted = ack_receiver.recv().expect("ack submitted");
        assert_eq!(submitted, BURST_BLOCKS);

        for _ in 0..submitted {
            tracker.record_submit(burst, BLOCK_SIZE as u64);
            stats.submitted_count = stats.submitted_count.saturating_add(1);
        }
        stats.queued_bytes = tracker.queued_bytes();
        stats.max_queued_bytes = stats.max_queued_bytes.max(stats.queued_bytes);
        stats.max_pending_count = stats.max_pending_count.max(tracker.pending_count());

        let completed_bursts = burst.saturating_add(1);
        let observation = kv_policy_observation(queue, &stats, &tracker, completed_bursts);
        if case.should_drain(observation)
            && drain_kv_policy_batch(queue, pool, &mut tracker, completed_bursts, &mut stats) > 0
        {
            stats.policy_drains = stats.policy_drains.saturating_add(1);
        }
    }

    while stats.drained_count < stats.submitted_count {
        if drain_kv_policy_batch(queue, pool, &mut tracker, BURSTS, &mut stats) == 0 {
            thread::yield_now();
        }
    }

    let queue_stats = queue.stats();
    assert_eq!(queue_stats.pending_count, 0);
    assert_eq!(queue_stats.disconnected_count, 0);
    assert!(tracker.is_empty());
    stats.full_count = queue_stats.full_count;
    stats
}

fn kv_policy_observation(
    queue: &RemoteFreeQueue<KvBlockHandle>,
    stats: &KvPolicyStats,
    tracker: &RemoteFreeDrainTracker,
    current_burst: u64,
) -> RemoteFreeDrainObservation {
    let queue_stats = queue.stats();
    let observation = tracker.observation(current_burst);

    assert_eq!(observation.pending_count, queue_stats.pending_count);
    assert_eq!(observation.queued_bytes, stats.queued_bytes);

    observation
}

fn drain_kv_policy_batch(
    queue: &mut RemoteFreeQueue<KvBlockHandle>,
    pool: &mut KvBlockPool,
    tracker: &mut RemoteFreeDrainTracker,
    current_burst: u64,
    stats: &mut KvPolicyStats,
) -> usize {
    let drained = queue.drain_batch(|handle| {
        let drain_record = tracker
            .record_drain(BLOCK_SIZE as u64)
            .expect("tracked drain");
        pool.free(handle).expect("free block");
        let wait_bursts = current_burst.saturating_sub(drain_record.submit_turn);
        stats.queued_bytes = tracker.queued_bytes();
        stats.released_bytes = stats
            .released_bytes
            .saturating_add(drain_record.released_bytes);
        stats.max_wait_bursts = stats.max_wait_bursts.max(wait_bursts);
        stats.total_wait_bursts = stats.total_wait_bursts.saturating_add(wait_bursts);
        stats.drained_count = stats.drained_count.saturating_add(1);
    });

    if drained.drained > 0 {
        stats.drain_rounds = stats.drain_rounds.saturating_add(1);
    }

    drained.drained
}

fn format_milli(value: u64) -> String {
    format!("{}.{:03}", value / 1000, value % 1000)
}

criterion_group!(
    benches,
    kv_remote_free_tracker_end_drain,
    kv_remote_free_tracker_max_wait2
);
criterion_main!(benches);
