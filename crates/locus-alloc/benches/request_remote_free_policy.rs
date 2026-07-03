#![allow(missing_docs)]

use std::{alloc::Layout, num::NonZeroU64, sync::mpsc::sync_channel, thread};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use locus_alloc::{
    RemoteFreeDrainController, RemoteFreeDrainPolicy, RemoteFreeQueue,
    RemoteFreeQueuedByteDrainConfig, RemoteFreeQueuedByteDriftReport,
    RemoteFreeQueuedByteRetuneAction, RemoteFreeQueuedByteRetuneHint, RequestScratchPool,
};
use locus_core::{NodeId, RequestHome, RequestId};

const REQUESTS_U64: u64 = 16;
const BURSTS: u64 = 4;
const BURST_REQUESTS: usize = 4;
const BURST_REQUESTS_U64: u64 = 4;
const ARENA_CAPACITY: usize = 32 * 1024;
const ARENA_CAPACITY_U64: u64 = 32 * 1024;
const REQUEST_ALLOCS: usize = 64;
const ALLOC_SIZE: usize = 256;
const ALLOC_ALIGN: usize = 64;
const QUEUE_CAPACITY: usize = 16;
const BATCH_LIMIT: usize = 8;
const TOTAL_TRACKED_BYTES: u64 = REQUESTS_U64 * ARENA_CAPACITY_U64;
const TARGET_PENDING_REQUESTS: u64 = 8;

enum RemoteCompletionCommand {
    Run(Vec<RequestId>),
    Stop,
}

#[derive(Debug, Clone, Copy)]
struct RequestPolicyCase {
    label: &'static str,
    drain_policy: RemoteFreeDrainPolicy,
    drift_config: RemoteFreeQueuedByteDrainConfig,
    expected: ExpectedRequestPolicyDrift,
}

#[derive(Debug, Clone, Copy)]
struct ExpectedRequestPolicyDrift {
    max_pending_over_target: u64,
    max_queued_bytes_over_budget: u64,
    queue_backpressure_observed: u64,
    retune_hint: RemoteFreeQueuedByteRetuneHint,
    retune_action: RemoteFreeQueuedByteRetuneAction,
}

#[derive(Debug, Clone, Copy)]
struct RequestPolicyStats {
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
    max_pending_over_target: u64,
    max_queued_bytes_over_budget: u64,
    queue_backpressure_observed: u64,
    retune_hint: RemoteFreeQueuedByteRetuneHint,
    retune_action: RemoteFreeQueuedByteRetuneAction,
}

#[derive(Debug, Clone, Copy)]
struct CounterSummary {
    min: u64,
    max: u64,
    sum: u64,
}

impl RequestPolicyCase {
    fn end_drain() -> Self {
        Self {
            label: "end_drain",
            drain_policy: RemoteFreeDrainPolicy::new(),
            drift_config: request_queued_byte_config(),
            expected: ExpectedRequestPolicyDrift {
                max_pending_over_target: 8,
                max_queued_bytes_over_budget: 262_144,
                queue_backpressure_observed: 0,
                retune_hint: RemoteFreeQueuedByteRetuneHint::ReviewMultipleSignals,
                retune_action: RemoteFreeQueuedByteRetuneAction::DrainEarlier,
            },
        }
    }

    fn max_wait2() -> Self {
        Self {
            label: "max_wait2",
            drain_policy: RemoteFreeDrainPolicy::new()
                .with_max_pending_age(NonZeroU64::new(2).expect("non-zero")),
            drift_config: request_queued_byte_config(),
            expected: ExpectedRequestPolicyDrift {
                max_pending_over_target: 0,
                max_queued_bytes_over_budget: 0,
                queue_backpressure_observed: 0,
                retune_hint: RemoteFreeQueuedByteRetuneHint::KeepConfig,
                retune_action: RemoteFreeQueuedByteRetuneAction::KeepConfig,
            },
        }
    }

    fn max_queued256kib() -> Self {
        let config = request_queued_byte_config();
        Self {
            label: "max_queued256kib",
            drain_policy: config.drain_policy(),
            drift_config: config,
            expected: ExpectedRequestPolicyDrift {
                max_pending_over_target: 0,
                max_queued_bytes_over_budget: 0,
                queue_backpressure_observed: 0,
                retune_hint: RemoteFreeQueuedByteRetuneHint::KeepConfig,
                retune_action: RemoteFreeQueuedByteRetuneAction::KeepConfig,
            },
        }
    }
}

