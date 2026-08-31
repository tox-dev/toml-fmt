//! A mutable, format-preserving TOML document model.
//!
//! Parsing keeps every byte of the source, so writing an untouched [`Document`] back reproduces the
//! input exactly. Comments and blank lines attach to the item below them, which lets a formatter
//! reorder keys, tables and array members without moving trivia by hand.
//!
//! Lexing and grammar come from [`toml_parser`], which tracks TOML 1.1.0.
//!
//! ```
//! let source = "b = 2\na = 1\n";
//! let mut document = toml_doc::parse(source).unwrap();
//! document.root.sort_by(|left, right| left.key_value.key.to_string().cmp(&right.key_value.key.to_string()));
//! assert_eq!(document.to_string(), "a = 1\nb = 2\n");
//! ```

#![forbid(unsafe_code)]

mod build;
mod document;
mod text;
mod trivia;
mod validate;
mod value;

use std::fmt;
use std::ops::Range;

/// The text a writer spells, built rather than formatted.
///
/// Writing a document out cannot fail: every piece of it is text the parser read or a caller wrote
/// through a type that only takes what it names. Building the text says so, where formatting into
/// a sink would carry an error arm no writer here can take.
pub(crate) fn spelled(write: impl FnOnce(&mut String)) -> String {
    let mut out = String::new();
    write(&mut out);
    out
}

pub use crate::document::{Document, Entry, Header, Section, SectionKind, Trail};
pub use crate::text::{decode, decode_key, encode_basic, encode_key, fits_literal};
pub use crate::trivia::{Comment, LineEnding, Pad, Padding, Piece, Trivia, Ws};
pub use crate::value::{Array, InlineTable, Key, KeyPart, KeyValue, Member, Quoting, Repr, Value};

/// Why a source could not be parsed, with the byte range it was noticed at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub message: String,
    pub span: Range<usize>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at byte {}", self.message, self.span.start)
    }
}

impl std::error::Error for Error {}

/// Parse a TOML document, borrowing every unchanged run of text from `source`.
///
/// # Errors
///
/// Returns every syntax error the source contains, in the order they were found, and what the
/// document says that no reader could build a value from.
pub fn parse(source: &str) -> Result<Document<'_>, Vec<Error>> {
    let document = parse_syntax(source)?;
    // the grammar says how the file is written; this says whether what it writes is a document a
    // reader can build a value from
    validate::check(source, &document).map(|()| document)
}

/// How deep a value may nest.
///
/// TOML sets no limit, and reading, writing and dropping a value all walk it by calling themselves,
/// so a value nested past what a thread's stack holds would end the process rather than the read.
/// No file writes anything near this; the deepest in either corpus this is checked against is four.
pub const NESTING: usize = 256;

/// Where the source first opens a value past [`NESTING`], read from the tokens so a bracket inside
/// a string or a comment counts for nothing.
fn past_the_nesting(tokens: &[toml_parser::lexer::Token]) -> Option<std::ops::Range<usize>> {
    use toml_parser::lexer::TokenKind::{LeftCurlyBracket, LeftSquareBracket, RightCurlyBracket, RightSquareBracket};
    let mut depth: usize = 0;
    for token in tokens {
        match token.kind() {
            LeftSquareBracket | LeftCurlyBracket => {
                depth += 1;
                if depth > NESTING {
                    let span = token.span();
                    return Some(span.start()..span.end());
                }
            }
            RightSquareBracket | RightCurlyBracket => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    None
}

/// Read how a source is written, without asking whether what it writes is a document a reader can
/// build a value from.
///
/// This is for text a caller means to be read as written rather than as a document: a fragment, or
/// a document the caller is midway through rewriting. Everything else wants [`parse`].
///
/// # Errors
///
/// Returns every syntax error the source contains, in the order they were found.
pub fn parse_syntax(source: &str) -> Result<Document<'_>, Vec<Error>> {
    let mut events = Vec::new();
    let mut errors = Vec::new();
    let view = toml_parser::Source::new(source);
    let tokens = view.lex().into_vec();
    if let Some(at) = past_the_nesting(&tokens) {
        return Err(vec![Error {
            message: format!("the value is nested deeper than {NESTING} levels, which is more than this reads"),
            span: at,
        }]);
    }
    toml_parser::parser::parse_document(&tokens, &mut |event| events.push(event), &mut errors);
    decode_all(source, &events, &mut errors);

    let mut failures: Vec<Error> = errors.iter().map(|error| convert(error, source)).collect();
    // build even when the source is broken, so the builder sees the events a failed parse produced
    // and the caller hears every complaint from both halves at once
    match build::Builder::new(source, &events).document() {
        Ok(mut document) if failures.is_empty() => {
            document.bom = source.starts_with('\u{feff}');
            document.ends_without_newline = !source.is_empty() && !source.ends_with('\n');
            Ok(document)
        }
        Ok(_) => Err(failures),
        Err(build::Malformed) => {
            failures.push(Error {
                message: "unsupported syntax".to_owned(),
                span: 0..source.len(),
            });
            Err(failures)
        }
    }
}

/// `toml_parser` validates lazily: the grammar admits a lone `\r` as a line break, a control
/// character in a comment, an empty value after `=`, and any bytes at all inside a string, and only
/// decoding rejects them. A formatter that skipped this would rewrite broken input without saying so.
///
/// The encoding rides on the event rather than the span, so [`toml_parser::Source::get`] would hand
/// back a slice that decodes as an unquoted scalar and reject every string.
fn decode_all(source: &str, events: &[toml_parser::parser::Event], errors: &mut Vec<toml_parser::ParseError>) {
    use toml_parser::parser::EventKind::{Comment, Newline, Scalar, SimpleKey};

    for event in events {
        let kind = event.kind();
        if !matches!(kind, Newline | Comment | SimpleKey | Scalar) {
            continue;
        }
        let span = event.span();
        let text = source.get(span.start()..span.end()).unwrap_or_default();
        let raw = toml_parser::Raw::new_unchecked(text, event.encoding(), span);
        match kind {
            Newline => raw.decode_newline(errors),
            Comment => raw.decode_comment(errors),
            SimpleKey => raw.decode_key(&mut (), errors),
            _ => {
                let _ = raw.decode_scalar(&mut (), errors);
            }
        }
    }
}

fn convert(error: &toml_parser::ParseError, source: &str) -> Error {
    let context = error.context();
    let span = error.unexpected().or(context);
    Error {
        message: error.description().to_owned(),
        span: span.map_or(0..source.len(), |span| span.start()..span.end()),
    }
}
