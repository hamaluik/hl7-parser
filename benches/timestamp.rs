use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hl7_parser::datetime::*;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse timestamp", |b| {
        b.iter(|| {
            parse_timestamp(black_box("20230312195905.1234-0700"), false)
                .expect("can parse timestamp")
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
