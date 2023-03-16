use crate::Field;
use std::ops::Range;

/// The separators for the message
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Separators {
    /// The character that separates fields (default: `'|'`)
    pub field: char,
    /// The character that separates components (default: `'^'`)
    pub component: char,
    /// The character that indicates repeats (default: `'~'`)
    pub repeat: char,
    /// The escape character (default: `'\'`)
    pub escape: char,
    /// The character that separates sub-components (default: `'&'`)
    pub subcomponent: char,
}

impl Default for Separators {
    fn default() -> Self {
        Separators {
            field: '|',
            component: '^',
            repeat: '~',
            escape: '\\',
            subcomponent: '&',
        }
    }
}

impl Separators {
    /// Given an un-decoded source, will decode the string into its canonical form
    ///
    /// # Examples
    ///
    /// ```
    /// # use hl7_parser::Separators;
    /// let separators = Separators::default();
    /// assert_eq!(
    ///     separators.decode(r#"Pierre DuRho\S\ne \T\ Cie"#).as_str(),
    ///     r#"Pierre DuRho^ne & Cie"#
    /// );
    /// ```
    pub fn decode<S: AsRef<str>>(&self, source: S) -> String {
        let mut tmp = [0; 4];
        source
            .as_ref()
            .replace(r#"\F\"#, self.field.encode_utf8(&mut tmp))
            .replace(r#"\R\"#, self.repeat.encode_utf8(&mut tmp))
            .replace(r#"\S\"#, self.component.encode_utf8(&mut tmp))
            .replace(r#"\T\"#, self.subcomponent.encode_utf8(&mut tmp))
            .replace(r#"\.br\"#, "\r")
            .replace(r#"\X0A\"#, "\n")
            .replace(r#"\X0D\"#, "\r")
            .replace(r#"\E\"#, self.escape.encode_utf8(&mut tmp))
    }
}

#[derive(Debug)]
pub(crate) struct Msh {
    pub range: Range<usize>,
    pub separators: Separators,
    pub fields: Vec<Field>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_decode_encoding_characters() {
        let separators = Separators::default();
        assert_eq!(separators.decode(r#"\.br\\X0A\\X0D\"#).as_str(), "\r\n\r");
        assert_eq!(separators.decode(r#"\F\\R\\S\\T\\E\"#).as_str(), r#"|~^&\"#);
        assert_eq!(separators.decode(r#"\E\\F\\E\"#).as_str(), r#"\|\"#);
    }
}
