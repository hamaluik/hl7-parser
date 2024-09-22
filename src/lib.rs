//! HL7v2 message parsing in Rust.
//!
//! Parses the structure of HL7v2 messages, but does not validate the correctness
//! of the messages.
//!
//! # Examples
//!
//! ```
//! use hl7_parser::Message;
//!
//! let message =
//! Message::parse("MSH|^~\\&|foo|bar|baz|quux|20010101000000||ADT^A01|1234|P|2.3|||").unwrap();
//! let msh = message.segment("MSH").unwrap();
//! assert_eq!(msh.field(3).unwrap().raw_value(), "foo");
//! ```

/// Structs for representing HL7 messages.
pub mod message;
pub use message::Message;

/// Structs for displaying parsed HL7 message values. Especially useful for decoding
/// escaped values.
pub mod display;

/// Functions to parse various parts of an HL7 message. Probably not useful to you
/// (use the `Message::parse` method instead).
pub mod parser;

/// Parses an HL7 message into a structured form. Equivalent to calling `Message::parse`.
pub fn parse_message(message: &str) -> Result<Message, String> {
    Message::parse(message)
}

// TODO list:
//
// - [ ] Timestamp parsing (chrono & time)
// - [ ] Add lenient parsing for segment separators (e.g. allow \n or \r\n as well as \r)
// - [ ] Add cursor location
// - [ ] Add query functions to get fields, components, etc. by name
// - [ ] Add ability to convert parsed messages into a mutable form that can be modified and then serialized back into a hl7 message
// - [X] Add serde support
// - [ ] this_error errors
// - [ ] More tests
// - [ ] More documentation
// - [ ] More examples
