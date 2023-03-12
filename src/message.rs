use crate::{
    Component, ComponentAccessor, Field, ParseError, Segment, Segments, Separators, SubComponent,
    SubComponentAccessor,
};
use std::{collections::HashMap, num::NonZeroUsize};

#[derive(Debug, Clone)]
pub struct Message<'s> {
    pub source: &'s str,
    pub separators: Separators,
    pub segments: HashMap<&'s str, Segments>,
}

pub struct LocatedData<'s> {
    pub segment: Option<(&'s str, NonZeroUsize, &'s Segment)>,
    pub field: Option<(NonZeroUsize, &'s Field)>,
    pub component: Option<(NonZeroUsize, &'s Component)>,
    pub sub_component: Option<(NonZeroUsize, &'s SubComponent)>,
}

impl<'s> Message<'s> {
    pub fn parse(source: &'s str) -> Result<Message<'s>, ParseError> {
        let (_, message) = crate::parser::parse_message(crate::parser::Span::new(source))
            .map_err(|_| ParseError::Failed)?;
        Ok(message)
    }

    pub fn has_segment(&'s self, segment: &str) -> bool {
        self.segments.contains_key(segment)
    }

    pub fn segment(&'s self, segment: &str) -> Option<&'s Segment> {
        self.segments.get(segment).map(|seg| seg.nth(0)).flatten()
    }

    pub fn segment_count(&'s self, segment: &str) -> usize {
        self.segments
            .get(segment)
            .map(|seg| seg.count())
            .unwrap_or_default()
    }

    pub fn segment_n(&'s self, segment: &str, n: NonZeroUsize) -> Option<&'s Segment> {
        self.segments
            .get(segment)
            .map(|seg| seg.nth(n.get() - 1))
            .flatten()
    }

    pub fn get_field_source(
        &'s self,
        segment: (&str, NonZeroUsize),
        field: NonZeroUsize,
    ) -> Option<&'s str> {
        let Some(seg) = self.segment_n(segment.0, segment.1) else {
            return None;
        };

        seg.field(field).map(|f| f.source(self.source))
    }

    pub fn get_component_source(
        &'s self,
        segment: (&str, NonZeroUsize),
        field: NonZeroUsize,
        component: NonZeroUsize,
    ) -> Option<&'s str> {
        let Some(seg) = self.segment_n(segment.0, segment.1) else {
            return None;
        };

        seg.field(field)
            .component(component)
            .map(|c| c.source(self.source))
    }

    pub fn get_sub_component_source(
        &'s self,
        segment: (&str, NonZeroUsize),
        field: NonZeroUsize,
        component: NonZeroUsize,
        sub_component: NonZeroUsize,
    ) -> Option<&'s str> {
        let Some(seg) = self.segment_n(segment.0, segment.1) else {
            return None;
        };

        seg.field(field)
            .component(component)
            .sub_component(sub_component)
            .map(|s| s.source(self.source))
    }

    pub fn segment_at_cursor(
        &'s self,
        cursor: usize,
    ) -> Option<(&'s str, NonZeroUsize, &'s Segment)> {
        self.segments
            .iter()
            .find_map(|(id, segs)| segs.segment_at_cursor(cursor).map(|(n, seg)| (*id, n, seg)))
    }

    pub fn locate_cursor(&'s self, cursor: usize) -> LocatedData<'s> {
        let segment = self.segment_at_cursor(cursor);
        let field = segment
            .map(|(_, _, segment)| segment.field_at_cursor(cursor))
            .flatten();
        let component = field
            .map(|(_, field)| field.component_at_cursor(cursor))
            .flatten();
        let sub_component = component
            .map(|(_, component)| component.sub_component_at_cursor(cursor))
            .flatten();
        LocatedData {
            segment,
            field,
            component,
            sub_component,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_locate_cursor() {
        let cursor = 26;
        let message = include_str!("../test_assets/sample_adt_a01.hl7")
            .replace("\r\n", "\r")
            .replace('\n', "\r");
        let message = Message::parse(message.as_str()).expect("can parse message");

        let (id, n, seg) = message
            .segment_at_cursor(cursor)
            .expect("can get segment at cursor");
        assert_eq!(id, "MSH");
        assert_eq!(n, NonZeroUsize::new(1).unwrap());

        let (n, _field) = seg
            .field_at_cursor(cursor)
            .expect("can get field at cursor");
        assert_eq!(n, NonZeroUsize::new(7).unwrap());

        let cursor = 0x458;
        let (id, n, seg) = message
            .segment_at_cursor(cursor)
            .expect("can get segment at cursor");
        assert_eq!(id, "IN1");
        assert_eq!(n, NonZeroUsize::new(1).unwrap());

        let (n, field) = seg
            .field_at_cursor(cursor)
            .expect("can get field at cursor");
        assert_eq!(n, NonZeroUsize::new(5).unwrap());

        let (n, component) = field
            .component_at_cursor(cursor)
            .expect("can get component at cursor");
        assert_eq!(n, NonZeroUsize::new(3).unwrap());
        assert_eq!(component.source(message.source), "HOLLYWOOD");
    }
}
