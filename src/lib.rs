use std::{collections::HashMap, num::NonZeroUsize};

use nom::{
    bytes::complete::{tag, take, take_till},
    character::complete::char,
    multi::separated_list0,
    IResult,
};
use nom_locate::{position, LocatedSpan};

mod time;

pub type Span<'a> = LocatedSpan<&'a str>;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Separators<'s> {
    source_field: &'s str,
    source_encoding: &'s str,
    pub field: char,
    pub component: char,
    pub repeat: char,
    pub escape: char,
    pub subcomponent: char,
}

impl Default for Separators<'static> {
    fn default() -> Self {
        Separators {
            source_field: "|",
            source_encoding: "^~\\&",
            field: '|',
            component: '^',
            repeat: '~',
            escape: '\\',
            subcomponent: '&',
        }
    }
}

impl<'s> Separators<'s> {
    pub fn decode(&'s self, source: &str) -> String {
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
pub struct SubComponent<'s> {
    pub source: &'s str,
    pub position: Span<'s>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component<'s> {
    pub source: &'s str,
    pub position: Span<'s>,
    pub sub_components: Vec<SubComponent<'s>>,
}

impl<'s> Component<'s> {
    pub fn sub_component(&'s self, sub_component: NonZeroUsize) -> Option<&'s SubComponent<'s>> {
        self.sub_components.get(sub_component.get() - 1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field<'s> {
    pub source: &'s str,
    pub position: Span<'s>,
    pub components: Vec<Component<'s>>,
}

impl<'s> Field<'s> {
    pub fn component(&'s self, component: NonZeroUsize) -> Option<&'s Component<'s>> {
        self.components.get(component.get() - 1)
    }
}

#[derive(Debug)]
pub struct MSH<'s> {
    pub source: &'s str,
    pub separators: Separators<'s>,
    pub fields: Vec<Field<'s>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment<'s> {
    pub source: &'s str,
    pub fields: Vec<Field<'s>>,
}

impl<'s> Segment<'s> {
    pub fn field(&'s self, field: NonZeroUsize) -> Option<&'s Field<'s>> {
        self.fields.get(field.get() - 1)
    }
}

impl<'s> From<MSH<'s>> for Segment<'s> {
    fn from(msh: MSH<'s>) -> Self {
        let MSH {
            source,
            separators,
            mut fields,
        } = msh;
        fields.insert(
            0,
            Field {
                source: separators.source_field,
                position: unsafe { Span::new_from_raw_offset(3, 0, separators.source_field, ()) },
                components: Vec::with_capacity(0),
            },
        );
        fields.insert(
            1,
            Field {
                source: separators.source_encoding,
                position: unsafe {
                    Span::new_from_raw_offset(4, 0, separators.source_encoding, ())
                },
                components: Vec::with_capacity(0),
            },
        );
        Segment { source, fields }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segments<'s> {
    Single(Segment<'s>),
    Many(Vec<Segment<'s>>),
}

impl<'s> Segments<'s> {
    fn nth(&'s self, n: usize) -> Option<&'s Segment<'s>> {
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

impl<'s> From<Segment<'s>> for Segments<'s> {
    fn from(value: Segment<'s>) -> Self {
        Segments::Single(value)
    }
}

impl<'s> From<Vec<Segment<'s>>> for Segments<'s> {
    fn from(value: Vec<Segment<'s>>) -> Self {
        Segments::Many(value)
    }
}

#[derive(Debug, Clone)]
pub struct Message<'s> {
    pub separators: Separators<'s>,
    pub segments: HashMap<&'s str, Segments<'s>>,
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
        source_field,
        source_encoding,
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
                source: source.fragment(),
                position,
            },
        ))
    }
}

fn sub_components_parser(
    separators: Separators,
) -> impl Fn(Span) -> IResult<Span, Vec<SubComponent>> + '_ {
    move |s: Span| -> IResult<Span, Vec<SubComponent>> {
        let parse_sub_component = sub_component_parser(separators);
        separated_list0(char(separators.subcomponent), parse_sub_component)(s)
    }
}

