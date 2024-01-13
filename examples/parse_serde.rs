#[cfg(feature = "serde")]
fn main() {
    use hl7_parser::ParsedMessage;
    let message = include_str!("../test_assets/sample_adt_a01.hl7")
        .replace("\r\n", "\r")
        .replace('\n', "\r");

    let parsed_message = ParsedMessage::parse(&message, false).expect("can parse message");
    let serialized_message =
        serde_json::to_string_pretty(&parsed_message).expect("can serialize message");
    println!("{}", serialized_message);
}

#[cfg(not(feature = "serde"))]
fn main() {
    println!("This example requires the serde feature to be enabled");
}
