use std::ops::Range;

/// Represents an HL7v2 sub-component
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubComponent {
    /// The range (in char indices) in the original message where the sub-component is located
    pub range: Range<usize>,
}

impl SubComponent {
    /// Given the source for the original message, extract the (raw) string for this sub-component
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
    /// let repeat = field.repeat(NonZeroUsize::new(1).unwrap()).expect("can get repeat 1");
    /// let component = repeat.component(NonZeroUsize::new(4).unwrap()).expect("can get component 4");
    /// let sub_component = component.sub_component(NonZeroUsize::new(3).unwrap()).expect("can get sub-component 3");
    ///
    /// assert_eq!(sub_component.source(message.source), "ISO");
    /// ```
    #[inline]
    pub fn source<'s>(&self, message_source: &'s str) -> &'s str {
        &message_source[self.range.clone()]
    }
}
