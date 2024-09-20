use std::{fmt::Display, borrow::Cow, ops::{Deref, DerefMut}};
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
pub struct Subcomponent<'m>(pub Cow<'m, str>);

impl<'m> Subcomponent<'m> {
    /// Create a new subcomponent with the given value. The value must be a raw
    /// value with proper escape sequences
    ///
    /// # Examples
    ///
    /// ```
    /// use hl7_parser::message::Subcomponent;
    /// let subcomponent = Subcomponent::new_raw(r"foo\F\bar");
    /// assert_eq!(subcomponent.raw_value(), r"foo\F\bar");
    /// ```
    pub fn new_raw<V: Into<Cow<'m, str>>>(value: V) -> Self {
        Self(value.into())
    }

    /// Create a new subcomponent with the given value. The value provided is
    /// not escaped, and will be encoded using the given Separators
    ///
    /// # Examples
    ///
    /// ```
    /// use hl7_parser::message::{Separators, Subcomponent};
    /// let separators = Separators::default();
    ///
    /// let subcomponent = Subcomponent::new(r"foo|bar", &separators);
    /// assert_eq!(subcomponent.raw_value(), r"foo\F\bar");
    /// ```
    pub fn new<V: AsRef<str>>(value: V, separators: &Separators) -> Self {
        Self(
            separators.encode(value.as_ref()).to_string().into()
        )
    }

    /// Display the subcomponent value, using the separators to decode escape sequences
    /// by default. Note: if you want to display the raw value without decoding escape
    /// sequences, use the `#` flag, e.g. `format!("{:#}", subcomponent.display(separators))`
    ///
    /// # Examples
    ///
    /// ```
    /// use hl7_parser::message::{Separators, Subcomponent};
    /// let separators = Separators::default();
    ///
    /// let subcomponent = Subcomponent::new_raw(r"foo\F\bar");
    ///
    /// assert_eq!(format!("{}", subcomponent.display(&separators)), "foo|bar");
    /// assert_eq!(format!("{:#}", subcomponent.display(&separators)), r"foo\F\bar");
    /// ```
    pub fn display(&'m self, separators: &'m Separators) -> SubcomponentDisplay<'m> {
        SubcomponentDisplay {
            value: &self,
            separators,
        }
    }

    /// Get the raw value of the subcomponent, without decoding escape sequences
    pub fn raw_value(&self) -> &Cow<'m, str> {
        &self
    }

    /// Get a mutable reference to the raw value of the subcomponent,
    /// without decoding escape sequences. Note that any modifications to this
    /// value should be encoded using the `encode` method on [Separators]
    /// so that the escape sequences are properly encoded.
    ///
    /// # Examples
    /// ```
    /// use hl7_parser::message::{Separators, Subcomponent};
    /// let separators = Separators::default();
    ///
    /// let mut subcomponent = Subcomponent::new_raw("foo");
    ///
    /// *subcomponent.raw_value_mut() = separators.encode(r"foo|bar").to_string().into();
    /// assert_eq!(subcomponent.raw_value(), r"foo\F\bar");
    /// ```
    pub fn raw_value_mut(&mut self) -> &mut Cow<'m, str> {
        &mut self.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SubcomponentDisplay<'m> {
    value: &'m str,
    separators: &'m Separators,
}

impl Display for SubcomponentDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "{}", self.value)
        }
        else {
            write!(f, "{}", self.separators.decode(self.value))
        }
    }
}


impl<'m> Deref for Subcomponent<'m> {
    type Target = Cow<'m, str>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'m> DerefMut for Subcomponent<'m> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
