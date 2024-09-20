use std::borrow::Cow;
use super::Field;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment<'m> {
    pub name: Cow<'m, str>,
    pub fields: Vec<Field<'m>>,
}
