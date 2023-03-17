use crate::{
    Component, ComponentAccessor, Field, LocationQuery, ParseError, Repeat, RepeatAccessor,
    Segment, Segments, Separators, SubComponent, SubComponentAccessor,
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

/// A parsed message that owns its string slice. The message structure is valid, but the contents may or may not be.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageBuf {
    /// The original source message, generally used to extract items using ranges
    pub source: String,
    /// The separators & encoding characters defined at the beginning of the MSH segment
    pub separators: Separators,
    /// All the segments stored within the message
    pub segments: HashMap<String, Segments>,
}

impl<'s> From<Message<'s>> for MessageBuf {
    fn from(value: Message<'s>) -> Self {
        let Message {
            source,
            separators,
            segments,
        } = value;
        let source = source.to_string();
        let segments = segments
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();

        MessageBuf {
            source,
            separators,
            segments,
        }
    }
}

/// Results from locating a cursor within a message
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocatedData<'s> {
    /// The (segment ID, segment ID repeat # (0-based), and segment) containing the cursor
    pub segment: Option<(&'s str, usize, &'s Segment)>,
    /// The (1-based field ID, field) containing the cursor
    pub field: Option<(NonZeroUsize, &'s Field)>,
    /// The (1-based repeat ID, repeat) containing the cursor
    pub repeat: Option<(NonZeroUsize, &'s Repeat)>,
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
        let (_, message) = crate::parser::parse_message(crate::parser::Span::new(source))?;
        Ok(message)
    }

    /// Whether the message contains any of the given segment identifier (`MSH`, `PID`, `PV1`, etc)
    pub fn has_segment<S: AsRef<str>>(&'s self, segment: S) -> bool {
        self.segments.contains_key(segment.as_ref())
    }

    /// Access the first segment identified by `segment`
    pub fn segment<S: AsRef<str>>(&'s self, segment: S) -> Option<&'s Segment> {
        self.segments
            .get(segment.as_ref())
            .and_then(|seg| seg.get(0))
    }

    /// Return the number of times segments identified by `segment` are present in the message
    pub fn segment_count<S: AsRef<str>>(&'s self, segment: S) -> usize {
        self.segments
            .get(segment.as_ref())
            .map(|seg| seg.count())
            .unwrap_or_default()
    }

    /// Get the 0-based nth segment identified by `segment` (i.e., if there were two `OBX` segments
    /// and you wanted the second one, call `message.segment_n("OBX", 1)`)
    pub fn segment_n<S: AsRef<str>>(&'s self, segment: S, n: usize) -> Option<&'s Segment> {
        self.segments
            .get(segment.as_ref())
            .and_then(|seg| seg.get(n))
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
    pub fn get_field_source<S: AsRef<str>>(
        &'s self,
        segment: (S, usize),
        field: NonZeroUsize,
    ) -> Option<&'s str> {
        let Some(seg) = self.segment_n(segment.0, segment.1) else {
            return None;
        };

        seg.field(field).map(|f| f.source(self.source))
    }

    /// Directly get the source (not yet decoded) for a given field and repeat, if it exists in the message. The
    /// field is identified by the segment identifier, segment repeat identifier, 1-based field identifier, and the repeat identifier
    ///
    /// # Examples
    ///
    /// ```
    /// # use hl7_parser::Message;
    /// # use std::num::NonZeroUsize;
    /// let message = include_str!("../test_assets/sample_adt_a04.hl7")
    ///     .replace("\r\n", "\r")
    ///     .replace('\n', "\r");
    /// let message = Message::parse(&message).expect("can parse message");
    ///
    /// let allergy_reaction_2 = message.get_repeat_source(
    ///     ("AL1", 0),
    ///     NonZeroUsize::new(5).unwrap(),
    ///     NonZeroUsize::new(2).unwrap());
    /// assert_eq!(allergy_reaction_2.unwrap(), "RASH");
    /// ```
    pub fn get_repeat_source<S: AsRef<str>>(
        &'s self,
        segment: (S, usize),
        field: NonZeroUsize,
        repeat: NonZeroUsize,
    ) -> Option<&'s str> {
        let Some(seg) = self.segment_n(segment.0, segment.1) else {
            return None;
        };

        seg.field(field)
            .repeat(repeat)
            .map(|r| r.source(self.source))
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
    ///     NonZeroUsize::new(1).unwrap(),
    ///     NonZeroUsize::new(2).unwrap());
    /// assert_eq!(trigger_event.unwrap(), "A01");
    /// ```
    pub fn get_component_source<S: AsRef<str>>(
        &'s self,
        segment: (S, usize),
        field: NonZeroUsize,
        repeat: NonZeroUsize,
        component: NonZeroUsize,
    ) -> Option<&'s str> {
        let Some(seg) = self.segment_n(segment.0, segment.1) else {
            return None;
        };

        seg.field(field)
            .repeat(repeat)
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
    ///     NonZeroUsize::new(1).unwrap(),
    ///     NonZeroUsize::new(4).unwrap(),
    ///     NonZeroUsize::new(2).unwrap());
    /// assert_eq!(universal_id.unwrap(), "1.2.840.114398.1.100");
    /// ```
    pub fn get_sub_component_source<S: AsRef<str>>(
        &'s self,
        segment: (S, usize),
        field: NonZeroUsize,
        repeat: NonZeroUsize,
        component: NonZeroUsize,
        sub_component: NonZeroUsize,
    ) -> Option<&'s str> {
        let Some(seg) = self.segment_n(segment.0, segment.1) else {
            return None;
        };

        seg.field(field)
            .repeat(repeat)
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
        let field = segment.and_then(|(_, _, segment)| segment.field_at_cursor(cursor));
        let repeat = field.and_then(|(_, field)| field.repeat_at_cursor(cursor));
        let component = repeat.and_then(|(_, repeat)| repeat.component_at_cursor(cursor));
        let sub_component =
            component.and_then(|(_, component)| component.sub_component_at_cursor(cursor));
        LocatedData {
            segment,
            field,
            repeat,
            component,
            sub_component,
        }
    }

    /// Query the message for a given segment, field, component, or sub-comonent.
    ///
    /// # Arguments
    ///
    /// * `query` - a [LocationQuery] targeting you want to access
    ///
    /// # Returns
    ///
    /// * [Result::Err] if the location query couldn't be parsed
    /// * [Result::Ok] if the item location query could be parsed message
    ///   + [Option::Some] containing the item source if the queried item was found in the message
    ///   + [Option::None] if the queried item was _not_ found in the message
    /// message
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
    /// let trigger_event = message.query("MSH.9.2").expect("can parse location query");
    /// assert_eq!(trigger_event, Some("A01"));
    /// ```
    pub fn query<Q, QErr>(&'s self, query: Q) -> Result<Option<&'s str>, QErr>
    where
        Q: TryInto<LocationQuery, Error = QErr>,
    {
        let LocationQuery {
            segment,
            field,
            repeat,
            component,
            sub_component,
        } = query.try_into()?;

        let repeat = if repeat.is_none() && component.is_some() || sub_component.is_some() {
            Some(NonZeroUsize::new(1).unwrap())
        } else {
            repeat
        };

        Ok(match (field, repeat, component, sub_component) {
            (Some(f), Some(r), Some(c), Some(s)) => {
                self.get_sub_component_source((segment, 0), f, r, c, s)
            }
            (Some(f), Some(r), Some(c), _) => self.get_component_source((segment, 0), f, r, c),
            (Some(f), Some(r), _, _) => self.get_repeat_source((segment, 0), f, r),
            (Some(f), _, _, _) => self.get_field_source((segment, 0), f),
            _ => self.segment(segment).map(|seg| seg.source(self.source)),
        })
    }
}

impl MessageBuf {
    /// Parse a string to obtain the underlying message
    pub fn parse<'s, S: ToString + 's>(source: S) -> Result<MessageBuf, ParseError> {
        let source = source.to_string();
        let (_, message) = crate::parser::parse_message(crate::parser::Span::new(&source))?;
        Ok(message.into())
    }

    /// Whether the message contains any of the given segment identifier (`MSH`, `PID`, `PV1`, etc)
    pub fn has_segment<S: AsRef<str>>(&self, segment: S) -> bool {
        self.segments.contains_key(segment.as_ref())
    }

    /// Access the first segment identified by `segment`
    pub fn segment<S: AsRef<str>>(&self, segment: S) -> Option<&Segment> {
        self.segments
            .get(segment.as_ref())
            .and_then(|seg| seg.get(0))
    }

    /// Return the number of times segments identified by `segment` are present in the message
    pub fn segment_count<S: AsRef<str>>(&self, segment: S) -> usize {
        self.segments
            .get(segment.as_ref())
            .map(|seg| seg.count())
            .unwrap_or_default()
    }

    /// Get the 0-based nth segment identified by `segment` (i.e., if there were two `OBX` segments
    /// and you wanted the second one, call `message.segment_n("OBX", 1)`)
    pub fn segment_n<S: AsRef<str>>(&self, segment: S, n: usize) -> Option<&Segment> {
        self.segments
            .get(segment.as_ref())
            .and_then(|seg| seg.get(n))
    }

    /// Directly get the source (not yet decoded) for a given field, if it exists in the message. The
    /// field is identified by the segment identifier, segment repeat identifier, and 1-based field
    /// identifier.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hl7_parser::MessageBuf;
    /// # use std::num::NonZeroUsize;
    /// let message = include_str!("../test_assets/sample_adt_a01.hl7")
    ///     .replace("\r\n", "\r")
    ///     .replace('\n', "\r");
    /// let message = MessageBuf::parse(message).expect("can parse message");
    ///
    /// let message_type = message.get_field_source(("MSH", 0), NonZeroUsize::new(9).unwrap());
    /// assert_eq!(message_type.unwrap(), "ADT^A01");
    /// ```
    pub fn get_field_source<S: AsRef<str>>(
        &self,
        segment: (S, usize),
        field: NonZeroUsize,
    ) -> Option<&str> {
        let Some(seg) = self.segment_n(segment.0, segment.1) else {
            return None;
        };

        seg.field(field).map(|f| f.source(self.source.as_str()))
    }

    /// Directly get the source (not yet decoded) for a given field and repeat, if it exists in the message. The
    /// field is identified by the segment identifier, segment repeat identifier, 1-based field identifier, and the repeat identifier
    ///
    /// # Examples
    ///
    /// ```
    /// # use hl7_parser::MessageBuf;
    /// # use std::num::NonZeroUsize;
    /// let message = include_str!("../test_assets/sample_adt_a04.hl7")
    ///     .replace("\r\n", "\r")
    ///     .replace('\n', "\r");
    /// let message = MessageBuf::parse(message).expect("can parse message");
    ///
    /// let allergy_reaction_2 = message.get_repeat_source(
    ///     ("AL1", 0),
    ///     NonZeroUsize::new(5).unwrap(),
    ///     NonZeroUsize::new(2).unwrap());
    /// assert_eq!(allergy_reaction_2.unwrap(), "RASH");
    /// ```
    pub fn get_repeat_source<S: AsRef<str>>(
        &self,
        segment: (S, usize),
        field: NonZeroUsize,
        repeat: NonZeroUsize,
    ) -> Option<&str> {
        let Some(seg) = self.segment_n(segment.0, segment.1) else {
            return None;
        };

        seg.field(field)
            .repeat(repeat)
            .map(|r| r.source(self.source.as_str()))
    }

    /// Directly get the source (not yet decoded) for a given component, if it exists in the message. The
    /// component is identified by the segment identifier, segment repeat identifier, 1-based field
    /// identifier, and 1-based component identifier
    ///
    /// # Examples
    ///
    /// ```
    /// # use hl7_parser::MessageBuf;
    /// # use std::num::NonZeroUsize;
    /// let message = include_str!("../test_assets/sample_adt_a01.hl7")
    ///     .replace("\r\n", "\r")
    ///     .replace('\n', "\r");
    /// let message = MessageBuf::parse(message).expect("can parse message");
    ///
    /// let trigger_event = message.get_component_source(
    ///     ("MSH", 0),
    ///     NonZeroUsize::new(9).unwrap(),
    ///     NonZeroUsize::new(1).unwrap(),
    ///     NonZeroUsize::new(2).unwrap());
    /// assert_eq!(trigger_event.unwrap(), "A01");
    /// ```
    pub fn get_component_source<S: AsRef<str>>(
        &self,
        segment: (S, usize),
        field: NonZeroUsize,
        repeat: NonZeroUsize,
        component: NonZeroUsize,
    ) -> Option<&str> {
        let Some(seg) = self.segment_n(segment.0, segment.1) else {
            return None;
        };

        seg.field(field)
            .repeat(repeat)
            .component(component)
            .map(|c| c.source(self.source.as_str()))
    }

    /// Directly get the source (not yet decoded) for a given sub-component, if it exists in the message.
    /// The component is identified by the segment identifier, segment repeat identifier, 1-based field
    /// identifier, 1-based component identifier, and 1-based sub-component identifier
    ///
    /// # Examples
    ///
    /// ```
    /// # use hl7_parser::MessageBuf;
    /// # use std::num::NonZeroUsize;
    /// let message = include_str!("../test_assets/sample_oru_r01_generic.hl7")
    ///     .replace("\r\n", "\r")
    ///     .replace('\n', "\r");
    /// let message = MessageBuf::parse(message).expect("can parse message");
    ///
    /// let universal_id = message.get_sub_component_source(
    ///     ("PID", 0),
    ///     NonZeroUsize::new(3).unwrap(),
    ///     NonZeroUsize::new(1).unwrap(),
    ///     NonZeroUsize::new(4).unwrap(),
    ///     NonZeroUsize::new(2).unwrap());
    /// assert_eq!(universal_id.unwrap(), "1.2.840.114398.1.100");
    /// ```
    pub fn get_sub_component_source<S: AsRef<str>>(
        &self,
        segment: (S, usize),
        field: NonZeroUsize,
        repeat: NonZeroUsize,
        component: NonZeroUsize,
        sub_component: NonZeroUsize,
    ) -> Option<&str> {
        let Some(seg) = self.segment_n(segment.0, segment.1) else {
            return None;
        };

        seg.field(field)
            .repeat(repeat)
            .component(component)
            .sub_component(sub_component)
            .map(|s| s.source(self.source.as_str()))
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
    pub fn segment_at_cursor(&self, cursor: usize) -> Option<(&str, usize, &Segment)> {
        self.segments.iter().find_map(|(id, segs)| {
            segs.segment_at_cursor(cursor)
                .map(|(n, seg)| (id.as_str(), n, seg))
        })
    }

    /// Deeply locate the cursor by returning the sub-component, component, field, and segment that
    /// the cursor is located in (if any)
    pub fn locate_cursor(&self, cursor: usize) -> LocatedData {
        let segment = self.segment_at_cursor(cursor);
        let field = segment.and_then(|(_, _, segment)| segment.field_at_cursor(cursor));
        let repeat = field.and_then(|(_, field)| field.repeat_at_cursor(cursor));
        let component = repeat.and_then(|(_, repeat)| repeat.component_at_cursor(cursor));
        let sub_component =
            component.and_then(|(_, component)| component.sub_component_at_cursor(cursor));
        LocatedData {
            segment,
            field,
            repeat,
            component,
            sub_component,
        }
    }

    /// Query the message for a given segment, field, component, or sub-comonent.
    ///
    /// # Arguments
    ///
    /// * `query` - a [LocationQuery] targeting you want to access
    ///
    /// # Returns
    ///
    /// * [Result::Err] if the location query couldn't be parsed
    /// * [Result::Ok] if the item location query could be parsed message
    ///   + [Option::Some] containing the item source if the queried item was found in the message
    ///   + [Option::None] if the queried item was _not_ found in the message
    ///
    /// # Examples
    ///
    /// ```
    /// # use hl7_parser::MessageBuf;
    /// # use std::num::NonZeroUsize;
    /// let message = include_str!("../test_assets/sample_adt_a01.hl7")
    ///     .replace("\r\n", "\r")
    ///     .replace('\n', "\r");
    /// let message = MessageBuf::parse(message).expect("can parse message");
    ///
    /// let trigger_event = message.query("MSH.9.2").expect("can parse location query");
    /// assert_eq!(trigger_event, Some("A01"));
    /// ```
    pub fn query<Q, QErr>(&self, query: Q) -> Result<Option<&str>, QErr>
    where
        Q: TryInto<LocationQuery, Error = QErr>,
    {
        let LocationQuery {
            segment,
            field,
            repeat,
            component,
            sub_component,
        } = query.try_into()?;

        let repeat = if repeat.is_none() && component.is_some() || sub_component.is_some() {
            Some(NonZeroUsize::new(1).unwrap())
        } else {
            repeat
        };

        Ok(match (field, repeat, component, sub_component) {
            (Some(f), Some(r), Some(c), Some(s)) => {
                self.get_sub_component_source((segment, 0), f, r, c, s)
            }
            (Some(f), Some(r), Some(c), _) => self.get_component_source((segment, 0), f, r, c),
            (Some(f), Some(r), _, _) => self.get_repeat_source((segment, 0), f, r),
            (Some(f), _, _, _) => self.get_field_source((segment, 0), f),
            _ => self
                .segment(segment)
                .map(|seg| seg.source(self.source.as_str())),
        })
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
        assert_eq!(n, 1);

        let (n, field) = seg
            .field_at_cursor(cursor)
            .expect("can get field at cursor");
        assert_eq!(n, NonZeroUsize::new(5).unwrap());

        let (n, repeat) = field
            .repeat_at_cursor(cursor)
            .expect("can get repeat at cursor");
        assert_eq!(n.get(), 1);

        let (n, component) = repeat
            .component_at_cursor(cursor)
            .expect("can get component at cursor");
        assert_eq!(n, NonZeroUsize::new(3).unwrap());
        assert_eq!(component.source(message.source), "HOLLYWOOD");
    }

    #[test]
    fn can_locate_cursor_at_empty_fields() {
        let message = include_str!("../test_assets/sample_adt_a01.hl7")
            .replace("\r\n", "\r")
            .replace('\n', "\r");
        let message = Message::parse(message.as_str()).expect("can parse message");
        for field in message.segment("MSH").unwrap().fields.iter() {
            eprintln!("Field: {field:?} source: {}", field.source(message.source));
        }
        let location = message.locate_cursor(19);
        assert!(location.segment.is_some());
        assert!(location.field.is_some());
        assert!(location.component.is_none());
        assert!(location.sub_component.is_none());
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

    #[test]
    fn can_create_owned_version() {
        let raw_message = include_str!("../test_assets/sample_adt_a01.hl7")
            .replace("\r\n", "\r")
            .replace('\n', "\r");

        let message = Message::parse(raw_message.as_str()).expect("can parse message");
        let message_from = MessageBuf::from(message);

        let message_direct = MessageBuf::parse(raw_message).expect("can parse message");

        assert_eq!(message_from, message_direct);
    }

    #[test]
    fn has_a_not_terrible_error_message() {
        assert_eq!(
            Message::parse("MSH|^~\\&$")
                .expect_err("Message parsing to fail")
                .to_string()
                .as_str(),
            "Message parsing failed at position 8 (line 1 column 9): `$`"
        );
    }

    #[test]
    fn message_and_message_buf_have_the_same_errors() {
        let err = Message::parse("MSH|^~\\&$").expect_err("Message parsing to fail");
        let err_buf = MessageBuf::parse("MSH|^~\\&$").expect_err("Message parsing to fail");
        assert_eq!(err, err_buf);
        assert_eq!(err.to_string(), err_buf.to_string());
    }
}
