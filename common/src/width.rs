//! How wide a piece of text is written, and where a line may break.
//!
//! `column_width` names columns rather than bytes, so a CJK character takes two of them and a
//! combining mark none. A line breaks between graphemes, since a decomposed accent or an emoji
//! written out of several scalars is one character wherever it is read.

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// How many columns the text takes to write.
#[must_use]
pub fn columns(text: &str) -> usize {
    text.width()
}

/// The offsets a line may end at, up to the first one past `max_columns`.
///
/// A character and a TOML escape each stand for one thing the file says, so breaking inside either
/// would write something it does not.
#[must_use]
pub fn break_points(text: &str, max_columns: usize) -> Vec<usize> {
    let mut points = Vec::new();
    let mut at = 0;
    let mut width = 0;
    while at < text.len() {
        // the body was written by `encode_basic`, whose longest escape is `\uXXXX`
        let held = if text.as_bytes()[at] == b'\\' {
            if text.as_bytes().get(at + 1) == Some(&b'u') {
                6
            } else {
                2
            }
        } else {
            text[at..].graphemes(true).next().map_or(1, str::len)
        };
        at = (at + held).min(text.len());
        width += columns(&text[at - held..at]).max(1);
        points.push(at);
        if width > max_columns {
            break;
        }
    }
    points
}
