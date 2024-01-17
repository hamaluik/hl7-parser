use std::borrow::Cow;
use std::fmt::Display;

pub use message::Separators;

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
    pub subcomponents: Vec<Subcomponent<'m>>,
}

impl<'m> Component<'m> {
    pub fn display(&'m self, separators: &'m Separators, escape: bool) -> ComponentDisplay<'m> {
        ComponentDisplay {
            subcomponents: self
                .subcomponents
                .iter()
                .map(|s| s.display(separators, escape))
                .collect(),
            separators,
            escape,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentDisplay<'m> {
    subcomponents: Vec<SubcomponentDisplay<'m>>,
    separators: &'m Separators,
    escape: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subcomponent<'m> {
    pub value: Cow<'m, str>,
}

impl<'m> Subcomponent<'m> {
    pub fn display(&'m self, separators: &'m Separators, escape: bool) -> SubcomponentDisplay<'m> {
        SubcomponentDisplay {
            value: &self.value,
            separators,
            escape,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SubcomponentDisplay<'m> {
    value: &'m str,
    separators: &'m Separators,
    escape: bool,
}

impl Display for SubcomponentDisplay<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.escape {
            for c in self.value.chars() {
                if c == self.separators.escape {
                    write!(f, "{}{}", self.separators.escape, self.separators.escape)?;
                } else if c == self.separators.subcomponent {
                    write!(
                        f,
                        "{}{}",
                        self.separators.escape, self.separators.subcomponent
                    )?;
                } else {
                    write!(f, "{}", c)?;
                }
            }
        } else {
            write!(f, "{}", self.value)?;
        }
        Ok(())
    }
}
