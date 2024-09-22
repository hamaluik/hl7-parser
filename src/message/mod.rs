mod separators;
pub use separators::*;
mod subcomponent;
pub use subcomponent::*;
mod component;
pub use component::*;
mod repeat;
pub use repeat::*;
mod field;
pub use field::*;
mod segment;
pub use segment::*;

/// A parsed HL7 message. This is the top-level structure that you get when you parse a message.
/// It contains the segments of the message, as well as the separators used in the message.
///
/// # Examples
///
/// ```
/// let message = hl7_parser::Message::parse("MSH|^~\\&|").unwrap();
/// let msh = message.segment("MSH").unwrap();
/// assert_eq!(msh.name, "MSH");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message<'m> {
    pub(crate) source: &'m str,
    pub segments: Vec<Segment<'m>>,
    pub separators: Separators,
}

impl<'m> Message<'m> {
    /// Parse a message from a string.
    /// This will return an error if the message is not a valid HL7 message.
    ///
    /// # Examples
    ///
    /// ```
    /// let message = hl7_parser::Message::parse("MSH|^~\\&|EPIC|EPICADT|SMS|SMSADT|199912271408|CHARRIS|ADT^A04|1817457|D|2.5|\rEVN|A04|199912271408|||CHARRIS\rPID||0493575^^^2^ID 1|454721||DOE^JOHN^^^^|DOE^JOHN^^^^|19480203|M||B|254 MYSTREET AVE^^MYTOWN^OH^44123^USA||(216)123-4567|||M|NON|400003403~1129086|\rNK1||ROE^MARIE^^^^|SPO||(216)123-4567||EC|||||||||||||||||||||||||||\rPV1||O|168 ~219~C~PMA^^^^^^^^^||||277^ALLEN MYLASTNAME^BONNIE^^^^|||||||||| ||2688684|||||||||||||||||||||||||199912271408||||||002376853").unwrap();
    /// let msh = message.segment("MSH").unwrap();
    /// assert_eq!(msh.field(4).unwrap().raw_value(), "EPICADT");
    /// let pid = message.segment("PID").unwrap();
    /// let patient_name = pid.field(5).unwrap();
    /// assert_eq!(patient_name.raw_value(), "DOE^JOHN^^^^");
    /// let first_name = patient_name.component(2).unwrap();
    /// let last_name = patient_name.component(1).unwrap();
    /// assert_eq!(first_name.raw_value(), "JOHN");
    /// assert_eq!(last_name.raw_value(), "DOE");
    /// ```
    pub fn parse(input: &'m str) -> Result<Self, String> {
        crate::parser::message::message()(input.into())
            .map(|(_, m)| m)
            .map_err(move |e| format!("Failed to parse message: {:?}", e))
    }

    /// Find a segment with the given name. If there are more than one segments
    /// with this name, return the first one.
    ///
    /// # Examples
    ///
    /// ```
    /// let message = hl7_parser::Message::parse("MSH|^~\\&|EPIC|EPICADT|SMS|SMSADT|199912271408|CHARRIS|ADT^A04|1817457|D|2.5|\rEVN|A04|199912271408|||CHARRIS\rPID||0493575^^^2^ID 1|454721||DOE^JOHN^^^^|DOE^JOHN^^^^|19480203|M||B|254 MYSTREET AVE^^MYTOWN^OH^44123^USA||(216)123-4567|||M|NON|400003403~1129086|\rNK1||ROE^MARIE^^^^|SPO||(216)123-4567||EC|||||||||||||||||||||||||||\rPV1||O|168 ~219~C~PMA^^^^^^^^^||||277^ALLEN MYLASTNAME^BONNIE^^^^|||||||||| ||2688684|||||||||||||||||||||||||199912271408||||||002376853").unwrap();
    /// let msh = message.segment("MSH").unwrap();
    /// assert_eq!(msh.name, "MSH");
    /// assert_eq!(msh.field(4).unwrap().raw_value(), "EPICADT");
    /// let pid = message.segment("PID").unwrap();
    /// assert_eq!(pid.field(2).unwrap().raw_value(), "0493575^^^2^ID 1");
    /// ```
    pub fn segment(&self, name: &str) -> Option<&Segment<'m>> {
        self.segments.iter().find(|s| s.name == name)
    }

    /// Find the nth segment with the given name. If there are fewer than n segments
    /// with this name, return `None`.
    /// Segments are 0-indexed.
    ///
    /// # Examples
    ///
    /// ```
    /// let message =
    /// hl7_parser::Message::parse("MSH|^~\\&|\rABC|foo\rXYZ|bar\rABC|baz").unwrap();
    /// let abc1 = message.segment_n("ABC", 0).unwrap();
    /// assert_eq!(abc1.field(1).unwrap().raw_value(), "foo");
    /// let abc2 = message.segment_n("ABC", 1).unwrap();
    /// assert_eq!(abc2.field(1).unwrap().raw_value(), "baz");
    /// let abc3 = message.segment_n("ABC", 2);
    /// assert_eq!(abc3, None);
    /// ```
    pub fn segment_n(&self, name: &str, n: usize) -> Option<&Segment<'m>> {
        self.segments.iter().filter(|s| s.name == name).nth(n)
    }

    /// An iterator over the segments of the message
    pub fn segments(&self) -> impl Iterator<Item = &Segment<'m>> {
        self.segments.iter()
    }

    /// Get the raw value of the message. This is the value as it appears in the message,
    /// without any decoding of escape sequences, and including all segments and
    /// their separators.
    /// This is the same as the input string that was used to parse the message.
    pub fn raw_value(&self) -> &'m str {
        self.source
    }
}
