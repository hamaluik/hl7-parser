use super::{Separators, Subcomponent};
use std::{fmt::Display, ops::Range};

/// A component is a part of a field, and is separated from other components by the component
/// separator character. A component is composed of 0 or more subcomponents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component<'m> {
    pub(crate) source: &'m str,
    /// The subcomponents of the component
    pub subcomponents: Vec<Subcomponent<'m>>,
    /// The range of the component in the original message
    pub range: Range<usize>,
}

impl<'m> Component<'m> {
    pub(crate) fn new_single(source: &'m str, range: Range<usize>) -> Self {
        Component {
            source,
            subcomponents: vec![Subcomponent::new_single(source, range.clone())],
            range,
        }
    }

    #[inline]
    /// An iterator over the subcomponents of the component
    pub fn subcomponents(&self) -> impl Iterator<Item = &Subcomponent<'m>> {
        self.subcomponents.iter()
    }

    #[inline]
    /// Display the component value, using the separators to decode escape sequences
    /// by default. Note: if you want to display the raw value without decoding escape
    /// sequences, use the `#` flag, e.g. `format!("{:#}", component.display(separators))`.
    /// Components will be separated by the component separator character.
    pub fn display(&'m self, separators: &'m Separators) -> ComponentDisplay<'m> {
        ComponentDisplay {
            subcomponents: &self.subcomponents,
            separators,
        }
    }

    #[inline]
    /// Get the raw value of the component. This is the value as it appears in the message,
    /// without any decoding of escape sequences, and including all subcomponents and
    /// their separators.
    ///
    /// # Examples
    ///
    /// ```
    /// let component = hl7_parser::parser::parse_component("foo&bar").unwrap();
    /// assert_eq!(component.subcomponents.len(), 2);
    /// assert_eq!(component.raw_value(), "foo&bar");
    /// ```
    pub fn raw_value(&self) -> &'m str {
        self.source
    }

    #[inline]
    /// Returns true if the component has more than one subcomponent. Note that
    /// if the component has only one subcomponent, the value of that subcomponent
    /// is essentially the value of the component, so the value of the component
    /// can be obtained using `raw_value()`.
    ///
    /// # Examples
    ///
    /// ```
    /// let component = hl7_parser::parser::parse_component("foo&bar").unwrap();
    /// assert_eq!(component.has_subcomponents(), true);
    /// let component = hl7_parser::parser::parse_component("foo").unwrap();
    /// assert_eq!(component.has_subcomponents(), false);
    /// ```
    pub fn has_subcomponents(&self) -> bool {
        self.subcomponents.len() > 1
    }

    /// Returns true if the component has no subcomponents, or if all subcomponents
    /// have empty values.
    ///
    /// # Examples
    ///
    /// ```
    /// let component = hl7_parser::parser::parse_component("foo&bar").unwrap();
    /// assert_eq!(component.is_empty(), false);
    /// let component = hl7_parser::parser::parse_component("foo").unwrap();
    /// assert_eq!(component.is_empty(), false);
    /// let component = hl7_parser::parser::parse_component("foo&").unwrap();
    /// assert_eq!(component.is_empty(), false);
    /// let component = hl7_parser::parser::parse_component("").unwrap();
    /// assert_eq!(component.is_empty(), true);
    /// let component = hl7_parser::parser::parse_component("&").unwrap();
    /// assert_eq!(component.is_empty(), true);
    /// ```
    pub fn is_empty(&self) -> bool {
        self.subcomponents.is_empty() || self.subcomponents.iter().all(|s| s.value.is_empty())
    }

    /// Get a subcomponent by its number. Subcomponent numbers are 1-based.
    /// Returns `None` if the subcomponent number is out of range.
    ///
    /// # Examples
    ///
    /// ```
    /// let component = hl7_parser::parser::parse_component("foo&bar").unwrap();
    /// assert_eq!(component.subcomponent(1).unwrap().value, "foo");
    /// assert_eq!(component.subcomponent(2).unwrap().value, "bar");
    /// assert_eq!(component.subcomponent(3), None);
    /// ```
    pub fn subcomponent(&self, number: usize) -> Option<&Subcomponent<'m>> {
        debug_assert!(number > 0, "Subcomponent numbers are 1-based");
        self.subcomponents.get(number - 1)
    }
}

/// A display implementation for components.
/// This will decode the escape sequences in the component values
/// using the separators. If the `#` flag is used, the raw value
/// will be displayed without decoding the escape sequences.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ComponentDisplay<'m> {
    subcomponents: &'m Vec<Subcomponent<'m>>,
    separators: &'m Separators,
}

impl Display for ComponentDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //     write!(f, "{}", self.separators.decode(self.value))
        let mut first: bool = true;
        for subcomponent in self.subcomponents {
            if first {
                first = false;
            } else {
                write!(f, "{}", self.separators.subcomponent)?;
            }
            if f.alternate() {
                write!(f, "{}", subcomponent.value)?;
            } else {
                write!(f, "{}", subcomponent.display(self.separators))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_get_component_raw_value() {
        let component = Component {
            source: "foo&bar",
            subcomponents: vec![
                Subcomponent {
                    value: "foo",
                    range: 0..1, // ignore
                },
                Subcomponent {
                    value: "bar",
                    range: 0..1, // ignore
                },
            ],
            range: 0..1, // ignore
        };

        assert_eq!(component.raw_value(), "foo&bar");
    }

    #[test]
    fn components_can_display_raw() {
        let component = Component {
            source: r"foo\F\bar&baz",
            subcomponents: vec![
                Subcomponent {
                    value: r"foo\F\bar",
                    range: 0..1, // ignore
                },
                Subcomponent {
                    value: r"baz",
                    range: 0..1, // ignore
                },
            ],
            range: 0..1, // ignore
        };

        let formatted = format!("{:#}", component.display(&Separators::default()));
        assert_eq!(formatted, r"foo\F\bar&baz");
    }

    #[test]
    fn components_can_display_decoded() {
        let component = Component {
            source: r"foo\F\bar&baz",
            subcomponents: vec![
                Subcomponent {
                    value: r"foo\F\bar",
                    range: 0..1, // ignore
                },
                Subcomponent {
                    value: r"baz",
                    range: 0..1, // ignore
                },
            ],
            range: 0..1, // ignore
        };

        let formatted = format!("{}", component.display(&Separators::default()));
        assert_eq!(formatted, "foo|bar&baz");
    }
}
