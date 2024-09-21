use std::borrow::Cow;
use super::{Component, Repeat};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field<'m> {
    pub value: Cow<'m, str>,
    pub components: Vec<Component<'m>>,
    pub repeats: Vec<Repeat<'m>>,
}


// #[derive(Debug, Clone, PartialEq, Eq)]
// pub enum Field<'m> {
//     Value(Cow<'m, str>),
//     Components(Vec<Component<'m>>),
//     Repeats(Vec<Repeat<'m>>),
// }
