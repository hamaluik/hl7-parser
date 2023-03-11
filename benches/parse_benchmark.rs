use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hl7_parser::*;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse adt^a01", |b| {
        let message = include_str!("../test_assets/sample_adt_a01.hl7")
            .replace("\r\n", "\r")
            .replace('\n', "\r");
        b.iter(|| {
            Message::parse(black_box(message.as_str())).expect("can parse message");
        })
    });

    let separators = Separators::default();
    c.bench_function("decode encoded sequences", |b| {
        b.iter(|| separators.decode(black_box(r#"\F\\R\\S\\T\\E\"#)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
