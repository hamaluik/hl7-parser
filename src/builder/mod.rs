//! # HL7 Message Builder
//!
//! This module provides a builder for constructing HL7 messages. The builder can be used to
//! construct messages from scratch, or to modify existing messages.
//!
//! ## Examples
//!
//! ```
//! use hl7_parser::builder::prelude::*;
//!
//! let message = MessageBuilder::new(Separators::default())
//!     .with_segment(SegmentBuilder::new("MSH")
//!         .with_field_value(3, "SendingApp")
//!         .with_field_value(4, "SendingFac")
//!         .with_field_value(5, "ReceivingApp")
//!         .with_field_value(6, "ReceivingFac")
//!         .with_field(9,
//!             FieldBuilder::default()
//!                 .with_component(1, "ADT")
//!                 .with_component(2, "A01"))
//!         .with_field_value(10, "123456")
//!         .with_field_value(11, "P")
//!         .with_field_value(12, "2.3"))
//!     .with_segment(SegmentBuilder::new("PID")
//!         .with_field_value(3, "123456")
//!         .with_field(5,
//!             FieldBuilder::default()
//!                 .with_component(1, "Doe")
//!                 .with_component(2, "John"))
//!         .with_field_value(7, "19700101"))
//!     .render_with_newlines().to_string();
//!
//! assert_eq!(message,
//! "MSH|^~\\&|SendingApp|SendingFac|ReceivingApp|ReceivingFac|||ADT^A01|123456|P|2.3\nPID|||123456||Doe^John||19700101");
//! ```

mod segment;
use std::fmt::Display;

use display::MessageBuilderDisplay;
pub use segment::*;

mod field;
pub use field::*;

mod repeat;
pub use repeat::*;

mod component;
pub use component::*;

use crate::{message::Separators, Message};

/// Prelude for building HL7 messages.
pub mod prelude {
    pub use super::*;
    pub use crate::message::Separators;
}

/// A builder for constructing HL7 messages.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MessageBuilder {
    separators: Separators,
    segments: Vec<SegmentBuilder>,
}

impl Default for MessageBuilder {
    fn default() -> Self {
        MessageBuilder {
            separators: Separators::default(),
            segments: Vec::new(),
        }
    }
}

impl MessageBuilder {
    /// Create a new message builder with the given separators. No segments are added.
    pub fn new(separators: Separators) -> Self {
        MessageBuilder {
            separators,
            segments: Vec::new(),
        }
    }

    /// Append a segment to the message. Segments will be output in the order they are added.
    pub fn push_segment(&mut self, segment: SegmentBuilder) {
        self.segments.push(segment);
    }

    /// Get the separators used by the message.
    pub fn separators(&self) -> &Separators {
        &self.separators
    }

    /// Get the segments in the message.
    pub fn segments(&self) -> &[SegmentBuilder] {
        &self.segments
    }

    /// Get a mutable reference to the segments in the message.
    pub fn segments_mut(&mut self) -> &mut Vec<SegmentBuilder> {
        &mut self.segments
    }

    /// Get a segment by index (0-based).
    pub fn segment(&self, index: usize) -> Option<&SegmentBuilder> {
        self.segments.get(index)
    }

    /// Get a mutable reference to a segment by index (0-based).
    pub fn segment_mut(&mut self, index: usize) -> Option<&mut SegmentBuilder> {
        self.segments.get_mut(index)
    }

    /// Remove a segment by index (0-based).
    pub fn remove_segment(&mut self, index: usize) -> Option<SegmentBuilder> {
        if index >= self.segments.len() {
            return None;
        }
        Some(self.segments.remove(index))
    }

    /// Get the first segment with the given name.
    pub fn segment_named<S: AsRef<str>>(&self, name: S) -> Option<&SegmentBuilder> {
        self.segments
            .iter()
            .find(|segment| segment.name() == name.as_ref())
    }

    /// Get a mutable reference to the first segment with the given name.
    pub fn segment_named_mut<S: AsRef<str>>(&mut self, name: S) -> Option<&mut SegmentBuilder> {
        self.segments
            .iter_mut()
            .find(|segment| segment.name() == name.as_ref())
    }

