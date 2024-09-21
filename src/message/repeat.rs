use std::borrow::Cow;

use super::Component;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Repeat<'m> {
    Value(Cow<'m, str>),
    Component(Component<'m>),
    Components(Vec<Component<'m>>),
}

