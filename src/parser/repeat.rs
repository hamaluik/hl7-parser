use super::component::component;
use crate::{Component, Repeat, Separators};
use nom::{
    branch::alt,
    bytes::complete::{escaped, tag, take_while},
    character::complete::{alpha1, char, none_of, one_of},
    combinator::consumed,
    multi::separated_list0,
    sequence::terminated,
    IResult,
};

pub fn repeat<'i>(seps: Separators) -> impl FnMut(&'i str) -> IResult<&'i str, Repeat<'i>> {
    move |i| parse_repeat(i, seps)
}

fn parse_repeat<'i>(i: &'i str, seps: Separators) -> IResult<&'i str, Repeat<'i>> {
    let (i, (subc_src, v)) = consumed(separated_list0(char(seps.component), component(seps)))(i)?;

    let v = if v.len() == 1 {
        Repeat {
            value: v[0].value,
            components: vec![],
        }
    } else {
        Repeat {
            value: subc_src,
            components: v,
        }
    };
    Ok((i, v))
}

#[cfg(test)]
mod tests {
    use crate::Component;

    use super::*;

    #[test]
    fn can_parse_repeat_basic() {
        let separators = Separators::default();

        let input = "foo";
        let expected = Repeat {
            value: "foo",
            components: vec![],
        };
        let actual = parse_repeat(input, separators).unwrap().1;
        assert_eq!(expected, actual);
    }

    #[test]
    fn can_parse_component_with_subcomponents() {
        let separators = Separators::default();

        let input = "foo^bar";
        let expected = Repeat {
            value: "foo^bar",
            components: vec![
                Component {
                    value: "foo",
                    subcomponents: vec![],
                },
                Component {
                    value: "bar",
                    subcomponents: vec![],
                },
            ],
        };
        let actual = parse_repeat(input, separators).unwrap().1;
        assert_eq!(expected, actual);
    }

    #[test]
    fn can_parse_repeat_with_no_components_and_escaped_component_separator() {
        let separators = Separators::default();

        let input = r"foo\^bar";
        let expected = Repeat {
            value: r"foo\^bar",
            components: vec![],
        };
        let actual = parse_repeat(input, separators).unwrap().1;
        assert_eq!(expected, actual);
    }
}
