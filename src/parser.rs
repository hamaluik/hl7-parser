use super::*;
use nom::{
    bytes::complete::{tag, take, take_till},
    character::complete::char,
    error::{ErrorKind, ParseError},
    Err, IResult, InputLength, Parser,
};
use nom_locate::{position, LocatedSpan};
use std::collections::HashMap;

pub(crate) type Span<'a> = LocatedSpan<&'a str>;

/// More or less lifted from nom; added capacity allocation
fn separated_list0_cap<I, O, O2, E, F, G>(
    mut sep: G,
    mut f: F,
    cap: usize,
) -> impl FnMut(I) -> IResult<I, Vec<O>, E>
where
    I: Clone + InputLength,
    F: Parser<I, O, E>,
    G: Parser<I, O2, E>,
    E: ParseError<I>,
{
    move |mut i: I| {
        let mut res = Vec::with_capacity(cap);

        match f.parse(i.clone()) {
            Err(Err::Error(_)) => return Ok((i, res)),
            Err(e) => return Err(e),
            Ok((i1, o)) => {
                res.push(o);
                i = i1;
            }
        }

        loop {
            let len = i.input_len();
            match sep.parse(i.clone()) {
                Err(Err::Error(_)) => return Ok((i, res)),
                Err(e) => return Err(e),
                Ok((i1, _)) => {
                    // infinite loop check: the parser must always consume
                    if i1.input_len() == len {
                        return Err(Err::Error(E::from_error_kind(i1, ErrorKind::SeparatedList)));
                    }

                    match f.parse(i1.clone()) {
                        Err(Err::Error(_)) => return Ok((i, res)),
                        Err(e) => return Err(e),
                        Ok((i2, o)) => {
                            res.push(o);
                            i = i2;
                        }
                    }
                }
            }
        }
    }
}

fn parse_separators(s: Span) -> IResult<Span, Separators> {
    let (s, source_field) = take(1u8)(s)?;
    let (s, source_encoding) = take(4u8)(s)?;

    let source_field = source_field.fragment();
    let field = source_field.chars().next().unwrap();

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
            c == separators.field
                || c == separators.component
                || c == '\r'
                || c == separators.repeat
                || c == separators.subcomponent
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
        separated_list0_cap(char(separators.subcomponent), parse_sub_component, 1)(s)
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
        separated_list0_cap(char(separators.component), parse_component, 10)(s)
    }
}

fn repeat_parser(separators: Separators) -> impl Fn(Span) -> IResult<Span, Repeat> {
    move |s: Span| -> IResult<Span, Repeat> {
        let parse_components = components_parser(separators);

        let (s, start_pos) = position(s)?;
        let (s, components) = parse_components(s)?;
        let (s, end_pos) = position(s)?;

        Ok((
            s,
            Repeat {
                range: start_pos.location_offset()..end_pos.location_offset(),
                components,
            },
        ))
    }
}

fn repeats_parser(separators: Separators) -> impl Fn(Span) -> IResult<Span, Vec<Repeat>> {
    move |s: Span| -> IResult<Span, Vec<Repeat>> {
        let parse_repeat = repeat_parser(separators);
        separated_list0_cap(char(separators.repeat), parse_repeat, 1)(s)
    }
}

fn field_parser(separators: Separators) -> impl Fn(Span) -> IResult<Span, Field> {
    move |s: Span| -> IResult<Span, Field> {
        let parse_repeats = repeats_parser(separators);

        let (s, start_pos) = position(s)?;
        let (s, repeats) = parse_repeats(s)?;
        let (s, end_pos) = position(s)?;

        Ok((
            s,
            Field {
                range: start_pos.location_offset()..end_pos.location_offset(),
                repeats,
            },
        ))
    }
}

fn fields_parser(separators: Separators) -> impl Fn(Span) -> IResult<Span, Vec<Field>> {
    move |s: Span| -> IResult<Span, Vec<Field>> {
        let parse_field = field_parser(separators);
        separated_list0_cap(char(separators.field), parse_field, 20)(s)
    }
}

