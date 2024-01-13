#[cfg(any(feature = "time", feature = "chrono"))]
use criterion::{black_box, criterion_group, criterion_main, Criterion};
#[cfg(any(feature = "time", feature = "chrono"))]
use hl7_parser::*;

#[cfg(feature = "time")]
fn criterion_benchmark_time(c: &mut Criterion) {
    c.bench_function("parse timestamp", |b| {
        b.iter(|| parse_timestamp_time(black_box("20230312195905.1234-0700")).expect("can parse timestamp"))
    });
}

#[cfg(feature = "chrono")]
fn criterion_benchmark_chrono(c: &mut Criterion) {
    c.bench_function("parse timestamp", |b| {
        b.iter(|| parse_timestamp_chrono(black_box("20230312195905.1234-0700")).expect("can parse timestamp"))
    });
}

#[cfg(all(feature = "time", not(feature = "chrono")))]
criterion_group!(benches, criterion_benchmark_time);
#[cfg(all(feature = "chrono", not(feature = "time")))]
criterion_group!(benches, criterion_benchmark_chrono);
#[cfg(all(feature = "chrono", feature = "time"))]
criterion_group!(benches, criterion_benchmark_chrono, criterion_benchmark_time);
#[cfg(all(feature = "time", feature = "chrono"))]
criterion_main!(benches);

#[cfg(not(any(feature = "time", feature = "chrono")))]
fn main() {
    println!("This benchmark requires either the `time` or `chrono` feature to be enabled.");
}
