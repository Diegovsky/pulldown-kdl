#![feature(let_chains)]
use std::borrow::Cow;
use std::ops::Range;
use std::str;

mod parser;
mod prelude;
mod string;
mod utils;

use parser::Parse;
use prelude::*;
pub use string::KdlString;
use string::{is_equals, is_whitespace, ParseString};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum State {
    #[default]
    Initial,
    Document,
    NodeEntries,
    Final,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KdlNodeEntry<'text> {
    Argument(KdlValue<'text>),
    Property {
        key: KdlString<'text>,
        value: KdlValue<'text>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event<'text> {
    StartDocument,
    EndDocument,
    Indentation(usize),
    NodeName(KdlString<'text>),
    NodeEntry(KdlNodeEntry<'text>),
    NodeEnd { inline: bool },
}

pub type Text<'a> = Cow<'a, str>;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum KdlValue<'text> {
    String(KdlString<'text>),
}

impl<'text> KdlValue<'text> {
    fn into_static(self) -> KdlValue<'static> {
        match self {
            KdlValue::String(val) => KdlValue::String(val.into_static()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseErrorCause {
    ExpectedString,
    ExpectedValue,
    ExpectedSequence { sequence: &'static str },
    InvalidKey { value: KdlValue<'static> },
    NeedsMoreData,
}

#[derive(Clone, Debug)]
pub struct ParseError {
    cause: ParseErrorCause,
    at: usize,
    end: Option<usize>,
}

impl ParseError {
    fn with_length(mut self, end: usize) -> Self {
        self.end = Some(self.at + end);
        self
    }
}

type Item<T> = Option<(T, Range<usize>)>;
type ResultItem<T> = Result<(T, Range<usize>), ParseError>;
pub(crate) fn item<T>(t: T, r: Range<usize>) -> Item<T> {
    Some((t, r))
}

#[derive(Default, Clone, Copy)]
pub struct Parser<'text> {
    acc: Acc<'text>,
    document_depth: usize,
    looking_for_newline: bool,
    state: State,
}

impl<'text> Parser<'text> {
    pub fn new(source: &'text str) -> Self {
        Self {
            acc: Acc::new(source),
            ..Default::default()
        }
    }

    fn peek_next_event(&mut self) -> Result<Item<Event<'text>>, ParseError> {
        // Looks for indentation after newline
        if self.looking_for_newline {
            self.looking_for_newline = false;
            let (ws, range) = self.acc.consume_whitespace()?;
            return Ok(item(Event::Indentation(ws), range));
        }
        match self.state {
            State::Initial => {
                self.set_state(State::Document);
                self.document_depth = 0;
                Ok(item(Event::StartDocument, 0..0))
            }
            State::Final => return Ok(None),
            State::Document => {
                let _ = self.acc.consume_whitespace()?;
                // Check if the document has ended
                if let Some(((), range)) = self.check_end() {
                    self.end_document();
                    if self.is_root_document() {
                        self.set_state(State::Final);
                    }
                    return Ok(item(Event::EndDocument, range));
                }
                // TODO: parse type cast
                let (name, range) = self.acc.string().ok_or_cause(&self.acc, ExpectedString)?;
                self.set_state(State::NodeEntries);
                Ok(item(Event::NodeName(name), range))
            }
            State::NodeEntries => {
                // check for children start
                let _ = self.acc.consume_whitespace()?;
                let Some(c) = self.acc.peek_char() else {
                    self.set_state(State::Document);
                    return if self.document_depth == 0 {
                        Ok(item(Event::NodeEnd { inline: true }, 0..0))
                    } else {
                        Err(self.acc.error(NeedsMoreData))
                    };
                };
                let c_range = 0..1;
                if c == '{' {
                    self.set_state(State::Document);
                    self.start_document();
                    return Ok(item(Event::StartDocument, c_range));
                } else if string::is_newline(c) {
                    self.set_state(State::Document);
                    return Ok(item(Event::NodeEnd { inline: false }, c_range));
                } else if c == ';' {
                    self.set_state(State::Document);
                    return Ok(item(Event::NodeEnd { inline: true }, c_range));
                }

                let mut sub = self.acc.sub_accumulator(0);
                // TODO: parse type cast
                let (value, range) = sub.expect_value()?;
                sub.consume_range(&range);
                if let Some(c) = sub.peek_char()
                    && is_equals(c)
                {
                    sub.consume_next_char();
                    // parse property
                    match value {
                        KdlValue::String(key) => {
                            let (value, value_range) = sub.expect_value()?;
                            sub.consume_range(&value_range);
                            return Ok(item(
                                Event::NodeEntry(KdlNodeEntry::Property { key, value }),
                                0..sub.end,
                            ));
                        }
                        _ => {
                            return Err(self.acc.error(InvalidKey {
                                value: value.into_static(),
                            }))
                        }
                    }
                }
                // parse argument
                return Ok(item(
                    Event::NodeEntry(KdlNodeEntry::Argument(value)),
                    0..sub.end,
                ));
            }
        }
    }

    pub fn next_event(&mut self) -> Result<Item<Event<'text>>, ParseError> {
        let mut evt = self.peek_next_event();
        if let Ok(Some((_evt, range))) = &mut evt {
            // Updates the range to be absolute
            *range = range.offset_by(self.acc.end);

            // Advances the current index past the parsed event.
            self.acc.set_end(range.end);
        }
        evt
    }

    fn start_document(&mut self) {
        self.document_depth += 1;
    }

    fn is_root_document(&self) -> bool {
        self.document_depth == 0
    }

    fn end_document(&mut self) {
        self.document_depth = self.document_depth.saturating_sub(1);
    }

    fn set_state(&mut self, new_state: State) {
        self.state = new_state;
    }

    fn check_end(&self) -> Item<()> {
        let rem = self.acc.remaining_text();
        if self.document_depth == 0 && rem.trim_end().is_empty() {
            return item((), 0..rem.trim_end().len());
        } else {
            return item((), 0..rem.find("}")?);
        }
    }
}

#[cfg(test)]
mod test;
