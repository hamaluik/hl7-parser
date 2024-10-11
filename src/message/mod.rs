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

use crate::locate::LocatedCursor;

use crate::{
    parser::ParseError,
    query::{LocationQuery, LocationQueryResult},
};

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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    pub fn parse(input: &'m str) -> Result<Self, ParseError> {
        crate::parser::message::message(false)(input.into())
            .map(|(_, m)| m)
            .map_err(|e| e.into())
    }

    /// Parse a message from a string, allowing lenient newlines.
    /// This will return an error if the message is not a valid HL7 message.
    /// If `lenient_newlines` is true, this will allow `\n` and `\r\n` to be treated
    /// the same as `\r` as the separator for segments.
    /// This is useful for parsing messages that come as standard text files
    /// where each segment is separated by platform-specific newlines.
    ///
    /// # Examples
    ///
    /// ```
    /// let message = hl7_parser::Message::parse_with_lenient_newlines("MSH|^~\\&|EPIC|EPICADT|SMS|SMSADT|199912271408|CHARRIS|ADT^A04|1817457|D|2.5|\nEVN|A04|199912271408|||CHARRIS\nPID||0493575^^^2^ID 1|454721||DOE^JOHN^^^^|DOE^JOHN^^^^|19480203|M||B|254 MYSTREET AVE^^MYTOWN^OH^44123^USA||(216)123-4567|||M|NON|400003403~1129086|\nNK1||ROE^MARIE^^^^|SPO||(216)123-4567||EC|||||||||||||||||||||||||||\nPV1||O|168 ~219~C~PMA^^^^^^^^^||||277^ALLEN MYLASTNAME^BONNIE^^^^|||||||||| ||2688684|||||||||||||||||||||||||199912271408||||||002376853", true).unwrap();
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
    pub fn parse_with_lenient_newlines(
        input: &'m str,
        lenient_newlines: bool,
    ) -> Result<Self, ParseError> {
        crate::parser::message::message(lenient_newlines)(input.into())
            .map(|(_, m)| m)
            .map_err(|e| e.into())
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
    /// Segments are 1-indexed.
    ///
    /// # Examples
    ///
    /// ```
    /// let message =
    /// hl7_parser::Message::parse("MSH|^~\\&|\rABC|foo\rXYZ|bar\rABC|baz").unwrap();
    /// let abc1 = message.segment_n("ABC", 1).unwrap();
    /// assert_eq!(abc1.field(1).unwrap().raw_value(), "foo");
    /// let abc2 = message.segment_n("ABC", 2).unwrap();
    /// assert_eq!(abc2.field(1).unwrap().raw_value(), "baz");
    /// let abc3 = message.segment_n("ABC", 3);
    /// assert_eq!(abc3, None);
    /// ```
    pub fn segment_n(&self, name: &str, n: usize) -> Option<&Segment<'m>> {
        debug_assert!(n > 0, "Segments are 1-indexed");
        self.segments.iter().filter(|s| s.name == name).nth(n - 1)
    }

    /// Count the number of segments with the given name.
    pub fn segment_count(&self, name: &str) -> usize {
        self.segments.iter().filter(|s| s.name == name).count()
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

    /// Locate the cursor within the message. Equivalent to calling
    /// `hl7_parser::locate::locate_cursor` with the message and the cursor position.
    pub fn locate_cursor(&self, cursor: usize) -> Option<LocatedCursor> {
        crate::locate::locate_cursor(self, cursor)
    }

    /// Query the message for a specific location. This is a more flexible way to
    /// access the fields, components, and subcomponents of the message.
    ///
    /// # Examples
    /// ```
    /// let message =
    /// hl7_parser::Message::parse("MSH|^~\\&|foo|bar|baz|quux|20010504094523||ADT^A01|1234|P|2.3|||").unwrap();
    /// let field = message.query("MSH.3").unwrap().raw_value();
    /// assert_eq!(field, "foo");
    /// let component = message.query("MSH.7.1").unwrap().raw_value();
    /// assert_eq!(component, "20010504094523");
    /// ```
    pub fn query<Q>(&'m self, query: Q) -> Option<LocationQueryResult<'m>>
    where
        Q: TryInto<LocationQuery>,
    {
        let query = query.try_into().ok()?;
        let segment_index = query.segment_index.unwrap_or(1);

        if let Some(field) = query.field {
            let repeat = query.repeat.unwrap_or(1);
            if let Some(component) = query.component {
                if let Some(subcomponent) = query.subcomponent {
                    self.segment_n(&query.segment, segment_index)
                        .and_then(|s| s.field(field))
                        .and_then(|f| f.repeat(repeat))
                        .and_then(|r| r.component(component))
                        .and_then(|c| c.subcomponent(subcomponent))
                        .map(LocationQueryResult::Subcomponent)
                } else {
                    self.segment_n(&query.segment, segment_index)
                        .and_then(|s| s.field(field))
                        .and_then(|f| f.repeat(repeat))
                        .and_then(|r| r.component(component))
                        .map(LocationQueryResult::Component)
                }
            } else if query.repeat.is_some() {
                self.segment_n(&query.segment, segment_index)
                    .and_then(|s| s.field(field))
                    .and_then(|f| f.repeat(repeat))
                    .map(LocationQueryResult::Repeat)
            } else {
                self.segment_n(&query.segment, segment_index)
                    .and_then(|s| s.field(field))
                    .map(LocationQueryResult::Field)
            }
        } else {
            self.segment_n(&query.segment, segment_index)
                .map(LocationQueryResult::Segment)
        }
    }
}
