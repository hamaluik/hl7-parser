use crate::Component;
use std::{num::NonZeroUsize, ops::Range};

/// Represents an HL7v2 field
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    /// The range (in char indices) in the original message where the field is located
    pub range: Range<usize>,
    /// The components found within the component
    pub components: Vec<Component>,
}

impl Field {
    /// Access a component via the 1-based HL7 component index
    ///
    /// # Returns
    ///
    /// A reference to the component
    #[inline]
    pub fn component(&self, component: NonZeroUsize) -> Option<&Component> {
        self.components.get(component.get() - 1)
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
    /// # use hl7_parser::Message;
    /// let message = include_str!("../test_assets/sample_oru_r01_generic.hl7")
    ///     .replace("\r\n", "\r")
    ///     .replace('\n', "\r");
    /// let message = Message::parse(&message).expect("can parse message");
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

    /// Locate a component at the cursor position
    ///
    /// # Arguments
    ///
    /// * `cursor` - The cursor location (0-based character index of the original message)
    ///
    /// # Returns
    ///
    /// A tuple containing the HL7 component index (1-based) and a reference to the component.
    /// If the field doesn't contain the cursor, returns `None`
    pub fn component_at_cursor(&self, cursor: usize) -> Option<(NonZeroUsize, &Component)> {
        if !self.range.contains(&cursor) {
            return None;
        }
        self.components
            .iter()
            .enumerate()
            .find(|(_, component)| {
                component.range.contains(&cursor) || component.range.start == cursor
            })
            .map(|(i, sc)| (NonZeroUsize::new(i + 1).unwrap(), sc))
    }
}

/// A trait for accessing components on fields, to extend Option<&Field> with short-circuit access
pub trait ComponentAccessor {
    /// Access the component given by 1-based indexing
    fn component(&self, component: NonZeroUsize) -> Option<&Component>;
}

impl ComponentAccessor for Option<&Field> {
    fn component(&self, component: NonZeroUsize) -> Option<&Component> {
        match self {
            None => None,
            Some(field) => field.component(component),
        }
    }
}
