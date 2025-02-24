use hl7_parser::parse_message_with_lenient_newlines;

static ADT_SRC: &str = include_str!("../test_assets/sample_adt_a08.hl7");

#[test]
fn locate_the_cursor() {
    let message = parse_message_with_lenient_newlines(ADT_SRC).expect("Can parse message");

    let cursor = message.locate_cursor(1331).expect("Can locate cursor");
    assert_eq!(format!("{cursor}"), "IN2[2].4");
    assert_eq!(cursor.field.unwrap().1.raw_value(), "Cartoon Ducks Inc");

    let cursor = message.locate_cursor(117).expect("Can locate cursor");
    assert_eq!(format!("{cursor}"), "PID.5.1");
    assert_eq!(cursor.component.unwrap().1.raw_value(), "DUCK");
}
