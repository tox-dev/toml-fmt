//! Reading a string's characters and writing them back out.

use toml_doc::{Key, Repr, Value};

#[test]
fn escapes_resolve_to_the_characters_they_name() {
    let decoded: Vec<String> = [
        r#"a = "one\ttwo""#,
        r"a = 'one\ttwo'",
        "a = \"\"\"\nkeep\n\"\"\"",
        "a = '''\nraw\n'''",
        "a = 12",
    ]
    .iter()
    .map(|source| toml_doc::decode(&scalar(source)).expect("decodable"))
    .collect();

    assert_eq!(decoded, ["one\ttwo", r"one\ttwo", "keep\n", "raw\n", "12"]);
}

/// A repr the caller writes out is read back before it is held, so nothing downstream carries a
/// failure path for a token that says nothing.
#[test]
fn text_that_is_not_a_string_written_in_its_form_makes_no_repr() {
    use toml_doc::Quoting;

    let refused = [
        Repr::string(r#""\q""#, Quoting::Basic),
        Repr::string(r#""one" and "two""#, Quoting::Basic),
        Repr::string("ends in nothing", Quoting::Basic),
        Repr::string("\"", Quoting::Basic),
    ];

    assert_eq!(
        (
            refused.iter().filter(|held| held.is_ok()).count(),
            Repr::string(r#""ok""#, Quoting::Basic).map(|held| held.to_string()),
            Repr::string("'''one\ntwo'''", Quoting::MlLiteral).map(|held| held.to_string()),
        ),
        (0, Ok(String::from(r#""ok""#)), Ok(String::from("'''one\ntwo'''")))
    );
}

/// Text a literal string cannot hold comes back as a basic string rather than as something no
/// parser would read back.
#[test]
fn a_literal_string_that_cannot_hold_the_text_falls_back_to_escaping() {
    let repr = Repr::literal_string("can't");

    assert_eq!(repr.text(), "\"can't\"");
    assert_eq!(toml_doc::decode(&repr).expect("valid"), "can't");
}

#[test]
fn encoding_escapes_what_a_basic_string_cannot_hold() {
    assert_eq!(
        toml_doc::encode_basic("tab\there \"q\" \u{1}\u{8}\u{c}\r"),
        r#""tab\there \"q\" \u0001\b\f\r""#
    );
}

#[test]
fn a_quote_keeps_text_out_of_a_literal_string() {
    let allowed: Vec<bool> = ["plain", "it's", "tab\there", "bell\u{7}"]
        .iter()
        .map(|text| toml_doc::fits_literal(text))
        .collect();

    assert_eq!(allowed, [true, false, true, false]);
}

#[test]
fn a_round_trip_through_encode_and_decode_keeps_the_characters() {
    let text = "quote \" backslash \\ newline \n done";

    assert_eq!(toml_doc::decode(&Repr::basic_string(text)).expect("decodable"), text);
}

#[test]
fn dotted_keys_read_back_without_their_spacing() {
    let document = toml_doc::parse("[ tool . ruff . lint ]\n\"quoted key\" = 1\n").expect("valid source");

    assert_eq!(document.sections[0].header.key.path(), "tool.ruff.lint");
    assert_eq!(document.sections[0].entries[0].key_value.key.path(), "quoted key");
}

#[test]
fn a_built_key_quotes_only_what_needs_it() {
    assert_eq!(Key::new(["tool", "a b"]).to_string(), "tool.\"a b\"");
}

#[test]
fn a_literal_string_holds_its_text_unescaped() {
    assert_eq!(Repr::literal_string(r"c:\path").to_string(), r"'c:\path'");
}

/// A quoted segment holding a dot is one segment, so it is not the dotted path that reads the same.
#[test]
fn a_key_matches_a_dotted_name_segment_by_segment() {
    let document =
        toml_doc::parse(concat!("[a.b]\n", "[\"a.b\"]\n", "['c.d']\n", "[\"e\\u0066\"]\n",)).expect("valid source");
    let key = |at: usize| &document.sections[at].header.key;

    assert!(key(0).is_path("a.b"));
    assert!(!key(0).is_path("a"));
    assert!(!key(0).is_path("a.b.c"));
    assert!(!key(1).is_path("a.b"));
    assert!(!key(2).is_path("c.d"));
    assert!(key(3).is_path("ef"));
    assert!(!key(3).is_path("e\\u0066"));
}

/// Nothing that reads a key carries a path for one that names nothing, so building one says so at
/// once rather than at the first read.
#[test]
#[should_panic(expected = "a key names at least one segment")]
fn a_key_that_names_nothing_cannot_be_built() {
    let _ = Key::new(Vec::<&str>::new());
}

/// A name is written the way a key is read back: bare where TOML reads it, quoted where it does
/// not.
#[test]
fn a_name_is_written_as_the_key_it_stands_for() {
    let written: Vec<String> = ["plain", "a b", "a.b", "", "say \"it\""]
        .into_iter()
        .map(toml_doc::encode_key)
        .collect();

    assert_eq!(written, ["plain", "\"a b\"", "\"a.b\"", "\"\"", "\"say \\\"it\\\"\""]);
}

/// A token holds text that reads back, whichever form the file wrote it in, so reading one asks
/// nothing of the caller.
#[test]
fn a_token_reads_back_the_characters_it_stands_for() {
    let document = toml_doc::parse("bare = 1\nescaped = \"a\\tb\"\nliteral = 'a\\tb'\n").expect("valid source");
    let held = |at: usize| match &document.root[at].key_value.value {
        Value::Scalar(repr) => repr.decoded(),
        _ => panic!("the value is a scalar"),
    };

    assert_eq!(held(0), "1");
    assert_eq!(held(1), "a\tb");
    assert_eq!(held(2), "a\\tb");
}

fn scalar(source: &str) -> Repr<'static> {
    let document = toml_doc::parse(source).expect("valid source");
    let Value::Scalar(repr) = &document.root[0].key_value.value else {
        panic!("expected a scalar");
    };
    repr.clone().into_owned()
}