    /// Remove the first segment with the given name.
    pub fn remove_segment_named<S: AsRef<str>>(&mut self, name: S) -> Option<SegmentBuilder> {
        let index = self
            .segments
            .iter()
            .position(|segment| segment.name() == name.as_ref())?;
        Some(self.segments.remove(index))
    }

    /// Get the nth segment with the given name (1-based).
    pub fn segment_n<S: AsRef<str>>(&self, name: S, n: usize) -> Option<&SegmentBuilder> {
        debug_assert!(n > 0, "Segment numbers are 1-based");
        let name = name.as_ref();
        self.segments
            .iter()
            .filter(|s| s.name.as_str() == name)
            .nth(n - 1)
    }

    /// Get a mutable reference to the nth segment with the given name (1-based).
    pub fn segment_n_mut<S: AsRef<str>>(
        &mut self,
        name: S,
        n: usize,
    ) -> Option<&mut SegmentBuilder> {
        debug_assert!(n > 0, "Segment numbers are 1-based");
        let name = name.as_ref();
        self.segments
            .iter_mut()
            .filter(|s| s.name.as_str() == name)
            .nth(n - 1)
    }

    /// Remove the nth segment with the given name (1-based).
    pub fn remove_segment_n<S: AsRef<str>>(&mut self, name: S, n: usize) -> Option<SegmentBuilder> {
        debug_assert!(n > 0, "Segment numbers are 1-based");
        let name = name.as_ref();
        let index = self
            .segments
            .iter()
            .enumerate()
            .find(|(_, s)| s.name.as_str() == name)
            .map(|(i, _)| i)?;
        Some(self.segments.remove(index))
    }

    /// Check if the message is empty (i.e. has no segments).
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Clear all segments from the message.
    pub fn clear(&mut self) {
        self.segments.clear();
    }

    /// Set the separators used by the message.
    pub fn set_separators(&mut self, separators: Separators) {
        self.separators = separators;
    }

    /// Add a segment to the message.
    pub fn with_segment(mut self, segment: SegmentBuilder) -> Self {
        self.push_segment(segment);
        self
    }

    /// Display the message with the default line endings for the current platform.
    ///
    /// On Windows, this will use `\r\n` as the line ending. On other platforms, it will use `\n`.
    pub fn render_with_newlines<'a>(&'a self) -> MessageBuilderDisplay<'a> {
        MessageBuilderDisplay {
            message: self,
            #[cfg(windows)]
            line_endings: "\r\n",
            #[cfg(not(windows))]
            line_endings: "\n",
        }
    }

    /// Display the message with the given line endings.
    pub fn render_with_segment_separators<'a>(
        &'a self,
        line_endings: &'a str,
    ) -> MessageBuilderDisplay<'a> {
        MessageBuilderDisplay {
            message: self,
            line_endings,
        }
    }
}

/// Render the message using the proper `\r` segment separators.
impl Display for MessageBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for segment in &self.segments {
            if first {
                first = false;
            } else {
                write!(f, "\r")?;
            }
            write!(f, "{}", segment.display(&self.separators))?;
        }
        Ok(())
    }
}

mod display {
    use super::*;

    /// Display the message with the given (non-standard) line endings.
    pub struct MessageBuilderDisplay<'a> {
        pub(super) message: &'a MessageBuilder,
        pub(super) line_endings: &'a str,
    }

    impl<'a> Display for MessageBuilderDisplay<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut first = true;
            for segment in &self.message.segments {
                if first {
                    first = false;
                } else {
                    write!(f, "{}", self.line_endings)?;
                }
                write!(f, "{}", segment.display(&self.message.separators))?;
            }
            Ok(())
        }
    }
}

/// Convert a message into a message builder.
impl<'m> From<&'m Message<'m>> for MessageBuilder {
    fn from(message: &'m Message) -> Self {
        let mut builder = MessageBuilder::new(message.separators);
        builder.segments = message.segments().map(SegmentBuilder::from).collect();
        builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions_sorted::assert_eq;

    #[test]
    fn can_roundtrip_messages() {
        let message_src = include_str!("../../test_assets/sample_adt_a01.hl7");
        let message = crate::parser::parse_message_with_lenient_newlines(message_src, true)
            .expect("Can parse message");

        let builder: MessageBuilder = MessageBuilder::from(&message);
        let display = builder.render_with_newlines().to_string();
        assert_eq!(message_src.trim(), display);
    }
}
