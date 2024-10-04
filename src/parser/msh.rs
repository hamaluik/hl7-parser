use std::ops::Range;

use super::Span;
use crate::{
    message::{Field, Segment, Separators},
    parser::field::field,
};
use nom::{
    bytes::complete::{tag, take_while_m_n},
    combinator::opt,
    multi::separated_list0,
    sequence::preceded,
    IResult,
};

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub(crate) struct MSH<'m> {
    pub(crate) separators: Separators,
    source: &'m str,
    fields: Vec<Field<'m>>,
    range: Range<usize>,
}

pub fn msh<'i>(lenient_newlines: bool) -> impl FnMut(Span<'i>) -> IResult<Span<'i>, MSH<'i>> {
    move |i| parse_msh(i, lenient_newlines)
}

fn msh_name<'i>() -> impl FnMut(Span<'i>) -> IResult<Span<'i>, Span<'i>> {
    move |i| tag("MSH")(i)
}

fn separators<'i>(lenient_newlines: bool) -> impl FnMut(Span<'i>) -> IResult<Span<'i>, Separators> {
    move |i| {
        let (i, seps) = take_while_m_n(5, 5, |c: char| c.is_ascii())(i)?;
        let mut chars = seps.chars();
        let seps = Separators {
            field: chars.next().expect("field separator"),
            component: chars.next().expect("component separator"),
            repetition: chars.next().expect("repetition separator"),
            escape: chars.next().expect("escape"),
            subcomponent: chars.next().expect("subcomponent separator"),
            lenient_newlines,
        };
        Ok((i, seps))
    }
}

fn parse_msh(i: Span<'_>, lenient_newlines: bool) -> IResult<Span<'_>, MSH<'_>> {
    let input_src = i.fragment();
    let pos_start = i.location_offset();

    let (i, _) = msh_name()(i)?;
    let (i, separators) = separators(lenient_newlines)(i)?;
    let (i, mut fields) = preceded(
        opt(nom::character::complete::char(separators.field)),
        separated_list0(
            nom::character::complete::char(separators.field),
            field(separators),
        ),
    )(i)?;

    let pos_end = i.location_offset();
    let msh_src = &input_src[..pos_end];

    let field_separator = Field::new_single(&input_src[3..4], 3..4);
    let encoding_characters = Field::new_single(&input_src[4..8], 4..8);

    fields.insert(0, encoding_characters);
    fields.insert(0, field_separator);

    Ok((
        i,
        MSH {
            separators,
            source: msh_src,
            fields,
            range: pos_start..pos_end,
        },
    ))
}

impl<'m> From<MSH<'m>> for Segment<'m> {
    fn from(msh: MSH<'m>) -> Self {
        Segment {
            source: msh.source,
            name: "MSH",
            fields: msh.fields,
            range: msh.range,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions_sorted::assert_eq;

    #[test]
    fn can_parse_msh_start() {
        let input = Span::new(r"MSH|^~\&|");
        let actual = parse_msh(input, false).unwrap().1;
        assert_eq!(actual.fields.len(), 3);
        assert_eq!(actual.range, 0..9);
        assert_eq!(actual.separators.field, '|');
        assert_eq!(actual.separators.component, '^');
        assert_eq!(actual.separators.repetition, '~');
        assert_eq!(actual.separators.escape, '\\');
        assert_eq!(actual.separators.subcomponent, '&');
        assert_eq!(actual.fields[0].raw_value(), "|");
        assert_eq!(actual.fields[1].raw_value(), "^~\\&");
    }

    #[test]
    fn can_parse_msh() {
        let input = Span::new(r"MSH|^~\&|AccMgr|1");
        let actual = parse_msh(input, false).unwrap().1;
        assert_eq!(actual.fields.len(), 4);
        assert_eq!(actual.range, 0..17);
        assert_eq!(actual.fields[2].raw_value(), "AccMgr");
        assert_eq!(actual.fields[3].raw_value(), "1");
    }
}
