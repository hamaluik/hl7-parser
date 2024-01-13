# hl7-parser &emsp; [![Build Status]][actions] [![Latest Version]][crates.io] [![Docs]][docs] [![License]][license]

[Build Status]: https://img.shields.io/github/actions/workflow/status/hamaluik/hl7-parser/rust.yml
[actions]: https://github.com/hamaluik/hl7-parser/actions?query=branch%3Amain
[Latest Version]: https://img.shields.io/crates/v/hl7-parser.svg
[crates.io]: https://crates.io/crates/hl7-parser
[Docs]: https://img.shields.io/docsrs/hl7-parser
[docs]: https://docs.rs/hl7-parser/latest/hl7_parser/
[License]: https://img.shields.io/github/license/hamaluik/hl7-parser
[license]: https://github.com/hamaluik/hl7-parser/blob/main/LICENSE

Parses the structure of [HL7v2] messages, but does not validate the correctness of the messages.

> [!WARNING]  
> Although a best effort has been made to make this parse HL7v2 messages correctly,
> there are no guarantees that it is actually correct. Use at your own risk.

[HL7v2]: https://github.com/hamaluik/hl7-parser/blob/main/LICENSE

## Features

- [x] Parse HL7v2 messages into a structure that can be queried
- [x] Parse HL7v2 timestamps into [chrono] or [time] types
- [x] Decode HL7v2 encoded strings
- [x] Locate a cursor within a message based on a character index
- [x] Optional lenient parsing of segment separators (allow `\r\n`, `\n`, and `\r` to count as segment separators instead of just `\r`)
- [ ] Non-ASCII/UTF-8 encodings

(Unchecked features are not yet implemented, but planned for future releases).

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
hl7-parser = "0.2"
```

and then you can parse HL7v2 messages:

```rust
use hl7_parser::ParsedMessage;

let message = include_str!("../test_assets/sample_adt_a01.hl7");
let message = ParsedMessage::parse(&message, true).expect("can parse message");

let trigger_event = message.query_value("MSH.9.2").expect("can parse location query");
assert_eq!(trigger_event, Some("A01"));
```

### Optional Cargo Features

By default, no optional features are enabled.

- `serde`: enable [serde] support for all data structures
- `time`: enable [time] support for parsing timestamps
- `chrono`: enable [chrono] support for parsing timestamps

[serde]: https://crates.io/crates/serde
[time]: https://crates.io/crates/time
[chrono]: https://crates.io/crates/chrono

## Examples

### Parsing a ParsedMessage

```rust
use hl7_parser::ParsedMessage;
use std::num::NonZeroUsize;

let message = r#"
MSH|^~\&|AccMgr|1|||20050110045504||ADT^A01|599102|P|2.3|||
PID|1||10006579^^^1^MRN^1||DUCK^DONALD^D||19241010|M||1|111 DUCK ST^^FOWL^CA^999990000^^M|1|8885551212|8885551212|1|2||40007716^^^AccMgr^VN^1|123121234|||||||||||NO NK1|1|DUCK^HUEY|SO|3583 DUCK RD^^FOWL^CA^999990000|8885552222||Y||||||||||||||
PV1|1|I|PREOP^101^1^1^^^S|3|||37^DISNEY^WALT^^^^^^AccMgr^^^^CI|||01||||1|||37^DISNEY^WALT^^^^^^AccMgr^^^^CI|2|40007716^^^AccMgr^VN|4|||||||||||||||||||1||G|||20050110045253||||||
"#;

let message = ParsedMessage::parse(message.trim(), true).expect("can parse message");
let message_type = message.get_field_source(("MSH", 0), NonZeroUsize::new(9).unwrap());
assert_eq!(message_type.unwrap(), "ADT^A01");
```

### Querying a ParsedMessage

```rust
use hl7_parser::ParsedMessage;

let message = include_str!("../test_assets/sample_adt_a01.hl7");
let message = ParsedMessage::parse(&message, true).expect("can parse message");

let trigger_event = message.query_value("MSH.9.2").expect("can parse location query");
assert_eq!(trigger_event, Some("A01"));
```

### Locating the Cursor Within A ParsedMessage

(The cursor being the character index of some point within the buffer)

```rust
use hl7_parser::ParsedMessage;
use std::num::NonZeroUsize;

let message = r#"
MSH|^~\&|AccMgr|1|||20050110045504||ADT^A01|599102|P|2.3|||
PID|1||10006579^^^1^MRN^1||DUCK^DONALD^D||19241010|M||1|111 DUCK ST^^FOWL^CA^999990000^^M|1|8885551212|8885551212|1|2||40007716^^^AccMgr^VN^1|123121234|||||||||||NO NK1|1|DUCK^HUEY|SO|3583 DUCK RD^^FOWL^CA^999990000|8885552222||Y||||||||||||||
PV1|1|I|PREOP^101^1^1^^^S|3|||37^DISNEY^WALT^^^^^^AccMgr^^^^CI|||01||||1|||37^DISNEY^WALT^^^^^^AccMgr^^^^CI|2|40007716^^^AccMgr^VN|4|||||||||||||||||||1||G|||20050110045253||||||
"#;

let message = ParsedMessage::parse(message.trim(), true).expect("can parse message");
let location = message.locate_cursor(25);
assert_eq!(location.segment.unwrap().0, "MSH");
assert_eq!(location.field.unwrap().0.get(), 7);
assert_eq!(location.field.unwrap().1.source(message.source), "20050110045504");
```

### Parsing ParsedMessage Timestamps

#### Using the `time` crate

```rust
#[cfg(feature = "time")]
use hl7_parser::parse_timestamp_time;

let ts = "20230312195905-0700";
let ts = parse_timestamp_time(ts).expect("can parse timestamp");

assert_eq!(ts.year(), 2023);
assert_eq!(ts.month(), time::Month::March);
assert_eq!(ts.day(), 12);
assert_eq!(ts.hour(), 19);
assert_eq!(ts.minute(), 59);
assert_eq!(ts.second(), 05);
assert_eq!(ts.microsecond(), 0);
assert_eq!(ts.offset().whole_hours(), -7);
assert_eq!(ts.offset().minutes_past_hour(), 0);
```

#### Using the `chrono` crate

```rust
#[cfg(feature = "chrono")]
use hl7_parser::parse_timestamp_chrono;
use chrono::prelude::*;

let ts = "20230312195905-0700";
let ts = parse_timestamp_chrono(ts).expect("can parse timestamp");

assert_eq!(ts.year(), 2023);
assert_eq!(ts.month(), 3);
assert_eq!(ts.day(), 12);
assert_eq!(ts.hour(), 19);
assert_eq!(ts.minute(), 59);
assert_eq!(ts.second(), 05);
assert_eq!(ts.nanosecond(), 123_400_000);
assert_eq!(ts.offset().local_minus_utc() / 3600, -7);
assert_eq!(ts.offset().local_minus_utc() % 3600, 0);
```

### Decoding Encoded Strings

```rust
use hl7_parser::Separators;

let separators = Separators::default();
assert_eq!(
    separators.decode(r#"Pierre DuRho\S\ne \T\ Cie"#).as_str(),
    r#"Pierre DuRho^ne & Cie"#
);
```

