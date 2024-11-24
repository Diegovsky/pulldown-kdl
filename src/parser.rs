use crate::{prelude::*, string::ParseString, utils::OptionExt, Item, KdlValue, ResultItem};

pub(crate) trait Parse<'text>: Buffer<'text> + ParseString<'text> {
    fn value(&self) -> Item<KdlValue<'text>> {
        self.string()
            .map(|(string, range)| (KdlValue::String(string), range))
    }

    fn expect_value(&self) -> ResultItem<KdlValue<'text>> {
        self.value().ok_or_cause(self.buffer(), ExpectedValue)
    }
}

impl<'text, B> Parse<'text> for B where B: Buffer<'text> + ParseString<'text> {}
