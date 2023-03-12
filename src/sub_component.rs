use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubComponent {
    pub range: Range<usize>,
}

impl SubComponent {
    #[inline]
    pub fn source<'s>(&self, s: &'s str) -> &'s str {
        &s[self.range.clone()]
    }
}
