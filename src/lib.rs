use std::borrow::Cow;

use message::Separators;

pub mod message;
mod parser;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message<'m> {
    pub segments: Vec<Segment<'m>>,
    pub separators: Separators,
}

impl<'m> Message<'m> {
    pub fn from_str(input: &'m str) -> Result<Self, String> {
        parser::message::message()(input)
            .map(|(_, m)| m)
            .map_err(move |e| format!("Failed to parse message: {:?}", e))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment<'m> {
    pub name: Cow<'m, str>,
    pub fields: Vec<Field<'m>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field<'m> {
    pub value: Cow<'m, str>,
    pub components: Vec<Component<'m>>,
    pub repeats: Vec<Repeat<'m>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repeat<'m> {
    pub value: Cow<'m, str>,
    pub components: Vec<Component<'m>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component<'m> {
    pub value: Cow<'m, str>,
    pub subcomponents: Vec<message::Subcomponent<'m>>,
}

