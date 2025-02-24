# hl7-parser &emsp; [![Build Status]][actions] [![Latest Version]][crates.io] [![Docs]][docslink] [![License]][licenseblob]

[Build Status]: https://img.shields.io/github/actions/workflow/status/hamaluik/hl7-parser/ci.yml
[actions]: https://github.com/hamaluik/hl7-parser/actions?query=branch%3Amain
[Latest Version]: https://img.shields.io/crates/v/hl7-parser.svg
[crates.io]: https://crates.io/crates/hl7-parser
[Docs]: https://img.shields.io/docsrs/hl7-parser
[docslink]: https://docs.rs/hl7-parser/latest/hl7_parser/
[License]: https://img.shields.io/github/license/hamaluik/hl7-parser
[licenseblob]: https://github.com/hamaluik/hl7-parser/blob/main/LICENSE

Parses the structure of [HL7v2] messages, but does not validate the correctness of the messages.

> [!WARNING]  
> Although a best effort has been made to make this parse HL7v2 messages correctly,
> there are no guarantees that it is actually correct. Use at your own risk.

[HL7v2]: https://github.com/hamaluik/hl7-parser/blob/main/LICENSE

## Features

- [x] Parse HL7v2 messages into a structure that can be queried
- [x] Parse HL7v2 timestamps into [chrono], [time], and [jiff] types
- [x] Decode HL7v2 encoded strings
- [x] Locate a cursor within a message based on a character index
- [x] Optional lenient parsing of segment separators (allow `\r\n`, `\n`, and `\r` to count as segment separators instead of just `\r`)
- [ ] Non-ASCII/UTF-8 encodings

(Unchecked features are not yet implemented, but planned for future releases).

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
hl7-parser = "0.3"
```

and then you can parse HL7v2 messages:

```rust
use hl7_parser::{Message, datetime::TimeStamp};
use std::str::FromStr;

let message =
Message::parse("MSH|^~\\&|foo|bar|baz|quux|20010504094523||ADT^A01|1234|P|2.3|||").unwrap();
let msh = message.segment("MSH").unwrap();
assert_eq!(msh.field(3).unwrap().raw_value(), "foo");

let message_time = msh.field(7).unwrap();
let time: TimeStamp = message_time.raw_value().parse().unwrap();
assert_eq!(time.year, 2001);
assert_eq!(time.month, Some(5));
assert_eq!(time.day, Some(4));
```

### Optional Cargo Features

By default, no optional features are enabled.

- `serde`: enable [serde] support for all data structures
- `time`: enable [time] support for parsing timestamps
- `chrono`: enable [chrono] support for parsing timestamps
- `jiff`: enable [jiff] support for parsing timestamps

[serde]: https://crates.io/crates/serde
[time]: https://crates.io/crates/time
[chrono]: https://crates.io/crates/chrono
[jiff]: https://crates.io/crates/jiff

## Additional Examples

### Querying a Message

```rust
let message =
hl7_parser::Message::parse("MSH|^~\\&|foo|bar|baz|quux|20010504094523||ADT^A01|1234|P|2.3|||").unwrap();
let field = message.query("MSH.3").unwrap().raw_value();
assert_eq!(field, "foo");
let component = message.query("MSH.7.1").unwrap().raw_value();
assert_eq!(component, "20010504094523");
```

### Locating the Cursor Within A Message

(The cursor being the character index of some point within the buffer)

```rust
let message = Message::parse("MSH|^~\\&|asdf\rPID|1|0").unwrap();
let cursor = locate_cursor(&message, 19).expect("cursor is located");
assert_eq!(cursor.segment.unwrap().0, "PID");
assert_eq!(cursor.segment.unwrap().1, 1);
assert_eq!(cursor.field.unwrap().0, 1);
assert_eq!(cursor.field.unwrap().1.raw_value(), "1");
```

### Decoding Encoded Strings

```rust
use hl7_parser::message::Separators;
let separators = Separators::default(); // or, from a parsed message
let input = "foo|bar^baz&quux~quuz\\corge\rquack\nduck";
let expected = r"foo\F\bar\S\baz\T\quux\R\quuz\E\corge\X0D\quack\X0A\duck";
let actual = separators.encode(input).to_string();
assert_eq!(expected, actual);
```

### Parsing Timestamps

```rust
use hl7_parser::datetime::{parse_timestamp, TimeStamp, TimeStampOffset};

let ts: TimeStamp = parse_timestamp("20230312195905.1234-0700", false).expect("can parse timestamp");

assert_eq!(ts.year, 2023);
assert_eq!(ts.month, Some(3));
assert_eq!(ts.day, Some(12));
assert_eq!(ts.hour, Some(19));
assert_eq!(ts.minute, Some(59));
assert_eq!(ts.second, Some(5));
assert_eq!(ts.microsecond, Some(123_400));
assert_eq!(ts.offset, Some(TimeStampOffset {
    hours: -7,
    minutes: 0,
}));
```

These `TimeStamp` values can then be converted to and from `chrono`, `time`,
and `jiff` types if the corresponding features are enabled using [std::convert::From]
and [std::convert::TryFrom].

### Building HL7 messages

```rust
use hl7_parser::builder::prelude::*;

let message = MessageBuilder::new(Separators::default())
    .with_segment(SegmentBuilder::new("MSH")
        .with_field_value(3, "SendingApp")
        .with_field_value(4, "SendingFac")
        .with_field_value(5, "ReceivingApp")
        .with_field_value(6, "ReceivingFac")
        .with_field(9,
            FieldBuilder::default()
                .with_component(1, "ADT")
                .with_component(2, "A01"))
        .with_field_value(10, "123456")
        .with_field_value(11, "P")
        .with_field_value(12, "2.3"))
    .with_segment(SegmentBuilder::new("PID")
        .with_field_value(3, "123456")
        .with_field(5,
            FieldBuilder::default()
                .with_component(1, "Doe")
                .with_component(2, "John"))
        .with_field_value(7, "19700101"))
    .render_with_newlines().to_string();

assert_eq!(message,
"MSH|^~\\&|SendingApp|SendingFac|ReceivingApp|ReceivingFac|||ADT^A01|123456|P|2.3\nPID|||123456||Doe^John||19700101");
```