fn request_queued_byte_config() -> RemoteFreeQueuedByteDrainConfig {
    RemoteFreeQueuedByteDrainConfig::from_item_shape(
        QUEUE_CAPACITY,
        BATCH_LIMIT,
        TARGET_PENDING_REQUESTS,
        ARENA_CAPACITY_U64,
    )
    .expect("drain config")
}

impl CounterSummary {
    fn new() -> Self {
        Self {
            min: u64::MAX,
            max: 0,
            sum: 0,
        }
    }

    fn observe(&mut self, value: u64) {
        self.min = self.min.min(value);
        self.max = self.max.max(value);
        self.sum = self.sum.saturating_add(value);
    }

    fn mean_milli(self, samples: u64) -> u64 {
        self.sum.saturating_mul(1000) / samples
    }

    fn mean(self, samples: u64) -> u64 {
        self.sum / samples
    }
}

impl RequestPolicyStats {
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
            max_pending_over_target: 0,
            max_queued_bytes_over_budget: 0,
            queue_backpressure_observed: 0,
            retune_hint: RemoteFreeQueuedByteRetuneHint::KeepConfig,
            retune_action: RemoteFreeQueuedByteRetuneAction::KeepConfig,
        }
    }

    fn observe_drift(&mut self, report: RemoteFreeQueuedByteDriftReport) {
        self.max_pending_over_target = self
            .max_pending_over_target
            .max(report.pending_items_over_target());
        self.max_queued_bytes_over_budget = self
            .max_queued_bytes_over_budget
            .max(report.queued_bytes_over_budget());
        if report.has_queue_backpressure() {
            self.queue_backpressure_observed = 1;
        }
        self.retune_hint = report.retune_hint();
        self.retune_action = report.retune_action();
    }

    fn mean_wait_milli(self) -> u64 {
        if self.drained_count == 0 {
            return 0;
        }

        self.total_wait_bursts.saturating_mul(1000) / self.drained_count
    }
}

fn request_remote_free_tracker_end_drain(c: &mut Criterion) {
    let case = RequestPolicyCase::end_drain();
    print_request_policy_sample(case);
    request_remote_free_tracker_benchmark(
        c,
        "request_remote_free_tracker_capacity16_batch8_end_drain_16x64x256b",
        case,
    );
}

fn request_remote_free_tracker_max_wait2(c: &mut Criterion) {
    let case = RequestPolicyCase::max_wait2();
    print_request_policy_sample(case);
    request_remote_free_tracker_benchmark(
        c,
        "request_remote_free_tracker_capacity16_batch8_max_wait2_16x64x256b",
        case,
    );
}

fn request_remote_free_tracker_max_queued256kib(c: &mut Criterion) {
    let case = RequestPolicyCase::max_queued256kib();
    print_request_policy_sample(case);
    request_remote_free_tracker_benchmark(
        c,
        "request_remote_free_tracker_capacity16_batch8_max_queued256kib_16x64x256b",
        case,
    );
}

fn request_remote_free_tracker_benchmark(
    c: &mut Criterion,
    name: &'static str,
    case: RequestPolicyCase,
) {
    c.bench_function(name, |bench| {
        let homes = request_homes();
        let layout = request_layout();
        let mut pool = RequestScratchPool::new();
        let mut queue = RemoteFreeQueue::new(QUEUE_CAPACITY, BATCH_LIMIT).expect("queue");
        let (command_sender, ack_receiver, remote_completion) = spawn_remote_completion(&queue);

        bench.iter(|| {
            let stats = run_request_policy_iteration(
                &homes,
                layout,
                &mut pool,
                &mut queue,
                &command_sender,
                &ack_receiver,
                case,
            );
            assert_eq!(stats.submitted_count, REQUESTS_U64);
            assert_eq!(stats.drained_count, REQUESTS_U64);
            assert_eq!(stats.queued_bytes, 0);
            assert_eq!(stats.released_bytes, TOTAL_TRACKED_BYTES);
            assert_eq!(pool.pool_stats().active_requests, 0);
            assert_request_policy_drift(case, stats);
            black_box(stats);
        });

        command_sender
            .send(RemoteCompletionCommand::Stop)
            .expect("send stop");
        remote_completion.join().expect("remote completion thread");
    });
}

