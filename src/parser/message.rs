use crate::{
    message::{Message, Segment},
    parser::{msh::msh, segment::segment},
};
use nom::{
    branch::alt, bytes::complete::tag, character::complete::char, combinator::opt,
    multi::separated_list0, sequence::preceded, IResult,
};
use nom_locate::position;

use super::Span;

pub fn message<'i>(
    lenient_newlines: bool,
) -> impl FnMut(Span<'i>) -> IResult<Span<'i>, Message<'i>> {
    move |i| parse_message(i, lenient_newlines)
}

fn parse_message(i: Span<'_>, lenient_newlines: bool) -> IResult<Span<'_>, Message<'_>> {
    let input_src = i.fragment();
    let (i, msh) = msh(lenient_newlines)(i)?;
    let mut separators = msh.separators;
    separators.lenient_newlines = lenient_newlines;
    let msh: Segment = msh.into();
    let (i, mut segments) = if lenient_newlines {
        preceded(
            opt(alt((tag("\r\n"), tag("\n"), tag("\r")))),
            separated_list0(
                alt((tag("\r\n"), tag("\n"), tag("\r"))),
                segment(separators),
            ),
        )(i)?
    } else {
        preceded(
            opt(char('\r')),
            separated_list0(char('\r'), segment(separators)),
        )(i)?
    };
    segments.insert(0, msh);
    let (i, pos_end) = position(i)?;
    let source = &input_src[..pos_end.location_offset()];

    Ok((
        i,
        Message {
            source,
            segments,
            separators,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_message() {
        let input = Span::new("MSH|^~\\&|EPIC|EPICADT|SMS|SMSADT|199912271408|CHARRIS|ADT^A04|1817457|D|2.5|\rEVN|A04|199912271408|||CHARRIS\rPID||0493575^^^2^ID 1|454721||DOE^JOHN^^^^|DOE^JOHN^^^^|19480203|M||B|254 MYSTREET AVE^^MYTOWN^OH^44123^USA||(216)123-4567|||M|NON|400003403~1129086|\rNK1||ROE^MARIE^^^^|SPO||(216)123-4567||EC|||||||||||||||||||||||||||\rPV1||O|168 ~219~C~PMA^^^^^^^^^||||277^ALLEN MYLASTNAME^BONNIE^^^^|||||||||| ||2688684|||||||||||||||||||||||||199912271408||||||002376853");

        let (_, message) = message(false)(input).unwrap();
        assert_eq!(message.segments.len(), 5);
        assert_eq!(message.segments[0].name, "MSH");
        assert_eq!(message.segments[1].name, "EVN");
        assert_eq!(message.segments[2].name, "PID");
        assert_eq!(message.segments[3].name, "NK1");
        assert_eq!(message.segments[4].name, "PV1");
        assert_eq!(message.segments[1].fields[4].raw_value(), "CHARRIS");
    }

    #[test]
    fn cant_parse_message_with_lf_instead_of_cr_strict() {
        let input = Span::new("MSH|^~\\&|EPIC|EPICADT|SMS|SMSADT|199912271408|CHARRIS|ADT^A04|1817457|D|2.5|\nEVN|A04|199912271408|||CHARRIS\nPID||0493575^^^2^ID 1|454721||DOE^JOHN^^^^|DOE^JOHN^^^^|19480203|M||B|254 MYSTREET AVE^^MYTOWN^OH^44123^USA||(216)123-4567|||M|NON|400003403~1129086|\nNK1||ROE^MARIE^^^^|SPO||(216)123-4567||EC|||||||||||||||||||||||||||\nPV1||O|168 ~219~C~PMA^^^^^^^^^||||277^ALLEN MYLASTNAME^BONNIE^^^^|||||||||| ||2688684|||||||||||||||||||||||||199912271408||||||002376853");

        let (_, message) = message(false)(input).unwrap();
        assert_eq!(message.segments.len(), 1);
    }

    #[test]
    fn can_parse_message_with_lf_instead_of_cr() {
        let input = Span::new("MSH|^~\\&|EPIC|EPICADT|SMS|SMSADT|199912271408|CHARRIS|ADT^A04|1817457|D|2.5|\nEVN|A04|199912271408|||CHARRIS\nPID||0493575^^^2^ID 1|454721||DOE^JOHN^^^^|DOE^JOHN^^^^|19480203|M||B|254 MYSTREET AVE^^MYTOWN^OH^44123^USA||(216)123-4567|||M|NON|400003403~1129086|\nNK1||ROE^MARIE^^^^|SPO||(216)123-4567||EC|||||||||||||||||||||||||||\nPV1||O|168 ~219~C~PMA^^^^^^^^^||||277^ALLEN MYLASTNAME^BONNIE^^^^|||||||||| ||2688684|||||||||||||||||||||||||199912271408||||||002376853");

        let (_, message) = message(true)(input).unwrap();
        assert_eq!(message.segments.len(), 5);
        assert_eq!(message.segments[0].name, "MSH");
        assert_eq!(message.segments[1].name, "EVN");
        assert_eq!(message.segments[2].name, "PID");
        assert_eq!(message.segments[3].name, "NK1");
        assert_eq!(message.segments[4].name, "PV1");
        assert_eq!(message.segments[1].fields[4].raw_value(), "CHARRIS");
    }

    #[test]
    fn can_parse_message_with_crlf_instead_of_cr() {
        let input = Span::new("MSH|^~\\&|EPIC|EPICADT|SMS|SMSADT|199912271408|CHARRIS|ADT^A04|1817457|D|2.5|\r\nEVN|A04|199912271408|||CHARRIS\r\nPID||0493575^^^2^ID 1|454721||DOE^JOHN^^^^|DOE^JOHN^^^^|19480203|M||B|254 MYSTREET AVE^^MYTOWN^OH^44123^USA||(216)123-4567|||M|NON|400003403~1129086|\r\nNK1||ROE^MARIE^^^^|SPO||(216)123-4567||EC|||||||||||||||||||||||||||\r\nPV1||O|168 ~219~C~PMA^^^^^^^^^||||277^ALLEN MYLASTNAME^BONNIE^^^^|||||||||| ||2688684|||||||||||||||||||||||||199912271408||||||002376853");

        let (_, message) = message(true)(input).unwrap();
        assert_eq!(message.segments.len(), 5);
        assert_eq!(message.segments[0].name, "MSH");
        assert_eq!(message.segments[1].name, "EVN");
        assert_eq!(message.segments[2].name, "PID");
        assert_eq!(message.segments[3].name, "NK1");
        assert_eq!(message.segments[4].name, "PV1");
        assert_eq!(message.segments[1].fields[4].raw_value(), "CHARRIS");
    }
}
