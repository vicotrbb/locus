#![allow(missing_docs)]

use criterion::{criterion_group, criterion_main, Criterion};

#[path = "remote_free_service/application_harness.rs"]
mod remote_free_service_application_harness;
#[path = "remote_free_service/dry_run_harness.rs"]
mod remote_free_service_dry_run_harness;
#[path = "remote_free_service/guarded_harness.rs"]
mod remote_free_service_guarded_harness;
#[path = "remote_free_service/guarded_runtime_harness.rs"]
mod remote_free_service_guarded_runtime_harness;
#[path = "remote_free_service/harness.rs"]
mod remote_free_service_harness;
#[path = "remote_free_service/runtime_collected_harness.rs"]
mod remote_free_service_runtime_collected_harness;
#[path = "remote_free_service/runtime_collected_multi_owner_harness.rs"]
mod remote_free_service_runtime_collected_multi_owner_harness;
#[path = "remote_free_service/runtime_collected_rollback_harness.rs"]
mod remote_free_service_runtime_collected_rollback_harness;
#[path = "remote_free_service/runtime_coordinator_harness.rs"]
mod remote_free_service_runtime_coordinator_harness;
#[path = "remote_free_service/runtime_registry_harness.rs"]
mod remote_free_service_runtime_registry_harness;
#[path = "remote_free_service/runtime_service_window_harness.rs"]
mod remote_free_service_runtime_service_window_harness;

fn remote_free_service_telemetry(c: &mut Criterion) {
    remote_free_service_harness::benchmark_service_telemetry(c);
    remote_free_service_application_harness::benchmark_runtime_application(c);
    remote_free_service_dry_run_harness::benchmark_dry_run_sequence(c);
    remote_free_service_guarded_harness::benchmark_guarded_sequences(c);
    remote_free_service_guarded_runtime_harness::benchmark_guarded_runtime_sequence(c);
    remote_free_service_runtime_coordinator_harness::benchmark_runtime_coordinator_sequence(c);
    remote_free_service_runtime_collected_harness::benchmark_runtime_collected_guarded_confirm(c);
    remote_free_service_runtime_collected_multi_owner_harness::benchmark_runtime_collected_multi_owner_mutation_limit(c);
    remote_free_service_runtime_collected_rollback_harness::benchmark_runtime_collected_rollback(c);
    remote_free_service_runtime_registry_harness::benchmark_runtime_registry_sequence(c);
    remote_free_service_runtime_service_window_harness::benchmark_runtime_service_window_sequence(
        c,
    );
    remote_free_service_runtime_service_window_harness::benchmark_runtime_window_collection_sequence(
        c,
    );
}

criterion_group!(benches, remote_free_service_telemetry);
criterion_main!(benches);