fn print_request_policy_sample(case: RequestPolicyCase) {
    let homes = request_homes();
    let layout = request_layout();
    let mut pool = RequestScratchPool::new();
    let mut queue = RemoteFreeQueue::new(QUEUE_CAPACITY, BATCH_LIMIT).expect("queue");
    let (command_sender, ack_receiver, remote_completion) = spawn_remote_completion(&queue);

    let stats = run_request_policy_iteration(
        &homes,
        layout,
        &mut pool,
        &mut queue,
        &command_sender,
        &ack_receiver,
        case,
    );
    let label = case.label;
    println!(
        "request_remote_free_policy_sample={label} requests={REQUESTS_U64} bursts={BURSTS} burst_requests={BURST_REQUESTS_U64} capacity={QUEUE_CAPACITY} batch_limit={BATCH_LIMIT} submitted_count={} drained_count={} full_count={} policy_drains={} drain_rounds={} max_pending_count={} max_queued_bytes={} released_bytes={} max_wait_bursts={} mean_wait_bursts={} max_pending_over_target={} max_queued_bytes_over_budget={} queue_backpressure_observed={} retune_hint={} retune_action={}",
        stats.submitted_count,
        stats.drained_count,
        stats.full_count,
        stats.policy_drains,
        stats.drain_rounds,
        stats.max_pending_count,
        stats.max_queued_bytes,
        stats.released_bytes,
        stats.max_wait_bursts,
        format_milli(stats.mean_wait_milli()),
        stats.max_pending_over_target,
        stats.max_queued_bytes_over_budget,
        stats.queue_backpressure_observed,
        stats.retune_hint.as_str(),
        stats.retune_action.as_str()
    );

    assert_eq!(stats.submitted_count, REQUESTS_U64);
    assert_eq!(stats.drained_count, REQUESTS_U64);
    assert_eq!(stats.queued_bytes, 0);
    assert_eq!(stats.released_bytes, TOTAL_TRACKED_BYTES);
    assert_eq!(pool.pool_stats().active_requests, 0);
    assert_request_policy_drift(case, stats);

    command_sender
        .send(RemoteCompletionCommand::Stop)
        .expect("send stop");
    remote_completion.join().expect("remote completion thread");

    print_request_policy_sample_summary(case);
}

