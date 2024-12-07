use std::fmt::Display;

use crate::KdlString;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum KdlValue<'text> {
    String(KdlString<'text>),
    Num(f64),
    Bool(bool),
    Null,
}

use KdlValue::*;

impl<'text> KdlValue<'text> {
    /// Converts the borrowed value into an owned one, eliminating the lifetime.
    pub fn into_owned(self) -> KdlValue<'static> {
        match self {
            String(val) => KdlValue::String(val.into_static()),
            Num(v) => Num(v),
            Bool(v) => Bool(v),
            Null => Null,
        }
    }
}

impl<'text> Display for KdlValue<'text> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            String(string) => string.string.fmt(f),
            Num(v) => v.fmt(f),
            Bool(v) => v.fmt(f),
            Null => write!(f, "null"),
        }
    }
}
