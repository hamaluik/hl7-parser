use super::{subcomponent::subcomponent, Span};
use crate::message::{Component, Separators};
use nom::{character::complete::char, combinator::consumed, multi::separated_list0, IResult};

pub fn component<'i>(seps: Separators) -> impl FnMut(Span<'i>) -> IResult<Span<'i>, Component<'i>> {
    move |i| parse_component(i, seps)
}

fn parse_component(i: Span<'_>, seps: Separators) -> IResult<Span<'_>, Component<'_>> {
    let pos_start = i.offset;
    let (i, (component_src, v)) =
        consumed(separated_list0(char(seps.subcomponent), subcomponent(seps)))(i)?;
    let pos_end = i.offset;

    let v = Component {
        source: component_src.input,
        subcomponents: v,
        range: pos_start..pos_end,
    };
    Ok((i, v))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions_sorted::assert_eq;

    #[test]
    fn can_parse_component_basic() {
        let separators = Separators::default();

        let input = Span::new("foo");
        let actual = parse_component(input, separators).unwrap().1;
        dbg!(&actual);
        assert_eq!(actual.subcomponents.len(), 1);
        assert_eq!(actual.display(&separators).to_string(), "foo");
        assert_eq!(actual.range, 0..3);
    }

    #[test]
    fn can_parse_component_with_subcomponents() {
        let separators = Separators::default();

        let input = Span::new("foo&bar");
        let actual = parse_component(input, separators).unwrap().1;
        dbg!(&actual);
        assert_eq!(actual.subcomponents.len(), 2);
        assert_eq!(actual.display(&separators).to_string(), "foo&bar");
        assert_eq!(actual.range, 0..7);
        assert_eq!(actual.subcomponents[0].value, "foo");
        assert_eq!(actual.subcomponents[1].value, "bar");
        assert_eq!(actual.subcomponents[0].range, 0..3);
        assert_eq!(actual.subcomponents[1].range, 4..7);
    }

    #[test]
    fn can_parse_component_with_no_subcomponents_and_escaped_subcomponent_separator() {
        let separators = Separators::default();

        let input = Span::new(r"foo\T\bar");
        let actual = parse_component(input, separators).unwrap().1;
        dbg!(&actual);
        assert_eq!(actual.subcomponents.len(), 1);
        assert_eq!(actual.subcomponents[0].value, r"foo\T\bar");
    }

    #[test]
    fn can_parse_component_at_end_of_line() {
        let separators = Separators::default();

        let input = Span::new("foo\rbar");
        let actual = parse_component(input, separators).unwrap().1;
        assert_eq!(actual.subcomponents.len(), 1);
        assert_eq!(actual.display(&separators).to_string(), "foo");
    }

    #[test]
    fn can_parse_component_in_field() {
        let separators = Separators::default();

        let input = Span::new("foo|bar");
        let actual = parse_component(input, separators).unwrap().1;
        assert_eq!(actual.subcomponents.len(), 1);
        assert_eq!(actual.display(&separators).to_string(), "foo");

        let input = Span::new("foo&bar|baz");
        let actual = parse_component(input, separators).unwrap().1;
        assert_eq!(actual.subcomponents.len(), 2);
        assert_eq!(actual.display(&separators).to_string(), "foo&bar");
        assert_eq!(actual.subcomponents[0].value, "foo");
        assert_eq!(actual.subcomponents[1].value, "bar");
    }
}
