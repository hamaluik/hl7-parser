use std::borrow::Cow;

use super::field::field;
use crate::message::{Field, Segment, Separators};
use nom::{
    bytes::complete::{tag, take_while_m_n},
    character::complete::char,
    combinator::{consumed, opt},
    multi::separated_list0,
    sequence::preceded,
    IResult,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MSH<'i> {
    pub separators: Separators,
    pub fields: Vec<Field<'i>>,
}

pub fn msh<'i>() -> impl FnMut(&'i str) -> IResult<&'i str, MSH<'i>> {
    move |i| parse_msh(i)
}

fn msh_name<'i>() -> impl FnMut(&'i str) -> IResult<&'i str, &'i str> {
    move |i| tag("MSH")(i)
}

fn field_separator<'i>() -> impl FnMut(&'i str) -> IResult<&'i str, &str> {
    move |i| take_while_m_n(1, 1, |c: char| c.is_ascii())(i)
}

fn separators<'i>(field: char) -> impl FnMut(&'i str) -> IResult<&'i str, Separators> {
    move |i| {
        let (i, seps) = take_while_m_n(4, 4, |c: char| c.is_ascii())(i)?;
        let mut chars = seps.chars();
        let seps = Separators {
            field,
            component: chars.next().expect("char 0: component separator"),
            repetition: chars.next().expect("char 1: repetition separator"),
            escape: chars.next().expect("char 2: escape"),
            subcomponent: chars.next().expect("char 3: subcomponent separator"),
        };
        Ok((i, seps))
    }
}

fn parse_msh<'i>(i: &'i str) -> IResult<&'i str, MSH<'i>> {
    let (i, _) = msh_name()(i)?;
    let (i, f) = field_separator()(i)?;
    let (i, (sep_src, seps)) = consumed(separators(f.chars().next().expect("char 0: field")))(i)?;
    let (i, mut fields) = preceded(
        opt(char(seps.field)),
        separated_list0(char(seps.field), field(seps)),
    )(i)?;
    fields.insert(
        0,
        Field {
            value: Cow::Borrowed(sep_src),
            repeats: vec![],
            components: vec![],
        },
    );
    fields.insert(
        0,
        Field {
            value: Cow::Borrowed(f),
            repeats: vec![],
            components: vec![],
        },
    );

    Ok((
        i,
        MSH {
            separators: seps,
            fields,
        },
    ))
}

impl<'i> From<MSH<'i>> for Segment<'i> {
    fn from(value: MSH<'i>) -> Self {
        Segment {
            name: Cow::Borrowed("MSH"),
            fields: value.fields,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions_sorted::assert_eq;

    #[test]
    fn can_parse_msh_start() {
        let input = r"MSH|^~\&|";
        let expected = MSH {
            separators: Separators {
                field: '|',
                component: '^',
                subcomponent: '&',
                repetition: '~',
                escape: '\\',
            },
            fields: vec![
                Field {
                    value: Cow::Borrowed("|"),
                    repeats: vec![],
                    components: vec![],
                },
                Field {
                    value: Cow::Borrowed(r"^~\&"),
                    repeats: vec![],
                    components: vec![],
                },
                Field {
                    value: Cow::Borrowed(""),
                    repeats: vec![],
                    components: vec![],
                },
            ],
        };
        let actual = parse_msh(input).unwrap().1;
        assert_eq!(expected, actual);
    }

    #[test]
    fn can_parse_msh() {
        let input = r"MSH|^~\&|AccMgr|1";
        let expected = MSH {
            separators: Separators {
                field: '|',
                component: '^',
                subcomponent: '&',
                repetition: '~',
                escape: '\\',
            },
            fields: vec![
                Field {
                    value: Cow::Borrowed("|"),
                    repeats: vec![],
                    components: vec![],
                },
                Field {
                    value: Cow::Borrowed(r"^~\&"),
                    repeats: vec![],
                    components: vec![],
                },
                Field {
                    value: Cow::Borrowed("AccMgr"),
                    repeats: vec![],
                    components: vec![],
                },
                Field {
                    value: Cow::Borrowed("1"),
                    repeats: vec![],
                    components: vec![],
                },
            ],
        };
        let actual = msh()(input).unwrap().1;
        assert_eq!(expected, actual);
    }
}
