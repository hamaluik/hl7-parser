use crate::{
    Component, ComponentAccessor, Field, ParseError, Segment, Segments, Separators, SubComponent,
    SubComponentAccessor,
};
use std::{collections::HashMap, num::NonZeroUsize};

/// A parsed message. The message structure is valid, but the contents may or may not be.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message<'s> {
    /// The original source message, generally used to extract items using ranges
    pub source: &'s str,
    /// The separators & encoding characters defined at the beginning of the MSH segment
    pub separators: Separators,
    /// All the segments stored within the message
    pub segments: HashMap<&'s str, Segments>,
}

/// Results from locating a cursor within a message
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocatedData<'s> {
    /// The (segment ID, segment ID repeat # (0-based), and segment) containing the cursor
    pub segment: Option<(&'s str, usize, &'s Segment)>,
    /// The (1-based field ID, field) containing the cursor
    pub field: Option<(NonZeroUsize, &'s Field)>,
    /// The (1-based component ID, component) containing the cursor
    pub component: Option<(NonZeroUsize, &'s Component)>,
    /// The (1-based sub-component ID, sub-component) containing the cursor
    pub sub_component: Option<(NonZeroUsize, &'s SubComponent)>,
}

impl<'s> std::fmt::Display for LocatedData<'s> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(segment) = self.segment {
            write!(f, "{}", segment.0)?;
        } else {
            return Ok(());
        }
        if let Some(field) = self.field {
            write!(f, ".{}", field.0)?;
        } else {
            return Ok(());
        }
        if let Some(component) = self.component {
            write!(f, ".{}", component.0)?;
        } else {
            return Ok(());
        }
        if let Some(sub_component) = self.sub_component {
            write!(f, ".{}", sub_component.0)?;
        } else {
            return Ok(());
        }
        Ok(())
    }
}

