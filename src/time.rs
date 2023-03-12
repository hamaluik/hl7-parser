use crate::Span;
use time::OffsetDateTime;

pub fn parse_time<'s>(s: &'s str) -> Result<OffsetDateTime, nom::Err<nom::error::Error<Span<'s>>>> {
    let s = Span::new(s);
    todo!()
}
