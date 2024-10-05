use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse query", |b| {
        b.iter(|| hl7_parser::query::parse_location_query(black_box("MSH.1.2.3")).expect("Can parse location query"))
    });

    c.bench_function("query message", |b| {
        let message = hl7_parser::parse_message_with_lenient_newlines(include_str!("../test_assets/sample_adt_a08.hl7"))
            .expect("Can parse message");
        let query = hl7_parser::query::parse_location_query("PID.5.2").expect("Can parse location query");
        b.iter(|| message.query(black_box(query.clone())).expect("Can query message"))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
