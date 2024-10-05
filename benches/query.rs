use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse query", |b| {
        b.iter(|| hl7_parser::query::parse_location_query(black_box("MSH.1.2.3")).expect("Can parse location query"))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
