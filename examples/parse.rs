use hl7_parser::Message;

fn main() {
    let message = include_str!("../test_assets/sample_adt_a01.hl7")
        .replace("\r\n", "\r")
        .replace('\n', "\r");
    for _ in 0..100_000 {
        Message::parse(message.as_str()).expect("can parse message");
    }
}
