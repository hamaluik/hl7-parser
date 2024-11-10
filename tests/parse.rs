static ADT_SRC: &str = include_str!("../test_assets/sample_adt_a08.hl7");
static ADT_SRC_ENCODED: &str = include_str!("../test_assets/sample_adt_a01_encoded.hl7");

#[test]
pub fn can_parse() {
    hl7_parser::parse_message(ADT_SRC).expect("can parse ADT");
    hl7_parser::parse_message_with_lenient_newlines(ADT_SRC).expect("can parse ADT with lenient newlines");

    let encoded_message = hl7_parser::parse_message_with_lenient_newlines(ADT_SRC_ENCODED).expect("can parse ADT");
    let msh_5 = encoded_message.query("MSH.5").expect("can query message").display(&encoded_message.separators).to_string();
    assert_eq!(msh_5, "Isaac^2");
}

#[test]
#[cfg(not(any(feature = "chrono", feature = "time")))]
pub fn can_parse_timestamp() {
    let message = hl7_parser::parse_message(ADT_SRC).expect("can parse ADT");

    let timestamp_raw = message
        .query("MSH.7")
        .expect("can query message")
        .raw_value();
    let timestamp =
        hl7_parser::datetime::parse_timestamp(timestamp_raw, false).expect("can parse timestamp");

    println!("parsed timestamp: {timestamp}");
}

#[test]
#[cfg(feature = "chrono")]
pub fn can_parse_chrono() {
    use chrono::{DateTime, Utc};

    let message = hl7_parser::parse_message(ADT_SRC).expect("can parse ADT");

    let timestamp_raw = message
        .query("MSH.7")
        .expect("can query message")
        .raw_value();
    let timestamp =
        hl7_parser::datetime::parse_timestamp(timestamp_raw, false).expect("can parse timestamp");

    let timestamp: DateTime<Utc> = timestamp.try_into().expect("can convert to chrono");
    println!("Parsed timestamp: {timestamp}");
}

#[test]
#[cfg(feature = "time")]
pub fn can_parse_time_crate() {
    use time::PrimitiveDateTime;

    let message = hl7_parser::parse_message(ADT_SRC).expect("can parse ADT");

    let timestamp_raw = message
        .query("MSH.7")
        .expect("can query message")
        .raw_value();
    let timestamp =
        hl7_parser::datetime::parse_timestamp(timestamp_raw, false).expect("can parse timestamp");

    let timestamp: PrimitiveDateTime = timestamp.try_into().expect("can convert to time");
    println!("Parsed timestamp: {timestamp}");
}