fn component_parser(separators: Separators) -> impl Fn(Span) -> IResult<Span, Component> + '_ {
    move |s: Span| -> IResult<Span, Component> {
        let parse_sub_components = sub_components_parser(separators);

        let start = s;
        let (s, start_pos) = position(s)?;
        let (s, sub_components) = parse_sub_components(s)?;

        let (s, end_pos) = position(s)?;
        let source = &start[..(end_pos.location_offset() - start_pos.location_offset())];

        Ok((
            s,
            Component {
                source,
                position: start_pos,
                sub_components,
            },
        ))
    }
}

fn components_parser(
    separators: Separators,
) -> impl Fn(Span) -> IResult<Span, Vec<Component>> + '_ {
    move |s: Span| -> IResult<Span, Vec<Component>> {
        let parse_component = component_parser(separators);
        separated_list0(char(separators.component), parse_component)(s)
    }
}

fn field_parser(separators: Separators) -> impl Fn(Span) -> IResult<Span, Field> + '_ {
    move |s: Span| -> IResult<Span, Field> {
        let parse_components = components_parser(separators);

        let start = s;
        let (s, start_pos) = position(s)?;
        let (s, components) = parse_components(s)?;

        let (s, end_pos) = position(s)?;
        let source = &start[..(end_pos.location_offset() - start_pos.location_offset())];

        Ok((
            s,
            Field {
                source,
                components,
                position: start_pos,
            },
        ))
    }
}

fn fields_parser(separators: Separators) -> impl Fn(Span) -> IResult<Span, Vec<Field>> + '_ {
    move |s: Span| -> IResult<Span, Vec<Field>> {
        let parse_field = field_parser(separators);
        separated_list0(char(separators.field), parse_field)(s)
    }
}

fn parse_msh(s: Span) -> IResult<Span, MSH> {
    let start = s;
    let (s, start_pos) = position(s)?;

    let (s, _) = tag("MSH")(s)?;
    let (s, separators) = parse_separators(s)?;
    let (s, _) = char(separators.field)(s)?;

    let parse_fields = fields_parser(separators);
    let (s, fields) = parse_fields(s)?;

    let (s, pos) = position(s)?;
    let source = &start[..(pos.location_offset() - start_pos.location_offset())];

    Ok((
        s,
        MSH {
            source,
            separators,
            fields,
        },
    ))
}

fn segment_parser(separators: Separators) -> impl Fn(Span) -> IResult<Span, (&str, Segment)> + '_ {
    move |s: Span| -> IResult<Span, (&str, Segment)> {
        let start = s;
        let (s, start_pos) = position(s)?;

        let (s, identifier) = take(3u8)(s)?;
        let (s, _) = char(separators.field)(s)?;

        let parse_fields = fields_parser(separators);
        let (s, fields) = parse_fields(s)?;

        let (s, pos) = position(s)?;
        let source = &start[..(pos.location_offset() - start_pos.location_offset())];

        Ok((s, (identifier.fragment(), Segment { source, fields })))
    }
}

