//! Whitespace, comments and line breaks, kept verbatim so a document round-trips byte for byte.

use std::borrow::Cow;
use std::fmt;

/// Horizontal whitespace: spaces and tabs, as written.
///
/// A type of its own rather than the text itself, so nothing written into a spacing slot can run
/// two tokens together or close a line the document means to keep open.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Ws<'a>(Cow<'a, str>);

impl<'a> Ws<'a> {
    /// The spacing `text` writes.
    ///
    /// # Panics
    ///
    /// If the text holds anything but spaces and tabs.
    #[must_use]
    pub fn new(text: impl Into<Cow<'a, str>>) -> Self {
        Self::written(text).expect("spacing is written with spaces and tabs")
    }

    fn written(text: impl Into<Cow<'a, str>>) -> Option<Self> {
        let text = text.into();
        text.bytes()
            .all(|byte| byte == b' ' || byte == b'\t')
            .then_some(Self(text))
    }

    /// The spacing a parser read, held as it was read.
    ///
    /// The grammar decides what whitespace is, so whatever the parser read goes in as it was read,
    /// and a document round-trips byte for byte even where the source did not parse.
    pub(crate) fn read(text: impl Into<Cow<'a, str>>) -> Self {
        Self(text.into())
    }
}

impl<'a> From<&'a str> for Ws<'a> {
    fn from(text: &'a str) -> Self {
        Self::new(text)
    }
}

impl From<String> for Ws<'static> {
    fn from(text: String) -> Self {
        Ws::new(text)
    }
}

impl std::ops::Deref for Ws<'_> {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

/// A comment as written, `#` included.
///
/// A type of its own so nothing written into a comment slot can leave off the `#` that makes it
/// one, or carry the line break that ends it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Comment<'a>(Cow<'a, str>);

impl<'a> Comment<'a> {
    /// The comment `text` writes, `#` included.
    ///
    /// # Panics
    ///
    /// If the text does not open with `#`, or holds a line break.
    #[must_use]
    pub fn new(text: impl Into<Cow<'a, str>>) -> Self {
        Self::written(text).expect("a comment opens with # and runs to the end of its line")
    }

    fn written(text: impl Into<Cow<'a, str>>) -> Option<Self> {
        let text = text.into();
        (text.starts_with('#') && !text.contains(['\n', '\r'])).then_some(Self(text))
    }

    /// The comment a parser read, held as it was read.
    pub(crate) fn read(text: impl Into<Cow<'a, str>>) -> Self {
        Self(text.into())
    }
}

impl From<String> for Comment<'static> {
    fn from(text: String) -> Self {
        Comment::new(text)
    }
}

impl std::ops::Deref for Comment<'_> {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

/// A line break, held apart from other whitespace so a file's own ending survives a rewrite.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LineEnding {
    /// `\n`
    #[default]
    Lf,
    /// `\r\n`
    Crlf,
}

impl LineEnding {
    /// The characters this ending writes.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Lf => "\n",
            Self::Crlf => "\r\n",
        }
    }

    pub(crate) fn of(source: &str) -> Self {
        if source.starts_with('\r') { Self::Crlf } else { Self::Lf }
    }
}

/// A line carrying no key, table header or value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Piece<'a> {
    /// A line holding nothing but whitespace.
    Blank { indent: Ws<'a>, ending: LineEnding },
    /// A line holding only a comment.
    Comment {
        indent: Ws<'a>,
        text: Comment<'a>,
        ending: LineEnding,
    },
}

impl Piece<'_> {
    /// Whether the line is empty.
    #[must_use]
    pub const fn is_blank(&self) -> bool {
        matches!(self, Self::Blank { .. })
    }
}

/// The blank and comment lines that lead an item.
///
/// Trivia attaches to what follows it, so reordering items carries their comments along. Whatever
/// trails the last item belongs to [`Document::trailing`](crate::Document::trailing) instead.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Trivia<'a> {
    pieces: Vec<Piece<'a>>,
}

