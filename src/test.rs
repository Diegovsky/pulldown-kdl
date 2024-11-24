use crate::*;
use assert_matches::assert_matches;

fn parse(text: &str) -> Parser {
    return Parser::new(text);
}

#[test]
fn parse_string_ident() {
    let parser = parse("foo  rest");
    assert_eq!(
        parser.acc.string().unwrap(),
        (
            KdlString {
                string: Cow::from("foo"),
            },
            0..3,
        )
    )
}

#[test]
fn parse_string_ident_non_indent() {
    let mut parser = parse(" foo=");
    parser.acc.advance_bytes(1);
    assert_eq!(
        parser.acc.string().unwrap(),
        (KdlString::from_str("foo"), 0..3)
    )
}

#[test]
fn parse_string_dquote() {
    let parser = parse(r#""foo""#);
    assert_eq!(
        parser.acc.string().unwrap(),
        (
            KdlString {
                string: Cow::from(r#""foo""#),
            },
            0..5,
        )
    )
}

#[test]
fn parse_string_dquote2() {
    let parser = parse(r#""foo bar ?=/''" rest"#);
    assert_eq!(
        parser.acc.string().unwrap(),
        (
            KdlString {
                string: Cow::from(r#""foo bar ?=/''""#),
            },
            0..15,
        )
    )
}

#[track_caller]
fn parse_into_vec<'a>(input: &'a str) -> Vec<(Event<'a>, Range<usize>)> {
    let mut parser = parse(input);
    let mut v = vec![];
    loop {
        let evt = match parser.next_event() {
            Ok(Some(v)) => v,
            Err(e) if e.cause != ParseErrorCause::NeedsMoreData => {
                panic!("Error while parsing: {e:?}")
            }
            _ => break,
        };
        v.push(dbg!(evt))
    }
    v
}

type Item<I> = (I, Range<usize>);

const fn node_name(name: &str, range: Range<usize>) -> Item<Event> {
    (Event::NodeName(KdlString::from_str(name)), range)
}

const fn node_argument(value: &str, range: Range<usize>) -> Item<Event> {
    (
        Event::NodeEntry(KdlNodeEntry::Argument(KdlValue::String(
            KdlString::from_str(value),
        ))),
        range,
    )
}

const fn node_prop<'a>(key: &'a str, value: &'a str, range: Range<usize>) -> Item<Event<'a>> {
    (
        Event::NodeEntry(KdlNodeEntry::Property {
            key: KdlString::from_str(key),
            value: KdlValue::String(KdlString::from_str(value)),
        }),
        range,
    )
}

const fn document_end(range: Range<usize>) -> Item<Event<'static>> {
    (Event::EndDocument, range)
}

const fn node_end(inline: bool, range: Range<usize>) -> Item<Event<'static>> {
    (Event::NodeEnd { inline }, range)
}

const DOCUMENT_START: (Event<'static>, Range<usize>) = (Event::StartDocument, 0..0);

#[test]
fn parse_inline_node() {
    let events = parse_into_vec(r#"node arg prop=value"#);
    assert_eq!(
        events,
        vec![
            DOCUMENT_START,
            node_name("node", 0..4),
            node_argument("arg", 5..8),
            node_prop("prop", "value", 9..19),
            node_end(true, 19..19),
            document_end(19..19)
        ]
    );
}

#[test]
fn parse_inline_node2() {
    let events = parse_into_vec(r#"node arg prop=value; "#);
    assert_eq!(
        events,
        vec![
            DOCUMENT_START,
            node_name("node", 0..4),
            node_argument("arg", 5..8),
            node_prop("prop", "value", 9..19),
            node_end(true, 19..20),
            document_end(21..21)
        ]
    );
}

#[test]
fn parse_inline_node_args() {
    let events = parse_into_vec(r#"node arg                        "double quoted" third "#);
    assert_eq!(
        events,
        vec![
            DOCUMENT_START,
            node_name("node", 0..4),
            node_argument("arg", 5..8),
            node_argument(r#""double quoted""#, 32..47),
            node_argument("third", 48..53),
            node_end(true, 54..54),
            document_end(54..54)
        ]
    );
}

#[test]
fn parse_inline_node_props() {
    let events = parse_into_vec(r#"node "key 1"=val1 key2=" double quoted " key3=val3 "#);
    assert_eq!(
        events,
        vec![
            DOCUMENT_START,
            node_name("node", 0..4),
            node_prop(r#""key 1""#, "val1", 5..17),
            node_prop("key2", r#"" double quoted ""#, 18..40),
            node_prop("key3", "val3", 41..50),
            node_end(true, 51..51),
            document_end(51..51)
        ]
    );
}

#[test]
fn parse_inline_very_spaced_inline_end() {
    let events = parse_into_vec(
        r#"n                                                                      ;"#,
    );
    assert_eq!(
        events,
        vec![
            DOCUMENT_START,
            node_name("n", 0..1),
            node_end(true, 71..72),
            document_end(72..72)
        ]
    );
}

#[test]
fn parse_stringly_named_inline_node() {
    let events = parse_into_vec(r#""name with spaces" arg ;"#);
    assert_eq!(
        events,
        vec![
            DOCUMENT_START,
            node_name(r#""name with spaces""#, 0..18),
            node_argument("arg", 19..22),
            node_end(true, 23..24),
            document_end(24..24)
        ]
    );
}

#[test]
fn parse_root_multiple_inline() {
    let events = parse_into_vec(r#"node a; node b ; node c;"#);
    assert_eq!(
        events,
        vec![
            DOCUMENT_START,
            node_name("node", 0..4),
            node_argument("a", 5..6),
            node_end(true, 6..7),
            node_name("node", 8..12),
            node_argument("b", 13..14),
            node_end(true, 15..16),
            node_name("node", 17..21),
            node_argument("c", 22..23),
            node_end(true, 23..24),
            document_end(24..24)
        ]
    );
}
