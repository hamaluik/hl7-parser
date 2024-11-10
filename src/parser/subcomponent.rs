use crate::message::{Separators, Subcomponent};
use nom::{bytes::complete::take_till, IResult};

use super::Span;

pub fn subcomponent<'i>(
    seps: Separators,
) -> impl FnMut(Span<'i>) -> IResult<Span<'i>, Subcomponent<'i>> {
    move |i| subcomponent_parser(i, seps)
}

fn subcomponent_parser(i: Span, seps: Separators) -> IResult<Span, Subcomponent<'_>> {
    let pos_start = i.offset;

    let sep = if seps.lenient_newlines {
        &[
            seps.field,
            seps.component,
            '\n',
            '\r',
            seps.subcomponent,
            seps.repetition,
        ][..]
    } else {
        &[
            seps.field,
            seps.component,
            '\r',
            seps.subcomponent,
            seps.repetition,
        ][..]
    };
    let (i, v) = take_till(|c: char| sep.contains(&c))(i)?;

    let pos_end = i.offset;
    let value = v.input;

    Ok((
        i,
        Subcomponent {
            range: pos_start..pos_end,
            value,
        },
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
    fn can_parse_subcomponent_with_lenient_newlines() {
        let separators = Separators::default().with_lenient_newlines(true);

        let input = Span::new("foo\rbar");
        let (input, actual) = subcomponent_parser(input, separators).unwrap();
        assert_eq!(actual.value, "foo");
        assert_eq!(actual.range, 0..3);
        assert_eq!(input.input, "\rbar");

        let input = Span::new("foo\nbar");
        let (input, actual) = subcomponent_parser(input, separators).unwrap();
        assert_eq!(actual.value, "foo");
        assert_eq!(actual.range, 0..3);
        assert_eq!(input.input, "\nbar");

        let input = Span::new("foo\r\nbar");
        let (input, actual) = subcomponent_parser(input, separators).unwrap();
        assert_eq!(actual.value, "foo");
        assert_eq!(actual.range, 0..3);
        assert_eq!(input.input, "\r\nbar");
    }

    #[test]
    fn can_parse_subcomponent_escape() {
        let separators = Separators::default().with_lenient_newlines(true);

        let input = Span::new(r"foo\T\bar");
        let (input, actual) = subcomponent_parser(input, separators).expect("can parse");
        assert_eq!(actual.value, r"foo\T\bar");
        assert_eq!(actual.range, 0..9);
        assert_eq!(input.input, "");
    }
}
