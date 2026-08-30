//! Moving between a string's source form and the characters it stands for.

use std::borrow::Cow;
use std::fmt::Write as _;

use crate::Ws;
use crate::value::{Quoting, Repr};

impl crate::value::Key<'_> {
    /// The dotted key with its whitespace dropped, as `tool.ruff.lint`.
    ///
    /// A quoted segment holding a dot reads back as two segments here; use [`Key::segments`] where
    /// that difference matters.
    ///
    /// # Panics
    ///
    /// See [`Key::segments`].
    #[must_use]
    pub fn path(&self) -> String {
        self.segments().join(".")
    }

    /// The key's segments with their quoting resolved, so a dot inside a quoted segment stays part
    /// of that segment rather than splitting it.
    ///
    /// # Panics
    ///
    /// If a segment's text is not a valid TOML key. Parsing rejects such a key, and [`Key::new`]
    /// quotes what needs quoting, so only a hand-built [`Repr`] can carry one.
    #[must_use]
    pub fn segments(&self) -> Vec<String> {
        self.parts
            .iter()
            .map(|part| decode_key(&part.repr).expect("a key part holds a valid TOML key"))
            .collect()
    }

    /// Whether the key names exactly the segments of `name`, which is read as a dotted path of
    /// bare names.
    ///
    /// A quoted segment holding a dot is one segment, so `[a."b.c"]` is not `a.b.c`. Comparing
    /// segment by segment is also what keeps this off the allocations [`Key::path`] makes.
    #[must_use]
    pub fn is_path(&self, name: &str) -> bool {
        let mut wanted = name.split('.');
        self.parts
            .iter()
            .all(|part| wanted.next().is_some_and(|want| part.holds(want)))
            && wanted.next().is_none()
    }

    /// Whether the key opens with these segments, read without building the name to find out.
    #[must_use]
    pub fn opens_with(&self, segments: &[String]) -> bool {
        segments.len() <= self.parts.len() && segments.iter().zip(&self.parts).all(|(want, part)| part.holds(want))
    }

    /// A key made of the given segments, quoted where TOML needs it.
    ///
    /// # Panics
    ///
    /// If no segment is given. A key names something, and nothing that reads one carries a path
    /// for a key that names nothing.
    #[must_use]
    pub fn new<'b>(segments: impl IntoIterator<Item = &'b str>) -> crate::value::Key<'static> {
        let parts: Vec<crate::value::KeyPart<'static>> = segments
            .into_iter()
            .map(|segment| crate::value::KeyPart {
                lead: Ws::default(),
                repr: Repr::key(segment),
                trail: Ws::default(),
            })
            .collect();
        assert!(!parts.is_empty(), "a key names at least one segment");
        crate::value::Key { parts }
    }
}

impl crate::value::KeyPart<'_> {
    /// Whether the segment stands for `want`, read without building a `String` where the text
    /// already is what it stands for.
    pub(crate) fn holds(&self, want: &str) -> bool {
        match self.repr.quoting() {
            None | Some(Quoting::Literal | Quoting::MlLiteral) => self.repr.body() == want,
            Some(Quoting::Basic | Quoting::MlBasic) if !self.repr.body().contains('\\') => self.repr.body() == want,
            Some(_) => decode_key(&self.repr).expect("a key part holds a valid TOML key") == want,
        }
    }
}

