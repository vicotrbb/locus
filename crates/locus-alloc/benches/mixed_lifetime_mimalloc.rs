#![allow(missing_docs)]

use criterion::{criterion_group, criterion_main, Criterion};
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[path = "mixed_lifetime_malloc/trace.rs"]
mod trace;

fn bench(c: &mut Criterion) {
    trace::bench_mixed_lifetime_malloc(c, "mimalloc");
}

criterion_group!(benches, bench);
criterion_main!(benches);
