static ADT_SRC: &str = include_str!("../test_assets/sample_adt_a08.hl7");

pub fn main() {
    let message = hl7_parser::parse_message(ADT_SRC).expect("can parse ADT");
    println!("{:#?}", message);

    #[cfg(feature = "serde")]
    {
        let as_json = serde_json::to_string_pretty(&message).expect("can serialize to JSON");
        println!("{}", as_json);
    }
}

