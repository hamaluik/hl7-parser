use std::fmt::Display;

/// Separators used in HL7 messages
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Separators {
    pub field: char,
    pub component: char,
    pub subcomponent: char,
    pub repetition: char,
    pub escape: char,
    /// Not a separator, but a flag to indicate that newlines should be leniently parsed
    /// as part of the message. Enabling this flag will allow `\n` and `\r\n` to be treated
    /// the same as `\r` as the separator for segments.
    pub lenient_newlines: bool,
}

impl Default for Separators {
    /// Default separators for HL7 messages
    ///
    /// # Examples
    ///
    /// ```
    /// use hl7_parser::message::Separators;
    /// let separators = Separators::default();
    /// assert_eq!(separators.field, '|');
    /// assert_eq!(separators.component, '^');
    /// assert_eq!(separators.subcomponent, '&');
    /// assert_eq!(separators.repetition, '~');
    /// assert_eq!(separators.escape, '\\');
    /// ```
    fn default() -> Self {
        Separators {
            field: '|',
            component: '^',
            subcomponent: '&',
            repetition: '~',
            escape: '\\',
            lenient_newlines: false,
        }
    }
}

impl Separators {
    /// Encode a string that has separators into a string that escapes the separators
    /// with the escape characters
    ///
    /// # Examples
    ///
    /// ```
    /// use hl7_parser::message::Separators;
    /// let separators = Separators::default();
    /// let input = "foo|bar^baz&quux~quuz\\corge\rquack\nduck";
    /// let expected = r"foo\F\bar\S\baz\T\quux\R\quuz\E\corge\X0D\quack\X0A\duck";
    /// let actual = separators.encode(input).to_string();
    /// assert_eq!(expected, actual);
    /// ```
    pub fn encode<'m>(&'m self, value: &'m str) -> EncodedSeparatorsDisplay<'m> {
        EncodedSeparatorsDisplay {
            separators: self,
            value,
        }
    }

    /// Decode a string that has separators encoding values
    ///
    /// # Examples
    ///
    /// ```
    /// use hl7_parser::message::Separators;
    /// let separators = Separators::default();
    /// let input = r"foo\F\bar\S\baz\T\quux\R\quuz\E\corge\X0D\quack\X0A\duck\.br\";
    /// let expected = "foo|bar^baz&quux~quuz\\corge\rquack\nduck\r";
    /// let actual = separators.decode(input).to_string();
    /// assert_eq!(expected, actual);
    /// ```
    pub fn decode<'m>(&'m self, value: &'m str) -> DecodedSeparatorsDisplay<'m> {
        DecodedSeparatorsDisplay {
            separators: self,
            value,
        }
    }

    /// Allow lenient newlines in the message. This will allow `\n` and `\r\n` to be treated
    /// the same as `\r` as the separator for segments.
    pub fn with_lenient_newlines(&mut self, lenient_newlines: bool) -> Self {
        self.lenient_newlines = lenient_newlines;
        *self
    }
}

impl Display for Separators {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}{}{}",
            self.field, self.component, self.repetition, self.escape, self.subcomponent
        )
    }
}

/// A display implementation which encodes the separators in the value. (i.e. replaces them with
/// escape sequences)
pub struct EncodedSeparatorsDisplay<'m> {
    pub(crate) separators: &'m Separators,
    pub(crate) value: &'m str,
}

impl Display for EncodedSeparatorsDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for c in self.value.chars() {
            if c == '\r' {
                write!(f, "{escape}X0D{escape}", escape = self.separators.escape)?;
            } else if c == '\n' {
                write!(f, "{escape}X0A{escape}", escape = self.separators.escape)?;
            } else if c == self.separators.field {
                write!(f, "{escape}F{escape}", escape = self.separators.escape)?;
            } else if c == self.separators.repetition {
                write!(f, "{escape}R{escape}", escape = self.separators.escape)?;
            } else if c == self.separators.component {
                write!(f, "{escape}S{escape}", escape = self.separators.escape)?;
            } else if c == self.separators.subcomponent {
                write!(f, "{escape}T{escape}", escape = self.separators.escape)?;
            } else if c == self.separators.escape {
                write!(f, "{escape}E{escape}", escape = self.separators.escape)?;
            } else {
                write!(f, "{}", c)?;
            }
        }
        Ok(())
    }
}

/// A display implementation which decodes the escape sequences in the value using the separators.
pub struct DecodedSeparatorsDisplay<'m> {
    pub(crate) separators: &'m Separators,
    pub(crate) value: &'m str,
}

impl Display for DecodedSeparatorsDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut escaped = false;
        let mut escape_i: usize = 0;
        for (i, c) in self.value.chars().enumerate() {
            if c == self.separators.escape {
                if escaped {
                    escaped = false;
                    match &self.value[escape_i..i] {
                        "F" => write!(f, "{}", self.separators.field)?,
                        "R" => write!(f, "{}", self.separators.repetition)?,
                        "S" => write!(f, "{}", self.separators.component)?,
                        "T" => write!(f, "{}", self.separators.subcomponent)?,
                        "E" => write!(f, "{}", self.separators.escape)?,
                        "X0A" => writeln!(f)?,
                        "X0D" | ".br" => write!(f, "\r")?,
                        v => write!(f, "{v}")?,
                    }
                } else {
                    escape_i = i + 1;
                    escaped = true;
                }
            } else if !escaped {
                write!(f, "{}", c)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions_sorted::assert_eq;

    #[test]
    fn separators_can_encode() {
        let separators = Separators::default();

        let input = "foo|bar^baz&quux~quuz\\corge\rquack\nduck";
        let expected = r"foo\F\bar\S\baz\T\quux\R\quuz\E\corge\X0D\quack\X0A\duck";
        let actual = separators.encode(input).to_string();
        assert_eq!(expected, actual);
    }

    #[test]
    fn sample_encode() {
        let separators = Separators::default();

        let input = "Pierre DuRho^ne & Cie";
        let expected = r"Pierre DuRho\S\ne \T\ Cie";
        let actual = separators.encode(input).to_string();
        assert_eq!(expected, actual);
    }

    #[test]
    fn separators_can_decode() {
        let separators = Separators::default();

        let input = r"foo\F\bar\S\baz\T\quux\R\quuz\E\corge\X0D\quack\X0A\duck\.br\";
        let expected = "foo|bar^baz&quux~quuz\\corge\rquack\nduck\r";
        let actual = separators.decode(input).to_string();
        assert_eq!(expected, actual);
    }
}
