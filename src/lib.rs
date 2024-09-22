pub(crate) mod message;
pub use message::*;

/// Functions to parse various parts of an HL7 message. Probably not useful to you
/// (use the `Message::parse` method instead).
pub mod parser;

// TODO list:
//
// - Add lenient parsing for segment separators (e.g. allow \n or \r\n as well as \r)
// - Add cursor location
// - Add query functions to get fields, components, etc. by name
// - Add ability to convert parsed messages into a mutable form that can be modified and then
// serialized back into a hl7 message
// - Add serde support
// - More tests
// - More documentation
// - More examples
