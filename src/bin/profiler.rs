use std::time::Instant;

static ADT_SRC: &'static str = include_str!("../../test_assets/sample_adt_a08.hl7");
// static ORU_SRC: &'static str = include_str!("../../test_assets/sample_oru_r01_lab.hl7");

pub fn main() {
    let start = Instant::now();
    let n = 50_000;
    for i in 0..n {
        let _ = hl7_parser::parse_message(ADT_SRC).expect("can parse ADT");
        // let _ = hl7_parser::parse_message(ORU_SRC).expect("can parse ORU");
        if i % 10_000 == 0 {
            let progress = i as f64 / n as f64 * 100.0;
            println!("Progress: {:.1}%", progress);
        }
    }
    let duration = start.elapsed();
    println!(
        "Parsing {n} messages took: {duration:#?} ({:.1} msg/s [{:.1} μs/msg])",
        n as f64 / duration.as_secs_f64(),
        1_000_000.0 * duration.as_secs_f64() / n as f64
    );
}
