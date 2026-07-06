#![allow(missing_docs)]

use criterion::{criterion_group, criterion_main, Criterion};
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[path = "locus_eval/workloads.rs"]
mod workloads;

#[path = "locus_eval/malloc_runner.rs"]
mod malloc_runner;

fn bench(c: &mut Criterion) {
    malloc_runner::bench_locus_eval_malloc(c, "mimalloc");
}

criterion_group!(benches, bench);
criterion_main!(benches);
