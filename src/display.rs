use crate::message::{Component, Field, Repeat, Separators, Subcomponent};
use std::fmt::Display;

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
