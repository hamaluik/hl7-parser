use hl7_parser::parse_message_with_lenient_newlines;

static ADT_SRC: &str = include_str!("../test_assets/sample_adt_a08.hl7");

#[test]
fn query_a_message() {
    let message = parse_message_with_lenient_newlines(ADT_SRC).expect("Can parse message");
    let query =
        hl7_parser::query::parse_location_query("PID.5.2").expect("Can parse location query");

    let result = message.query(query).expect("Can query message");
    assert_eq!(result.raw_value(), "DONALD");
}
