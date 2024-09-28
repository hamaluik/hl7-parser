use crate::message::{Component, Field, Repeat, Separators, Subcomponent};
use std::fmt::Display;

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

/// A display implementation for segments.
/// This will decode the escape sequences in the segment value
/// using the separators. If the `#` flag is used, the raw value
/// will be displayed without decoding the escape sequences.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SegmentDisplay<'m> {
    pub(crate) name: &'m str,
    pub(crate) fields: &'m Vec<Field<'m>>,
    pub(crate) separators: &'m Separators,
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct FieldDisplay<'m> {
    pub(crate) repeats: &'m Vec<Repeat<'m>>,
    pub(crate) separators: &'m Separators,
}

impl Display for FieldDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first: bool = true;
        for repeat in self.repeats {
            if first {
                first = false;
            } else {
                write!(f, "{}", self.separators.repetition)?;
            }
            if f.alternate() {
                write!(f, "{:#}", repeat.display(self.separators))?;
            } else {
                write!(f, "{}", repeat.display(self.separators))?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RepeatDisplay<'m> {
    pub(crate) components: &'m Vec<Component<'m>>,
    pub(crate) separators: &'m Separators,
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

/// A display implementation for components.
/// This will decode the escape sequences in the component values
/// using the separators. If the `#` flag is used, the raw value
/// will be displayed without decoding the escape sequences.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ComponentDisplay<'m> {
    pub(crate) subcomponents: &'m Vec<Subcomponent<'m>>,
    pub(crate) separators: &'m Separators,
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

/// A display implementation for subcomponents.
/// This will decode the escape sequences in the subcomponent value
/// using the separators. If the `#` flag is used, the raw value
/// will be displayed without decoding the escape sequences.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SubcomponentDisplay<'m> {
    pub(crate) value: &'m str,
    pub(crate) separators: &'m Separators,
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
