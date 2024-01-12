use crate::Repeat;
use std::{
    num::NonZeroUsize,
    ops::{Index, Range},
};

/// Represents an HL7v2 field
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Field {
    /// The range (in char indices) in the original message where the field is located
    pub range: Range<usize>,
    /// The repeats found within the component. This will always be at least 1 element long, and
    /// the vast majority of the time will only be 1 element long. However, in the case of field
    /// repeats ("arrays", really), all of the repeat data will be stored here
    pub repeats: Vec<Repeat>,
}

impl Field {
    /// Access a repeat via 1-based HL7 repeat index
    ///
    /// # Returns
    ///
    /// A reference to the repeat
    #[inline]
    pub fn repeat(&self, repeat: NonZeroUsize) -> Option<&Repeat> {
        self.repeats.get(repeat.get() - 1)
    }

    /// Mutably access a repeat via 1-based HL7 repeat index
    ///
    /// # Returns
    ///
    /// A mutable reference to the repeat
    #[inline]
    pub fn repeat_mut(&mut self, repeat: NonZeroUsize) -> Option<&mut Repeat> {
        self.repeats.get_mut(repeat.get() - 1)
    }

    /// Given the source for the original message, extract the (raw) string for this field
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
    /// let message = include_str!("../test_assets/sample_oru_r01_generic.hl7");
    /// let message = ParsedMessage::parse(&message, true).expect("can parse message");
    ///
    /// let segment = message.segment("PID").expect("can get PID segment");
    /// let field = segment.field(NonZeroUsize::new(3).unwrap()).expect("can get field 3");
    ///
    /// assert_eq!(field.source(message.source), "12345^^^MIE&1.2.840.114398.1.100&ISO^MR");
    /// ```
    #[inline]
    pub fn source<'s>(&self, message_source: &'s str) -> &'s str {
        &message_source[self.range.clone()]
    }

    /// Locate a repeat at the cursor position
    ///
    /// # Arguments
    ///
    /// * `cursor` - The cursor location (0-based character index of the original message)
    ///
    /// # Returns
    ///
    /// A tuple containing the HL7 repeat index (1-based) and a reference to the repeat.
    /// If the field doesn't contain the cursor, returns `None`
    pub fn repeat_at_cursor(&self, cursor: usize) -> Option<(NonZeroUsize, &Repeat)> {
        if !self.range.contains(&cursor) {
            return None;
        }
        self.repeats
            .iter()
            .enumerate()
            .find(|(_, repeat)| repeat.range.contains(&cursor) || repeat.range.start == cursor)
            .map(|(i, sc)| (NonZeroUsize::new(i + 1).unwrap(), sc))
    }
}

impl<I: Into<usize>> Index<I> for &Field {
    type Output = Repeat;

    fn index(&self, index: I) -> &Self::Output {
        &self.repeats[index.into()]
    }
}

/// A trait for accessing repeat on fields, to extend Option<&Field> with short-circuit access
pub trait RepeatAccessor {
    /// Access the component given by 1-based indexing
    fn repeat(&self, repeat: NonZeroUsize) -> Option<&Repeat>;
}

impl RepeatAccessor for Option<&Field> {
    #[inline]
    fn repeat(&self, repeat: NonZeroUsize) -> Option<&Repeat> {
        match self {
            None => None,
            Some(field) => field.repeat(repeat),
        }
    }
}

/// A trait for accessing repeat on fields, to extend Option<&mut Field> with short-circuit access
pub trait RepeatAccessorMut {
    /// Access the component given by 1-based indexing
    fn repeat_mut(&mut self, repeat: NonZeroUsize) -> Option<&mut Repeat>;
}

impl RepeatAccessorMut for Option<&mut Field> {
    #[inline]
    fn repeat_mut(&mut self, repeat: NonZeroUsize) -> Option<&mut Repeat> {
        match self {
            None => None,
            Some(field) => field.repeat_mut(repeat),
        }
    }
}
