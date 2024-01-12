use indexmap::IndexMap;

use crate::{
    Component, ComponentAccessor, Field, LocationQuery, ParseError, Repeat, RepeatAccessor,
    Segment, Segments, Separators, SubComponent, SubComponentAccessor,
};
use std::{num::NonZeroUsize, ops::Range};

/// A parsed message. The message structure is valid, but the contents may or may not be.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ParsedMessage<'s> {
    /// The original source message, generally used to extract items using ranges
    pub source: &'s str,
    /// The separators & encoding characters defined at the beginning of the MSH segment
    pub separators: Separators,
    /// All the segments stored within the message
    pub segments: IndexMap<&'s str, Segments>,
}

/// A parsed message that owns its string slice. The message structure is valid, but the contents may or may not be.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParsedMessageOwned {
    /// The original source message, generally used to extract items using ranges
    pub source: String,
    /// The separators & encoding characters defined at the beginning of the MSH segment
    pub separators: Separators,
    /// All the segments stored within the message
    pub segments: IndexMap<String, Segments>,
}

impl<'s> From<ParsedMessage<'s>> for ParsedMessageOwned {
    fn from(value: ParsedMessage<'s>) -> Self {
        let ParsedMessage {
            source,
            separators,
            segments,
        } = value;
        let source = source.to_string();
        let segments = segments
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();

        ParsedMessageOwned {
            source,
            separators,
            segments,
        }
    }
}

