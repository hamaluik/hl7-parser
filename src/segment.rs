use std::{num::NonZeroUsize, ops::Range};

use crate::{Field, MSH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    pub range: Range<usize>,
    pub fields: Vec<Field>,
}

impl Segment {
    #[inline]
    pub fn field(&self, field: NonZeroUsize) -> Option<&Field> {
        self.fields.get(field.get() - 1)
    }

    #[inline]
    pub fn source<'s>(&self, s: &'s str) -> &'s str {
        &s[self.range.clone()]
    }
}

pub trait FieldAccessor {
    fn field(&self, field: NonZeroUsize) -> Option<&Field>;
}

impl FieldAccessor for Option<&Segment> {
    fn field(&self, field: NonZeroUsize) -> Option<&Field> {
        match self {
            None => None,
            Some(seg) => seg.field(field),
        }
    }
}

impl From<MSH> for Segment {
    fn from(msh: MSH) -> Self {
        let MSH {
            range, mut fields, ..
        } = msh;
        fields.insert(
            0,
            Field {
                range: 3..4,
                components: Vec::with_capacity(0),
            },
        );
        fields.insert(
            1,
            Field {
                range: 4..8,
                components: Vec::with_capacity(0),
            },
        );
        Segment { range, fields }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segments {
    Single(Segment),
    Many(Vec<Segment>),
}

impl Segment {
    pub fn field_at_cursor(&self, cursor: usize) -> Option<(NonZeroUsize, &Field)> {
        self.fields
            .iter()
            .enumerate()
            .find(|(_, field)| field.range.contains(&cursor))
            .map(|(i, sc)| (NonZeroUsize::new(i + 1).unwrap(), sc))
    }
}

impl Segments {
    pub fn nth(&self, n: usize) -> Option<&Segment> {
        match self {
            Segments::Single(seg) if n == 0 => Some(seg),
            Segments::Many(segs) if n < segs.len() => Some(&segs[n]),
            _ => None,
        }
    }

    pub fn count(&self) -> usize {
        match self {
            Segments::Single(_) => 1,
            Segments::Many(segs) => segs.len(),
        }
    }

    pub fn field_at_cursor(&self, cursor: usize) -> Option<(NonZeroUsize, NonZeroUsize, &Field)> {
        match self {
            Segments::Single(seg) => seg
                .field_at_cursor(cursor)
                .map(|(n, f)| (NonZeroUsize::new(1).unwrap(), n, f)),
            Segments::Many(segs) => segs.iter().enumerate().find_map(|(i, seg)| {
                seg.field_at_cursor(cursor)
                    .map(|(n, f)| (NonZeroUsize::new(i + 1).unwrap(), n, f))
            }),
        }
    }

    pub fn segment_at_cursor(&self, cursor: usize) -> Option<(NonZeroUsize, &Segment)> {
        match self {
            Segments::Single(seg) => {
                if seg.range.contains(&cursor) {
                    Some((NonZeroUsize::new(1).unwrap(), seg))
                } else {
                    None
                }
            }
            Segments::Many(segs) => segs
                .iter()
                .enumerate()
                .find(|(_, seg)| seg.range.contains(&cursor))
                .map(|(i, seg)| (NonZeroUsize::new(i + 1).unwrap(), seg)),
        }
    }
}

impl From<Segment> for Segments {
    fn from(value: Segment) -> Self {
        Segments::Single(value)
    }
}

impl From<Vec<Segment>> for Segments {
    fn from(value: Vec<Segment>) -> Self {
        Segments::Many(value)
    }
}
