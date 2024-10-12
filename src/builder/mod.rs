mod segment;
use std::fmt::Display;

pub use segment::*;

mod field;
pub use field::*;

mod repeat;
pub use repeat::*;

mod component;
pub use component::*;

use crate::{message::Separators, Message};

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
    pub fn new(separators: Separators) -> Self {
        MessageBuilder {
            separators,
            segments: Vec::new(),
        }
    }

    pub fn push_segment(&mut self, segment: SegmentBuilder) {
        self.segments.push(segment);
    }

    pub fn separators(&self) -> &Separators {
        &self.separators
    }

    pub fn segments(&self) -> &[SegmentBuilder] {
        &self.segments
    }

    pub fn segments_mut(&mut self) -> &mut Vec<SegmentBuilder> {
        &mut self.segments
    }

    pub fn segment(&self, index: usize) -> Option<&SegmentBuilder> {
        self.segments.get(index)
    }

    pub fn segment_mut(&mut self, index: usize) -> Option<&mut SegmentBuilder> {
        self.segments.get_mut(index)
    }

    pub fn remove_segment(&mut self, index: usize) -> Option<SegmentBuilder> {
        if index >= self.segments.len() {
            return None;
        }
        Some(self.segments.remove(index))
    }

    pub fn segment_named<S: AsRef<str>>(&self, name: S) -> Option<&SegmentBuilder> {
        self.segments
            .iter()
            .find(|segment| segment.name() == name.as_ref())
    }

    pub fn segment_named_mut<S: AsRef<str>>(&mut self, name: S) -> Option<&mut SegmentBuilder> {
        self.segments
            .iter_mut()
            .find(|segment| segment.name() == name.as_ref())
    }

    pub fn remove_segment_named<S: AsRef<str>>(&mut self, name: S) -> Option<SegmentBuilder> {
        let index = self
            .segments
            .iter()
            .position(|segment| segment.name() == name.as_ref())?;
        Some(self.segments.remove(index))
    }

    pub fn segment_n<S: AsRef<str>>(&self, name: S, n: usize) -> Option<&SegmentBuilder> {
        debug_assert!(n > 0, "Segment numbers are 1-based");
        let name = name.as_ref();
        self.segments
            .iter()
            .filter(|s| s.name.as_str() == name)
            .nth(n - 1)
    }

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

    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    pub fn clear(&mut self) {
        self.segments.clear();
    }

    pub fn set_separators(&mut self, separators: Separators) {
        self.separators = separators;
    }

    pub fn display_with_newlines<'a>(&'a self) -> MessageBuilderDisplay<'a> {
        MessageBuilderDisplay {
            message: self,
            #[cfg(windows)]
            line_endings: "\r\n",
            #[cfg(not(windows))]
            line_endings: "\n",
        }
    }

    pub fn display_with_segment_separators<'a>(
        &'a self,
        line_endings: &'a str,
    ) -> MessageBuilderDisplay<'a> {
        MessageBuilderDisplay {
            message: self,
            line_endings,
        }
    }
}

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

pub struct MessageBuilderDisplay<'a> {
    message: &'a MessageBuilder,
    line_endings: &'a str,
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
        let display = builder.display_with_newlines().to_string();
        assert_eq!(message_src.trim(), display);
    }
}
