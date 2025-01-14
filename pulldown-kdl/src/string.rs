use std::borrow::Cow;

use crate::prelude::*;
use crate::ParseResult;
use crate::Ranged;
use crate::{item, Item, Text};

pub(crate) const fn is_digit(c: char) -> bool {
    match c {
        '0'..='9' => true,
        _ => false,
    }
}

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

use ownable::IntoOwned;

#[derive(IntoOwned, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KdlString<'text> {
    pub string: Text<'text>,
}

impl<'text> KdlString<'text> {
    /// A const-enabled constructor that creates a [`KdlString`] from a string slice.
    pub const fn from_str(data: &'text str) -> Self {
        Self {
            string: Cow::Borrowed(data),
        }
    }
}

impl<'text> From<&'text str> for KdlString<'text> {
    fn from(data: &'text str) -> Self {
        Self::from_str(data)
    }
}

pub(crate) trait ParseString<'text>: Buffer<'text> {
    fn peek_whitespace(&self) -> Item<usize> {
        let amount = self
            .remaining_text()
            .chars()
            .take_while(|c| is_whitespace(*c))
            .count();
        item(amount, 0..amount)
    }

    fn peek_blankspace(&self) -> Item<usize> {
        let mut char_count = 0;
        let mut space_amount = 0;
        // look for the previously visited char to check if it was a newline
        //        if self
        //            .text()
        //            .chars()
        //            .last()
        //            .map(is_newline)
        //            .unwrap_or_default()
        //        {
        //            visited_newline = true;
        //        }
        for (count, c) in self.remaining_text().chars().enumerate() {
            if is_whitespace(c) {
                char_count = count + 1;
                if c == '\t' {
                    space_amount += 4;
                } else {
                    space_amount += 1;
                }
            } else if is_newline(c) {
                char_count = count + 1;
                space_amount = 0;
            } else {
                break;
            }
        }
        item(space_amount, 0..char_count)
    }

    fn consume_whitespace(&mut self) -> ParseResult<()> {
        let v = self.peek_whitespace().ok_or_eof()?;
        self.advance_bytes(v.0);
        Ok(())
    }

    fn peek_string(&self) -> ParseResult<Ranged<KdlString<'text>>> {
        let mut end_sequence = None;
        let mut acc = self.sub_accumulator();

        match acc.peek_char().ok_or(ParseErrorCause::NeedsMoreData)? {
            q @ '"' => end_sequence = Some(q),
            c if is_non_identifier(c) || is_digit(c) => {
                return Err(ParseErrorCause::InvalidStringCharacter { c })
            }
            _ => (),
        };

        acc.consume_next_char();

        while let Some(c) = acc.consume_next_char() {
            // TODO: handle escape sequences
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

        Ok((KdlString::from_str(acc.text()), acc.range()))
    }
}

impl<'text, B> ParseString<'text> for B where B: Buffer<'text> {}
