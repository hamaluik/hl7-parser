use std::{fmt::Display, ops::Range};
use super::{Component, Separators};

/// A repeat represents an item in a list of field values. Most fields have a
/// single value, but some fields can have multiple values, called repeats. Each
/// repeat is separated by the repetition separator character and is composed of
/// 0 or more components.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repeat<'m> {
    pub(crate) source: &'m str,
    /// The components of the repeat
    pub components: Vec<Component<'m>>,
    /// The range of the repeat in the original message
    pub range: Range<usize>,
}

impl<'m> Repeat<'m> {
    pub(crate) fn new_single(source: &'m str, range: Range<usize>) -> Self {
        Repeat {
            source,
            components: vec![Component::new_single(source, range.clone())],
            range,
        }
    }

    #[inline]
    /// An iterator over the components of the repeat
    pub fn components(&self) -> impl Iterator<Item = &Component<'m>> {
        self.components.iter()
    }

    #[inline]
    /// Display the repeat value, using the separators to decode escape sequences
    /// by default. Note: if you want to display the raw value without decoding escape
    /// sequences, use the `#` flag, e.g. `format!("{:#}", repeat.display(separators))`.
    /// Repeats will be separated by the repeat separator character.
    pub fn display(&'m self, separators: &'m Separators) -> RepeatDisplay<'m> {
        RepeatDisplay {
            components: &self.components,
            separators,
        }
    }

    #[inline]
    /// Get the raw value of the repeat. This is the value as it appears in the message,
    /// without any decoding of escape sequences, and including all components and
    /// their separators.
    ///
    /// # Examples
    ///
    /// ```
    /// let repeat = hl7_parser::parser::parse_repeat("foo^bar").unwrap();
    /// assert_eq!(repeat.components.len(), 2);
    /// assert_eq!(repeat.raw_value(), "foo^bar");
    /// ```
    pub fn raw_value(&self) -> &'m str {
        self.source
    }

    #[inline]
    /// Returns true if the repeat has more than one component. Note that
    /// if the repeat has only one component, the value of that components
    /// is essentially the value of the repeat, so the value of the repeat
    /// can be obtained using `raw_value()`.
    ///
    /// # Examples
    ///
    /// ```
    /// let repeat = hl7_parser::parser::parse_repeat("foo^bar").unwrap();
    /// assert_eq!(repeat.has_components(), true);
    /// let repeat = hl7_parser::parser::parse_repeat("foo").unwrap();
    /// assert_eq!(repeat.has_components(), false);
    /// ```
    pub fn has_components(&self) -> bool {
        self.components.len() > 1
    }

    /// Returns true if the repeat has no components, or if all components
    /// have empty values.
    ///
    /// # Examples
    ///
    /// ```
    /// let repeat = hl7_parser::parser::parse_repeat("foo^bar").unwrap();
    /// assert_eq!(repeat.is_empty(), false);
    /// let repeat = hl7_parser::parser::parse_repeat("").unwrap();
    /// assert_eq!(repeat.is_empty(), true);
    /// let repeat = hl7_parser::parser::parse_repeat("^").unwrap();
    /// assert_eq!(repeat.is_empty(), true);
    /// ```
    pub fn is_empty(&self) -> bool {
        self.components.is_empty() || self.components.iter().all(|c| c.is_empty())
    }

    /// Get the component at the specified 1-based index
    /// Returns None if the index is out of bounds
    ///
    /// # Examples
    ///
    /// ```
    /// let repeat = hl7_parser::parser::parse_repeat("foo^bar").unwrap();
    /// assert_eq!(repeat.component(1).unwrap().raw_value(), "foo");
    /// assert_eq!(repeat.component(2).unwrap().raw_value(), "bar");
    /// assert_eq!(repeat.component(3), None);
    /// ```
    pub fn component(&self, number: usize) -> Option<&Component<'m>> {
        debug_assert!(number > 0, "Component numbers are 1-based");
        self.components.get(number - 1)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RepeatDisplay<'m> {
    components: &'m Vec<Component<'m>>,
    separators: &'m Separators,
}

impl Display for RepeatDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first: bool = true;
        for component in self.components {
            if first {
                first = false;
            } else {
                write!(f, "{}", self.separators.repetition)?;
            }
            if f.alternate() {
                write!(f, "{:#}", component.display(self.separators))?;
            } else {
                write!(f, "{}", component.display(self.separators))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::message::Subcomponent;

    use super::*;

    #[test]
    fn repeats_can_display_raw() {
        let separators = Separators::default();

        let component = Component {
            source: r"foo\F\bar",
            subcomponents: vec![
                Subcomponent {
                    value: r"foo\F\bar",
                    range: 0..1, // ignore
                },
            ],
            range: 0..1, // ignore
        };

        let repeat = Repeat {
            source: r"foo\F\bar~foo\F\bar",
            components: vec![component.clone(), component.clone()],
            range: 0..1, // ignore
        };

        let formatted = format!("{:#}", repeat.display(&separators));
        assert_eq!(formatted, r"foo\F\bar~foo\F\bar");
    }

    #[test]
    fn repeats_can_display_decoded() {
        let separators = Separators::default();

        let component = Component {
            source: r"foo\F\bar",
            subcomponents: vec![
                Subcomponent {
                    value: r"foo\F\bar",
                    range: 0..1, // ignore
                },
            ],
            range: 0..1, // ignore
        };

        let repeat = Repeat {
            source: r"foo\F\bar~foo\F\bar",
            components: vec![component.clone(), component.clone()],
            range: 0..1, // ignore
        };

        let formatted = format!("{}", repeat.display(&separators));
        assert_eq!(formatted, r"foo|bar~foo|bar");
    }
}


