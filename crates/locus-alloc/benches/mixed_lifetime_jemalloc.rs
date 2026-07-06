#![allow(missing_docs)]

use criterion::{criterion_group, criterion_main, Criterion};
use tikv_jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[path = "mixed_lifetime_malloc/trace.rs"]
mod trace;

fn bench(c: &mut Criterion) {
    trace::bench_mixed_lifetime_malloc(c, "jemalloc");
}

criterion_group!(benches, bench);
criterion_main!(benches);
