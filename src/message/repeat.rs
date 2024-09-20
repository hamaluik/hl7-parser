use std::borrow::Cow;
use super::Component;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Repeat<'m> {
    Value(Cow<'m, str>),
    Components(Vec<Component<'m>>),
}

