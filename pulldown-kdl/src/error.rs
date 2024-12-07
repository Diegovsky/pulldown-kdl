use std::borrow::Cow;

use miette::LabeledSpan;

use crate::value::KdlValue;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum ParseErrorCause {
    InvalidNodeName,
    ExpectedValue,
    ExpectedSequence { sequence: &'static str },
    InvalidKey { value: KdlValue<'static> },
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
        use ParseErrorCause::*;
        match &self.cause {
            ExpectedSequence { sequence } => write!(f, "Expected the sequence '{sequence}'"),
            ExpectedValue => write!(f, "Expected a value"),
            InvalidNodeName => write!(f, "Got an invalid node name"),
            InvalidKey { value } => write!(f, "Expected a valid string, but got a {value} instead"),
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
    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.source)
    }
}
