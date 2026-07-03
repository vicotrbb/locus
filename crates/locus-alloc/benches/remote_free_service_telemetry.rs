#![allow(missing_docs)]

use criterion::{criterion_group, criterion_main, Criterion};

#[path = "remote_free_service/dry_run_harness.rs"]
mod remote_free_service_dry_run_harness;
#[path = "remote_free_service/harness.rs"]
mod remote_free_service_harness;

fn remote_free_service_telemetry(c: &mut Criterion) {
    remote_free_service_harness::benchmark_service_telemetry(c);
    remote_free_service_dry_run_harness::benchmark_dry_run_sequence(c);
}

criterion_group!(benches, remote_free_service_telemetry);
criterion_main!(benches);
