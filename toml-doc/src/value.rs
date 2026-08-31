//! Keys and values, each holding the source text that produced it.

use std::borrow::Cow;
use std::fmt;

use crate::trivia::{Pad, Padding, Ws};

/// The quoting a key or string was written with. Bare keys and non-string scalars carry none.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Quoting {
    /// `"..."`
    Basic,
    /// `'...'`
    Literal,
    /// `"""..."""`
    MlBasic,
    /// `'''...'''`
    MlLiteral,
}

impl Quoting {
    /// The delimiter the quoting opens and closes with.
    #[must_use]
    pub const fn delimiter(self) -> &'static str {
        match self {
            Self::Basic => "\"",
            Self::Literal => "'",
            Self::MlBasic => "\"\"\"",
            Self::MlLiteral => "'''",
        }
    }

    /// Whether the form may span several lines.
    #[must_use]
    pub const fn is_multiline(self) -> bool {
        matches!(self, Self::MlBasic | Self::MlLiteral)
    }
}

/// A key or scalar as it appears in the source, delimiters included.
///
/// The text and the quoting have to agree, so the two are set together: a `Basic` repr always
/// opens and closes with `"`, and an unquoted one carries no delimiters. Every constructor holds
/// that, which is what lets [`Repr::body`] and key decoding work without a fallible return.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repr<'a> {
    text: Cow<'a, str>,
    quoting: Option<Quoting>,
}

impl<'a> Repr<'a> {
    /// A repr for a token the parser has already read, delimiters included.
    ///
    /// # Panics
    ///
    /// If the text does not open and close with `quoting`'s delimiter.
    #[must_use]
    pub(crate) fn written(text: impl Into<Cow<'a, str>>, quoting: Option<Quoting>) -> Self {
        Self::parsed(text, quoting).expect("the text is written in that form")
    }

    /// [`Repr::written`], or `None` when the text does not open and close with `quoting`'s
    /// delimiter, as a token read out of a source that failed to parse may not.
    #[must_use]
    pub(crate) fn parsed(text: impl Into<Cow<'a, str>>, quoting: Option<Quoting>) -> Option<Self> {
        let text = text.into();
        if let Some(quoting) = quoting {
            let delimiter = quoting.delimiter();
            if text.len() < 2 * delimiter.len() || !text.starts_with(delimiter) || !text.ends_with(delimiter) {
                return None;
            }
        }
        Some(Self { text, quoting })
    }

    /// A string written out in `quoting`'s form, delimiters and escapes included, as a caller that
    /// spells a value itself writes one.
    ///
    /// The text is read back before it is held, so what comes back is a repr whose text says what
    /// it is written to say. Text that is not a string, and text a caller has rather than writes,
    /// go through [`Repr::basic_string`] or [`Repr::literal_string`] instead.
    ///
    /// # Errors
    ///
    /// Returns why the text is not a string written in that form, such as an escape that stands
    /// for nothing or a delimiter that closes it early.
    pub fn string(text: &str, quoting: Quoting) -> Result<Repr<'static>, crate::Error> {
        let held = Repr::parsed(Cow::Owned(text.to_owned()), Some(quoting)).ok_or_else(|| crate::Error {
            message: format!("the text is not written as {}", quoting.delimiter()),
            span: 0..text.len(),
        })?;
        crate::decode(&held).map(|_| held)
    }

    /// The source text, delimiters included.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// How the text is quoted, or `None` for a bare key or a non-string scalar.
    #[must_use]
    pub const fn quoting(&self) -> Option<Quoting> {
        self.quoting
    }

    /// The text inside the quotes, or the whole token when unquoted. Escape sequences stay as
    /// written.
    #[must_use]
    pub fn body(&'a self) -> &'a str {
        self.quoting.map_or(&self.text, |quoting| {
            let width = quoting.delimiter().len();
            &self.text[width..self.text.len() - width]
        })
    }
}

impl fmt::Display for Repr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

impl Repr<'_> {
    pub(crate) fn write_into(&self, out: &mut String) {
        out.push_str(&self.text);
    }
}

/// One dot-separated segment of a key, with the whitespace hugging its dots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyPart<'a> {
    /// Whitespace between the preceding dot and this segment; empty on the first segment.
    pub lead: Ws<'a>,
    /// The segment as written. Held privately: a key and a value read different grammars, and a
    /// scalar token put here would write a document no parser reads back.
    pub(crate) repr: Repr<'a>,
    /// Whitespace between this segment and the following dot; empty on the last segment.
    pub trail: Ws<'a>,
}

