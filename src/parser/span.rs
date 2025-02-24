use nom::{Compare, Err, InputIter, InputLength, InputTake, InputTakeAtPosition, Offset, Slice};
use std::{
    ops::Deref,
    str::{CharIndices, Chars},
};

#[derive(Debug, Copy, Clone)]
pub struct Span<'i> {
    pub input: &'i str,
    pub offset: usize,
}

impl<'i> Span<'i> {
    pub fn new(data: &'i str) -> Self {
        Self {
            input: data,
            offset: 0,
        }
    }
}

impl<'a, 'b> Compare<&'b str> for Span<'a> {
    fn compare(&self, t: &'b str) -> nom::CompareResult {
        self.input.compare(t)
    }

    fn compare_no_case(&self, t: &'b str) -> nom::CompareResult {
        self.input.compare_no_case(t)
    }
}

impl<'i> InputIter for Span<'i> {
    type Item = char;
    type Iter = CharIndices<'i>;
    type IterElem = Chars<'i>;

    fn iter_indices(&self) -> Self::Iter {
        self.input.iter_indices()
    }

    fn iter_elements(&self) -> Self::IterElem {
        self.input.iter_elements()
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.input.position(predicate)
    }

    fn slice_index(&self, count: usize) -> Result<usize, nom::Needed> {
        self.input.slice_index(count)
    }
}

impl<'i> InputLength for Span<'i> {
    fn input_len(&self) -> usize {
        self.input.len()
    }
}

impl<'i> InputTake for Span<'i> {
    fn take(&self, count: usize) -> Self {
        self.slice(..count)
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        (self.slice(count..), self.slice(..count))
    }
}

impl<'i> InputTakeAtPosition for Span<'i> {
    type Item = char;

    fn split_at_position<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.input.position(predicate) {
            Some(n) => Ok(self.take_split(n)),
            None => Err(Err::Incomplete(nom::Needed::new(1))),
        }
    }

    fn split_at_position1<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
        _e: nom::error::ErrorKind,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.input.position(predicate) {
            Some(n) => Ok(self.take_split(n)),
            None => Err(Err::Incomplete(nom::Needed::new(1))),
        }
    }

    fn split_at_position_complete<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.split_at_position(predicate) {
            Err(Err::Incomplete(_)) => Ok(self.take_split(self.input_len())),
            res => res,
        }
    }

    fn split_at_position1_complete<P, E: nom::error::ParseError<Self>>(
        &self,
        predicate: P,
        e: nom::error::ErrorKind,
    ) -> nom::IResult<Self, Self, E>
    where
        P: Fn(Self::Item) -> bool,
    {
        match self.input.position(predicate) {
            Some(0) => Err(Err::Error(E::from_error_kind(*self, e))),
            Some(n) => Ok(self.take_split(n)),
            None => {
                if self.input.input_len() == 0 {
                    Err(Err::Error(E::from_error_kind(*self, e)))
                } else {
                    Ok(self.take_split(self.input_len()))
                }
            }
        }
    }
}

impl<'i> Offset for Span<'i> {
    fn offset(&self, second: &Self) -> usize {
        let a = self.offset;
        let b = second.offset;
        b - a
    }
}

impl<'i, R> Slice<R> for Span<'i>
where
    &'i str: Slice<R>,
{
    fn slice(&self, range: R) -> Self {
        let next_input = self.input.slice(range);
        let consumed_len = self.input.offset(next_input);
        if consumed_len == 0 {
            return Span {
                offset: self.offset,
                input: next_input,
            };
        }

        let next_offset = self.offset + consumed_len;
        Span {
            offset: next_offset,
            input: next_input,
        }
    }
}

impl<'i> From<&'i str> for Span<'i> {
    fn from(s: &'i str) -> Self {
        Span::new(s)
    }
}

impl Deref for Span<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.input
    }
}
