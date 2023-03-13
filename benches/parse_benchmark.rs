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

    c.bench_function("decode encoded sequences", |b| {
        let separators = Separators::default();
        b.iter(|| separators.decode(black_box(r#"\F\\R\\S\\T\\E\"#)))
    });

    c.bench_function("locate cursor", |b| {
        let message = include_str!("../test_assets/sample_adt_a01.hl7")
            .replace("\r\n", "\r")
            .replace('\n', "\r");
        let message = Message::parse(message.as_str()).expect("can parse message");
        b.iter(|| message.locate_cursor(black_box(0x458)))
    });

    c.bench_function("parse timestamp", |b| {
        b.iter(|| parse_time(black_box("20230312195905.1234-0700")).expect("can parse timestamp"))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
