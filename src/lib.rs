//! HL7v2 message parsing in Rust.
//!
//! Parses the structure of HL7v2 messages, but does not validate the correctness
//! of the messages.
//!
//! # Examples
//!
//! ```
//! use hl7_parser::{Message, datetime::TimeStamp};
//! use std::str::FromStr;
//!
//! let message =
//! Message::parse("MSH|^~\\&|foo|bar|baz|quux|20010504094523||ADT^A01|1234|P|2.3|||").unwrap();
//! let msh = message.segment("MSH").unwrap();
//! assert_eq!(msh.field(3).unwrap().raw_value(), "foo");
//!
//! let message_time = msh.field(7).unwrap();
//! let time: TimeStamp = message_time.raw_value().parse().unwrap();
//! assert_eq!(time.year, 2001);
//! assert_eq!(time.month, Some(5));
//! assert_eq!(time.day, Some(4));
//! ```

/// Structs for representing HL7 messages.
pub mod message;
pub use message::Message;

pub mod builder;

/// Structs for displaying parsed HL7 message values. Especially useful for decoding
/// escaped values.
pub mod display;

/// Utilities for locating a cursor within an HL7 message.
pub mod locate;

/// Human-readable location queries for HL7 messages.
///
/// i.e. parsing "PID.5.1" to get the value of the first component of the fifth field
pub mod query;

/// Functions to parse various parts of an HL7 message. Probably not useful to you
/// (use the `Message::parse` method instead).
pub mod parser;

/// Timestamp parsing and utilities to translate to and from the `chrono` and
/// `time` crates.
pub mod datetime;

/// Parses an HL7 message into a structured form. Equivalent to calling `Message::parse(message)`.
pub fn parse_message(message: &str) -> Result<Message, parser::ParseError> {
    Message::parse(message)
}

/// Parses an HL7 message into a structured form, allowing lenient newlines. Equivalent to calling
/// `Message::parse_with_lenient_newlines(message, true)`.
pub fn parse_message_with_lenient_newlines(message: &str) -> Result<Message, parser::ParseError> {
    Message::parse_with_lenient_newlines(message, true)
}

// TODO list:
//
// - [x] Timestamp parsing
// - [x] Chrono support
// - [x] Time support
// - [x] Add lenient parsing for segment separators (e.g. allow \n or \r\n as well as \r)
// - [x] Add cursor location
// - [x] Add query functions to get fields, components, etc. by name
// - [ ] Add ability to convert parsed messages into a mutable form that can be modified and then serialized back into a hl7 message
// - [X] Add serde support
// - [x] this_error errors
// - [x] More tests
// - [x] More documentation
// - [x] More examples
// - [x] benchmarks
