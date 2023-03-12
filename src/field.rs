use crate::Component;
use std::{num::NonZeroUsize, ops::Range};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub range: Range<usize>,
    pub components: Vec<Component>,
}

impl Field {
    #[inline]
    pub fn component(&self, component: NonZeroUsize) -> Option<&Component> {
        self.components.get(component.get() - 1)
    }

    #[inline]
    pub fn source<'s>(&self, s: &'s str) -> &'s str {
        &s[self.range.clone()]
    }

    pub fn component_at_cursor(&self, cursor: usize) -> Option<(NonZeroUsize, &Component)> {
        self.components
            .iter()
            .enumerate()
            .find(|(_, component)| component.range.contains(&cursor))
            .map(|(i, sc)| (NonZeroUsize::new(i + 1).unwrap(), sc))
    }
}

pub trait ComponentAccessor {
    fn component(&self, component: NonZeroUsize) -> Option<&Component>;
}

impl ComponentAccessor for Option<&Field> {
    fn component(&self, component: NonZeroUsize) -> Option<&Component> {
        match self {
            None => None,
            Some(field) => field.component(component),
        }
    }
}
