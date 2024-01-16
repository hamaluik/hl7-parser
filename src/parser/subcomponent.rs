use std::borrow::Cow;

use crate::{Separators, Subcomponent};
use nom::{
    branch::alt,
    bytes::complete::{escaped, tag, take_while},
    character::complete::{none_of, one_of},
    combinator::map,
    sequence::terminated,
    IResult, Parser,
};

pub fn subcomponent<'i>(
    seps: Separators,
) -> impl FnMut(&'i str) -> IResult<&'i str, Subcomponent<'i>> {
    move |i| subcomponent_parser(i, seps)
}

fn subcomponent_parser<'i>(i: &'i str, seps: Separators) -> IResult<&'i str, Subcomponent<'i>> {
    let sep = [
        seps.subcomponent,
        seps.component,
        seps.repetition,
        seps.field,
        seps.escape,
        '\r',
    ];

    let (i, v): (&str, &str) = escaped(none_of(&sep[..]), seps.escape, one_of(&sep[..]))(i)?;

    Ok((i, Subcomponent { value: Cow::Borrowed(v)}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions_sorted::assert_eq;

    #[test]
    fn can_parse_subcomponent_basic() {
        let separators = Separators::default();

        let input = "foo";
        let actual = subcomponent_parser(input, separators).unwrap().1;
        assert_eq!(actual.value, "foo");
    }

    #[test]
    fn can_parse_subcomponent_until_next_field() {
        let separators = Separators::default();

        let input = "foo^bar";
        let actual = subcomponent_parser(input, separators).unwrap().1;
        assert_eq!(actual.value, "foo");

        let input = "foo|bar";
        let actual = subcomponent_parser(input, separators).unwrap().1;
        assert_eq!(actual.value, "foo");

        let input = "foo\rbar";
        let actual = subcomponent_parser(input, separators).unwrap().1;
        assert_eq!(actual.value, "foo");
    }

    #[test]
    fn can_parse_subcomponent_with_escape() {
        let separators = Separators::default();

        let input = r"foo|bar\baz^qux";
        let actual = subcomponent_parser(input, separators).unwrap().1;
        assert_eq!(actual.value, "foo");

        let input = r"foo\|bar\\baz\^qux";
        let actual = subcomponent_parser(input, separators).unwrap().1;
        assert_eq!(actual.value, r"foo\|bar\\baz\^qux");
    }
}
