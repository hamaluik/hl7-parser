use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hl7_parser::*;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("decode encoded sequences", |b| {
        let separators = Separators::default();
        b.iter(|| separators.decode(black_box(r#"\F\\R\\S\\T\\E\"#)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
