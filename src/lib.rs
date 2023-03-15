//! HL7v2 message parsing in Rust.
//!
//! Parses the structure of HL7v2 messages, but does not validate the correctness of the messages.
//!
//! Parsing is centered around the [Message] type.
//!
//! # Examples
//!
//! ## Parsing a Message
//!
//! ```
//! use hl7_parser::Message;
//! use std::num::NonZeroUsize;
//!
//! let message = r#"
//! MSH|^~\&|AccMgr|1|||20050110045504||ADT^A01|599102|P|2.3|||
//! PID|1||10006579^^^1^MRN^1||DUCK^DONALD^D||19241010|M||1|111 DUCK ST^^FOWL^CA^999990000^^M|1|8885551212|8885551212|1|2||40007716^^^AccMgr^VN^1|123121234|||||||||||NO NK1|1|DUCK^HUEY|SO|3583 DUCK RD^^FOWL^CA^999990000|8885552222||Y||||||||||||||
//! PV1|1|I|PREOP^101^1^1^^^S|3|||37^DISNEY^WALT^^^^^^AccMgr^^^^CI|||01||||1|||37^DISNEY^WALT^^^^^^AccMgr^^^^CI|2|40007716^^^AccMgr^VN|4|||||||||||||||||||1||G|||20050110045253||||||
//! "#.replace("\r\n", "\r").replace('\n', "\r");
//!
//! let message = Message::parse(message.trim()).expect("can parse message");
//! let message_type = message.get_field_source(("MSH", 0), NonZeroUsize::new(9).unwrap());
//! assert_eq!(message_type.unwrap(), "ADT^A01");
//! ```
//!
//! ## Querying a Message
//!
//! ```
//! use hl7_parser::Message;
//!
//! let message = include_str!("../test_assets/sample_adt_a01.hl7")
//!     .replace("\r\n", "\r")
//!     .replace('\n', "\r");
//! let message = Message::parse(&message).expect("can parse message");
//!
//! let trigger_event = message.query("MSH.9.2").expect("can parse location query");
//! assert_eq!(trigger_event, Some("A01"));
//! ```
//!
//! ## Locating the Cursor Within A Message
//!
//! (The cursor being the character index of some point within the buffer)
//!
//! ```
//! use hl7_parser::Message;
//! use std::num::NonZeroUsize;
//!
//! let message = r#"
//! MSH|^~\&|AccMgr|1|||20050110045504||ADT^A01|599102|P|2.3|||
//! PID|1||10006579^^^1^MRN^1||DUCK^DONALD^D||19241010|M||1|111 DUCK ST^^FOWL^CA^999990000^^M|1|8885551212|8885551212|1|2||40007716^^^AccMgr^VN^1|123121234|||||||||||NO NK1|1|DUCK^HUEY|SO|3583 DUCK RD^^FOWL^CA^999990000|8885552222||Y||||||||||||||
//! PV1|1|I|PREOP^101^1^1^^^S|3|||37^DISNEY^WALT^^^^^^AccMgr^^^^CI|||01||||1|||37^DISNEY^WALT^^^^^^AccMgr^^^^CI|2|40007716^^^AccMgr^VN|4|||||||||||||||||||1||G|||20050110045253||||||
//! "#.replace("\r\n", "\r").replace('\n', "\r");
//!
//! let message = Message::parse(message.trim()).expect("can parse message");
//! let location = message.locate_cursor(25);
//! assert_eq!(location.segment.unwrap().0, "MSH");
//! assert_eq!(location.field.unwrap().0.get(), 7);
//! assert_eq!(location.field.unwrap().1.source(message.source), "20050110045504");
//! ```
//!
//! ## Parsing Message Timestamps
//!
//! ```
//! use hl7_parser::parse_time;
//!
//! let ts = "20230312195905-0700";
//! let ts = parse_time(ts).expect("can parse timestamp");
//!
//! assert_eq!(ts.year(), 2023);
//! assert_eq!(ts.month(), time::Month::March);
//! assert_eq!(ts.day(), 12);
//! assert_eq!(ts.hour(), 19);
//! assert_eq!(ts.minute(), 59);
//! assert_eq!(ts.second(), 05);
//! assert_eq!(ts.microsecond(), 0);
//! assert_eq!(ts.offset().whole_hours(), -7);
//! assert_eq!(ts.offset().minutes_past_hour(), 0);
//! ```
//!
//! ## Decoding Encoded Strings
//!
//! ```
//! use hl7_parser::Separators;
//!
//! let separators = Separators::default();
//! assert_eq!(
//!     separators.decode(r#"Pierre DuRho\S\ne \T\ Cie"#).as_str(),
//!     r#"Pierre DuRho^ne & Cie"#
//! );
//! ```

mod component;
mod error;
mod field;
mod header;
mod message;
mod parser;
mod query;
mod segment;
mod sub_component;
mod time_parser;

pub use component::*;
pub use error::*;
pub use field::*;
pub use header::*;
pub use message::*;
pub use query::*;
pub use segment::*;
pub use sub_component::*;
pub use time_parser::*;
