fn main() {
    afl::fuzz!(|data: &[u8]| {
        if let Ok(s) = std::str::from_utf8(data) {
            hl7_parser::Message::parse(s).ok();
        }
    });
}