/// Results from locating a cursor within a message
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
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
        if let Some(repeat) = self.repeat {
            write!(f, "[{}]", repeat.0)?;
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

impl<'s> ParsedMessage<'s> {
    /// Parse a string to obtain the underlying message
    pub fn parse(source: &'s str, lenient_segment_separators: bool) -> Result<ParsedMessage<'s>, ParseError> {
        let (_, message) = crate::parser::parse_message(crate::parser::Span::new(source), lenient_segment_separators)?;
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
            .map(|seg| seg.len())
            .unwrap_or_default()
    }

    /// Get the 0-based nth segment identified by `segment` (i.e., if there were two `OBX` segments
    /// and you wanted the second one, call `message.segment_n("OBX", 1)`)
    pub fn segment_n<S: AsRef<str>>(&'s self, segment: S, n: usize) -> Option<&'s Segment> {
        self.segments
            .get(segment.as_ref())
            .and_then(|seg| seg.get(n))
    }

    /// Mutable access to the 0-based nth segment identified by `segment` (i.e., if there were two `OBX` Segments
    /// and you wanted the second one, call `message.segment_n_mut("OBX", 1)`)
    pub fn segment_n_mut<S: AsRef<str>>(&mut self, segment: S, n: usize) -> Option<&mut Segment> {
        self.segments
            .get_mut(segment.as_ref())
            .and_then(|seg| seg.get_mut(n))
    }

    /// Directly get the source (not yet decoded) for a given field, if it exists in the message. The
    /// field is identified by the segment identifier, segment repeat identifier, and 1-based field
    /// identifier.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hl7_parser::ParsedMessage;
    /// # use std::num::NonZeroUsize;
    /// let message = include_str!("../test_assets/sample_adt_a01.hl7");
    /// let message = ParsedMessage::parse(&message, true).expect("can parse message");
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

    /// Get the field for a given field, if it exists in the message. The field
    /// is identified by the segment identifier, segment repeat identifier, and 1-based field
    /// identifier.
    pub fn get_field<S: AsRef<str>>(
        &'s self,
        segment: (S, usize),
        field: NonZeroUsize,
    ) -> Option<&'s Field> {
        let Some(seg) = self.segment_n(segment.0, segment.1) else {
            return None;
        };

        seg.field(field)
    }

    /// Directly get the source (not yet decoded) for a given field and repeat, if it exists in the message. The
    /// field is identified by the segment identifier, segment repeat identifier, 1-based field identifier, and the repeat identifier
    ///
    /// # Examples
    ///
    /// ```
    /// # use hl7_parser::ParsedMessage;
    /// # use std::num::NonZeroUsize;
    /// let message = include_str!("../test_assets/sample_adt_a04.hl7");
    /// let message = ParsedMessage::parse(&message, true).expect("can parse message");
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

    /// Directly get the repeat for a given field and repeat, if it exists in
    /// the message. The field is identified by the segment identifier, segment
    /// repeat identifier, 1-based field identifier, and the repeat identifier
    pub fn get_repeat<S: AsRef<str>>(
        &'s self,
        segment: (S, usize),
        field: NonZeroUsize,
        repeat: NonZeroUsize,
    ) -> Option<&'s Repeat> {
        let Some(seg) = self.segment_n(segment.0, segment.1) else {
            return None;
        };

        seg.field(field).and_then(|f| f.repeat(repeat))
    }

    /// Directly get the source (not yet decoded) for a given component, if it exists in the message. The
    /// component is identified by the segment identifier, segment repeat identifier, 1-based field
    /// identifier, and 1-based component identifier
    ///
    /// # Examples
    ///
    /// ```
    /// # use hl7_parser::ParsedMessage;
    /// # use std::num::NonZeroUsize;
    /// let message = include_str!("../test_assets/sample_adt_a01.hl7");
    /// let message = ParsedMessage::parse(&message, true).expect("can parse message");
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

    /// Directly get the component for a given component, if it exists in the message. The
    /// component is identified by the segment identifier, segment repeat identifier, 1-based field
    /// identifier, and 1-based component identifier
    pub fn get_component<S: AsRef<str>>(
        &'s self,
        segment: (S, usize),
        field: NonZeroUsize,
        repeat: NonZeroUsize,
        component: NonZeroUsize,
    ) -> Option<&'s Component> {
        let Some(seg) = self.segment_n(segment.0, segment.1) else {
            return None;
        };

        seg.field(field)
            .and_then(|f| f.repeat(repeat))
            .and_then(|r| r.component(component))
    }

    /// Directly get the source (not yet decoded) for a given sub-component, if it exists in the message.
    /// The component is identified by the segment identifier, segment repeat identifier, 1-based field
    /// identifier, 1-based component identifier, and 1-based sub-component identifier
    ///
    /// # Examples
    ///
    /// ```
    /// # use hl7_parser::ParsedMessage;
    /// # use std::num::NonZeroUsize;
    /// let message = include_str!("../test_assets/sample_oru_r01_generic.hl7");
    /// let message = ParsedMessage::parse(&message, true).expect("can parse message");
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

    /// Directly get the sub-component for a given sub-component, if it exists in the message.
    /// The component is identified by the segment identifier, segment repeat identifier, 1-based field
    /// identifier, 1-based component identifier, and 1-based sub-component identifier
    pub fn get_sub_component<S: AsRef<str>>(
        &'s self,
        segment: (S, usize),
        field: NonZeroUsize,
        repeat: NonZeroUsize,
        component: NonZeroUsize,
        sub_component: NonZeroUsize,
    ) -> Option<&'s SubComponent> {
        let Some(seg) = self.segment_n(segment.0, segment.1) else {
            return None;
        };

        seg.field(field)
            .and_then(|f| f.repeat(repeat))
            .and_then(|r| r.component(component))
            .and_then(|c| c.sub_component(sub_component))
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
        let multi_repeats = field.map(|(_, f)| f.repeats.len() > 1).unwrap_or_default();
        let repeat = field.and_then(|(_, field)| field.repeat_at_cursor(cursor));
        let component = repeat.and_then(|(_, repeat)| repeat.component_at_cursor(cursor));
        let sub_component =
            component.and_then(|(_, component)| component.sub_component_at_cursor(cursor));
        LocatedData {
            segment,
            field,
            repeat: if repeat.is_some() && multi_repeats {
                repeat
            } else {
                None
            },
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
    /// # use hl7_parser::ParsedMessage;
    /// # use std::num::NonZeroUsize;
    /// let message = include_str!("../test_assets/sample_adt_a01.hl7");
    /// let message = ParsedMessage::parse(&message, true).expect("can parse message");
    ///
    /// let trigger_event = message.query_value("MSH.9.2").expect("can parse location query");
    /// assert_eq!(trigger_event, Some("A01"));
    /// ```
    pub fn query_value<Q, QErr>(&'s self, query: Q) -> Result<Option<&'s str>, QErr>
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

    /// Query the message for a given segment, field, component, or sub-comonent,
    /// returning the range in the source string that the item occupies.
    ///
    /// # Arguments
    ///
    /// * `query` - a [LocationQuery] targeting you want to access
    ///
    /// # Returns
    ///
    /// * [Result::Err] if the location query couldn't be parsed
    /// * [Result::Ok] if the item location query could be parsed message
    ///   + [Option::Some] containing the range in the source if the queried item was found in the message
    ///   + [Option::None] if the queried item was _not_ found in the message
    /// message
    ///
    /// # Examples
    ///
    /// ```
    /// # use hl7_parser::ParsedMessage;
    /// # use std::num::NonZeroUsize;
    /// let message = include_str!("../test_assets/sample_adt_a01.hl7");
    /// let message = ParsedMessage::parse(&message, true).expect("can parse message");
    ///
    /// let trigger_event = message.query("MSH.9.2").expect("can parse location query");
    /// assert_eq!(trigger_event, Some(40..43));
    /// ```
    pub fn query<Q, QErr>(&self, query: Q) -> Result<Option<Range<usize>>, QErr>
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
            (Some(f), Some(r), Some(c), Some(s)) => self
                .get_sub_component((segment, 0), f, r, c, s)
                .map(|s| s.range.clone()),
            (Some(f), Some(r), Some(c), _) => self
                .get_component((segment, 0), f, r, c)
                .map(|c| c.range.clone()),
            (Some(f), Some(r), _, _) => {
                self.get_repeat((segment, 0), f, r).map(|r| r.range.clone())
            }
            (Some(f), _, _, _) => self.get_field((segment, 0), f).map(|f| f.range.clone()),
            _ => self.segment(segment).map(|seg| seg.range.clone()),
        })
    }
}