impl<'s> Message<'s> {
    /// Parse a string to obtain the underlying message
    pub fn parse(source: &'s str) -> Result<Message<'s>, ParseError> {
        let (_, message) =
            crate::parser::parse_message(crate::parser::Span::new(source)).map_err(|e| {
                if cfg!(debug_assertions) {
                    // TODO: better error messages
                    eprintln!("Parse error: {e:#?}");
                }
                ParseError::Failed
            })?;
        Ok(message)
    }

    /// Whether the message contains any of the given segment identifier (`MSH`, `PID`, `PV1`, etc)
    pub fn has_segment(&'s self, segment: &str) -> bool {
        self.segments.contains_key(segment)
    }

    /// Access the first segment identified by `segment`
    pub fn segment(&'s self, segment: &str) -> Option<&'s Segment> {
        self.segments.get(segment).map(|seg| seg.get(0)).flatten()
    }

    /// Return the number of times segments identified by `segment` are present in the message
    pub fn segment_count(&'s self, segment: &str) -> usize {
        self.segments
            .get(segment)
            .map(|seg| seg.count())
            .unwrap_or_default()
    }

    /// Get the 0-based nth segment identified by `segment` (i.e., if there were two `OBX` segments
    /// and you wanted the second one, call `message.segment_n("OBX", 1)`)
    pub fn segment_n(&'s self, segment: &str, n: usize) -> Option<&'s Segment> {
        self.segments.get(segment).map(|seg| seg.get(n)).flatten()
    }

    /// Directly get the source (not yet decoded) for a given field, if it exists in the message. The
    /// field is identified by the segment identifier, segment repeat identifier, and 1-based field
    /// identifier.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hl7_parser::Message;
    /// # use std::num::NonZeroUsize;
    /// let message = include_str!("../test_assets/sample_adt_a01.hl7")
    ///     .replace("\r\n", "\r")
    ///     .replace('\n', "\r");
    /// let message = Message::parse(&message).expect("can parse message");
    ///
    /// let message_type = message.get_field_source(("MSH", 0), NonZeroUsize::new(9).unwrap());
    /// assert_eq!(message_type.unwrap(), "ADT^A01");
    /// ```
    pub fn get_field_source(
        &'s self,
        segment: (&str, usize),
        field: NonZeroUsize,
    ) -> Option<&'s str> {
        let Some(seg) = self.segment_n(segment.0, segment.1) else {
            return None;
        };

        seg.field(field).map(|f| f.source(self.source))
    }

    /// Directly get the source (not yet decoded) for a given component, if it exists in the message. The
    /// component is identified by the segment identifier, segment repeat identifier, 1-based field
    /// identifier, and 1-based component identifier
    ///
    /// # Examples
    ///
    /// ```
    /// # use hl7_parser::Message;
    /// # use std::num::NonZeroUsize;
    /// let message = include_str!("../test_assets/sample_adt_a01.hl7")
    ///     .replace("\r\n", "\r")
    ///     .replace('\n', "\r");
    /// let message = Message::parse(&message).expect("can parse message");
    ///
    /// let trigger_event = message.get_component_source(
    ///     ("MSH", 0),
    ///     NonZeroUsize::new(9).unwrap(),
    ///     NonZeroUsize::new(2).unwrap());
    /// assert_eq!(trigger_event.unwrap(), "A01");
    /// ```
    pub fn get_component_source(
        &'s self,
        segment: (&str, usize),
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

    /// Directly get the source (not yet decoded) for a given sub-component, if it exists in the message.
    /// The component is identified by the segment identifier, segment repeat identifier, 1-based field
    /// identifier, 1-based component identifier, and 1-based sub-component identifier
    ///
    /// # Examples
    ///
    /// ```
    /// # use hl7_parser::Message;
    /// # use std::num::NonZeroUsize;
    /// let message = include_str!("../test_assets/sample_oru_r01_generic.hl7")
    ///     .replace("\r\n", "\r")
    ///     .replace('\n', "\r");
    /// let message = Message::parse(&message).expect("can parse message");
    ///
    /// let universal_id = message.get_sub_component_source(
    ///     ("PID", 0),
    ///     NonZeroUsize::new(3).unwrap(),
    ///     NonZeroUsize::new(4).unwrap(),
    ///     NonZeroUsize::new(2).unwrap());
    /// assert_eq!(universal_id.unwrap(), "1.2.840.114398.1.100");
    /// ```
    pub fn get_sub_component_source(
        &'s self,
        segment: (&str, usize),
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

    /// Locate a segment at the cursor position
    ///
    /// # Arguments
    ///
    /// * `cursor` - The cursor location (0-based character index of the original message)
    ///
    /// # Returns
    ///
    /// A tuple containing the HL7 segment identifier, 0-based segment repeat number and a
    /// reference to the field. If the segment doesn't contain the cursor, returns `None`
    pub fn segment_at_cursor(&'s self, cursor: usize) -> Option<(&'s str, usize, &'s Segment)> {
        self.segments
            .iter()
            .find_map(|(id, segs)| segs.segment_at_cursor(cursor).map(|(n, seg)| (*id, n, seg)))
    }

    /// Deeply locate the cursor by returning the sub-component, component, field, and segment that
    /// the cursor is located in (if any)
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
        assert_eq!(n, 0);

        let (n, _field) = seg
            .field_at_cursor(cursor)
            .expect("can get field at cursor");
        assert_eq!(n, NonZeroUsize::new(7).unwrap());

        let cursor = 0x458;
        let (id, n, seg) = message
            .segment_at_cursor(cursor)
            .expect("can get segment at cursor");
        assert_eq!(id, "IN1");
        assert_eq!(n, 0);

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

    #[test]
    fn can_display_hl7_path() {
        let cursor = 0x458;
        let message = include_str!("../test_assets/sample_adt_a01.hl7")
            .replace("\r\n", "\r")
            .replace('\n', "\r");
        let message = Message::parse(message.as_str()).expect("can parse message");
        let location = message.locate_cursor(cursor);
        let location = format!("{location}");
        assert_eq!(location, "IN1.5.3.1");
    }
}
