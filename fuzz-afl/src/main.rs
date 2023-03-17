fn main() {
    afl::fuzz!(|data: &[u8]| {
        if let Ok(s) = std::str::from_utf8(data) {
            hl7_parser::ParsedMessage::parse(s).ok();
        }
    });
}

