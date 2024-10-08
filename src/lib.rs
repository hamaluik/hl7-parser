//! HL7v2 message parsing in Rust.
//!
//! Parses the structure of HL7v2 messages, but does not validate the correctness of the messages.
//!
//! Parsing is centered around the [ParsedMessage] type.
//!
//! # Examples
//!
//! ## Parsing a ParsedMessage
//!
//! ```
//! use hl7_parser::ParsedMessage;
//! use std::num::NonZeroUsize;
//!
//! let message = r#"
//! MSH|^~\&|AccMgr|1|||20050110045504||ADT^A01|599102|P|2.3|||
//! PID|1||10006579^^^1^MRN^1||DUCK^DONALD^D||19241010|M||1|111 DUCK ST^^FOWL^CA^999990000^^M|1|8885551212|8885551212|1|2||40007716^^^AccMgr^VN^1|123121234|||||||||||NO NK1|1|DUCK^HUEY|SO|3583 DUCK RD^^FOWL^CA^999990000|8885552222||Y||||||||||||||
//! PV1|1|I|PREOP^101^1^1^^^S|3|||37^DISNEY^WALT^^^^^^AccMgr^^^^CI|||01||||1|||37^DISNEY^WALT^^^^^^AccMgr^^^^CI|2|40007716^^^AccMgr^VN|4|||||||||||||||||||1||G|||20050110045253||||||
//! "#;
//!
//! let message = ParsedMessage::parse(message.trim(), true).expect("can parse message");
//! let message_type = message.get_field_source(("MSH", 0), NonZeroUsize::new(9).unwrap());
//! assert_eq!(message_type.unwrap(), "ADT^A01");
//! ```
//!
//! ## Querying a ParsedMessage
//!
//! ```
//! use hl7_parser::ParsedMessage;
//!
//! let message = include_str!("../test_assets/sample_adt_a01.hl7");
//! let message = ParsedMessage::parse(&message, true).expect("can parse message");
//!
//! let trigger_event = message.query_value("MSH.9.2").expect("can parse location query");
//! assert_eq!(trigger_event, Some("A01"));
//! ```
//!
//! ## Locating the Cursor Within A ParsedMessage
//!
//! (The cursor being the character index of some point within the buffer)
//!
//! ```
//! use hl7_parser::ParsedMessage;
//! use std::num::NonZeroUsize;
//!
//! let message = r#"
//! MSH|^~\&|AccMgr|1|||20050110045504||ADT^A01|599102|P|2.3|||
//! PID|1||10006579^^^1^MRN^1||DUCK^DONALD^D||19241010|M||1|111 DUCK ST^^FOWL^CA^999990000^^M|1|8885551212|8885551212|1|2||40007716^^^AccMgr^VN^1|123121234|||||||||||NO NK1|1|DUCK^HUEY|SO|3583 DUCK RD^^FOWL^CA^999990000|8885552222||Y||||||||||||||
//! PV1|1|I|PREOP^101^1^1^^^S|3|||37^DISNEY^WALT^^^^^^AccMgr^^^^CI|||01||||1|||37^DISNEY^WALT^^^^^^AccMgr^^^^CI|2|40007716^^^AccMgr^VN|4|||||||||||||||||||1||G|||20050110045253||||||
//! "#;
//!
//! let message = ParsedMessage::parse(message.trim(), true).expect("can parse message");
//! let location = message.locate_cursor(25);
//! assert_eq!(location.segment.unwrap().0, "MSH");
//! assert_eq!(location.field.unwrap().0.get(), 7);
//! assert_eq!(location.field.unwrap().1.source(message.source), "20050110045504");
//! ```
//!
//! ## Parsing ParsedMessage Timestamps
//!
//! Only if the `time` or `chrono` features are enabled.
//!
//! ```
//! #[cfg(any(feature = "time", feature = "chrono"))]
//! {
//! # #[cfg(feature = "chrono")]
//! # use chrono::prelude::*;
//! # #[cfg(feature = "chrono")]
//! # use hl7_parser::parse_timestamp_chrono;
//! # #[cfg(feature = "time")]
//! # use hl7_parser::parse_timestamp_time;
//!
//! #[cfg(feature = "chrono")]
//! let ts = "20230312195905-0700";
//! #[cfg(feature = "chrono")]
//! let ts = parse_timestamp_chrono(ts)
//!     .expect("can parse timestamp")
//!     .earliest()
//!     .expect("can convert to datetime");
//!
//! #[cfg(feature = "time")]
//! let ts = "20230312195905-0700";
//! #[cfg(feature = "time")]
//! let ts = parse_timestamp_time(ts).expect("can parse timestamp");
//!
//! assert_eq!(ts.year(), 2023);
//! #[cfg(all(feature = "chrono", not(feature = "time")))]
//! assert_eq!(ts.month(), 3);
//! #[cfg(feature = "time")]
//! assert_eq!(ts.month(), time::Month::March);
//! assert_eq!(ts.day(), 12);
//! assert_eq!(ts.hour(), 19);
//! assert_eq!(ts.minute(), 59);
//! assert_eq!(ts.second(), 05);
//! }
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

#[cfg(feature = "chrono")]
mod chrono_parser;
mod component;
mod error;
mod field;
mod header;
mod message;
mod parser;
mod query;
mod repeat;
mod segment;
mod sub_component;
#[cfg(feature = "time")]
mod time_parser;

#[cfg(feature = "chrono")]
pub use chrono_parser::parse_timestamp_chrono;
pub use component::*;
pub use error::*;
pub use field::*;
pub use header::*;
pub use message::*;
pub use query::*;
pub use repeat::*;
pub use segment::*;
pub use sub_component::*;
#[cfg(feature = "time")]
pub use time_parser::parse_timestamp_time;
