use crate::SubComponent;
use std::{num::NonZeroUsize, ops::Range};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub range: Range<usize>,
    pub sub_components: Vec<SubComponent>,
}

impl Component {
    #[inline]
    pub fn sub_component(&self, sub_component: NonZeroUsize) -> Option<&SubComponent> {
        self.sub_components.get(sub_component.get() - 1)
    }

    #[inline]
    pub fn source<'s>(&self, s: &'s str) -> &'s str {
        &s[self.range.clone()]
    }

    pub fn sub_component_at_cursor(&self, cursor: usize) -> Option<(NonZeroUsize, &SubComponent)> {
        self.sub_components
            .iter()
            .enumerate()
            .find(|(_, sub_component)| sub_component.range.contains(&cursor))
            .map(|(i, sc)| (NonZeroUsize::new(i + 1).unwrap(), sc))
    }
}

pub trait SubComponentAccessor {
    fn sub_component(&self, sub_component: NonZeroUsize) -> Option<&SubComponent>;
}

impl SubComponentAccessor for Option<&Component> {
    #[inline]
    fn sub_component(&self, sub_component: NonZeroUsize) -> Option<&SubComponent> {
        match self {
            None => None,
            Some(component) => component.sub_component(sub_component),
        }
    }
}
