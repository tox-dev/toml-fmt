//! Making entries, arrays and sections that were never in the source.
//!
//! Everything built here comes out unspaced; the layout pass decides what the whitespace looks
//! like, so a builder only has to get the structure and the text right.

use toml_doc::{
    Array, Entry, Header, Key, KeyValue, LineEnding, Member, Padding, Repr, Section, SectionKind, Trail, Trivia, Value,
};

/// `key = "text"`.
#[must_use]
pub fn string_entry(key: &str, text: &str) -> Entry<'static> {
    entry(key, string(text))
}

/// `key = value`.
#[must_use]
pub fn entry<'a>(key: &str, value: Value<'a>) -> Entry<'a> {
    Entry {
        lead: Trivia::default(),
        indent: "".into(),
        key_value: key_value(key, value),
        trail: trail(),
    }
}

/// `key = value`, as a table written as a value holds one.
#[must_use]
pub fn key_value<'a>(key: &str, value: Value<'a>) -> KeyValue<'a> {
    KeyValue {
        key: Key::new(key.split('.')),
        pre_eq: " ".into(),
        post_eq: " ".into(),
        value,
    }
}

/// A basic string value.
#[must_use]
pub fn string(text: &str) -> Value<'static> {
    Value::Scalar(Repr::basic_string(text))
}

/// An array holding the given values.
#[must_use]
pub fn array<'a>(values: impl IntoIterator<Item = Value<'a>>) -> Value<'a> {
    Value::Array(Array {
        members: values.into_iter().map(member).collect(),
        trailing_comma: false,
        trailing: Padding::default(),
    })
}

/// One member of an array or inline table, with no spacing and no comma of its own.
#[must_use]
pub fn member<'a, T>(item: T) -> Member<'a, T> {
    Member {
        lead: Padding::default(),
        item,
        trail: Padding::default(),
        after: Padding::default(),
    }
}

/// A `[name]` or `[[name]]` section holding the given entries.
#[must_use]
pub fn section<'a>(name: &str, kind: SectionKind, entries: Vec<Entry<'a>>) -> Section<'a> {
    Section {
        header: Header {
            lead: Trivia::default(),
            indent: "".into(),
            kind,
            pre_key: "".into(),
            key: Key::new(name.split('.')),
            post_key: "".into(),
            trail: trail(),
        },
        entries,
    }
}

/// What ends a line the builder wrote: nothing after the value, and a newline.
fn trail() -> Trail<'static> {
    Trail {
        ws: "".into(),
        comment: None,
        ending: LineEnding::Lf,
    }
}
