use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hl7_parser::*;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse adt^a01 owned", |b| {
        let message = include_str!("../test_assets/sample_adt_a01.hl7")
            .replace("\r\n", "\r")
            .replace('\n', "\r");
        b.iter(|| {
            ParsedMessageOwned::parse(black_box(message.clone()), false).expect("can parse message");
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
