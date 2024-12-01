use std::{ffi::OsString, io::BufReader, path::Path};

use pulldown_kdl::{Event, KdlNodeEntry, KdlValue, Parser, Ranged};

enum Mode {
    Emit,
    Compare,
    Check,
}

struct Args {
    input_file: OsString,
    mode: Mode,
}

impl Args {
    fn cli() -> Result<Self, lexopt::Error> {
        use lexopt::prelude::*;
        let mut parser = lexopt::Parser::from_env();
        let mut input_file = None;
        let mut mode = Mode::Emit;
        while let Some(a) = parser.next()? {
            match a {
                Value(val) if input_file.is_none() => input_file = Some(val),
                Short('m') => {
                    mode = match parser.value()?.string()?.as_str() {
                        "emit" => Mode::Emit,
                        "check" => Mode::Check,
                        "compare" => Mode::Compare,
                        other => return Err(format!("Unexpected mode '{other}'"))?,
                    }
                }
                _ => return Err(a.unexpected()),
            }
        }
        Ok(Self {
            mode,
            input_file: input_file.ok_or("Missing filename")?,
        })
    }
}

type R = Result<(), Box<dyn std::error::Error>>;

fn emit(filename: &Path) -> R {
    let contents = std::fs::read_to_string(filename)?;
    let mut parser = pulldown_kdl::Parser::new(&contents);
    let mut events = vec![];
    while let Some(item) = parser.next_event()? {
        events.push(item);
    }
    let ron = serde_json::to_string_pretty(&events)?;
    std::fs::write(filename.with_extension("json"), ron)?;
    Ok(())
}

fn check(filename: &Path) -> R {
    let contents = std::fs::read_to_string(filename)?;
    let expected: Vec<Ranged<Event<'static>>> = serde_json::from_reader(BufReader::new(
        std::fs::File::open(filename.with_extension("json"))?,
    ))?;
    let mut depth = 0;
    // check if range in document corresponds to what is expected
    for (e, range) in expected {
        let expected = &contents[range];
        match e {
            Event::StartDocument => {
                if depth == 0 {
                    assert_eq!(expected, "");
                } else {
                    assert_eq!(expected, "{");
                }
                depth += 1;
            }
            Event::EndDocument => {
                depth -= 1;
                if depth == 0 {
                    assert_eq!(expected, "");
                } else {
                    assert_eq!(expected, "}");
                }
            }
            Event::NodeEnd { inline: true } => assert_eq!(expected, ";"),
            Event::NodeEnd { inline: false } => assert_eq!(expected, ""),
            Event::Indentation(_) => (), //nothing can be done
            Event::NodeName(name) => assert_eq!(expected, name.string),
            Event::NodeEntry(entry) => match entry {
                KdlNodeEntry::Argument(KdlValue::String(val)) => {
                    assert_eq!(expected, val.string)
                }
                KdlNodeEntry::Property {
                    key,
                    value: KdlValue::String(value),
                } => assert_eq!(expected, format!("{}={}", key.string, value.string)),
            },
        }
    }
    Ok(())
}

fn compare(filename: &Path) -> R {
    let contents = std::fs::read_to_string(filename)?;
    let expected: Vec<Ranged<Event<'static>>> = serde_json::from_reader(BufReader::new(
        std::fs::File::open(filename.with_extension("json"))?,
    ))?;
    let parser = pulldown_kdl::Parser::new(&contents);
    let generated = parser.collect::<Result<Vec<_>, _>>()?;
    assert_eq!(expected, generated);

    Ok(())
}

fn main() -> R {
    let args = Args::cli()?;
    let filename = Path::new(&args.input_file);

    match args.mode {
        Mode::Emit => emit(filename)?,
        Mode::Check => check(filename)?,
        Mode::Compare => compare(filename)?,
    }
    Ok(())
}
