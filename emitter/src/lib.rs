use std::io::{self, Write};

use pulldown_kdl::{Event, KdlNodeEntry, KdlString, KdlValue, Parser};

pub struct KdlEmitter<'input> {
    parser: Parser<'input>,
    space: bool,
}

impl<'input> KdlEmitter<'input> {
    pub fn new(parser: Parser<'input>) -> Self {
        Self {
            parser,
            space: false,
        }
    }

    fn emit_value(&self, value: KdlValue, writer: &mut dyn Write) -> io::Result<()> {
        match value {
            KdlValue::String(string) => self.emit_string(string, writer),
            _ => unreachable!(),
        }
    }

    fn emit_string(&self, string: KdlString, writer: &mut dyn Write) -> io::Result<()> {
        write!(writer, "{}", string.string)
    }

    fn signal_space(&mut self) {
        self.space = true;
    }

    pub fn emit(&mut self, writer: &mut dyn Write) -> Result<(), Box<dyn std::error::Error>> {
        while let Some((event, _)) = self.parser.next_event_borrowed()? {
            // Some elements have implicit spacing between them,
            // however, some elements shouldn't be preceded by space.
            // this code prevents these elements from having space before them.
            if self.space && !matches!(&event, Event::Indentation { .. } | Event::NodeEnd { .. }) {
                write!(writer, " ")?;
            }
            self.space = false;
            match event {
                Event::StartDocument => {
                    write!(writer, "{{")?;
                }
                Event::Indentation(amount) => write!(writer, "\n{}", " ".repeat(amount))?,
                Event::NodeName(name) => {
                    self.emit_string(name, writer)?;
                    self.signal_space();
                }
                Event::NodeEntry(entry) => {
                    match entry {
                        KdlNodeEntry::Argument(arg) => self.emit_value(arg, writer)?,
                        KdlNodeEntry::Property { key, value } => {
                            self.emit_string(key, writer)?;
                            write!(writer, "=")?;
                            self.emit_value(value, writer)?;
                        }
                    };
                    self.signal_space();
                }
                Event::NodeEnd { inline } => {
                    if inline {
                        write!(writer, ";")?;
                    }
                }
                Event::EndDocument => {
                    write!(writer, "}}")?;
                }
            }
        }
        Ok(())
    }
}
