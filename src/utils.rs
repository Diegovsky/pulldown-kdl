use std::ops::{Index, Range};

use crate::ParseErrorCause;

pub(crate) fn first_char(seq: &[u8]) -> Option<char> {
    let len = utf8_byte_len(*seq.get(0)?) as usize;
    std::str::from_utf8(&seq[..len]).ok()?.chars().next()
}

pub(crate) fn utf8_byte_len(i: u8) -> u8 {
    if (i >> 7 & 1) == 0 {
        return 1;
    } else if i >> 5 == 0b110 {
        2
    } else if i >> 4 == 0b1110 {
        3
    } else if i >> 3 == 0b11110 {
        4
    } else {
        unreachable!("Invalid utf-8 sequence: {}", i)
    }
}

pub(crate) trait Buffer<'a> {
    fn base(&self) -> &'a str;
    fn end(&self) -> usize;
    fn set_end(&mut self, new_end: usize);

    fn buffer(&self) -> &dyn Buffer<'a>;

    fn advance_bytes(&mut self, amount: usize) {
        self.set_end(self.end() + amount);
    }

    fn next_char_len(&self) -> usize {
        match self.remaining_bytes().get(0) {
            Some(c) => utf8_byte_len(*c) as usize,
            _ => 0,
        }
    }

    fn consume_next_char(&mut self) -> Option<char> {
        let c = self.peek_char()?;
        self.advance_bytes(c.len_utf8());
        Some(c)
    }

    fn sub_accumulator(&self, offset: usize) -> Acc<'a> {
        Acc {
            base: &self.base()[(self.end() + offset)..],
            end: 0,
        }
    }

    fn consume_range(&mut self, range: &Range<usize>) {
        debug_assert_eq!(range.start, 0);
        self.advance_bytes(range.end);
    }

    fn expect_sequence(&self, seq: &'static str) -> Result<Range<usize>, ParseErrorCause> {
        if !self.remaining_text().starts_with(seq) {
            return Err(ParseErrorCause::ExpectedSequence { sequence: seq });
        }
        Ok(0..seq.len())
    }

    fn peek_byte(&self) -> Option<u8> {
        self.bytes().get(self.end()).copied()
    }

    fn peek_char(&self) -> Option<char> {
        first_char(self.remaining_bytes())
    }

    fn unconsume_char(&mut self, c: char) {
        self.set_end(self.end() - c.len_utf8());
        debug_assert_eq!(self.peek_char().unwrap(), c);
    }

    fn remaining_bytes(&self) -> &'a [u8] {
        &self.base().as_bytes()[self.end()..]
    }

    fn remaining_text(&self) -> &'a str {
        &self.base()[self.end()..]
    }

    fn bytes(&self) -> &'a [u8] {
        &self.base().as_bytes()[..self.end()]
    }

    fn text(&self) -> &'a str {
        &self.base()[..self.end()]
    }

    fn range(&self) -> Range<usize> {
        0..self.end()
    }
}

pub(crate) trait OptionExt<'text, T>: Sized {
    fn ok_or_cause(self, cause: ParseErrorCause) -> Result<T, ParseErrorCause>;
    fn ok_or_eof(self) -> Result<T, ParseErrorCause> {
        self.ok_or_cause(ParseErrorCause::NeedsMoreData)
    }
}

impl<'text, T> OptionExt<'text, T> for Option<T> {
    fn ok_or_cause(self, cause: ParseErrorCause) -> Result<T, ParseErrorCause> {
        self.ok_or_else(|| cause)
    }
}

#[derive(Clone, Debug, Default, Copy)]
pub(crate) struct Acc<'a> {
    pub(crate) base: &'a str,
    pub(crate) end: usize,
}

impl<'a> Acc<'a> {
    pub(crate) fn new(base: &'a str) -> Self {
        Self { base, end: 0 }
    }
}

impl<'a> Buffer<'a> for Acc<'a> {
    fn base(&self) -> &'a str {
        self.base
    }

    fn buffer(&self) -> &dyn Buffer<'a> {
        self
    }

    fn end(&self) -> usize {
        self.end
    }

    fn set_end(&mut self, new_end: usize) {
        self.end = new_end
    }
}

impl<'a> Index<usize> for Acc<'a> {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        &self.bytes()[index]
    }
}

pub(crate) trait RangeExt: Sized {
    fn offset_by(&mut self, i: usize) -> Self;
}

impl RangeExt for Range<usize> {
    fn offset_by(&mut self, i: usize) -> Self {
        (self.start + i)..(self.end + i)
    }
}
