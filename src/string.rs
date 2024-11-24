use std::borrow::Cow;

use crate::prelude::*;
use crate::{item, Item, ParseError, Text};

pub(crate) const fn is_whitespace(c: char) -> bool {
    match c {
        '\u{0009}' | '\u{000B}' | '\u{0020}' | '\u{00A0}' | '\u{1680}' | '\u{2000}'
        | '\u{2001}' | '\u{2002}' | '\u{2003}' | '\u{2004}' | '\u{2005}' | '\u{2006}'
        | '\u{2007}' | '\u{2008}' | '\u{2009}' | '\u{200A}' | '\u{202F}' | '\u{205F}'
        | '\u{3000}' => true,
        _ => false,
    }
}

pub(crate) const fn is_equals(c: char) -> bool {
    match c {
        '=' | '﹦' | '＝' | '🟰' => true,
        _ => false,
    }
}

pub(crate) const fn is_newline(c: char) -> bool {
    match c {
        '\r' | '\n' => true,
        '\u{0085}' => true,
        '\u{000C}' => true,
        '\u{2028}' => true,
        '\u{2029}' => true,
        _ => false,
    }
}

pub(crate) const fn is_disallowed(c: char) -> bool {
    match c as u32 {
        0..=8 => true,
        0x7F => true,
        0xD800..=0xDFFF => true,
        // TODO: add more
        _ => false,
    }
}

pub(crate) const fn is_non_identifier(c: char) -> bool {
    match c {
        '(' | ')' | '{' | '}' | '[' | ']' | '/' | '\\' | '"' | '#' | ';' => true,
        c if is_equals(c) => true,
        c if is_whitespace(c) => true,
        c if is_newline(c) => true,
        _ => false,
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct KdlString<'text> {
    pub string: Text<'text>,
}

impl<'text> KdlString<'text> {
    pub(crate) fn into_static(self) -> KdlString<'static> {
        KdlString {
            string: self.string.into_owned().into(),
        }
    }

    pub const fn from_str(data: &'text str) -> Self {
        Self {
            string: Cow::Borrowed(data),
        }
    }
}

pub(crate) trait ParseString<'text>: Buffer<'text> {
    fn whitespace(&self) -> Item<usize> {
        let amount = self
            .remaining_text()
            .chars()
            .take_while(|c| is_whitespace(*c))
            .count();
        item(amount, 0..amount)
    }

    fn consume_whitespace(&mut self) -> Result<(usize, Range<usize>), ParseError> {
        let v = self.whitespace().ok_or_eof(self.buffer())?;
        self.advance_bytes(v.0);
        Ok(v)
    }

    fn string(&self) -> Item<KdlString<'text>> {
        let mut end_sequence = None;
        let mut acc = self.sub_accumulator(0);

        match acc.peek_char()? {
            q @ '"' => end_sequence = Some(q),
            c if is_non_identifier(c) => return None,
            _ => (),
        };

        acc.consume_next_char();

        while let Some(c) = acc.consume_next_char() {
            match end_sequence {
                // Dquoted string
                Some(end_sequence) if end_sequence == c => break,
                // Indentifier string
                None if is_non_identifier(c) => {
                    acc.unconsume_char(c);
                    break;
                }
                // If the char isn't special, keep consuming it.
                _ => (),
            }
        }

        item(KdlString::from_str(acc.text()), acc.range())
    }
}

impl<'text, B> ParseString<'text> for B where B: Buffer<'text> {}
