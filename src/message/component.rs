use std::borrow::Cow;

use super::Subcomponent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Component<'m> {
    Value(Cow<'m, str>),
    Subcomponents(Vec<Subcomponent<'m>>),
}

