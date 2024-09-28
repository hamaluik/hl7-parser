use super::{repeat::repeat, Span};
use crate::message::{Field, Separators};
use nom::{character::complete::char, combinator::consumed, multi::separated_list0, IResult};
use nom_locate::position;

pub fn field<'i>(seps: Separators) -> impl FnMut(Span<'i>) -> IResult<Span<'i>, Field<'i>> {
    move |i| parse_field(i, seps)
}

fn parse_field(i: Span, seps: Separators) -> IResult<Span, Field> {
    let (i, pos_start) = position(i)?;
    let (i, (field_src, v)) = consumed(separated_list0(char(seps.repetition), repeat(seps)))(i)?;
    let (i, pos_end) = position(i)?;

    let v = Field {
        source: field_src.fragment(),
        repeats: v,
        range: pos_start.location_offset()..pos_end.location_offset(),
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
