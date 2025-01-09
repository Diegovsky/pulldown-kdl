use crate::{
    prelude::*, string::ParseString, utils::OptionExt, Item, KdlValue, ParseResult, Ranged,
};

pub(crate) trait Parse<'text>: Buffer<'text> + ParseString<'text> {
    fn peek_value(&self) -> ParseResult<Ranged<KdlValue<'text>>> {
        self.peek_string()
            .map(|(string, range)| (KdlValue::String(string), range))
    }

    fn consume_value(&mut self) -> ParseResult<KdlValue<'text>> {
        let (value, range) = self.peek_value()?;
        self.consume_range(&range);
        Ok(value)
    }
}

impl<'text, B> Parse<'text> for B where B: Buffer<'text> + ParseString<'text> {}
