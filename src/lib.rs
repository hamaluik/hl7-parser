use std::borrow::Cow;

#[allow(unused)]
mod parser;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Separators {
    pub field: char,
    pub component: char,
    pub subcomponent: char,
    pub repetition: char,
    pub escape: char,
}

impl Default for Separators {
    fn default() -> Self {
        Separators {
            field: '|',
            component: '^',
            subcomponent: '&',
            repetition: '~',
            escape: '\\',
        }
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
    pub subcomponents: Vec<Subcomponent<'m>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subcomponent<'m> {
    pub value: Cow<'m, str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message<'m> {
    pub segments: Vec<Segment<'m>>,
    pub separators: Separators,
}
