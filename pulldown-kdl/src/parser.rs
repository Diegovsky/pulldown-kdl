use crate::{
    prelude::*, string::ParseString, utils::OptionExt, Item, KdlValue, ParseResult, Ranged,
};

pub(crate) trait Parse<'text>: Buffer<'text> + ParseString<'text> {
    fn peek_value(&self) -> ParseResult<Ranged<KdlValue<'text>>> {
        self.peek_string()
            .map(|(string, range)| (KdlValue::String(string), range))
    }

    fn consume_value(&mut self) -> ParseResult<Ranged<KdlValue<'text>>> {
        let item = self.peek_value()?;
        self.consume_range(&item.1);
        Ok(item)
    }
}

impl<'text, B> Parse<'text> for B where B: Buffer<'text> + ParseString<'text> {}