fn parse_message(s: Span) -> IResult<Span, Message> {
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

    pub fn segment(&'s self, segment: &str) -> Option<&'s Segment<'s>> {
        self.segments.get(segment).map(|seg| seg.nth(0)).flatten()
    }

    pub fn segment_count(&'s self, segment: &str) -> usize {
        self.segments
            .get(segment)
            .map(|seg| seg.count())
            .unwrap_or_default()
    }

    pub fn segment_n(&'s self, segment: &str, n: NonZeroUsize) -> Option<&'s Segment<'s>> {
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
        assert_eq!(sub_components[0].source, "abc");
        assert_eq!(sub_components[1].source, "def");
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
        assert_eq!(sc2.source, "ghi");
    }

    #[test]
    fn can_parse_components() {
        let parse_components = components_parser(Separators::default());

        let components = "ADT^A01";
        let (_, components) =
            parse_components(Span::new(components)).expect("can parse components");
        assert_eq!(components.len(), 2);
        assert_eq!(components[0].source, "ADT");
        assert_eq!(components[1].source, "A01");

        let components = "xyz";
        let (_, components) =
            parse_components(Span::new(components)).expect("can parse components");
        assert_eq!(components.len(), 1);
        assert_eq!(components[0].source, "xyz");
    }

    #[test]
    fn can_parse_field_components() {
        let parse_fields = fields_parser(Separators::default());

        let fields = "abc|def^hij";
        let (_, fields) = parse_fields(Span::new(fields)).expect("can parse fields");
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].source, "abc");
        assert_eq!(fields[1].source, "def^hij");
        let c1 = fields[1]
            .component(NonZeroUsize::new(1).unwrap())
            .expect("can get component 1");
        let c2 = fields[1]
            .component(NonZeroUsize::new(2).unwrap())
            .expect("can get component 2");
        assert_eq!(c1.source, "def");
        assert_eq!(c2.source, "hij");
    }

    #[test]
    fn can_parse_fields() {
        let parse_fields = fields_parser(Separators::default());

        let fields = "abc|def|hij^klm\r123";
        let (_, fields) = parse_fields(Span::new(fields)).expect("can parse fields");
        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].source, "abc");
        assert_eq!(fields[1].source, "def");
        assert_eq!(fields[2].source, "hij^klm");

        let fields = "abc";
        let (_, fields) = parse_fields(Span::new(fields)).expect("can parse fields");
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].source, "abc");
    }

    #[test]
    fn can_parse_msh() {
        let (_, msh) =
            parse_msh(Span::new("MSH|^~\\&|sfac|sapp|rfac|rapp")).expect("can parse msh");
        assert_eq!(msh.source, "MSH|^~\\&|sfac|sapp|rfac|rapp");
        assert_eq!(msh.fields.len(), 4);
        assert_eq!(msh.fields[0].source, "sfac");
        assert_eq!(msh.fields[0].position.location_offset(), 9);
        assert_eq!(msh.fields[1].source, "sapp");
        assert_eq!(msh.fields[1].position.location_offset(), 14);
        assert_eq!(msh.fields[2].source, "rfac");
        assert_eq!(msh.fields[2].position.location_offset(), 19);
        assert_eq!(msh.fields[3].source, "rapp");
        assert_eq!(msh.fields[3].position.location_offset(), 24);
    }

    #[test]
    fn msh_field_access_is_correct() {
        let (_, msh) =
            parse_msh(Span::new("MSH|^~\\&|sfac|sapp|rfac|rapp")).expect("can parse msh");
        let seg: Segment = msh.into();
        assert_eq!(
            seg.field(NonZeroUsize::new(1).unwrap())
                .expect("can get field 1")
                .source,
            "|"
        );
        assert_eq!(
            seg.field(NonZeroUsize::new(2).unwrap())
                .expect("can get field 2")
                .source,
            "^~\\&"
        );
        assert_eq!(
            seg.field(NonZeroUsize::new(3).unwrap())
                .expect("can get field 3")
                .source,
            "sfac"
        );
    }

    #[test]
    fn can_parse_segment() {
        let segment = "MSA|AA|1234|woohoo";
        let parse_segment = segment_parser(Separators::default());
        let (_, (identifier, segment)) =
            parse_segment(Span::new(segment)).expect("can parse segment");
        assert_eq!(segment.source, "MSA|AA|1234|woohoo");
        assert_eq!(identifier, "MSA");
        assert_eq!(
            segment
                .field(NonZeroUsize::new(1).unwrap())
                .expect("can get MSA.1")
                .source,
            "AA"
        );
        assert_eq!(
            segment
                .field(NonZeroUsize::new(2).unwrap())
                .expect("can get MSA.2")
                .source,
            "1234"
        );
        assert_eq!(
            segment
                .field(NonZeroUsize::new(3).unwrap())
                .expect("can get MSA.3")
                .source,
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
                .source,
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
        assert_eq!(
            message
                .segment_n("OBX", NonZeroUsize::new(14).unwrap())
                .expect("can get OBX 14")
                .field(NonZeroUsize::new(3).unwrap())
                .expect("can get OBX14.3")
                .component(NonZeroUsize::new(2).unwrap())
                .expect("can get OBX14.3.2")
                .source,
            "Basophils"
        )
    }
}