fn print_request_policy_sample_summary(case: RequestPolicyCase) {
    const SAMPLES: u64 = 8;

    let mut full = CounterSummary::new();
    let mut policy_drains = CounterSummary::new();
    let mut drain_rounds = CounterSummary::new();
    let mut max_pending = CounterSummary::new();
    let mut max_queued_bytes = CounterSummary::new();
    let mut max_wait = CounterSummary::new();
    let mut mean_wait = CounterSummary::new();
    let mut max_pending_over_target = CounterSummary::new();
    let mut max_queued_bytes_over_budget = CounterSummary::new();
    let mut queue_backpressure_observed = CounterSummary::new();

    for _ in 0..SAMPLES {
        let homes = request_homes();
        let layout = request_layout();
        let mut pool = RequestScratchPool::new();
        let mut queue = RemoteFreeQueue::new(QUEUE_CAPACITY, BATCH_LIMIT).expect("queue");
        let (command_sender, ack_receiver, remote_completion) = spawn_remote_completion(&queue);
        let stats = run_request_policy_iteration(
            &homes,
            layout,
            &mut pool,
            &mut queue,
            &command_sender,
            &ack_receiver,
            case,
        );

        full.observe(stats.full_count);
        policy_drains.observe(stats.policy_drains);
        drain_rounds.observe(stats.drain_rounds);
        max_pending.observe(stats.max_pending_count);
        max_queued_bytes.observe(stats.max_queued_bytes);
        max_wait.observe(stats.max_wait_bursts);
        mean_wait.observe(stats.mean_wait_milli());
        max_pending_over_target.observe(stats.max_pending_over_target);
        max_queued_bytes_over_budget.observe(stats.max_queued_bytes_over_budget);
        queue_backpressure_observed.observe(stats.queue_backpressure_observed);

        assert_eq!(stats.submitted_count, REQUESTS_U64);
        assert_eq!(stats.drained_count, REQUESTS_U64);
        assert_eq!(stats.queued_bytes, 0);
        assert_eq!(stats.released_bytes, TOTAL_TRACKED_BYTES);
        assert_eq!(pool.pool_stats().active_requests, 0);
        assert_request_policy_drift(case, stats);

        command_sender
            .send(RemoteCompletionCommand::Stop)
            .expect("send stop");
        remote_completion.join().expect("remote completion thread");
    }

    let label = case.label;
    println!(
        "request_remote_free_policy_sample_summary={label} requests={REQUESTS_U64} bursts={BURSTS} burst_requests={BURST_REQUESTS_U64} capacity={QUEUE_CAPACITY} batch_limit={BATCH_LIMIT} retune_hint={} retune_action={} samples={SAMPLES} full_min={} full_max={} full_mean={} policy_drains_min={} policy_drains_max={} policy_drains_mean={} drain_rounds_min={} drain_rounds_max={} drain_rounds_mean={} max_pending_min={} max_pending_max={} max_pending_mean={} max_queued_bytes_min={} max_queued_bytes_max={} max_queued_bytes_mean={} max_wait_min={} max_wait_max={} max_wait_mean={} mean_wait_min={} mean_wait_max={} mean_wait_mean={} max_pending_over_target_min={} max_pending_over_target_max={} max_pending_over_target_mean={} max_queued_bytes_over_budget_min={} max_queued_bytes_over_budget_max={} max_queued_bytes_over_budget_mean={} queue_backpressure_observed_min={} queue_backpressure_observed_max={} queue_backpressure_observed_mean={}",
        case.expected.retune_hint.as_str(),
        case.expected.retune_action.as_str(),
        full.min,
        full.max,
        format_milli(full.mean_milli(SAMPLES)),
        policy_drains.min,
        policy_drains.max,
        format_milli(policy_drains.mean_milli(SAMPLES)),
        drain_rounds.min,
        drain_rounds.max,
        format_milli(drain_rounds.mean_milli(SAMPLES)),
        max_pending.min,
        max_pending.max,
        format_milli(max_pending.mean_milli(SAMPLES)),
        max_queued_bytes.min,
        max_queued_bytes.max,
        max_queued_bytes.mean(SAMPLES),
        max_wait.min,
        max_wait.max,
        format_milli(max_wait.mean_milli(SAMPLES)),
        format_milli(mean_wait.min),
        format_milli(mean_wait.max),
        format_milli(mean_wait.mean(SAMPLES)),
        max_pending_over_target.min,
        max_pending_over_target.max,
        format_milli(max_pending_over_target.mean_milli(SAMPLES)),
        max_queued_bytes_over_budget.min,
        max_queued_bytes_over_budget.max,
        max_queued_bytes_over_budget.mean(SAMPLES),
        queue_backpressure_observed.min,
        queue_backpressure_observed.max,
        format_milli(queue_backpressure_observed.mean_milli(SAMPLES))
    );
}

fn request_homes() -> Vec<RequestHome> {
    (0..REQUESTS_U64)
        .map(|request| RequestHome {
            request_id: RequestId(request),
            node: Some(alternating_node(request)),
            reason: "bench",
        })
        .collect()
}

fn alternating_node(request: u64) -> NodeId {
    if request % 2 == 0 {
        NodeId(0)
    } else {
        NodeId(1)
    }
}

fn request_layout() -> Layout {
    Layout::from_size_align(ALLOC_SIZE, ALLOC_ALIGN).expect("layout")
}

fn spawn_remote_completion(
    queue: &RemoteFreeQueue<RequestId>,
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
                RemoteCompletionCommand::Run(requests) => {
                    let submitted = requests.len();
                    for request_id in requests {
                        sink.enqueue(request_id).expect("enqueue request");
                    }
                    ack_sender.send(submitted).expect("ack submitted");
                }
                RemoteCompletionCommand::Stop => break,
            }
        }
    });

    (command_sender, ack_receiver, remote_completion)
}

