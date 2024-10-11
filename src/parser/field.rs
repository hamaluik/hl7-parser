use super::{repeat::repeat, Span};
use crate::message::{Field, Separators};
use nom::{character::complete::char, combinator::consumed, multi::separated_list0, IResult};

pub fn field<'i>(seps: Separators) -> impl FnMut(Span<'i>) -> IResult<Span<'i>, Field<'i>> {
    move |i| parse_field(i, seps)
}

fn parse_field(i: Span, seps: Separators) -> IResult<Span, Field> {
    let pos_start = i.offset;
    let (i, (field_src, v)) = consumed(separated_list0(char(seps.repetition), repeat(seps)))(i)?;
    let pos_end = i.offset;

    let v = Field {
        source: field_src.input,
        repeats: v,
        range: pos_start..pos_end,
    };
    Ok((i, v))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions_sorted::assert_eq;

    #[test]
    fn can_parse_field_basic() {
        let separators = Separators::default();

        let input = Span::new("foo");
        let actual = parse_field(input, separators).unwrap().1;
        assert_eq!(actual.repeats.len(), 1);
        assert_eq!(actual.range, 0..3);
    }

    #[test]
    fn can_parse_field_with_repeats() {
        let separators = Separators::default();

        let input = Span::new("foo~bar");
        let actual = parse_field(input, separators).unwrap().1;
        assert_eq!(actual.repeats.len(), 2);
        assert_eq!(actual.range, 0..7);
    }

    #[test]
    fn can_parse_field_with_no_repeats_but_components() {
        let separators = Separators::default();

        let input = Span::new("foo^bar");
        let actual = parse_field(input, separators).unwrap().1;
        assert_eq!(actual.repeats.len(), 1);
        assert_eq!(actual.range, 0..7);
        assert_eq!(actual.repeats[0].components.len(), 2);
        assert_eq!(actual.repeats[0].range, 0..7);
        assert_eq!(actual.repeats[0].components[0].subcomponents.len(), 1);
        assert_eq!(actual.repeats[0].components[1].subcomponents.len(), 1);
    }
}
