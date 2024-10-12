use std::fmt::Display;

use crate::message::{Field, Separators};

use super::RepeatBuilder;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FieldBuilder {
    Value(String),
    Repeats(Vec<RepeatBuilder>),
}

impl Default for FieldBuilder {
    fn default() -> Self {
        FieldBuilder::Value(String::new())
    }
}

impl FieldBuilder {
    pub fn with_value(value: String) -> Self {
        FieldBuilder::Value(value)
    }

    pub fn with_repeats(repeats: Vec<RepeatBuilder>) -> Self {
        FieldBuilder::Repeats(repeats)
    }

    pub fn value(&self) -> Option<&String> {
        match self {
            FieldBuilder::Value(value) => Some(value),
            _ => None,
        }
    }

    pub fn repeats(&self) -> Option<&Vec<RepeatBuilder>> {
        match self {
            FieldBuilder::Repeats(repeats) => Some(repeats),
            _ => None,
        }
    }

    pub fn value_mut(&mut self) -> Option<&mut String> {
        match self {
            FieldBuilder::Value(value) => Some(value),
            _ => None,
        }
    }

    pub fn repeats_mut(&mut self) -> Option<&mut Vec<RepeatBuilder>> {
        match self {
            FieldBuilder::Repeats(repeats) => Some(repeats),
            _ => None,
        }
    }

    pub fn into_value(self) -> Option<String> {
        match self {
            FieldBuilder::Value(value) => Some(value),
            _ => None,
        }
    }

    pub fn into_repeats(self) -> Option<Vec<RepeatBuilder>> {
        match self {
            FieldBuilder::Repeats(repeats) => Some(repeats),
            _ => None,
        }
    }

    pub fn has_repeats(&self) -> bool {
        matches!(self, FieldBuilder::Repeats(_))
    }

    pub fn is_empty(&self) -> bool {
        match self {
            FieldBuilder::Value(value) => value.is_empty(),
            FieldBuilder::Repeats(repeats) => repeats.is_empty(),
        }
    }

    pub fn set_value(&mut self, value: String) {
        *self = FieldBuilder::Value(value);
    }

    pub fn set_repeats(&mut self, repeats: Vec<RepeatBuilder>) {
        *self = FieldBuilder::Repeats(repeats);
    }

    pub fn push_repeat(&mut self, repeat: RepeatBuilder) {
        match self {
            FieldBuilder::Repeats(repeats) => repeats.push(repeat),
            _ => *self = FieldBuilder::Repeats(vec![repeat]),
        }
    }

    pub fn clear(&mut self) {
        *self = FieldBuilder::Value(String::new());
    }

    pub fn repeat(&self, index: usize) -> Option<&RepeatBuilder> {
        match self {
            FieldBuilder::Repeats(repeats) => repeats.get(index),
            _ => None,
        }
    }

    pub fn repeat_mut(&mut self, index: usize) -> Option<&mut RepeatBuilder> {
        match self {
            FieldBuilder::Repeats(repeats) => repeats.get_mut(index),
            _ => None,
        }
    }

    pub fn remove_repeat(&mut self, index: usize) -> Option<RepeatBuilder> {
        match self {
            FieldBuilder::Repeats(repeats) => {
                if index < repeats.len() {
                    Some(repeats.remove(index))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn display<'a>(&'a self, separators: &'a Separators) -> FieldBuilderDisplay<'a> {
        FieldBuilderDisplay {
            field: self,
            separators,
        }
    }
}

pub struct FieldBuilderDisplay<'a> {
    field: &'a FieldBuilder,
    separators: &'a Separators,
}

impl<'a> Display for FieldBuilderDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.field {
            FieldBuilder::Value(value) => self.separators.encode(value).fmt(f),
            FieldBuilder::Repeats(repeats) => {
                let mut first = true;
                for repeat in repeats {
                    if first {
                        first = false;
                    } else {
                        write!(f, "{}", self.separators.repetition)?;
                    }
                    write!(f, "{}", repeat.display(self.separators))?;
                }
                Ok(())
            }
        }
    }
}

impl<S: ToString> From<S> for FieldBuilder {
    fn from(value: S) -> Self {
        FieldBuilder::Value(value.to_string())
    }
}

impl<'m> From<&'m Field<'m>> for FieldBuilder {
    fn from(field: &'m Field) -> Self {
        if field.has_repeats()
            || (!field.repeats.is_empty()
                && (field.repeats[0].has_components()
                    || (!field.repeats[0].components.is_empty()
                        && field.repeats[0].components[0].has_subcomponents())))
        {
            FieldBuilder::Repeats(field.repeats().map(RepeatBuilder::from).collect())
        } else {
            FieldBuilder::Value(field.raw_value().to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions_sorted::assert_eq;

    #[test]
    fn can_display_field_builder() {
        let separators = Separators::default();
        let field = crate::parser::parse_field("foo~bar").unwrap();
        let field = FieldBuilder::from(&field);
        let display = field.display(&separators).to_string();
        assert_eq!(display, "foo~bar");
    }
}
