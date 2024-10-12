static ADT_SRC: &'static str = include_str!("../test_assets/sample_adt_a08.hl7");

pub fn main() {
    let message = hl7_parser::parse_message(ADT_SRC).expect("can parse ADT");
    println!("{:#?}", message);
}

