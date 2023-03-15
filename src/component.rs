use crate::SubComponent;
use std::{num::NonZeroUsize, ops::Range};

/// Represents an HL7v2 sub-component
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    /// The range (in char indices) in the original message where the component is located
    pub range: Range<usize>,
    /// The sub-components found within the component
    pub sub_components: Vec<SubComponent>,
}

impl Component {
    /// Access a sub-component via the 1-based HL7 sub-component index
    ///
    /// # Returns
    ///
    /// A reference to the sub-component
    #[inline]
    pub fn sub_component(&self, sub_component: NonZeroUsize) -> Option<&SubComponent> {
        self.sub_components.get(sub_component.get() - 1)
    }

    /// Given the source for the original message, extract the (raw) string for this component
    ///
    /// # Arguments
    ///
    /// * `s` - A string slice representing the original message source that was parsed
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
    /// let component = field.component(NonZeroUsize::new(4).unwrap()).expect("can get component 4");
    ///
    /// assert_eq!(component.source(message.source), "MIE&1.2.840.114398.1.100&ISO");
    /// ```
    #[inline]
    pub fn source<'s>(&self, s: &'s str) -> &'s str {
        &s[self.range.clone()]
    }

    /// Locate a sub-component at the cursor position
    ///
    /// # Arguments
    ///
    /// * `cursor` - The cursor location (0-based character index of the original message)
    ///
    /// # Returns
    ///
    /// A tuple containing the HL7 sub-component index (1-based) and a reference to the sub-component.
    /// If the component doesn't contain the cursor, returns `None`
    pub fn sub_component_at_cursor(&self, cursor: usize) -> Option<(NonZeroUsize, &SubComponent)> {
        if !self.range.contains(&cursor) {
            return None;
        }
        self.sub_components
            .iter()
            .enumerate()
            .find(|(_, sub_component)| sub_component.range.contains(&cursor))
            .map(|(i, sc)| (NonZeroUsize::new(i + 1).unwrap(), sc))
    }
}

/// A trait for accessing sub-components on fields, to extend Option<&Component> with short-circuit access
pub trait SubComponentAccessor {
    /// Access the sub-component given by 1-based indexing
    fn sub_component(&self, sub_component: NonZeroUsize) -> Option<&SubComponent>;
}

impl SubComponentAccessor for Option<&Component> {
    #[inline]
    fn sub_component(&self, sub_component: NonZeroUsize) -> Option<&SubComponent> {
        match self {
            None => None,
            Some(component) => component.sub_component(sub_component),
        }
    }
}