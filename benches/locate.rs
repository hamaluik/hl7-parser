use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hl7_parser::*;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("locate cursor", |b| {
        let message = include_str!("../test_assets/sample_adt_a01.hl7")
            .replace("\r\n", "\r")
            .replace('\n', "\r");
        let message = ParsedMessage::parse(message.as_str(), false).expect("can parse message");
        b.iter(|| message.locate_cursor(black_box(0x458)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