fn parse_msh(s: Span) -> IResult<Span, Msh> {
    let (s, start_pos) = position(s)?;

    let (s, _) = tag("MSH")(s)?;
    let (s, separators) = parse_separators(s)?;
    let (s, _) = char(separators.field)(s)?;

    let parse_fields = fields_parser(separators);
    let (s, fields) = parse_fields(s)?;

    let (s, end_pos) = position(s)?;

    Ok((
        s,
        Msh {
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

pub(crate) fn parse_message(s: Span) -> IResult<Span, ParsedMessage> {
    let source = s.fragment();
    let (s, msh) = parse_msh(s)?;

    let separators = msh.separators;
    let msh: Segment = msh.into();
    let msh: Segments = msh.into();

    let mut segments = HashMap::default();
    segments.insert("MSH", msh);

    let (s, _) = char('\r')(s)?;
    let parse_segment = segment_parser(separators);
    let (s, segs) = separated_list0_cap(char('\r'), parse_segment, 10)(s)?;
    for (seg_id, seg) in segs.into_iter() {
        if segments.contains_key(seg_id) {
            let entry = segments.remove(seg_id).unwrap();
            match entry {
                Segments::Single(existing_seg) => {
                    segments.insert(seg_id, Segments::Many(vec![existing_seg, seg]));
                }
                Segments::Many(mut segs) => {
                    segs.push(seg);
                    segments.insert(seg_id, Segments::Many(segs));
                }
            }
        } else {
            segments.insert(seg_id, Segments::Single(seg));
        }
    }

    Ok((
        s,
        ParsedMessage {
            source,
            separators,
            segments,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroUsize;

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
        assert_eq!(
            fields[1]
                .repeat(NonZeroUsize::new(1).unwrap())
                .component(NonZeroUsize::new(1).unwrap())
                .expect("can get component 1")
                .source("abc|def^hij"),
            "def"
        );
        assert_eq!(
            fields[1]
                .repeat(NonZeroUsize::new(1).unwrap())
                .component(NonZeroUsize::new(2).unwrap())
                .expect("can get component 2")
                .source("abc|def^hij"),
            "hij"
        );
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
        let message = ParsedMessage::parse(&message).expect("can parse message");

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
    fn can_parse_multi_segments() {
        let message = include_str!("../test_assets/sample_oru_r01_lab.hl7")
            .replace("\r\n", "\r")
            .replace('\n', "\r");
        let message = ParsedMessage::parse(message.as_str()).expect("can parse message");

        assert!(message.has_segment("OBX"));
        assert_eq!(message.segment_count("OBX"), 14);
        assert_eq!(
            message
                .segment_n("OBX", 13)
                .field(NonZeroUsize::new(3).unwrap())
                .repeat(NonZeroUsize::new(1).unwrap())
                .component(NonZeroUsize::new(2).unwrap())
                .expect("can get OBX14.3.2")
                .source(message.source),
            "Basophils"
        );
        assert_eq!(
            message
                .get_component_source(
                    ("OBX", 13),
                    NonZeroUsize::new(3).unwrap(),
                    NonZeroUsize::new(1).unwrap(),
                    NonZeroUsize::new(2).unwrap(),
                )
                .expect("can get component"),
            "Basophils"
        );
    }

    #[test]
    fn can_parse_repeats() {
        let message = include_str!("../test_assets/sample_adt_a04.hl7")
            .replace("\r\n", "\r")
            .replace('\n', "\r");
        let message = ParsedMessage::parse(message.as_str()).expect("can parse message");

        assert_eq!(
            message
                .segment("AL1")
                .field(NonZeroUsize::new(5).unwrap())
                .expect("AL1.5 exists")
                .repeats
                .len(),
            2
        );
    }
}
