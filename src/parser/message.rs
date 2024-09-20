use crate::{
    parser::{msh::msh, segment::segment},
    message::{Message, Segment},
};
use nom::{
    character::complete::char, combinator::opt, multi::separated_list0, sequence::preceded, IResult,
};

pub fn message<'i>() -> impl FnMut(&'i str) -> IResult<&'i str, Message<'i>> {
    move |i| parse_message(i)
}

fn parse_message<'i>(i: &'i str) -> IResult<&'i str, Message<'i>> {
    let (i, msh) = msh()(i)?;
    let separators = msh.separators;
    let msh: Segment = msh.into();
    let (i, mut segments) = preceded(
        opt(char('\r')),
        separated_list0(char('\r'), segment(separators)),
    )(i)?;
    segments.insert(0, msh);

    Ok((
        i,
        Message {
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
        let input = "MSH|^~\\&|EPIC|EPICADT|SMS|SMSADT|199912271408|CHARRIS|ADT^A04|1817457|D|2.5|\rEVN|A04|199912271408|||CHARRIS\rPID||0493575^^^2^ID 1|454721||DOE^JOHN^^^^|DOE^JOHN^^^^|19480203|M||B|254 MYSTREET AVE^^MYTOWN^OH^44123^USA||(216)123-4567|||M|NON|400003403~1129086|\rNK1||ROE^MARIE^^^^|SPO||(216)123-4567||EC|||||||||||||||||||||||||||\rPV1||O|168 ~219~C~PMA^^^^^^^^^||||277^ALLEN MYLASTNAME^BONNIE^^^^|||||||||| ||2688684|||||||||||||||||||||||||199912271408||||||002376853";

        let (_, message) = message()(input).unwrap();
        assert_eq!(message.segments.len(), 5);
        assert_eq!(message.segments[0].name, "MSH");
        assert_eq!(message.segments[1].name, "EVN");
        assert_eq!(message.segments[2].name, "PID");
        assert_eq!(message.segments[3].name, "NK1");
        assert_eq!(message.segments[4].name, "PV1");
        assert_eq!(message.segments[1].fields[4].value, "CHARRIS");
    }
}
