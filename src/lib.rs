use std::{collections::HashMap, num::NonZeroUsize, ops::Range};

use nom::{
    bytes::complete::{tag, take, take_till},
    character::complete::char,
    multi::separated_list0,
    IResult,
};
use nom_locate::{position, LocatedSpan};

pub type Span<'a> = LocatedSpan<&'a str>;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Separators {
    pub field: char,
    pub component: char,
    pub repeat: char,
    pub escape: char,
    pub subcomponent: char,
}

impl Default for Separators {
    fn default() -> Self {
        Separators {
            field: '|',
            component: '^',
            repeat: '~',
            escape: '\\',
            subcomponent: '&',
        }
    }
}

impl Separators {
    pub fn decode(&self, source: &str) -> String {
        let mut tmp = [0; 4];
        source
            .replace(r#"\F\"#, self.field.encode_utf8(&mut tmp))
            .replace(r#"\R\"#, self.repeat.encode_utf8(&mut tmp))
            .replace(r#"\S\"#, self.component.encode_utf8(&mut tmp))
            .replace(r#"\T\"#, self.subcomponent.encode_utf8(&mut tmp))
            .replace(r#"\.br\"#, "\r")
            .replace(r#"\X0A\"#, "\n")
            .replace(r#"\X0D\"#, "\r")
            .replace(r#"\E\"#, self.escape.encode_utf8(&mut tmp))
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub range: Range<usize>,
    pub sub_components: Vec<SubComponent>,
}

impl Component {
    pub fn sub_component(&self, sub_component: NonZeroUsize) -> Option<&SubComponent> {
        self.sub_components.get(sub_component.get() - 1)
    }

    #[inline]
    pub fn source<'s>(&self, s: &'s str) -> &'s str {
        &s[self.range.clone()]
    }
}

pub trait SubComponentAccessor {
    fn sub_component(&self, sub_component: NonZeroUsize) -> Option<&SubComponent>;
}

impl SubComponentAccessor for Option<&Component> {
    fn sub_component(&self, sub_component: NonZeroUsize) -> Option<&SubComponent> {
        match self {
            None => None,
            Some(component) => component.sub_component(sub_component),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub range: Range<usize>,
    pub components: Vec<Component>,
}

impl Field {
    pub fn component(&self, component: NonZeroUsize) -> Option<&Component> {
        self.components.get(component.get() - 1)
    }

    #[inline]
    pub fn source<'s>(&self, s: &'s str) -> &'s str {
        &s[self.range.clone()]
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

#[derive(Debug)]
pub struct MSH {
    pub range: Range<usize>,
    pub separators: Separators,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    pub range: Range<usize>,
    pub fields: Vec<Field>,
}

impl Segment {
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

impl Segments {
    fn nth(&self, n: usize) -> Option<&Segment> {
        match self {
            Segments::Single(seg) if n == 0 => Some(seg),
            Segments::Many(segs) if n < segs.len() => Some(&segs[n]),
            _ => None,
        }
    }

    fn count(&self) -> usize {
        match self {
            Segments::Single(_) => 1,
            Segments::Many(segs) => segs.len(),
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

#[derive(Debug, Clone)]
pub struct Message<'s> {
    pub source: &'s str,
    pub separators: Separators,
    pub segments: HashMap<&'s str, Segments>,
}

fn parse_separators(s: Span) -> IResult<Span, Separators> {
    let (s, source_field) = take(1u8)(s)?;
    let (s, source_encoding) = take(4u8)(s)?;

    let source_field = source_field.fragment();
    let field = source_field.chars().nth(0).unwrap();

    let source_encoding = source_encoding.fragment();
    let mut ec = source_encoding.chars();
    let component = ec.next().unwrap();
    let repeat = ec.next().unwrap();
    let escape = ec.next().unwrap();
    let subcomponent = ec.next().unwrap();

    let separators = Separators {
        field,
        component,
        repeat,
        escape,
        subcomponent,
    };
    Ok((s, separators))
}

fn sub_component_parser(separators: Separators) -> impl Fn(Span) -> IResult<Span, SubComponent> {
    move |s: Span| -> IResult<Span, SubComponent> {
        let (s, position) = position(s)?;
        let (s, source) = take_till(|c| {
            c == separators.subcomponent
                || c == separators.component
                || c == separators.field
                || c == '\r'
        })(s)?;

        Ok((
            s,
            SubComponent {
                range: position.location_offset()
                    ..(position.location_offset() + source.fragment().len()),
            },
        ))
    }
}

fn sub_components_parser(
    separators: Separators,
) -> impl Fn(Span) -> IResult<Span, Vec<SubComponent>> {
    move |s: Span| -> IResult<Span, Vec<SubComponent>> {
        let parse_sub_component = sub_component_parser(separators);
        separated_list0(char(separators.subcomponent), parse_sub_component)(s)
    }
}

fn component_parser(separators: Separators) -> impl Fn(Span) -> IResult<Span, Component> {
    move |s: Span| -> IResult<Span, Component> {
        let parse_sub_components = sub_components_parser(separators);

        let (s, start_pos) = position(s)?;
        let (s, sub_components) = parse_sub_components(s)?;
        let (s, end_pos) = position(s)?;

        Ok((
            s,
            Component {
                range: start_pos.location_offset()..end_pos.location_offset(),
                sub_components,
            },
        ))
    }
}

fn components_parser(separators: Separators) -> impl Fn(Span) -> IResult<Span, Vec<Component>> {
    move |s: Span| -> IResult<Span, Vec<Component>> {
        let parse_component = component_parser(separators);
        separated_list0(char(separators.component), parse_component)(s)
    }
}

fn field_parser(separators: Separators) -> impl Fn(Span) -> IResult<Span, Field> {
    move |s: Span| -> IResult<Span, Field> {
        let parse_components = components_parser(separators);

        let (s, start_pos) = position(s)?;
        let (s, components) = parse_components(s)?;
        let (s, end_pos) = position(s)?;

        Ok((
            s,
            Field {
                range: start_pos.location_offset()..end_pos.location_offset(),
                components,
            },
        ))
    }
}

fn fields_parser(separators: Separators) -> impl Fn(Span) -> IResult<Span, Vec<Field>> {
    move |s: Span| -> IResult<Span, Vec<Field>> {
        let parse_field = field_parser(separators);
        separated_list0(char(separators.field), parse_field)(s)
    }
}

fn parse_msh(s: Span) -> IResult<Span, MSH> {
    let (s, start_pos) = position(s)?;

    let (s, _) = tag("MSH")(s)?;
    let (s, separators) = parse_separators(s)?;
    let (s, _) = char(separators.field)(s)?;

    let parse_fields = fields_parser(separators);
    let (s, fields) = parse_fields(s)?;

    let (s, end_pos) = position(s)?;

    Ok((
        s,
        MSH {
            range: start_pos.location_offset()..end_pos.location_offset(),
            separators,
            fields,
        },
    ))
}

fn segment_parser(separators: Separators) -> impl Fn(Span) -> IResult<Span, (&str, Segment)> {
    move |s: Span| -> IResult<Span, (&str, Segment)> {
        let (s, start_pos) = position(s)?;

        let (s, identifier) = take(3u8)(s)?;
        let (s, _) = char(separators.field)(s)?;

        let parse_fields = fields_parser(separators);
        let (s, fields) = parse_fields(s)?;
        let (s, end_pos) = position(s)?;

        Ok((
            s,
            (
                identifier.fragment(),
                Segment {
                    range: start_pos.location_offset()..end_pos.location_offset(),
                    fields,
                },
            ),
        ))
    }
}

fn parse_message(s: Span) -> IResult<Span, Message> {
    let source = s.fragment();
    let (s, msh) = parse_msh(s)?;

    let separators = msh.separators;
    let msh: Segment = msh.into();
    let msh: Segments = msh.into();

    let mut segments = HashMap::default();
    segments.insert("MSH", msh);

    let (s, _) = char('\r')(s)?;
    let parse_segment = segment_parser(separators);
    let (s, segs) = separated_list0(char('\r'), parse_segment)(s)?;
    for (seg_id, seg) in segs.into_iter() {
        let seg2 = seg.clone();
        segments
            .entry(seg_id)
            .and_modify(|entry| match entry {
                Segments::Single(existing_seg) => {
                    *entry = Segments::Many(vec![existing_seg.clone(), seg2])
                }
                Segments::Many(segs) => {
                    segs.push(seg2);
                }
            })
            .or_insert_with(|| Segments::Single(seg));
    }

    Ok((
        s,
        Message {
            source,
            separators,
            segments,
        },
    ))
}

impl<'s> Message<'s> {
    pub fn parse(source: &'s str) -> Result<Message<'s>, nom::Err<nom::error::Error<Span<'s>>>> {
        let (_, message) = parse_message(Span::new(source))?;
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_separators() {
        let (_, separators) = parse_separators(Span::new("|^~\\&")).expect("can parse separators");
        assert_eq!(separators, Separators::default());
    }

    #[test]
    fn can_parse_sub_components() {
        let parse_sub_components = sub_components_parser(Separators::default());

        let sub_components = "abc&def";
        let (_, sub_components) =
            parse_sub_components(Span::new(sub_components)).expect("can parse sub components");
        assert_eq!(sub_components.len(), 2);
        assert_eq!(sub_components[0].source("abc&def"), "abc");
        assert_eq!(sub_components[1].source("abc&def"), "def");
    }

    #[test]
    fn can_parse_component_subcomponents() {
        let parse_components = components_parser(Separators::default());

        let components = "abc^def&ghi^jkl";
        let (_, components) =
            parse_components(Span::new(components)).expect("can parse components");
        assert_eq!(components.len(), 3);
        let sc2 = components[1]
            .sub_component(NonZeroUsize::new(2).unwrap())
            .expect("can get subcomponent 2");
        assert_eq!(sc2.source("abc^def&ghi^jkl"), "ghi");
    }

    #[test]
    fn can_parse_components() {
        let parse_components = components_parser(Separators::default());

        let components = "ADT^A01";
        let (_, components) =
            parse_components(Span::new(components)).expect("can parse components");
        assert_eq!(components.len(), 2);
        assert_eq!(components[0].source("ADT^A01"), "ADT");
        assert_eq!(components[1].source("ADT^A01"), "A01");

        let components = "xyz";
        let (_, components) =
            parse_components(Span::new(components)).expect("can parse components");
        assert_eq!(components.len(), 1);
        assert_eq!(components[0].source("xyz"), "xyz");
    }

    #[test]
    fn can_parse_field_components() {
        let parse_fields = fields_parser(Separators::default());

        let fields = "abc|def^hij";
        let (_, fields) = parse_fields(Span::new(fields)).expect("can parse fields");
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].source("abc|def^hij"), "abc");
        assert_eq!(fields[1].source("abc|def^hij"), "def^hij");
        let c1 = fields[1]
            .component(NonZeroUsize::new(1).unwrap())
            .expect("can get component 1");
        let c2 = fields[1]
            .component(NonZeroUsize::new(2).unwrap())
            .expect("can get component 2");
        assert_eq!(c1.source("abc|def^hij"), "def");
        assert_eq!(c2.source("abc|def^hij"), "hij");
    }

    #[test]
    fn can_parse_fields() {
        let parse_fields = fields_parser(Separators::default());

        let fields = "abc|def|hij^klm\r123";
        let (_, fields) = parse_fields(Span::new(fields)).expect("can parse fields");
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].source("abc|def|hij^klm\r123"), "abc");
        assert_eq!(fields[1].source("abc|def|hij^klm\r123"), "def");
        assert_eq!(fields[2].source("abc|def|hij^klm\r123"), "hij^klm");

        let fields = "abc";
        let (_, fields) = parse_fields(Span::new(fields)).expect("can parse fields");
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].source("abc"), "abc");
    }

    #[test]
    fn can_parse_msh() {
        let (_, msh) =
            parse_msh(Span::new("MSH|^~\\&|sfac|sapp|rfac|rapp")).expect("can parse msh");
        assert_eq!(msh.fields.len(), 4);
        assert_eq!(
            msh.fields[0].source("MSH|^~\\&|sfac|sapp|rfac|rapp"),
            "sfac"
        );
        assert_eq!(msh.fields[0].range.start, 9);
        assert_eq!(
            msh.fields[1].source("MSH|^~\\&|sfac|sapp|rfac|rapp"),
            "sapp"
        );
        assert_eq!(msh.fields[1].range.start, 14);
        assert_eq!(
            msh.fields[2].source("MSH|^~\\&|sfac|sapp|rfac|rapp"),
            "rfac"
        );
        assert_eq!(msh.fields[2].range.start, 19);
        assert_eq!(
            msh.fields[3].source("MSH|^~\\&|sfac|sapp|rfac|rapp"),
            "rapp"
        );
        assert_eq!(msh.fields[3].range.start, 24);
    }

    #[test]
    fn msh_field_access_is_correct() {
        let (_, msh) =
            parse_msh(Span::new("MSH|^~\\&|sfac|sapp|rfac|rapp")).expect("can parse msh");
        let seg: Segment = msh.into();
        assert_eq!(
            seg.field(NonZeroUsize::new(1).unwrap())
                .expect("can get field 1")
                .source("MSH|^~\\&|sfac|sapp|rfac|rapp"),
            "|"
        );
        assert_eq!(
            seg.field(NonZeroUsize::new(2).unwrap())
                .expect("can get field 2")
                .source("MSH|^~\\&|sfac|sapp|rfac|rapp"),
            "^~\\&"
        );
        assert_eq!(
            seg.field(NonZeroUsize::new(3).unwrap())
                .expect("can get field 3")
                .source("MSH|^~\\&|sfac|sapp|rfac|rapp"),
            "sfac"
        );
    }

    #[test]
    fn can_parse_segment() {
        let segment = "MSA|AA|1234|woohoo";
        let parse_segment = segment_parser(Separators::default());
        let (_, (identifier, segment)) =
            parse_segment(Span::new(segment)).expect("can parse segment");
        assert_eq!(segment.source("MSA|AA|1234|woohoo"), "MSA|AA|1234|woohoo");
        assert_eq!(identifier, "MSA");
        assert_eq!(
            segment
                .field(NonZeroUsize::new(1).unwrap())
                .expect("can get MSA.1")
                .source("MSA|AA|1234|woohoo"),
            "AA"
        );
        assert_eq!(
            segment
                .field(NonZeroUsize::new(2).unwrap())
                .expect("can get MSA.2")
                .source("MSA|AA|1234|woohoo"),
            "1234"
        );
        assert_eq!(
            segment
                .field(NonZeroUsize::new(3).unwrap())
                .expect("can get MSA.3")
                .source("MSA|AA|1234|woohoo"),
            "woohoo"
        );
        assert!(segment.field(NonZeroUsize::new(4).unwrap()).is_none());
    }

    #[test]
    fn fails_to_parse_msh_without_id_and_starter_fields() {
        assert!(parse_msh(Span::new("abc|def")).is_err());
        assert!(parse_msh(Span::new("^~\\&")).is_err());
        assert!(parse_msh(Span::new("|^~\\&")).is_err());
        assert!(parse_msh(Span::new("MSH|^~\\&")).is_err());
        assert!(parse_msh(Span::new("MS|^~\\&")).is_err());
        assert!(parse_msh(Span::new("MS_|^~\\&")).is_err());
    }

    #[test]
    fn can_parse_message() {
        let message = include_str!("../test_assets/sample_adt_a01.hl7")
            .replace("\r\n", "\r")
            .replace('\n', "\r");
        let message = Message::parse(message.as_str()).expect("can parse message");

        assert!(message.has_segment("EVN"));
        assert!(message.has_segment("PID"));
        assert!(message.has_segment("PV1"));

        assert_eq!(
            message
                .segment("MSH")
                .expect("MSH segment exists")
                .field(NonZeroUsize::new(9).unwrap())
                .expect("field 9 exists")
                .source(message.source),
            "ADT^A01"
        );
    }

    #[test]
    fn can_decode_encoding_characters() {
        let separators = Separators::default();
        assert_eq!(
            separators.decode(r#"Pierre DuRho\S\ne \T\ Cie"#).as_str(),
            r#"Pierre DuRho^ne & Cie"#
        );
        assert_eq!(separators.decode(r#"\.br\\X0A\\X0D\"#).as_str(), "\r\n\r");
        assert_eq!(separators.decode(r#"\F\\R\\S\\T\\E\"#).as_str(), r#"|~^&\"#);
        assert_eq!(separators.decode(r#"\E\\F\\E\"#).as_str(), r#"\|\"#);
    }

    #[test]
    fn can_parse_multi_segments() {
        let message = include_str!("../test_assets/sample_oru_r01_lab.hl7")
            .replace("\r\n", "\r")
            .replace('\n', "\r");
        let message = Message::parse(message.as_str()).expect("can parse message");

        assert!(message.has_segment("OBX"));
        assert_eq!(message.segment_count("OBX"), 14);
        // let obx14_3_2 = message
        //     .segment_n("OBX", NonZeroUsize::new(14).unwrap())
        //     .field(NonZeroUsize::new(3).unwrap())
        //     .component(NonZeroUsize::new(2).unwrap())
        //     .expect("can get OBX14.3.2");
        assert_eq!(
            message
                .segment_n("OBX", NonZeroUsize::new(14).unwrap())
                .expect("can get OBX 14")
                .field(NonZeroUsize::new(3).unwrap())
                .expect("can get OBX14.3")
                .component(NonZeroUsize::new(2).unwrap())
                .expect("can get OBX14.3.2")
                .source(message.source),
            "Basophils"
        )
    }
}