fn run_request_policy_iteration(
    homes: &[RequestHome],
    layout: Layout,
    pool: &mut RequestScratchPool,
    queue: &mut RemoteFreeQueue<RequestId>,
    command_sender: &std::sync::mpsc::SyncSender<RemoteCompletionCommand>,
    ack_receiver: &std::sync::mpsc::Receiver<usize>,
    case: RequestPolicyCase,
) -> RequestPolicyStats {
    let mut controller = RemoteFreeDrainController::new(case.drain_policy);
    let mut stats = RequestPolicyStats::new();

    for burst in 0..BURSTS {
        let burst_start = usize::try_from(burst).expect("burst fits usize") * BURST_REQUESTS;
        let burst_end = burst_start + BURST_REQUESTS;
        let mut requests = Vec::with_capacity(BURST_REQUESTS);

        for home in &homes[burst_start..burst_end] {
            pool.open_request(home, ARENA_CAPACITY)
                .expect("open request");
            for _ in 0..REQUEST_ALLOCS {
                let allocation = pool
                    .alloc_bytes(home.request_id, layout)
                    .expect("allocation");
                black_box(allocation.as_mut_ptr());
            }
            requests.push(home.request_id);
        }

        command_sender
            .send(RemoteCompletionCommand::Run(requests))
            .expect("send requests");
        let submitted = ack_receiver.recv().expect("ack submitted");
        assert_eq!(submitted, BURST_REQUESTS);

        for _ in 0..submitted {
            controller.record_submit(burst, ARENA_CAPACITY_U64);
            stats.submitted_count = stats.submitted_count.saturating_add(1);
        }
        stats.queued_bytes = controller.queued_bytes();
        stats.max_queued_bytes = stats.max_queued_bytes.max(stats.queued_bytes);
        stats.max_pending_count = stats.max_pending_count.max(controller.pending_count());

        let completed_bursts = burst.saturating_add(1);
        let policy_report = controller
            .status_for_queue(queue, completed_bursts)
            .expect("controller status");
        assert_eq!(policy_report.observation.queued_bytes, stats.queued_bytes);
        stats.observe_drift(RemoteFreeQueuedByteDriftReport::from_status(
            case.drift_config,
            policy_report,
        ));
        if policy_report.decision.should_drain()
            && drain_request_policy_batch(
                queue,
                pool,
                &mut controller,
                completed_bursts,
                &mut stats,
            ) > 0
        {
            stats.policy_drains = stats.policy_drains.saturating_add(1);
        }
    }

    while stats.drained_count < stats.submitted_count {
        if drain_request_policy_batch(queue, pool, &mut controller, BURSTS, &mut stats) == 0 {
            thread::yield_now();
        }
    }

    let queue_stats = queue.stats();
    assert_eq!(queue_stats.pending_count, 0);
    assert_eq!(queue_stats.disconnected_count, 0);
    assert!(controller.is_empty());
    stats.full_count = queue_stats.full_count;
    stats
}

fn assert_request_policy_drift(case: RequestPolicyCase, stats: RequestPolicyStats) {
    assert_eq!(
        stats.max_pending_over_target,
        case.expected.max_pending_over_target
    );
    assert_eq!(
        stats.max_queued_bytes_over_budget,
        case.expected.max_queued_bytes_over_budget
    );
    assert_eq!(
        stats.queue_backpressure_observed,
        case.expected.queue_backpressure_observed
    );
    assert_eq!(stats.retune_hint, case.expected.retune_hint);
    assert_eq!(stats.retune_action, case.expected.retune_action);
}

fn drain_request_policy_batch(
    queue: &mut RemoteFreeQueue<RequestId>,
    pool: &mut RequestScratchPool,
    controller: &mut RemoteFreeDrainController,
    current_burst: u64,
    stats: &mut RequestPolicyStats,
) -> usize {
    let drained = queue.drain_batch(|request_id| {
        let drain_record = controller
            .record_drain(ARENA_CAPACITY_U64)
            .expect("tracked drain");
        black_box(pool.close_request(request_id).expect("close request"));
        let wait_bursts = current_burst.saturating_sub(drain_record.submit_turn);
        stats.queued_bytes = controller.queued_bytes();
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
    request_remote_free_tracker_end_drain,
    request_remote_free_tracker_max_wait2,
    request_remote_free_tracker_max_queued256kib
);
criterion_main!(benches);
