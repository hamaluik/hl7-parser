use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hl7_parser::*;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse query", |b| {
        b.iter(|| LocationQuery::new(black_box("MSH.1.2.3")).expect("Can parse location query"))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
