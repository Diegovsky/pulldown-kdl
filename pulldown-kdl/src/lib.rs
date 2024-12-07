#![doc = include_str!("../../README.md")]
#![feature(let_chains)]
use std::borrow::Cow;
use std::ops::Range;
use std::str;

pub(crate) mod error;
pub(crate) mod parser;
pub(crate) mod prelude;
pub(crate) mod string;
pub(crate) mod utils;
pub(crate) mod value;

pub use error::{ParseError, ParseErrorCause};

use parser::Parse;
use prelude::*;
pub use string::KdlString;
use string::{is_equals, ParseString};
pub use value::KdlValue;

macro_rules! tdbg {
    ($expr:expr) => {{
        if cfg!(feature = "debug") {
            dbg!($expr)
        } else {
            $expr
        }
    }};
}

pub(crate) use tdbg;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum State {
    #[default]
    Initial,
    Document,
    NodeEntries,
    Final,
}
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum KdlNodeEntry<'text> {
    Argument(KdlValue<'text>),
    Property {
        key: KdlString<'text>,
        value: KdlValue<'text>,
    },
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Event<'text> {
    StartDocument,
    EndDocument,
    Indentation(usize),
    NodeName(KdlString<'text>),
    NodeEntry(KdlNodeEntry<'text>),
    NodeEnd { inline: bool },
}

pub type Text<'a> = Cow<'a, str>;

pub type Ranged<T> = (T, Range<usize>);
type Item<T> = Option<Ranged<T>>;
type ResultItem<T> = Result<Ranged<T>, ParseErrorCause>;
pub(crate) fn item<T>(t: T, r: Range<usize>) -> Item<T> {
    Some((t, r))
}

#[derive(Default, Clone, Copy)]
pub struct Parser<'text> {
    acc: Acc<'text>,
    document_depth: usize,
    state: State,
}

impl<'text> Parser<'text> {
    pub fn new(source: &'text str) -> Self {
        Self {
            acc: Acc::new(source),
            ..Default::default()
        }
    }

    fn peek_next_event(&mut self) -> Result<Item<Event<'text>>, ParseErrorCause> {
        // Looks for indentation
        if self.state != State::NodeEntries
            && let Some((ws, ws_range)) = tdbg!(self.acc.blankspace())
            && !ws_range.is_empty()
        {
            return Ok(item(Event::Indentation(ws), ws_range));
        }
        tdbg!(self.state);
        tdbg!(self.acc.remaining_text());
        match self.state {
            State::Initial => {
                self.set_state(State::Document);
                self.document_depth = 0;
                Ok(item(Event::StartDocument, 0..0))
            }
            State::Final => return Ok(None),
            State::Document => {
                // Check if the document has ended
                if let Some(((), range)) = self.check_end() {
                    self.end_document();
                    if self.is_root_document() {
                        self.set_state(State::Final);
                    }
                    return Ok(item(Event::EndDocument, range));
                }
                // TODO: parse type cast
                let (name, range) = self.acc.string().ok_or_cause(InvalidNodeName)?;
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
                        Err(NeedsMoreData)
                    };
                };
                let c_range = 0..1;
                if c == '{' {
                    self.set_state(State::Document);
                    self.start_document();
                    return Ok(item(Event::StartDocument, c_range));
                } else if string::is_newline(c) || c == '}' {
                    let c_range = 0..0;
                    self.set_state(State::Document);
                    return Ok(item(Event::NodeEnd { inline: false }, c_range));
                } else if c == ';' {
                    self.set_state(State::Document);
                    return Ok(item(Event::NodeEnd { inline: true }, c_range));
                }

                // TODO: parse type cast
                let (value, range) = self.acc.expect_value()?;
                self.acc.consume_range(&range);
                let mut sub = self.acc.sub_accumulator(0);
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
                            return Err(InvalidKey {
                                value: value.into_owned(),
                            })
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

    pub fn next_event_borrowed(&mut self) -> Result<Item<Event<'text>>, ParseError<'text>> {
        let mut evt = self.peek_next_event();
        if let Ok(Some((_evt, range))) = &mut evt {
            // Updates the range to be absolute
            *range = range.offset_by(self.acc.end);

            // Advances the current index past the parsed event.
            self.acc.set_end(range.end);
        }
        let evt = evt.map_err(|cause| ParseError {
            cause,
            at: self.acc.end,
            source: self.acc.base().into(),
        });
        tdbg!(evt)
    }

    pub fn next_event(&mut self) -> Result<Item<Event<'text>>, ParseError<'static>> {
        self.next_event_borrowed().map_err(|e| e.into_owned())
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
        if self.document_depth == 0 && rem.is_empty() {
            return item((), 0..rem.len());
        } else {
            let mut subacc = self.acc.sub_accumulator(0);
            subacc.consume_next_char().filter(|c| *c == '}')?;
            // strip trailing semicolon if it's there
            subacc.consume_whitespace().ok()?;
            if let Some(range) = subacc.expect_sequence(";").ok() {
                subacc.consume_range(&range);
            }
            return item((), 0..subacc.end);
        }
    }
}

impl<'text> std::iter::Iterator for Parser<'text> {
    type Item = Result<Ranged<Event<'text>>, ParseError<'static>>;
    fn next(&mut self) -> Option<Self::Item> {
        self.next_event().transpose()
    }
}

impl<'text> std::iter::FusedIterator for Parser<'text> {}