impl Repr<'_> {
    /// A bare key, or a quoted one when `name` holds anything outside `A-Za-z0-9_-`.
    ///
    /// Bare text is a key and nothing else, so this stays inside the crate: a caller names a
    /// segment through [`crate::KeyPart::set_name`] or [`crate::Key::new`], and nothing it can
    /// build is a token one grammar reads and the other does not.
    #[must_use]
    pub(crate) fn key(name: &str) -> Repr<'static> {
        if !name.is_empty()
            && name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
        {
            return Repr::written(Cow::Owned(name.to_owned()), None);
        }
        Repr::basic_string(name)
    }

    /// A basic string holding `text`, escaped as TOML requires.
    #[must_use]
    pub fn basic_string(text: &str) -> Repr<'static> {
        Repr::written(Cow::Owned(encode_basic(text)), Some(Quoting::Basic))
    }

    /// A literal string holding `text`, which carries no escapes of its own.
    ///
    /// Text a literal string cannot hold, per [`fits_literal`], comes back as a basic string
    /// instead: the value is what matters, and the form is what the writer can spell it in.
    #[must_use]
    pub fn literal_string(text: &str) -> Repr<'static> {
        if fits_literal(text) {
            return Repr::written(Cow::Owned(format!("'{text}'")), Some(Quoting::Literal));
        }
        Repr::basic_string(text)
    }

    /// Detach from the source so the value outlives the text it was parsed from.
    #[must_use]
    pub fn into_owned(self) -> Repr<'static> {
        Repr::written(Cow::Owned(self.text().to_owned()), self.quoting())
    }
}

impl Repr<'_> {
    /// The characters the token stands for, with escape sequences resolved.
    ///
    /// Every repr is one a parser read or a constructor spelled, and both hold text that reads
    /// back, so nothing here is left for a caller to handle.
    ///
    /// # Panics
    ///
    /// If the token does not read back, which no repr this crate can build holds.
    #[must_use]
    pub fn decoded(&self) -> String {
        decode(self).expect("a repr holds text that reads back")
    }
}

/// The characters a key or string stands for, with escape sequences resolved.
///
/// # Errors
///
/// Returns the reason the source form is not a valid TOML string, such as an unknown escape or a
/// code point outside Unicode.
pub fn decode(repr: &Repr<'_>) -> Result<String, crate::Error> {
    decode_with(repr, |raw, out, errors| {
        let _ = raw.decode_scalar(out, errors);
    })
}

/// The characters a key stands for. A bare key decodes to itself, where the same text read as a
/// value would be a number or a boolean.
///
/// # Errors
///
/// Returns the reason the source form is not a valid TOML key.
pub fn decode_key(repr: &Repr<'_>) -> Result<String, crate::Error> {
    decode_with(repr, |raw, out, errors| raw.decode_key(out, errors))
}

fn decode_with(
    repr: &Repr<'_>,
    run: impl Fn(&toml_parser::Raw<'_>, &mut String, &mut Vec<toml_parser::ParseError>),
) -> Result<String, crate::Error> {
    let mut decoded = String::new();
    let mut errors = Vec::new();
    let span = toml_parser::Span::new_unchecked(0, repr.text().len());
    let encoding = repr.quoting().map(|quoting| match quoting {
        Quoting::Basic => toml_parser::decoder::Encoding::BasicString,
        Quoting::Literal => toml_parser::decoder::Encoding::LiteralString,
        Quoting::MlBasic => toml_parser::decoder::Encoding::MlBasicString,
        Quoting::MlLiteral => toml_parser::decoder::Encoding::MlLiteralString,
    });
    run(
        &toml_parser::Raw::new_unchecked(repr.text(), encoding, span),
        &mut decoded,
        &mut errors,
    );
    errors.first().map_or(Ok(decoded), |error| {
        Err(crate::Error {
            message: error.description().to_owned(),
            span: 0..repr.text().len(),
        })
    })
}

/// Write `text` as a basic string, quotes included, escaping what TOML requires.
#[must_use]
pub fn encode_basic(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            control if control.is_control() => {
                let _ = write!(out, "\\u{:04X}", control as u32);
            }
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

/// Write `name` as a key: bare where TOML reads it as one, quoted where it does not.
#[must_use]
pub fn encode_key(name: &str) -> String {
    Repr::key(name).to_string()
}

/// Whether `text` can sit inside a literal string, which has no escapes of its own.
#[must_use]
pub fn fits_literal(text: &str) -> bool {
    !text.contains('\'')
        && !text
            .chars()
            .any(|character| character.is_control() && character != '\t')
}
