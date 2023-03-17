use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hl7_parser::*;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse adt^a01", |b| {
        let message = include_str!("../test_assets/sample_adt_a01.hl7")
            .replace("\r\n", "\r")
            .replace('\n', "\r");
        b.iter(|| {
            ParsedMessage::parse(black_box(message.as_str())).expect("can parse message");
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
