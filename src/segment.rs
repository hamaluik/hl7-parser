use std::{
    num::NonZeroUsize,
    ops::{Deref, DerefMut, Index, Range},
};

use crate::{Field, Msh};

/// Represents an HL7v2 segment
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Segment {
    /// The range (in char indices) in the original message where the segment is located
    pub range: Range<usize>,
    /// The fields found within the component
    pub fields: Vec<Field>,
}

impl Segment {
    /// Access a field via the 1-based HL7 field index
    ///
    /// # Returns
    ///
    /// A reference to the field
    #[inline]
    pub fn field(&self, field: NonZeroUsize) -> Option<&Field> {
        self.fields.get(field.get() - 1)
    }

    /// Mutably access a field via the 1-based HL7 field indexing
    ///
    /// # Returns
    ///
    /// A mutable reference to the field
    #[inline]
    pub fn field_mut(&mut self, field: NonZeroUsize) -> Option<&mut Field> {
        self.fields.get_mut(field.get() - 1)
    }

    /// Given the source for the original message, extract the (raw) string for this segment
    ///
    /// # Arguments
    ///
    /// * `message_source` - A string slice representing the original message source that was parsed
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::num::NonZeroUsize;
    /// # use hl7_parser::ParsedMessage;
    /// let message = include_str!("../test_assets/sample_oru_r01_generic.hl7")
    ///     .replace("\r\n", "\r")
    ///     .replace('\n', "\r");
    /// let message = ParsedMessage::parse(&message).expect("can parse message");
    ///
    /// let segment = message.segment("PID").expect("can get PID segment");
    ///
    /// assert_eq!(segment.source(message.source), "PID|1|12345|12345^^^MIE&1.2.840.114398.1.100&ISO^MR||MOUSE^MINNIE^S||19240101|F|||123 MOUSEHOLE LN^^FORT WAYNE^IN^46808|||||||||||||||||||");
    /// ```
    #[inline]
    pub fn source<'s>(&self, message_source: &'s str) -> &'s str {
        &message_source[self.range.clone()]
    }

    /// Locate a field at the cursor position
    ///
    /// # Arguments
    ///
    /// * `cursor` - The cursor location (0-based character index of the original message)
    ///
    /// # Returns
    ///
    /// A tuple containing the HL7 field index (1-based) and a reference to the field.
    /// If the segment doesn't contain the cursor, returns `None`
    pub fn field_at_cursor(&self, cursor: usize) -> Option<(NonZeroUsize, &Field)> {
        if !self.range.contains(&cursor) {
            return None;
        }
        self.fields
            .iter()
            .enumerate()
            .find(|(_, field)| field.range.contains(&cursor) || field.range.start == cursor)
            .map(|(i, sc)| (NonZeroUsize::new(i + 1).unwrap(), sc))
    }
}

impl<I: Into<usize>> Index<I> for &Segment {
    type Output = Field;

    fn index(&self, index: I) -> &Self::Output {
        &self.fields[index.into()]
    }
}

/// A trait for accessing fields on segments, to extend Option<&Segment> with short-circuit access
pub trait FieldAccessor {
    /// Access the field given by 1-based indexing
    fn field(&self, field: NonZeroUsize) -> Option<&Field>;
}

impl FieldAccessor for Option<&Segment> {
    #[inline]
    fn field(&self, field: NonZeroUsize) -> Option<&Field> {
        match self {
            None => None,
            Some(seg) => seg.field(field),
        }
    }
}

/// A trait for accessing fields on segments, to extend Option<&mut Segment> with short-circuit access
pub trait FieldAccessorMut {
    /// Access the field given by 1-based indexing
    fn field_mut(&mut self, field: NonZeroUsize) -> Option<&mut Field>;
}

impl FieldAccessorMut for Option<&mut Segment> {
    #[inline]
    fn field_mut(&mut self, field: NonZeroUsize) -> Option<&mut Field> {
        match self {
            None => None,
            Some(seg) => seg.field_mut(field),
        }
    }
}

impl From<Msh> for Segment {
    fn from(msh: Msh) -> Self {
        let Msh {
            range, mut fields, ..
        } = msh;
        fields.insert(
            0,
            Field {
                range: 3..4,
                repeats: Vec::with_capacity(0),
            },
        );
        fields.insert(
            1,
            Field {
                range: 4..8,
                repeats: Vec::with_capacity(0),
            },
        );
        Segment { range, fields }
    }
}

/// Wrapper around segments; HL7 messages can contain multiple segments of the same type
/// (ex: ORU messages often contain multiple OBX segments)
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Segments(pub Vec<Segment>);

impl Deref for Segments {
    type Target = Vec<Segment>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Segments {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Segments {
    /// Locate a field at the cursor position
    ///
    /// # Arguments
    ///
    /// * `cursor` - The cursor location (0-based character index of the original message)
    ///
    /// # Returns
    ///
    /// A tuple containing the segment index number (0-based), HL7 field index (1-based),
    /// and a reference to the field. If the segment(s) don't contain the cursor, returns `None`
    pub fn field_at_cursor(&self, cursor: usize) -> Option<(usize, NonZeroUsize, &Field)> {
        self.iter()
            .enumerate()
            .find_map(|(i, seg)| seg.field_at_cursor(cursor).map(|(n, f)| (i, n, f)))
    }

    /// Locate a segment at the cursor position
    ///
    /// # Arguments
    ///
    /// * `cursor` - The cursor location (0-based character index of the original message)
    ///
    /// # Returns
    ///
    /// A tuple containing the segment index number (0-based) and a reference to the field.
    /// If the segment(s) don't contain the cursor, returns `None`
    pub fn segment_at_cursor(&self, cursor: usize) -> Option<(usize, &Segment)> {
        self.iter()
            .enumerate()
            .find(|(_, seg)| seg.range.contains(&cursor))
            .map(|(i, seg)| (i, seg))
    }
}

impl From<Segment> for Segments {
    fn from(value: Segment) -> Self {
        Segments(vec![value])
    }
}

impl From<Vec<Segment>> for Segments {
    fn from(value: Vec<Segment>) -> Self {
        Segments(value)
    }
}
