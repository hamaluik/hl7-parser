use std::{fmt::Display, ops::Range};
use super::{Field, Separators};

/// A segment in an HL7 message. A segment is a collection of fields, separated by the field
/// separator character. Each segment has a name, which is the first field in the segment.
/// Segments are separated by the segment separator character.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment<'m> {
    pub(crate) source: &'m str,
    /// The name of the segment
    pub name: &'m str,
    /// The fields of the segment
    pub fields: Vec<Field<'m>>,
    /// The range of the segment in the original message
    pub range: Range<usize>,
}

impl<'m> Segment<'m> {
    #[inline]
    /// An iterator over the fields of the segment
    pub fn fields(&self) -> impl Iterator<Item = &Field<'m>> {
        self.fields.iter()
    }

    #[inline]
    /// Display the segment value, using the separators to decode escape sequences
    /// by default. Note: if you want to display the raw value without decoding escape
    /// sequences, use the `#` flag, e.g. `format!("{:#}", segment.display(separators))`.
    /// Repeats will be separated by the repeat separator character.
    /// Fields will be separated by the field separator character.
    /// Components will be separated by the component separator character.
    /// Subcomponents will be separated by the subcomponent separator character.
    /// Segments will be separated by the segment separator character.
    /// Escape sequences will be decoded using the escape character.
    pub fn display(&'m self, separators: &'m Separators) -> SegmentDisplay<'m> {
        SegmentDisplay {
            name: self.name,
            fields: &self.fields,
            separators,
        }
    }

    #[inline]
    /// Get the raw value of the segment. This is the value as it appears in the message,
    /// without any decoding of escape sequences, and including all fields and
    /// their separators.
    ///
    /// # Examples
    /// 
    /// ```
    /// let segment = hl7_parser::parser::parse_segment("ZFO|foo|bar").unwrap();
    /// assert_eq!(segment.name, "ZFO");
    /// assert_eq!(segment.fields.len(), 2);
    /// assert_eq!(segment.raw_value(), "ZFO|foo|bar");
    /// ```
    pub fn raw_value(&self) -> &'m str {
        self.source
    }

    #[inline]
    /// Get a specific field of the segment by number. Fields are numbered starting at 1.
    /// Returns `None` if the field number is out of range.
    ///
    /// # Examples
    /// 
    /// ```
    /// let segment = hl7_parser::parser::parse_segment("ZFO|foo|bar").unwrap();
    /// assert_eq!(segment.field(1).unwrap().raw_value(), "foo");
    /// assert_eq!(segment.field(2).unwrap().raw_value(), "bar");
    /// assert_eq!(segment.field(3), None);
    /// ```
    pub fn field(&self, number: usize) -> Option<&Field<'m>> {
        debug_assert!(number > 0);
        self.fields.get(number - 1)
    }
}

/// A display implementation for segments.
/// This will decode the escape sequences in the segment value
/// using the separators. If the `#` flag is used, the raw value
/// will be displayed without decoding the escape sequences.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SegmentDisplay<'m> {
    name: &'m str,
    fields: &'m Vec<Field<'m>>,
    separators: &'m Separators,
}

impl Display for SegmentDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)?;
        for field in self.fields {
            write!(f, "{}", self.separators.field)?;
            if f.alternate() {
                write!(f, "{:#}", field.display(self.separators))?;
            } else {
                write!(f, "{}", field.display(self.separators))?;
            }
        }
        Ok(())
    }
}

