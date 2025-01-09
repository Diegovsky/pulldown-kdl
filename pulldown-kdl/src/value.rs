use std::fmt::Display;

use crate::KdlString;
use ownable::IntoOwned;

#[derive(IntoOwned, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum KdlValue<'text> {
    String(KdlString<'text>),
    Num(f64),
    Bool(bool),
    Null,
}

use KdlValue::*;

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
