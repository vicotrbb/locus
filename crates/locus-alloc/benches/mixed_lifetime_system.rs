#![allow(missing_docs)]

use criterion::{criterion_group, criterion_main, Criterion};

#[path = "mixed_lifetime_malloc/trace.rs"]
mod trace;

fn bench(c: &mut Criterion) {
    trace::bench_mixed_lifetime_malloc(c, "system");
}

criterion_group!(benches, bench);
criterion_main!(benches);
