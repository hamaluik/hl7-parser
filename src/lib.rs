use std::ops::Deref;

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

impl From<&str> for Separators {
    fn from(value: &str) -> Self {
        Separators {
            field: value.chars().nth(0).expect("char 0: field separator"),
            component: value.chars().nth(1).expect("char 1: component separator"),
            repetition: value.chars().nth(2).expect("char 2: repetition separator"),
            escape: value.chars().nth(3).expect("char 3: escape"),
            subcomponent: value.chars().nth(4).expect("char 4: subcomponent separator"),
        }
    }
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
    pub name: &'m str,
    pub fields: Vec<Field<'m>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field<'m> {
    pub value: &'m str,
    pub components: Vec<Component<'m>>,
    pub repeats: Vec<Repeat<'m>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repeat<'m> {
    pub value: &'m str,
    pub components: Vec<Component<'m>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component<'m> {
    pub value: &'m str,
    pub subcomponents: Vec<Subcomponent<'m>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subcomponent<'m> {
    pub value: &'m str,
}

impl Deref for Subcomponent<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message<'m> {
    pub segments: Vec<Segment<'m>>,
    pub separators: Separators,
}