impl<'a> Trivia<'a> {
    /// The lines in source order.
    #[must_use]
    pub fn pieces(&self) -> &[Piece<'a>] {
        &self.pieces
    }

    /// The lines, for applying a blank-line or comment policy in place.
    pub const fn pieces_mut(&mut self) -> &mut Vec<Piece<'a>> {
        &mut self.pieces
    }

    /// Cap each run of blank lines at `max`.
    pub fn limit_blank_runs(&mut self, max: usize) {
        let mut run = 0_usize;
        self.pieces.retain(|piece| {
            if piece.is_blank() {
                run += 1;
                run <= max
            } else {
                run = 0;
                true
            }
        });
    }

    pub(crate) fn push(&mut self, piece: Piece<'a>) {
        self.pieces.push(piece);
    }
}

impl Trivia<'_> {
    pub(crate) fn write_into(&self, out: &mut String) {
        for piece in &self.pieces {
            let (indent, text, ending) = match piece {
                Piece::Blank { indent, ending } => (indent, None, ending),
                Piece::Comment { indent, text, ending } => (indent, Some(text), ending),
            };
            out.push_str(indent);
            if let Some(text) = text {
                out.push_str(text);
            }
            out.push_str(ending.as_str());
        }
    }
}

impl fmt::Display for Trivia<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::spelled(|out| self.write_into(out)))
    }
}

/// Whitespace, comments and line breaks inside an array or inline table, where the surrounding
/// container rather than the line start decides the layout.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Padding<'a> {
    parts: Vec<Pad<'a>>,
}

/// One element of a [`Padding`] run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pad<'a> {
    /// Spaces and tabs.
    Space(Ws<'a>),
    /// A comment.
    Comment(Comment<'a>),
    /// A line break.
    Newline(LineEnding),
}

impl<'a> Padding<'a> {
    /// The parts in source order.
    #[must_use]
    pub fn parts(&self) -> &[Pad<'a>] {
        &self.parts
    }

    /// The parts, for rewriting spacing in place.
    pub const fn parts_mut(&mut self) -> &mut Vec<Pad<'a>> {
        &mut self.parts
    }

    /// Whether the run forces its container across several lines.
    #[must_use]
    pub fn is_multiline(&self) -> bool {
        self.parts
            .iter()
            .any(|part| matches!(part, Pad::Newline(_) | Pad::Comment(_)))
    }

    /// Whether the run holds a comment, which no single line could keep.
    #[must_use]
    pub fn has_comment(&self) -> bool {
        self.parts.iter().any(|part| matches!(part, Pad::Comment(_)))
    }

    /// Cap each run of empty lines at `max`.
    ///
    /// A blank line inside a container is a line break, whatever spacing follows it, and another
    /// break; a comment closes the run, since the line it opens is not empty.
    pub fn limit_blank_runs(&mut self, max: usize) {
        let mut breaks = 0_usize;
        let mut kept = Vec::with_capacity(self.parts.len());
        for (at, part) in self.parts.iter().enumerate() {
            match part {
                Pad::Newline(_) => {
                    breaks += 1;
                    if breaks > max + 1 {
                        continue;
                    }
                }
                // spacing that only pads a line being dropped goes with that line; what indents
                // whatever comes next stays
                Pad::Space(_) if matches!(self.parts.get(at + 1), Some(Pad::Newline(_))) => {
                    if breaks > max {
                        continue;
                    }
                }
                Pad::Space(_) => {}
                Pad::Comment(_) => breaks = 0,
            }
            kept.push(part.clone());
        }
        self.parts = kept;
    }

    pub(crate) fn push(&mut self, part: Pad<'a>) {
        self.parts.push(part);
    }
}

impl Padding<'_> {
    pub(crate) fn write_into(&self, out: &mut String) {
        for part in &self.parts {
            match part {
                Pad::Space(space) => out.push_str(space),
                Pad::Comment(text) => out.push_str(text),
                Pad::Newline(ending) => out.push_str(ending.as_str()),
            }
        }
    }
}

impl fmt::Display for Padding<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&crate::spelled(|out| self.write_into(out)))
    }
}
