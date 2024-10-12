use hl7_parser::{builder::MessageBuilder, parse_message_with_lenient_newlines};

static ADT_SRC: &str = include_str!("../test_assets/sample_adt_a08.hl7");

pub fn main() {
    let message = parse_message_with_lenient_newlines(ADT_SRC).expect("can parse ADT");
    let message: MessageBuilder = MessageBuilder::from(&message);
    let as_json = serde_json::to_string_pretty(&message).expect("can serialize to JSON");
    println!("{}", as_json);
}

