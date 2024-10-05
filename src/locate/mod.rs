use std::{collections::HashMap, fmt::Display};

use crate::{
    message::{Component, Field, Repeat, Segment, Subcomponent},
    Message,
};

/// Results from locating a cursor within a message
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct LocatedCursor<'s> {
    pub message: &'s Message<'s>,
    /// The (segment name, and 1-based segment index) containing the cursor
    pub segment: Option<(&'s str, usize, &'s Segment<'s>)>,
    /// The (1-based field index, field) containing the cursor
    pub field: Option<(usize, &'s Field<'s>)>,
    /// The (1-based repeat index, repeat) containing the cursor
    pub repeat: Option<(usize, &'s Repeat<'s>)>,
    /// The (1-based component index, component) containing the cursor
    pub component: Option<(usize, &'s Component<'s>)>,
    /// The (1-based sub-component index, sub-component) containing the cursor
    pub sub_component: Option<(usize, &'s Subcomponent<'s>)>,
}

/// Locate a cursor within a message. The offset is the character (distinct from byte) offset
/// within the message.
pub fn locate_cursor<'m>(message: &'m Message<'m>, offset: usize) -> Option<LocatedCursor<'m>> {
    let mut cursor = LocatedCursor {
        message,
        segment: None,
        field: None,
        repeat: None,
        component: None,
        sub_component: None,
    };

    if offset >= message.source.len() {
        return None;
    }
    let mut seg_indices: HashMap<&str, usize> = HashMap::new();
    for seg in message.segments() {
        *seg_indices.entry(seg.name).or_insert(0) += 1;
        // note: inclusive range at both ends to include the segment separator
        if offset >= seg.range.start && offset <= seg.range.end {
            cursor.segment = Some((
                seg.name,
                *seg_indices.get(seg.name).expect("seg exists in hashmap"),
                seg,
            ));
            break;
        }
    }

    cursor.segment?;
    for (i, field) in cursor.segment.as_ref().unwrap().2.fields().enumerate() {
        // note: inclusive range at both ends to include the field separator
        if offset >= field.range.start && offset <= field.range.end {
            cursor.field = Some((i + 1, field));
            break;
        }
    }

    if cursor.field.is_none() {
        return Some(cursor);
    }
    for (i, repeat) in cursor.field.as_ref().unwrap().1.repeats().enumerate() {
        // note: inclusive range at both ends to include the repetition separator
        if offset >= repeat.range.start && offset <= repeat.range.end {
            cursor.repeat = Some((i + 1, repeat));
            break;
        }
    }

    if cursor.repeat.is_none() {
        return Some(cursor);
    }
    for (i, component) in cursor.repeat.as_ref().unwrap().1.components().enumerate() {
        // note: inclusive range at both ends to include the component separator
        if offset >= component.range.start && offset <= component.range.end {
            cursor.component = Some((i + 1, component));
            break;
        }
    }

    if cursor.component.is_none() {
        return Some(cursor);
    }
    for (i, sub_component) in cursor
        .component
        .as_ref()
        .unwrap()
        .1
        .subcomponents()
        .enumerate()
    {
        // note: inclusive range at both ends to include the sub-component separator
        if offset >= sub_component.range.start && offset <= sub_component.range.end {
            cursor.sub_component = Some((i + 1, sub_component));
            break;
        }
    }

    Some(cursor)
}

impl Display for LocatedCursor<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some((seg_name, seg_idx, _)) = self.segment {
            write!(f, "{}", seg_name)?;
            if self.message.segment_count(seg_name) > 1 {
                write!(f, "[{}]", seg_idx)?;
            }
        }
        if let Some((field_idx, field)) = self.field {
            write!(f, ".{}", field_idx)?;
            if let Some((repeat_idx, repeat)) = self.repeat {
                if field.has_repeats() {
                    write!(f, "[{}]", repeat_idx)?;
                }
                if let Some((component_idx, component)) = self.component {
                    if repeat.has_components() {
                        write!(f, ".{}", component_idx)?;
                    }
                    if let Some((sub_component_idx, _)) = self.sub_component {
                        if component.has_subcomponents() {
                            write!(f, ".{}", sub_component_idx)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_locate_cursor_in_segment() {
        let message = Message::parse("MSH|^~\\&|asdf\rPID|1|0").unwrap();
        let cursor = locate_cursor(&message, 0).expect("cursor is located");
        assert_eq!(cursor.segment.unwrap().0, "MSH");
        assert_eq!(cursor.segment.unwrap().1, 1);

        let cursor = locate_cursor(&message, 18).expect("cursor is located");
        assert_eq!(cursor.segment.unwrap().0, "PID");
        assert_eq!(cursor.segment.unwrap().1, 1);
        assert_eq!(cursor.field.unwrap().0, 1);
        assert_eq!(cursor.field.unwrap().1.raw_value(), "1");
    }

    #[test]
    fn can_locate_cursor_in_repeated_segment() {
        let message = Message::parse("MSH|^~\\&|asdf\rPID|1|0\rPID|2|1").unwrap();
        let cursor = locate_cursor(&message, 26).expect("cursor is located");
        assert_eq!(cursor.segment.unwrap().0, "PID");
        assert_eq!(cursor.segment.unwrap().1, 2);
        assert_eq!(cursor.field.unwrap().0, 1);
        assert_eq!(cursor.field.unwrap().1.raw_value(), "2");
    }

    #[test]
    fn can_locate_cursor_at_field_boundaries() {
        let message = Message::parse("MSH|^~\\&|asdf\rPID|1|0").unwrap();
        let cursor = locate_cursor(&message, 19).expect("cursor is located");
        assert_eq!(cursor.segment.unwrap().0, "PID");
        assert_eq!(cursor.segment.unwrap().1, 1);
        assert_eq!(cursor.field.unwrap().0, 1);
        assert_eq!(cursor.field.unwrap().1.raw_value(), "1");

        let message = Message::parse("MSH|^~\\&|asdf\rPID||0").unwrap();
        let cursor = locate_cursor(&message, 18).expect("cursor is located");
        assert_eq!(cursor.segment.unwrap().0, "PID");
        assert_eq!(cursor.segment.unwrap().1, 1);
        assert_eq!(cursor.field.unwrap().0, 1);
        assert_eq!(cursor.field.unwrap().1.raw_value(), "");
    }
}
