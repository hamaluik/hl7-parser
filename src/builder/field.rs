use super::RepeatBuilder;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FieldBuilder {
    Value(String),
    Repeats(Vec<RepeatBuilder>),
}

