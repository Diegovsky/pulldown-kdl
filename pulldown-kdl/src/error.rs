use std::borrow::Cow;

use miette::LabeledSpan;

use crate::value::KdlValue;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Expected {
    LineEnd,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum ParseErrorCause {
    InvalidStringCharacter { c: char },
    InvalidKey { value: KdlValue<'static> },
    Expected(Expected),
    NeedsMoreData,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ParseError<'text> {
    pub cause: ParseErrorCause,
    pub at: usize,
    pub(crate) source: Cow<'text, str>,
}

impl<'text> ParseError<'text> {
    /// Converts the borrowed error into an owned one, eliminating the lifetime.
    pub fn into_owned(self) -> ParseError<'static> {
        ParseError {
            source: Cow::Owned(self.source.into_owned()),
            ..self
        }
    }
}

impl<'test> std::fmt::Display for ParseError<'test> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use self::Expected as Ex;
        use ParseErrorCause::*;
        match &self.cause {
            InvalidStringCharacter { c } => {
                write!(f, "Got an invalid character '{c}' while parsing a string")
            }
            InvalidKey { value } => write!(f, "Expected a valid string, but got a {value} instead"),
            ParseErrorCause::Expected(Ex::LineEnd) => {
                write!(f, "A line end (';' or newline) was expected")
            }
            NeedsMoreData => write!(f, "The source ended abrubtly"),
        }
    }
}

impl std::error::Error for ParseError<'_> {}

impl<'text> miette::Diagnostic for ParseError<'text> {
    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        Some(Box::new(
            vec![LabeledSpan::new(Some("here".into()), self.at, 1)].into_iter(),
        ))
    }
    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        if let ParseErrorCause::Expected(Expected::LineEnd) = self.cause {
            Some(Box::new("Insert a line break or a semicolon"))
        } else {
            None
        }
    }
    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.source)
    }
}