/// A simple or dotted key. Whitespace outside the dots belongs to the surrounding item.
///
/// A key names at least one segment, which every way of building or changing one holds to: nothing
/// that reads a key carries a path for one that names nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Key<'a> {
    pub(crate) parts: Vec<KeyPart<'a>>,
}

impl<'a> Key<'a> {
    /// The segments as written, in order.
    #[must_use]
    pub fn parts(&self) -> &[KeyPart<'a>] {
        &self.parts
    }

    /// The segments, to rewrite in place. A slice, so what a caller writes back names as much as
    /// what it was handed.
    pub fn parts_mut(&mut self) -> &mut [KeyPart<'a>] {
        &mut self.parts
    }

    /// A key made of parts already written, such as the ones another key was built from.
    ///
    /// # Panics
    ///
    /// If no part is given.
    #[must_use]
    pub fn from_parts(parts: Vec<KeyPart<'a>>) -> Self {
        assert!(!parts.is_empty(), "a key names at least one segment");
        Self { parts }
    }

    /// Write more segments after the ones the key already names.
    pub fn extend_parts(&mut self, parts: impl IntoIterator<Item = KeyPart<'a>>) {
        self.parts.extend(parts);
    }

    /// Write more segments ahead of the ones the key already names, as folding a table into its
    /// parent does.
    pub fn prepend_parts(&mut self, parts: impl IntoIterator<Item = KeyPart<'a>>) {
        let mut ahead: Vec<KeyPart<'a>> = parts.into_iter().collect();
        ahead.append(&mut self.parts);
        self.parts = ahead;
    }

    /// Take the first `width` segments out of the key and hand them back, as writing a table out of
    /// its parent does.
    ///
    /// # Panics
    ///
    /// If that would leave the key naming nothing.
    pub fn take_leading(&mut self, width: usize) -> Vec<KeyPart<'a>> {
        assert!(width < self.parts.len(), "a key names at least one segment");
        self.parts.drain(..width).collect()
    }
}

impl<'a> KeyPart<'a> {
    /// Write the segment as `name`, quoted where TOML needs it.
    pub fn set_name(&mut self, name: &str) {
        self.repr = Repr::key(name);
    }

    /// Write the segment as the quoted name `written` spells.
    ///
    /// A quoted key segment and a string value are written the same way, so a caller that has
    /// spelled one can hand it over here rather than through [`KeyPart::set_name`], which writes
    /// the name bare wherever TOML reads one bare.
    ///
    /// # Panics
    ///
    /// If `written` is not a quoted string on one line. A key reads a grammar of its own: a bare
    /// token spelled for a value names nothing a key can hold, and a name runs to the end of the
    /// line the key is written on.
    pub fn set_quoted(&mut self, written: Repr<'a>) {
        assert!(
            written.quoting().is_some_and(|quoting| !quoting.is_multiline()),
            "a key segment holds a name written on one line"
        );
        self.repr = written;
    }

    /// Whether the file wrote the segment in quotes.
    #[must_use]
    pub const fn is_quoted(&self) -> bool {
        self.repr.quoting().is_some()
    }

    /// The segment as written, quotes included.
    #[must_use]
    pub fn written(&self) -> &str {
        self.repr.text()
    }
}

impl Key<'_> {
    pub(crate) fn write_into(&self, out: &mut String) {
        for (index, part) in self.parts.iter().enumerate() {
            if index > 0 {
                out.push('.');
            }
            out.push_str(&part.lead);
            part.repr.write_into(out);
            out.push_str(&part.trail);
        }
    }
}

impl fmt::Display for Key<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::spelled(|out| self.write_into(out)))
    }
}

/// A value in any of TOML's forms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value<'a> {
    Scalar(Repr<'a>),
    Array(Array<'a>),
    InlineTable(InlineTable<'a>),
}

impl Value<'_> {
    pub(crate) fn write_into(&self, out: &mut String) {
        match self {
            Self::Scalar(repr) => repr.write_into(out),
            Self::Array(array) => array.write_into(out),
            Self::InlineTable(table) => table.write_into(out),
        }
    }
}

impl fmt::Display for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::spelled(|out| self.write_into(out)))
    }
}

/// A key bound to a value, shared by document entries and inline-table members.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyValue<'a> {
    pub key: Key<'a>,
    pub pre_eq: Ws<'a>,
    pub post_eq: Ws<'a>,
    pub value: Value<'a>,
}

