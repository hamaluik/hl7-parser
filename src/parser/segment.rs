use super::{field::field, Span};
use crate::message::{Segment, Separators};
use nom::{
    bytes::complete::take_while_m_n, character::complete::char, combinator::consumed,
    multi::separated_list0, sequence::separated_pair, IResult,
};

pub fn segment<'i>(seps: Separators) -> impl FnMut(Span<'i>) -> IResult<Span<'i>, Segment<'i>> {
    move |i| parse_segment(i, seps)
}

fn segment_name<'i>() -> impl FnMut(Span<'i>) -> IResult<Span<'i>, Span<'i>> {
    move |i| parse_segment_name(i)
}

pub(crate) fn parse_segment_name(i: Span) -> IResult<Span, Span> {
    take_while_m_n(3, 3, |c: char| c.is_alphanumeric())(i)
}

fn parse_segment(i: Span<'_>, seps: Separators) -> IResult<Span<'_>, Segment<'_>> {
    let pos_start = i.offset;
    let (i, (segment_src, (name, v))) = consumed(separated_pair(
        segment_name(),
        char(seps.field),
        separated_list0(char(seps.field), field(seps)),
    ))(i)?;
    let pos_end = i.offset;

    Ok((
        i,
        Segment {
            source: segment_src.input,
            name: name.input,
            fields: v,
            range: pos_start..pos_end,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::Separators;
    use pretty_assertions_sorted::assert_eq;

    #[test]
    fn can_parse_segment() {
        let separators = Separators::default();

        let input = Span::new("MSH|foo|bar|baz");
        let actual = parse_segment(input, separators).unwrap().1;
        assert_eq!(actual.name, "MSH");
        assert_eq!(actual.fields.len(), 3);
        assert_eq!(format!("{}", actual.fields[0].display(&separators)), "foo");
    }

    #[test]
    fn can_parse_segment_with_number() {
        let separators = Separators::default();

        let input = Span::new("DB1|foo|bar|baz");
        let actual = parse_segment(input, separators).unwrap().1;
        assert_eq!(actual.name, "DB1");
        assert_eq!(actual.fields.len(), 3);
        assert_eq!(format!("{}", actual.fields[0].display(&separators)), "foo");
    }

    #[test]
    fn cant_parse_segments_with_invalid_names() {
        let separators = Separators::default();

        let input = Span::new("MS|foo|bar|baz");
        let actual = parse_segment(input, separators);
        assert!(actual.is_err());

        let input = Span::new("MSAX|foo|bar|baz");
        let actual = parse_segment(input, separators);
        assert!(actual.is_err());
    }

    #[test]
    fn cant_parse_segments_with_no_names() {
        let separators = Separators::default();

        let input = Span::new("|foo|bar|baz");
        let actual = parse_segment(input, separators);
        assert!(actual.is_err());
    }

    #[test]
    fn cant_parse_segments_with_no_fields() {
        let separators = Separators::default();

        let input = Span::new("MSH");
        let actual = parse_segment(input, separators);
        assert!(actual.is_err());
    }

    #[test]
    fn can_parse_segment_with_repeats_components_and_subcomponents() {
        let separators = Separators::default();

        let input = Span::new(r"MSH|foo^b\~ar^baz&x~qux^quux^quuz|asdf|");
        let actual = parse_segment(input, separators).unwrap().1;
        // dbg!(&actual);
        assert_eq!(actual.name, "MSH");
        assert_eq!(actual.fields.len(), 3);
        assert_eq!(
            actual.fields[0].raw_value(),
            r"foo^b\~ar^baz&x~qux^quux^quuz"
        );
        assert_eq!(actual.fields[0].repeats.len(), 2);
        assert_eq!(actual.fields[0].repeats[0].raw_value(), r"foo^b\~ar^baz&x");
        assert_eq!(actual.fields[0].repeats[1].raw_value(), r"qux^quux^quuz");
        assert_eq!(actual.fields[0].repeats[0].components.len(), 3);
        assert_eq!(actual.fields[0].repeats[0].components[0].raw_value(), "foo");
        assert_eq!(
            actual.fields[0].repeats[0].components[1].raw_value(),
            r"b\~ar"
        );
        assert_eq!(
            actual.fields[0].repeats[0].components[2].raw_value(),
            "baz&x"
        );
        assert_eq!(
            actual.fields[0].repeats[0].components[2]
                .subcomponents
                .len(),
            2
        );
        assert_eq!(
            actual.fields[0].repeats[0].components[2].subcomponents[0].raw_value(),
            "baz"
        );
        assert_eq!(
            actual.fields[0].repeats[0].components[2].subcomponents[1].raw_value(),
            "x"
        );
    }

    #[test]
    fn can_parse_segment_with_trailing_empty_fields() {
        let separators = Separators::default();

        let input = Span::new("MSH|foo|bar|baz||");
        let actual = parse_segment(input, separators).unwrap().1;
        assert_eq!(actual.fields.len(), 5);
        assert_eq!(actual.fields[3].raw_value(), "");
        assert_eq!(actual.fields[4].raw_value(), "");
    }
}
