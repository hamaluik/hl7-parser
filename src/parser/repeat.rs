use super::{component::component, Span};
use crate::message::{Repeat, Separators};
use nom::{character::complete::char, combinator::consumed, multi::separated_list0, IResult};

pub fn repeat<'i>(seps: Separators) -> impl FnMut(Span<'i>) -> IResult<Span<'i>, Repeat<'i>> {
    move |i| parse_repeat(i, seps)
}

fn parse_repeat(i: Span, seps: Separators) -> IResult<Span, Repeat> {
    let pos_start = i.location_offset();
    let (i, (repeat_src, v)) = consumed(separated_list0(char(seps.component), component(seps)))(i)?;
    let pos_end = i.location_offset();

    let v = Repeat {
        source: repeat_src.fragment(),
        components: v,
        range: pos_start..pos_end,
    };
    Ok((i, v))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions_sorted::assert_eq;

    #[test]
    fn can_parse_repeat_basic() {
        let separators = Separators::default();

        let input = Span::new("foo");
        let actual = parse_repeat(input, separators).unwrap().1;
        assert_eq!(actual.components.len(), 1);
        assert_eq!(actual.range, 0..3);
    }

    #[test]
    fn can_parse_repeat_with_components() {
        let separators = Separators::default();

        let input = Span::new("foo^bar");
        let actual = parse_repeat(input, separators).unwrap().1;
        assert_eq!(actual.components.len(), 2);
        assert_eq!(actual.range, 0..7);
    }

    #[test]
    fn can_parse_repeat_with_no_subcomponents_and_escaped_component_separator() {
        let separators = Separators::default();

        let input = Span::new(r"foo\^bar");
        let actual = parse_repeat(input, separators).unwrap().1;
        assert_eq!(actual.components.len(), 1);
        assert_eq!(actual.range, 0..8);
        assert_eq!(actual.components[0].subcomponents.len(), 1);
    }
}
