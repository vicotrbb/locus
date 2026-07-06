#![allow(missing_docs)]

use criterion::{criterion_group, criterion_main, Criterion};
use tikv_jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[path = "locus_eval/workloads.rs"]
mod workloads;

#[path = "locus_eval/malloc_runner.rs"]
mod malloc_runner;

fn bench(c: &mut Criterion) {
    malloc_runner::bench_locus_eval_malloc(c, "jemalloc");
}

criterion_group!(benches, bench);
criterion_main!(benches);