impl ParsedMessageOwned {
    /// Parse a string to obtain the underlying message
    pub fn parse<'s, S: ToString + 's>(source: S, lenient_segment_separators: bool) -> Result<ParsedMessageOwned, ParseError> {
        let source = source.to_string();
        let (_, message) = crate::parser::parse_message(crate::parser::Span::new(&source), lenient_segment_separators)?;
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

    /// Mutable access to the first segment identified by `segment`
    pub fn segment_mut<S: AsRef<str>>(&mut self, segment: S) -> Option<&mut Segment> {
        self.segments
            .get_mut(segment.as_ref())
            .and_then(|seg| seg.get_mut(0))
    }

    /// Return the number of times segments identified by `segment` are present in the message
    pub fn segment_count<S: AsRef<str>>(&self, segment: S) -> usize {
        self.segments
            .get(segment.as_ref())
            .map(|seg| seg.len())
            .unwrap_or_default()
    }

    /// Get the 0-based nth segment identified by `segment` (i.e., if there were two `OBX` segments
    /// and you wanted the second one, call `message.segment_n("OBX", 1)`)
    pub fn segment_n<S: AsRef<str>>(&self, segment: S, n: usize) -> Option<&Segment> {
        self.segments
            .get(segment.as_ref())
            .and_then(|seg| seg.get(n))
    }

    /// Mutable access to the 0-based nth segment identified by `segment` (i.e., if there were two `OBX` Segments
    /// and you wanted the second one, call `message.segment_n_mut("OBX", 1)`)
    pub fn segment_n_mut<S: AsRef<str>>(&mut self, segment: S, n: usize) -> Option<&mut Segment> {
        self.segments
            .get_mut(segment.as_ref())
            .and_then(|seg| seg.get_mut(n))
    }

    /// Directly get the source (not yet decoded) for a given field, if it exists in the message. The
    /// field is identified by the segment identifier, segment repeat identifier, and 1-based field
    /// identifier.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hl7_parser::ParsedMessageOwned;
    /// # use std::num::NonZeroUsize;
    /// let message = include_str!("../test_assets/sample_adt_a01.hl7");
    /// let message = ParsedMessageOwned::parse(&message, true).expect("can parse message");
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

    /// Get the field for a given field, if it exists in the message. The field
    /// is identified by the segment identifier, segment repeat identifier, and 1-based field
    /// identifier.
    pub fn get_field<S: AsRef<str>>(
        &self,
        segment: (S, usize),
        field: NonZeroUsize,
    ) -> Option<&Field> {
        let Some(seg) = self.segment_n(segment.0, segment.1) else {
            return None;
        };

        seg.field(field)
    }

    /// Get a mutable reference to a field for a given field, if it exists in the message. The field
    /// is identified by the segment identifier, segment repeat identifier, and 1-based field
    /// identifier.
    pub fn get_field_mut<S: AsRef<str>>(
        &mut self,
        segment: (S, usize),
        field: NonZeroUsize,
    ) -> Option<&mut Field> {
        let Some(seg) = self.segment_n_mut(segment.0, segment.1) else {
            return None;
        };

        seg.field_mut(field)
    }

    /// Directly get the source (not yet decoded) for a given field and repeat, if it exists in the message. The
    /// field is identified by the segment identifier, segment repeat identifier, 1-based field identifier, and the repeat identifier
    ///
    /// # Examples
    ///
    /// ```
    /// # use hl7_parser::ParsedMessageOwned;
    /// # use std::num::NonZeroUsize;
    /// let message = include_str!("../test_assets/sample_adt_a04.hl7");
    /// let message = ParsedMessageOwned::parse(&message, true).expect("can parse message");
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

    /// Directly get the repeat for a given field and repeat, if it exists in
    /// the message. The field is identified by the segment identifier, segment
    /// repeat identifier, 1-based field identifier, and the repeat identifier
    pub fn get_repeat<S: AsRef<str>>(
        &self,
        segment: (S, usize),
        field: NonZeroUsize,
        repeat: NonZeroUsize,
    ) -> Option<&Repeat> {
        let Some(seg) = self.segment_n(segment.0, segment.1) else {
            return None;
        };

        seg.field(field).and_then(|f| f.repeat(repeat))
    }

    /// Get a mutable reference to the repeat for a given field and repeat, if it exists in
    /// the message. The field is identified by the segment identifier, segment
    /// repeat identifier, 1-based field identifier, and the repeat identifier
    pub fn get_repeat_mut<S: AsRef<str>>(
        &mut self,
        segment: (S, usize),
        field: NonZeroUsize,
        repeat: NonZeroUsize,
    ) -> Option<&mut Repeat> {
        let Some(seg) = self.segment_n_mut(segment.0, segment.1) else {
            return None;
        };

        seg.field_mut(field).and_then(|f| f.repeat_mut(repeat))
    }

    /// Directly get the source (not yet decoded) for a given component, if it exists in the message. The
    /// component is identified by the segment identifier, segment repeat identifier, 1-based field
    /// identifier, and 1-based component identifier
    ///
    /// # Examples
    ///
    /// ```
    /// # use hl7_parser::ParsedMessageOwned;
    /// # use std::num::NonZeroUsize;
    /// let message = include_str!("../test_assets/sample_adt_a01.hl7");
    /// let message = ParsedMessageOwned::parse(&message, true).expect("can parse message");
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

    /// Directly get the component for a given component, if it exists in the message. The
    /// component is identified by the segment identifier, segment repeat identifier, 1-based field
    /// identifier, and 1-based component identifier
    pub fn get_component<S: AsRef<str>>(
        &self,
        segment: (S, usize),
        field: NonZeroUsize,
        repeat: NonZeroUsize,
        component: NonZeroUsize,
    ) -> Option<&Component> {
        let Some(seg) = self.segment_n(segment.0, segment.1) else {
            return None;
        };

        seg.field(field)
            .and_then(|f| f.repeat(repeat))
            .and_then(|r| r.component(component))
    }

    /// Get a mutable reference to a component for a given component, if it exists in the message. The
    /// component is identified by the segment identifier, segment repeat identifier, 1-based field
    /// identifier, and 1-based component identifier
    pub fn get_component_mut<S: AsRef<str>>(
        &mut self,
        segment: (S, usize),
        field: NonZeroUsize,
        repeat: NonZeroUsize,
        component: NonZeroUsize,
    ) -> Option<&mut Component> {
        let Some(seg) = self.segment_n_mut(segment.0, segment.1) else {
            return None;
        };

        seg.field_mut(field)
            .and_then(|f| f.repeat_mut(repeat))
            .and_then(|r| r.component_mut(component))
    }

    /// Directly get the source (not yet decoded) for a given sub-component, if it exists in the message.
    /// The component is identified by the segment identifier, segment repeat identifier, 1-based field
    /// identifier, 1-based component identifier, and 1-based sub-component identifier
    ///
    /// # Examples
    ///
    /// ```
    /// # use hl7_parser::ParsedMessageOwned;
    /// # use std::num::NonZeroUsize;
    /// let message = include_str!("../test_assets/sample_oru_r01_generic.hl7");
    /// let message = ParsedMessageOwned::parse(message, true).expect("can parse message");
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

    /// Directly get the sub-component for a given sub-component, if it exists in the message.
    /// The component is identified by the segment identifier, segment repeat identifier, 1-based field
    /// identifier, 1-based component identifier, and 1-based sub-component identifier
    pub fn get_sub_component<S: AsRef<str>>(
        &self,
        segment: (S, usize),
        field: NonZeroUsize,
        repeat: NonZeroUsize,
        component: NonZeroUsize,
        sub_component: NonZeroUsize,
    ) -> Option<&SubComponent> {
        let Some(seg) = self.segment_n(segment.0, segment.1) else {
            return None;
        };

        seg.field(field)
            .and_then(|f| f.repeat(repeat))
            .and_then(|r| r.component(component))
            .and_then(|c| c.sub_component(sub_component))
    }

    /// Get a mutable reference to the sub-component for a given sub-component, if it exists in the message.
    /// The component is identified by the segment identifier, segment repeat identifier, 1-based field
    /// identifier, 1-based component identifier, and 1-based sub-component identifier
    pub fn get_sub_component_mut<S: AsRef<str>>(
        &mut self,
        segment: (S, usize),
        field: NonZeroUsize,
        repeat: NonZeroUsize,
        component: NonZeroUsize,
        sub_component: NonZeroUsize,
    ) -> Option<&mut SubComponent> {
        let Some(seg) = self.segment_n_mut(segment.0, segment.1) else {
            return None;
        };

        seg.field_mut(field)
            .and_then(|f| f.repeat_mut(repeat))
            .and_then(|r| r.component_mut(component))
            .and_then(|c| c.sub_component_mut(sub_component))
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
    /// # use hl7_parser::ParsedMessageOwned;
    /// # use std::num::NonZeroUsize;
    /// let message = include_str!("../test_assets/sample_adt_a01.hl7");
    /// let message = ParsedMessageOwned::parse(message, true).expect("can parse message");
    ///
    /// let trigger_event = message.query_value("MSH.9.2").expect("can parse location query");
    /// assert_eq!(trigger_event, Some("A01"));
    /// ```
    pub fn query_value<Q, QErr>(&self, query: Q) -> Result<Option<&str>, QErr>
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

    /// Query the message for a given segment, field, component, or sub-comonent,
    /// returning the range in the source string that the item occupies.
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
    /// # use hl7_parser::ParsedMessageOwned;
    /// # use std::num::NonZeroUsize;
    /// let message = include_str!("../test_assets/sample_adt_a01.hl7");
    /// let message = ParsedMessageOwned::parse(message, true).expect("can parse message");
    ///
    /// let trigger_event = message.query("MSH.9.2").expect("can parse location query");
    /// assert_eq!(trigger_event, Some(40..43));
    /// ```
    pub fn query<Q, QErr>(&self, query: Q) -> Result<Option<Range<usize>>, QErr>
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
            (Some(f), Some(r), Some(c), Some(s)) => self
                .get_sub_component((segment, 0), f, r, c, s)
                .map(|s| s.range.clone()),
            (Some(f), Some(r), Some(c), _) => self
                .get_component((segment, 0), f, r, c)
                .map(|c| c.range.clone()),
            (Some(f), Some(r), _, _) => {
                self.get_repeat((segment, 0), f, r).map(|r| r.range.clone())
            }
            (Some(f), _, _, _) => self.get_field((segment, 0), f).map(|f| f.range.clone()),
            _ => self.segment(segment).map(|seg| seg.range.clone()),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_locate_cursor() {
        let cursor = 26;
        let message = include_str!("../test_assets/sample_adt_a01.hl7");
        let message = ParsedMessage::parse(message, true).expect("can parse message");

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

        let message = include_str!("../test_assets/sample_adt_a04.hl7");
        let message = ParsedMessage::parse(message, true).expect("can parse message");
        let cursor = 0x1cc;

        let (id, n, seg) = message
            .segment_at_cursor(cursor)
            .expect("can get segment at cursor");
        assert_eq!(id, "AL1");
        assert_eq!(n, 0);

        let (n, field) = seg
            .field_at_cursor(cursor)
            .expect("can get field at cursor");
        assert_eq!(n, NonZeroUsize::new(5).unwrap());

        let (n, repeat) = field
            .repeat_at_cursor(cursor)
            .expect("can get repeat at cursor");
        assert_eq!(n.get(), 2);
        assert_eq!(repeat.source(message.source), "RASH");
    }

    #[test]
    fn can_locate_cursor_at_empty_fields() {
        let message = include_str!("../test_assets/sample_adt_a01.hl7");
        let message = ParsedMessage::parse(message, true).expect("can parse message");
        let location = message.locate_cursor(19);
        assert!(location.segment.is_some());
        assert!(location.field.is_some());
        assert!(location.component.is_none());
        assert!(location.sub_component.is_none());
    }

    #[test]
    fn can_display_hl7_path() {
        let cursor = 0x458;
        let message = include_str!("../test_assets/sample_adt_a01.hl7");
        let message = ParsedMessage::parse(message, true).expect("can parse message");
        let location = message.locate_cursor(cursor);
        let location = format!("{location}");
        assert_eq!(location, "IN1.5.3.1");

        let cursor = 0x1cc;
        let message = include_str!("../test_assets/sample_adt_a04.hl7");
        let message = ParsedMessage::parse(message, true).expect("can parse message");
        let location = message.locate_cursor(cursor);
        let location = format!("{location}");
        assert_eq!(location, "AL1.5[2].1.1");
    }

    #[test]
    fn can_create_owned_version() {
        let raw_message = include_str!("../test_assets/sample_adt_a01.hl7");
        let message = ParsedMessage::parse(raw_message, true).expect("can parse message");
        let message_from = ParsedMessageOwned::from(message);

        let message_direct = ParsedMessageOwned::parse(raw_message, true).expect("can parse message");

        assert_eq!(message_from, message_direct);
    }

    #[test]
    fn has_a_not_terrible_error_message() {
        assert_eq!(
            ParsedMessage::parse("MSH|^~\\&$", false)
                .expect_err("ParsedMessage parsing to fail")
                .to_string()
                .as_str(),
            "ParsedMessage parsing failed at position 8 (line 1 column 9): `$`"
        );
    }

    #[test]
    fn message_and_message_buf_have_the_same_errors() {
        let err = ParsedMessage::parse("MSH|^~\\&$", false).expect_err("ParsedMessage parsing to fail");
        let err_buf =
            ParsedMessageOwned::parse("MSH|^~\\&$", false).expect_err("ParsedMessage parsing to fail");
        assert_eq!(err, err_buf);
        assert_eq!(err.to_string(), err_buf.to_string());
    }
}
