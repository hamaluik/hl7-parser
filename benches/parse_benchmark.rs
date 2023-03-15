use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse adt^a01 hl7-parser", |b| {
        use hl7_parser::*;
        let message = include_str!("../test_assets/sample_adt_a01.hl7")
            .replace("\r\n", "\r")
            .replace('\n', "\r");
        b.iter(|| {
            Message::parse(black_box(message.as_str())).expect("can parse message");
        })
    });

    c.bench_function("parse adt^a01 rust-hl7", |b| {
        use rusthl7::*;
        let message = include_str!("../test_assets/sample_adt_a01.hl7")
            .replace("\r\n", "\r")
            .replace('\n', "\r");
        b.iter(|| {
            message::Message::try_from(black_box(message.as_str())).expect("can parse message");
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
