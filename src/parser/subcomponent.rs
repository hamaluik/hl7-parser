use crate::message::{Separators, Subcomponent};
use nom::{
    bytes::complete::escaped,
    character::complete::{none_of, one_of},
    IResult,
};
use nom_locate::position;

use super::Span;

pub fn subcomponent<'i>(
    seps: Separators,
) -> impl FnMut(Span<'i>) -> IResult<Span<'i>, Subcomponent<'i>> {
    move |i| subcomponent_parser(i, seps)
}

fn subcomponent_parser(i: Span, seps: Separators) -> IResult<Span, Subcomponent<'_>> {
    let sep = [
        seps.subcomponent,
        seps.component,
        seps.repetition,
        seps.field,
        seps.escape,
        '\r',
    ];

    let (i, pos_start) = position(i)?;
    let (i, v): (Span, Span) = escaped(none_of(&sep[..]), seps.escape, one_of(&sep[..]))(i)?;
    let (i, pos_end) = position(i)?;
    let value = v.fragment();

    Ok((
        i,
        Subcomponent {
            range: pos_start.location_offset()..pos_end.location_offset(),
            value,
        }
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions_sorted::assert_eq;

    #[test]
    fn can_parse_subcomponent_basic() {
        let separators = Separators::default();

        let input = Span::new("foo");
        let actual = subcomponent_parser(input, separators).unwrap().1;
        assert_eq!(actual.value, "foo");
        assert_eq!(actual.range, 0..3);
    }

    #[test]
    fn can_parse_subcomponent_until_next_field() {
        let separators = Separators::default();

        let input = Span::new("foo^bar");
        let actual = subcomponent_parser(input, separators).unwrap().1;
        assert_eq!(actual.value, "foo");
        assert_eq!(actual.range, 0..3);

        let input = Span::new("foo|bar");
        let actual = subcomponent_parser(input, separators).unwrap().1;
        assert_eq!(actual.value, "foo");
        assert_eq!(actual.range, 0..3);

        let input = Span::new("foo\rbar");
        let actual = subcomponent_parser(input, separators).unwrap().1;
        assert_eq!(actual.value, "foo");
        assert_eq!(actual.range, 0..3);
    }

    #[test]
    fn can_parse_subcomponent_with_escape() {
        let separators = Separators::default();

        let input = Span::new(r"foo|bar\baz^qux");
        let actual = subcomponent_parser(input, separators).unwrap().1;
        assert_eq!(actual.value, "foo");
        assert_eq!(actual.range, 0..3);

        let input = Span::new(r"foo\|bar\\baz\^qux");
        let actual = subcomponent_parser(input, separators).unwrap().1;
        assert_eq!(actual.value, r"foo\|bar\\baz\^qux");
        assert_eq!(actual.range, 0..18);
    }
}
