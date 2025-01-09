use std::{any::Any, ffi::OsString, io::BufReader, path::Path};

use miette::IntoDiagnostic;
use pulldown_kdl::{Event, KdlNodeEntry, KdlValue, ParseError, Parser, Ranged};

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

#[derive(Debug)]
enum Error {
    ParseError(ParseError<'static>),
    Message(String),
    Other(Box<dyn std::error::Error>),
}

impl<E> From<E> for Error
where
    E: std::error::Error + Any + 'static,
{
    fn from(value: E) -> Self {
        if let Some(err) = (&value as &dyn Any).downcast_ref::<ParseError>() {
            Self::ParseError(err.clone())
        } else {
            Self::Other(value.into())
        }
    }
}

type R = Result<(), Error>;

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
    let mut failed = false;
    // check if range in document corresponds to what is expected
    for (e, range) in expected {
        let found = &contents[range.clone()];

        macro_rules! assert_expected {
            ($found:expr, $expected:expr) => {{
                let found = $found;
                let expected = $expected;
                if found != expected {
                    failed = true;
                    eprintln!("EXPECTED: {expected:?}");
                    eprintln!("FOUND: {found:?}");
                    eprintln!("{}:{}", range.start, range.end);
                    eprintln!("")
                }
            }};
        }
        match e {
            Event::StartDocument => {
                if depth == 0 {
                    assert_expected!(found, "");
                } else {
                    assert_expected!(found, "{");
                }
                depth += 1;
            }
            Event::EndDocument => {
                depth -= 1;
                if depth == 0 {
                    assert_expected!(found, "");
                } else {
                    assert_expected!(found, "}");
                }
            }
            Event::NodeEnd { inline: true } => assert_expected!(found, ";"),
            Event::NodeEnd { inline: false } => assert_expected!(found, ""),
            Event::Indentation(_) => (), //nothing can be done
            Event::NodeName(name) => assert_expected!(found, name.string),
            Event::NodeEntry(entry) => match entry {
                KdlNodeEntry::Argument(val) => match val {
                    KdlValue::String(val) => assert_expected!(found, val.string),
                    _ => todo!(),
                },
                KdlNodeEntry::Property { key, value } => match value {
                    KdlValue::String(value) => {
                        assert_expected!(found, format!("{}={}", key.string, value.string))
                    }
                    _ => todo!(),
                },
            },
        }
    }
    if failed {
        Err(Error::Message("Document failed to check".into()))
    } else {
        Ok(())
    }
}

fn compare(filename: &Path) -> R {
    let contents = std::fs::read_to_string(filename)?;
    let expected: Vec<Ranged<Event<'static>>> = serde_json::from_reader(BufReader::new(
        std::fs::File::open(filename.with_extension("json"))?,
    ))?;
    let parser = pulldown_kdl::Parser::new(&contents);
    let generated = parser
        .map(|res| res.map_err(|e| e.into_owned()))
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(expected, generated);

    Ok(())
}

fn main() -> miette::Result<()> {
    let args = Args::cli().into_diagnostic()?;
    let filename = Path::new(&args.input_file);

    let result = match args.mode {
        Mode::Emit => emit(filename),
        Mode::Check => check(filename),
        Mode::Compare => compare(filename),
    };
    match result {
        Ok(()) => (),
        Err(e) => {
            let e: &dyn std::fmt::Display = match e {
                Error::Message(ref msg) => msg,
                Error::Other(ref e) => e,
                Error::ParseError(parse_error) => Err(parse_error)?,
            };
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };
    Ok(())
}