impl KeyValue<'_> {
    pub(crate) fn write_into(&self, out: &mut String) {
        self.key.write_into(out);
        out.push_str(&self.pre_eq);
        out.push('=');
        out.push_str(&self.post_eq);
        self.value.write_into(out);
    }
}

impl fmt::Display for KeyValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::spelled(|out| self.write_into(out)))
    }
}

/// A member of an array or inline table.
///
/// It owns the spacing on either side of the comma that follows it, and so the comment that closes
/// its line, but not the comma itself: a comma sits between members, wherever they end up, so the
/// container is what writes one out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member<'a, T> {
    pub lead: Padding<'a>,
    pub item: T,
    /// Spacing between the member and the comma that follows it.
    pub trail: Padding<'a>,
    /// What follows that comma on the same line, which is where a trailing comment sits.
    pub after: Padding<'a>,
}

/// Write the members with the commas that sit between them, and the one that closes the last member
/// where the container is written to stay open.
fn write_members<T>(
    out: &mut String,
    members: &[Member<'_, T>],
    trailing_comma: bool,
    trailing: &Padding<'_>,
    write_item: impl Fn(&T, &mut String),
) {
    let last = members.len().saturating_sub(1);
    for (index, member) in members.iter().enumerate() {
        let following = members.get(index + 1).map_or(trailing, |next| &next.lead);
        member.lead.write_into(out);
        write_item(&member.item, out);
        member.trail.write_into(out);
        if index < last || trailing_comma {
            // the comma comes next, and a comment would read it as more of itself
            close_the_line(out, &member.trail, &Padding::default());
            out.push(',');
        } else if member.after.parts().is_empty() {
            close_the_line(out, &member.trail, following);
        } else {
            close_the_line(out, &member.trail, &member.after);
        }
        member.after.write_into(out);
        close_the_line(out, &member.after, following);
    }
}

/// End the line a comment left open, where what comes next does not already start on a new one.
///
/// A comment runs to the end of its line, so a comma or a closing bracket written after one would be
/// read as more of the comment. The line break that closed a comment sits in the padding that
/// follows it, which a member moving somewhere else takes away.
fn close_the_line(out: &mut String, padding: &Padding<'_>, next: &Padding<'_>) {
    if leaves_a_comment_open(padding) && !starts_a_line(next) {
        out.push('\n');
    }
}

fn leaves_a_comment_open(padding: &Padding<'_>) -> bool {
    padding
        .parts()
        .iter()
        .rev()
        .find_map(|part| match part {
            Pad::Comment(_) => Some(true),
            Pad::Newline(_) => Some(false),
            Pad::Space(_) => None,
        })
        .unwrap_or(false)
}

/// Whether the padding opens with a line break, which is what puts what follows it on a line of its
/// own. What closes a line sits at the end of the padding before it, so a break that is coming sits
/// at the front of this one.
fn starts_a_line(padding: &Padding<'_>) -> bool {
    matches!(padding.parts().first(), Some(Pad::Newline(_)))
}

/// `[ ... ]`, holding values and the padding before the closing bracket.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Array<'a> {
    pub members: Vec<Member<'a, Value<'a>>>,
    /// Whether a comma closes the last member, which is how a file says it means to stay open.
    pub trailing_comma: bool,
    pub trailing: Padding<'a>,
}

impl Array<'_> {
    pub(crate) fn write_into(&self, out: &mut String) {
        out.push('[');
        write_members(
            out,
            &self.members,
            self.trailing_comma,
            &self.trailing,
            Value::write_into,
        );
        self.trailing.write_into(out);
        close_the_line(out, &self.trailing, &Padding::default());
        out.push(']');
    }
}

impl fmt::Display for Array<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::spelled(|out| self.write_into(out)))
    }
}

/// `{ ... }`, holding key-values and the padding before the closing brace.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InlineTable<'a> {
    pub members: Vec<Member<'a, KeyValue<'a>>>,
    /// Whether a comma closes the last member, which is how a file says it means to stay open.
    pub trailing_comma: bool,
    pub trailing: Padding<'a>,
}

impl InlineTable<'_> {
    pub(crate) fn write_into(&self, out: &mut String) {
        out.push('{');
        write_members(
            out,
            &self.members,
            self.trailing_comma,
            &self.trailing,
            KeyValue::write_into,
        );
        self.trailing.write_into(out);
        close_the_line(out, &self.trailing, &Padding::default());
        out.push('}');
    }
}

impl fmt::Display for InlineTable<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::spelled(|out| self.write_into(out)))
    }
}
