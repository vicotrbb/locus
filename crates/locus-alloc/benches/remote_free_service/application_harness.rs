#![allow(missing_docs)]

use locus_alloc::{
    RemoteFreeQueuedByteDrainConfig, RemoteFreeServiceRetuneCandidate,
    RemoteFreeServiceRetuneGuardDecision, RemoteFreeServiceRetunePolicyApplication,
    RemoteFreeServiceRetunePolicyApplicator,
};

use crate::remote_free_service_harness::{
    ServiceTelemetryCase, BATCH_LIMIT, BYTES_PER_BLOCK, QUEUE_CAPACITY, TARGET_PENDING_BLOCKS,
};

const QUEUE_CAPACITY_GROWTH_FACTOR: usize = 2;

pub(crate) fn candidate_case(candidate: RemoteFreeServiceRetuneCandidate) -> ServiceTelemetryCase {
    let applicator = RemoteFreeServiceRetunePolicyApplicator::try_new(QUEUE_CAPACITY_GROWTH_FACTOR)
        .expect("policy applicator");
    let current_config = current_config_for_candidate(candidate);
    let application = applicator
        .plan(
            current_config,
            RemoteFreeServiceRetuneGuardDecision::Apply { candidate },
        )
        .expect("policy application plan");

    let RemoteFreeServiceRetunePolicyApplication::Apply { candidate, config } = application else {
        panic!("apply decision produced no policy application");
    };

    service_case_for_applied_config(candidate, config)
}

fn current_config_for_candidate(
    candidate: RemoteFreeServiceRetuneCandidate,
) -> RemoteFreeQueuedByteDrainConfig {
    match candidate {
        RemoteFreeServiceRetuneCandidate::DrainEarlier
        | RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacity => service_config(QUEUE_CAPACITY),
        RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier => {
            service_config(QUEUE_CAPACITY / QUEUE_CAPACITY_GROWTH_FACTOR)
        }
        _ => panic!("candidate cannot be applied by guarded benchmark: {candidate:?}"),
    }
}

fn service_config(queue_capacity: usize) -> RemoteFreeQueuedByteDrainConfig {
    RemoteFreeQueuedByteDrainConfig::from_item_shape(
        queue_capacity,
        BATCH_LIMIT,
        TARGET_PENDING_BLOCKS,
        BYTES_PER_BLOCK,
    )
    .expect("service config")
}

fn service_case_for_applied_config(
    candidate: RemoteFreeServiceRetuneCandidate,
    config: RemoteFreeQueuedByteDrainConfig,
) -> ServiceTelemetryCase {
    assert_service_config(config, expected_capacity_for_candidate(candidate));

    match candidate {
        RemoteFreeServiceRetuneCandidate::DrainEarlier => {
            ServiceTelemetryCase::planner_candidate_drain_earlier()
        }
        RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier => {
            ServiceTelemetryCase::planner_candidate_capacity_and_drain_earlier()
        }
        RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacity => {
            panic!("capacity-only candidate has no guarded service benchmark case")
        }
        _ => panic!("candidate cannot be applied by guarded benchmark: {candidate:?}"),
    }
}

fn expected_capacity_for_candidate(candidate: RemoteFreeServiceRetuneCandidate) -> usize {
    match candidate {
        RemoteFreeServiceRetuneCandidate::DrainEarlier
        | RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacityAndDrainEarlier => QUEUE_CAPACITY,
        RemoteFreeServiceRetuneCandidate::IncreaseQueueCapacity => {
            QUEUE_CAPACITY * QUEUE_CAPACITY_GROWTH_FACTOR
        }
        _ => panic!("candidate cannot be applied by guarded benchmark: {candidate:?}"),
    }
}

fn assert_service_config(config: RemoteFreeQueuedByteDrainConfig, queue_capacity: usize) {
    assert_eq!(config.queue_capacity(), queue_capacity);
    assert_eq!(config.drain_batch_limit(), BATCH_LIMIT);
    assert_eq!(config.target_pending_items(), TARGET_PENDING_BLOCKS);
    assert_eq!(config.queued_byte_budget().bytes(), 262_144);
}
