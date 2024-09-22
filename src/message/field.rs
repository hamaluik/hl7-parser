use crate::display::FieldDisplay;

use super::Repeat;
use std::ops::Range;

/// A field in an HL7 message. A field is a collection of repeats, separated by the repeat
/// separator character. Fields are separated by the field separator character.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Field<'m> {
    pub(crate) source: &'m str,
    pub repeats: Vec<Repeat<'m>>,
    pub range: Range<usize>,
}

impl<'m> Field<'m> {
    pub(crate) fn new_single(source: &'m str, range: Range<usize>) -> Self {
        Field {
            source,
            repeats: vec![Repeat::new_single(source, range.clone())],
            range,
        }
    }

    #[inline]
    /// An iterator over the repeats of the field
    pub fn repeats(&self) -> impl Iterator<Item = &Repeat<'m>> {
        self.repeats.iter()
    }

    #[inline]
    /// Display the field value, using the separators to decode escape sequences
    /// by default. Note: if you want to display the raw value without decoding escape
    /// sequences, use the `#` flag, e.g. `format!("{:#}", field.display(separators))`.
    /// Repeats will be separated by the repeat separator character.
    /// Fields will be separated by the field separator character.
    /// Components will be separated by the component separator character.
    /// Subcomponents will be separated by the subcomponent separator character.
    pub fn display(&'m self, separators: &'m super::Separators) -> FieldDisplay<'m> {
        FieldDisplay {
            repeats: &self.repeats,
            separators,
        }
    }

    #[inline]
    /// Get the raw value of the field. This is the value as it appears in the message,
    /// without any decoding of escape sequences, and including all repeats and
    /// their separators.
    ///
    /// # Examples
    ///
    /// ```
    /// let field = hl7_parser::parser::parse_field("foo~bar").unwrap();
    /// assert_eq!(field.repeats.len(), 2);
    /// assert_eq!(field.raw_value(), "foo~bar");
    /// ```
    pub fn raw_value(&self) -> &'m str {
        self.source
    }

    #[inline]
    /// Returns true if the field has more than one repeat. Note that
    /// if the field has only one repeat, the value of that repeat
    /// is essentially the value of the field, so the value of the field
    /// can be obtained using `raw_value()`.
    ///
    /// # Examples
    ///
    /// ```
    /// let field = hl7_parser::parser::parse_field("foo~bar").unwrap();
    /// assert_eq!(field.has_repeats(), true);
    /// let field = hl7_parser::parser::parse_field("foo").unwrap();
    /// assert_eq!(field.has_repeats(), false);
    /// let field = hl7_parser::parser::parse_field("foo^bar").unwrap();
    /// assert_eq!(field.has_repeats(), false);
    /// ```
    pub fn has_repeats(&self) -> bool {
        self.repeats.len() > 1
    }

    /// Returns true if the field has no repeats, or if all repeats
    /// have empty values.
    ///
    /// # Examples
    ///
    /// ```
    /// let field = hl7_parser::parser::parse_field("foo~bar").unwrap();
    /// assert_eq!(field.is_empty(), false);
    /// let field = hl7_parser::parser::parse_field("").unwrap();
    /// assert_eq!(field.is_empty(), true);
    /// let field = hl7_parser::parser::parse_field("~").unwrap();
    /// assert_eq!(field.is_empty(), true);
    /// ```
    pub fn is_empty(&self) -> bool {
        self.repeats.is_empty() || self.repeats.iter().all(|r| r.is_empty())
    }

    /// Get the repeat at the specified 1-based index
    /// Returns None if the index is out of bounds
    ///
    /// # Examples
    ///
    /// ```
    /// let field = hl7_parser::parser::parse_field("foo~bar").unwrap();
    /// assert_eq!(field.repeat(1).unwrap().raw_value(), "foo");
    /// assert_eq!(field.repeat(2).unwrap().raw_value(), "bar");
    /// assert_eq!(field.repeat(3), None);
    /// ```
    pub fn repeat(&self, number: usize) -> Option<&Repeat<'m>> {
        self.repeats.get(number - 1)
    }

    /// Get the component at the specified 1-based index
    /// Returns None if the index is out of bounds
    /// If the field has multiple repeats, the component will be taken from the first repeat
    /// only.
    /// If the field has no repeats, this will return None.
    /// If the field has one or more repeats, this is equivalent to calling
    /// `repeat(1).component(number)`.
    ///
    /// This is a convenience method for the common case where a field has only one repeat.
    ///
    /// # Examples
    ///
    /// ```
    /// let field = hl7_parser::parser::parse_field("foo^bar~baz^qux").unwrap();
    /// assert_eq!(field.component(1).unwrap().raw_value(), "foo");
    /// assert_eq!(field.component(2).unwrap().raw_value(), "bar");
    /// assert_eq!(field.component(3), None);
    /// ```
    pub fn component(&self, number: usize) -> Option<&super::Component<'m>> {
        self.repeats.get(0).and_then(|r| r.component(number))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{Component, Separators, Subcomponent};

    #[test]
    fn fields_can_display() {
        let separators = Separators::default();

        let repeat = Repeat {
            source: r"foo\F\bar",
            components: vec![Component {
                source: r"foo\F\bar",
                subcomponents: vec![Subcomponent {
                    value: r"foo\F\bar",
                    range: 0..1, // ignore
                }],
                range: 0..1, // ignore
            }],
            range: 0..1, // ignore
        };

        let field = Field {
            source: r"foo\F\bar~foo\F\bar",
            repeats: vec![repeat.clone(), repeat],
            range: 0..1, // ignore
        };

        assert_eq!(format!("{}", field.display(&separators)), "foo|bar~foo|bar");
        assert_eq!(format!("{:#}", field.display(&separators)), r"foo\F\bar~foo\F\bar");
    }
}
