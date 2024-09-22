use std::{fmt::Display, ops::Range};

use super::Separators;

/// A subcomponent is the smallest unit of data in an HL7 message.
/// It is a string that may contain escape sequences to encode the separators.
/// It is the only type that does not have a separator character.
/// It is always contained within a component.
///
/// For parsing performance reasons, the subcomponent does not decode the escape
/// sequences when it is parsed. Instead, the escape sequences are decoded when
/// the subcomponent is displayed. This allows the subcomponent to be parsed
/// without allocating a new string for the decoded value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subcomponent<'m> {
    /// The raw value of the subcomponent, including escape sequences
    pub value: &'m str,
    /// The range of the subcomponent in the original message
    pub range: Range<usize>,
}

impl<'m> Subcomponent<'m> {
    pub(crate) fn new_single(source: &'m str, range: Range<usize>) -> Self {
        Subcomponent {
            value: source,
            range,
        }
    }

    #[inline]
    /// Display the subcomponent value, using the separators to decode escape sequences
    /// by default. Note: if you want to display the raw value without decoding escape
    /// sequences, use the `#` flag, e.g. `format!("{:#}", subcomponent.display(separators))`
    ///
    /// # Examples
    ///
    /// ```
    /// use hl7_parser::{Separators, Subcomponent};
    /// let separators = Separators::default();
    ///
    /// let subcomponent = Subcomponent {
    ///     value: r"foo\F\bar",
    ///     range: 0..1, // ignore
    /// };
    ///
    /// assert_eq!(format!("{}", subcomponent.display(&separators)), "foo|bar");
    /// assert_eq!(format!("{:#}", subcomponent.display(&separators)), r"foo\F\bar");
    /// ```
    pub fn display(&'m self, separators: &'m Separators) -> SubcomponentDisplay<'m> {
        SubcomponentDisplay {
            value: self.value,
            separators,
        }
    }

    #[inline]
    /// Get the raw value of the subcomponent. This is the value as it appears in the message,
    /// without any decoding of escape sequences.
    pub fn raw_value(&self) -> &'m str {
        self.value
    }
}

/// A display implementation for subcomponents.
/// This will decode the escape sequences in the subcomponent value
/// using the separators. If the `#` flag is used, the raw value
/// will be displayed without decoding the escape sequences.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SubcomponentDisplay<'m> {
    value: &'m str,
    separators: &'m Separators,
}

impl Display for SubcomponentDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "{}", self.value)
        } else {
            write!(f, "{}", self.separators.decode(self.value))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subcomponents_can_display_raw() {
        let separators = Separators::default();

        let subcomponent = Subcomponent {
            value: r"foo\F\bar",
            range: 0..1, // ignore
        };

        assert_eq!(
            format!("{:#}", subcomponent.display(&separators)),
            r"foo\F\bar"
        );
    }

    #[test]
    fn subcomponents_can_display_decoded() {
        let separators = Separators::default();

        let subcomponent = Subcomponent {
            value: r"foo\F\bar",
            range: 0..1, // ignore
        };

        assert_eq!(format!("{}", subcomponent.display(&separators)), "foo|bar");
    }
}
