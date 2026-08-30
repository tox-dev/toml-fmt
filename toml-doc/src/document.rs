//! The document: root key-values, table sections, and the trivia between them.

use std::fmt;

use crate::trivia::{LineEnding, Trivia, Ws};
use crate::value::{Key, KeyValue};

/// What closes out a line: spacing, an optional comment, and the line break.
///
/// Every line carries its own break, the last one included, so a line that moves closes where it
/// lands. Whether the file itself ends there is [`Document::ends_without_newline`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Trail<'a> {
    pub ws: Ws<'a>,
    pub comment: Option<crate::Comment<'a>>,
    pub ending: LineEnding,
}

impl Trail<'_> {
    pub(crate) fn write_into(&self, out: &mut String) {
        out.push_str(&self.ws);
        if let Some(comment) = &self.comment {
            out.push_str(comment);
        }
        out.push_str(self.ending.as_str());
    }
}

impl fmt::Display for Trail<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::spelled(|out| self.write_into(out)))
    }
}

/// A `key = value` line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry<'a> {
    pub lead: Trivia<'a>,
    pub indent: Ws<'a>,
    pub key_value: KeyValue<'a>,
    pub trail: Trail<'a>,
}

impl Entry<'_> {
    pub(crate) fn write_into(&self, out: &mut String) {
        self.lead.write_into(out);
        out.push_str(&self.indent);
        self.key_value.write_into(out);
        self.trail.write_into(out);
    }
}

impl fmt::Display for Entry<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::spelled(|out| self.write_into(out)))
    }
}

/// Whether a header opens a table or appends to an array of tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SectionKind {
    /// `[key]`
    Table,
    /// `[[key]]`
    ArrayOfTables,
}

impl SectionKind {
    const fn brackets(self) -> (&'static str, &'static str) {
        match self {
            Self::Table => ("[", "]"),
            Self::ArrayOfTables => ("[[", "]]"),
        }
    }
}

/// A `[key]` or `[[key]]` line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header<'a> {
    pub lead: Trivia<'a>,
    pub indent: Ws<'a>,
    pub kind: SectionKind,
    pub pre_key: Ws<'a>,
    pub key: Key<'a>,
    pub post_key: Ws<'a>,
    pub trail: Trail<'a>,
}

impl Header<'_> {
    pub(crate) fn write_into(&self, out: &mut String) {
        let (open, close) = self.kind.brackets();
        self.lead.write_into(out);
        out.push_str(&self.indent);
        out.push_str(open);
        out.push_str(&self.pre_key);
        self.key.write_into(out);
        out.push_str(&self.post_key);
        out.push_str(close);
        self.trail.write_into(out);
    }
}

impl fmt::Display for Header<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::spelled(|out| self.write_into(out)))
    }
}

/// A header with the entries written under it, the unit that moves when tables are reordered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section<'a> {
    pub header: Header<'a>,
    pub entries: Vec<Entry<'a>>,
}

impl Section<'_> {
    pub(crate) fn write_into(&self, out: &mut String) {
        self.header.write_into(out);
        for entry in &self.entries {
            entry.write_into(out);
        }
    }
}

impl fmt::Display for Section<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::spelled(|out| self.write_into(out)))
    }
}

/// A parsed TOML document.
///
/// Writing it back yields the source byte for byte until something is changed.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Document<'a> {
    /// Whether the source opened with a byte order mark, which TOML allows only at the very start.
    pub bom: bool,
    /// Whether the source ran out before its last line ended.
    ///
    /// Every line the document holds carries an ending of its own, so no item takes the end of the
    /// file with it when it moves; this is what says the file stops where it does.
    pub ends_without_newline: bool,
    /// Key-values written before the first header.
    pub root: Vec<Entry<'a>>,
    pub sections: Vec<Section<'a>>,
    /// Blank and comment lines after the last item, which no item can claim.
    pub trailing: Trivia<'a>,
}

impl Document<'_> {
    pub(crate) fn write_into(&self, out: &mut String) {
        if self.bom {
            out.push('\u{feff}');
        }
        for entry in &self.root {
            entry.write_into(out);
        }
        for section in &self.sections {
            section.write_into(out);
        }
        self.trailing.write_into(out);
        if !self.ends_without_newline {
            return;
        }
        // the line that closes the document keeps its ending while it is held, so it is written
        // without one only here, where the end of the file is
        for ending in [LineEnding::Crlf.as_str(), LineEnding::Lf.as_str()] {
            if let Some(rest) = out.strip_suffix(ending) {
                out.truncate(rest.len());
                return;
            }
        }
    }
}

impl fmt::Display for Document<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::spelled(|out| self.write_into(out)))
    }
}
