static ADT_SRC: &str = include_str!("../test_assets/sample_adt_a08.hl7");

pub fn main() {
    let message = hl7_parser::parse_message(ADT_SRC).expect("can parse ADT");

    let timestamp_raw = message
        .query("MSH.7")
        .expect("can query message")
        .raw_value();
    let timestamp =
        hl7_parser::datetime::parse_timestamp(timestamp_raw, false).expect("can parse timestamp");

    #[cfg(feature = "chrono")]
    {
        use chrono::{DateTime, Utc};
        let timestamp: DateTime<Utc> = timestamp.try_into().expect("can convert to chrono");
        println!("Parsed timestamp: {timestamp}");
    }

    #[cfg(feature = "time")]
    {
        use time::PrimitiveDateTime;
        let timestamp: PrimitiveDateTime = timestamp.try_into().expect("can convert to time");
        println!("Parsed timestamp: {timestamp}");
    }

    #[cfg(not(any(feature = "chrono", feature = "time")))]
    {
        println!("Parsed timestamp: {timestamp}");
    }
}
