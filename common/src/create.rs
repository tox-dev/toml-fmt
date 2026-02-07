//! Syntax node creation utilities using the parse-and-extract pattern.
//!
//! This module provides functions to create TOML syntax nodes by parsing valid TOML
//! and extracting the desired elements. While this approach involves parsing overhead,
//! it ensures:
//!
//! 1. **Correctness**: All created nodes are guaranteed to be valid TOML syntax
//! 2. **Proper escaping**: The parser handles all TOML escape sequences correctly
//! 3. **Simplicity**: Straightforward code that's easy to understand and maintain

use tombi_config::TomlVersion;
use tombi_syntax::SyntaxElement;
use tombi_syntax::SyntaxKind::{
    ARRAY, BASIC_STRING, COMMA, KEYS, LINE_BREAK, LITERAL_STRING, MULTI_LINE_BASIC_STRING, WHITESPACE,
};

fn escape(text: &str) -> String {
    let escaped = tombi_toml_text::to_basic_string(text);
    escaped[1..escaped.len() - 1].to_string()
}

fn parse(source: &str) -> tombi_syntax::SyntaxNode {
    tombi_parser::parse(source, TomlVersion::default())
        .syntax_node()
        .clone_for_update()
}

pub fn make_multiline_string_node(wrapped: &str) -> SyntaxElement {
    let expr = format!("a = {wrapped}");
    parse(&expr)
        .first_child()
        .expect("parsed TOML has a child")
        .children_with_tokens()
        .find(|n| n.kind() == MULTI_LINE_BASIC_STRING)
        .expect("KEY_VALUE contains MULTI_LINE_BASIC_STRING")
}

/// Create a BASIC_STRING (basic/double-quoted) syntax element by parsing valid TOML.
///
/// This function ensures proper TOML escaping by using the escape function
/// to handle backslash escaping, quote escaping, and control characters.
pub fn make_string_node(text: &str) -> SyntaxElement {
    let escaped = escape(text);
    let expr = format!("a = \"{escaped}\"");
    parse(&expr)
        .first_child()
        .expect("parsed TOML has a child")
        .children_with_tokens()
        .find(|n| n.kind() == BASIC_STRING)
        .expect("KEY_VALUE contains BASIC_STRING")
}

/// Create a LITERAL_STRING (literal/single-quoted) syntax element by parsing valid TOML.
///
/// Use this when content contains backslashes that would need escaping in a basic string.
/// Content must not contain single quotes (there's no way to escape them in literal strings).
pub fn make_literal_string_node(text: &str) -> SyntaxElement {
    let expr = format!("a = '{text}'");
    parse(&expr)
        .first_child()
        .expect("parsed TOML has a child")
        .children_with_tokens()
        .find(|n| n.kind() == LITERAL_STRING)
        .expect("KEY_VALUE contains LITERAL_STRING")
}

/// Create LINE_BREAK tokens for a blank line (two newlines).
///
/// Used for adding vertical spacing between sections in TOML files.
/// Returns two LINE_BREAK elements since tombi represents each newline separately.
pub fn make_empty_newline() -> Vec<SyntaxElement> {
    parse("\n\n")
        .children_with_tokens()
        .filter(|n| n.kind() == LINE_BREAK)
        .collect()
}

/// Create a single LINE_BREAK token.
pub fn make_newline() -> SyntaxElement {
    parse("\n")
        .children_with_tokens()
        .find(|n| n.kind() == LINE_BREAK)
        .expect("parsed newline contains LINE_BREAK")
}

/// Create a COMMA token for use in arrays.
pub fn make_comma() -> SyntaxElement {
    parse("a=[1,2]")
        .first_child()
        .expect("parsed TOML has KEY_VALUE")
        .children_with_tokens()
        .find(|n| n.kind() == ARRAY)
        .expect("KEY_VALUE has ARRAY")
        .as_node()
        .expect("ARRAY is a node")
        .children_with_tokens()
        .find(|n| n.kind() == COMMA)
        .expect("ARRAY contains COMMA")
}

/// Create a WHITESPACE token with a custom number of spaces.
pub fn make_whitespace_n(count: usize) -> SyntaxElement {
    let spaces = " ".repeat(count.max(1));
    let sample = format!("a=[1,{}2]", spaces);
    parse(&sample)
        .first_child()
        .expect("parsed TOML has KEY_VALUE")
        .children_with_tokens()
        .find(|n| n.kind() == ARRAY)
        .expect("KEY_VALUE has ARRAY")
        .as_node()
        .expect("ARRAY is a node")
        .children_with_tokens()
        .find(|n| n.kind() == WHITESPACE)
        .expect("ARRAY contains WHITESPACE")
}

/// Create a KEYS node with the given text.
///
/// Supports dotted keys like `"foo.bar"`.
pub fn make_key(text: &str) -> SyntaxElement {
    parse(format!("{text}=1").as_str())
        .first_child()
        .expect("parsed TOML has KEY_VALUE")
        .children_with_tokens()
        .find(|n| n.kind() == KEYS)
        .expect("KEY_VALUE has KEYS")
}

pub fn make_array(key: &str) -> SyntaxElement {
    let txt = format!("{key} = []");
    parse(txt.as_str())
        .first_child()
        .map(SyntaxElement::Node)
        .expect("parsed array has KEY_VALUE")
}

pub fn make_array_entry(key: &str) -> SyntaxElement {
    let txt = format!("a = [\"{key}\"]");
    parse(txt.as_str())
        .first_child()
        .expect("parsed TOML has KEY_VALUE")
        .children_with_tokens()
        .find(|n| n.kind() == ARRAY)
        .expect("KEY_VALUE has ARRAY")
        .as_node()
        .expect("ARRAY is a node")
        .children_with_tokens()
        .find(|n| n.kind() == BASIC_STRING)
        .expect("ARRAY contains BASIC_STRING")
}

pub fn make_entry_of_string(key: &String, value: &String) -> SyntaxElement {
    let txt = format!("{key} = \"{value}\"\n");
    parse(txt.as_str())
        .first_child()
        .map(SyntaxElement::Node)
        .expect("parsed entry has KEY_VALUE")
}

pub fn make_table_entry(key: &str) -> Vec<SyntaxElement> {
    let txt = format!("[{key}]\n");
    parse(txt.as_str()).children_with_tokens().collect()
}

pub fn make_entry_with_array_of_inline_tables(key: &str, inline_tables: &[String]) -> SyntaxElement {
    let tables_str = inline_tables.join(", ");
    let txt = format!("{key} = [{tables_str}]\n");
    parse(txt.as_str())
        .first_child()
        .map(SyntaxElement::Node)
        .expect("parsed entry has KEY_VALUE")
}

/// Create an array of tables entry (e.g., `[[project.authors]]`)
pub fn make_table_array_entry(key: &str) -> Vec<SyntaxElement> {
    let txt = format!("[[{key}]]\n");
    parse(txt.as_str()).children_with_tokens().collect()
}

/// Create a table array with entries (e.g., `[[project.authors]]\nname = "John"\nemail = "john@example.com"`)
pub fn make_table_array_with_entries(key: &str, entries: &[(String, String)]) -> Vec<SyntaxElement> {
    let entries_str = entries
        .iter()
        .map(|(k, v)| format!("{k} = {v}"))
        .collect::<Vec<_>>()
        .join("\n");
    let txt = format!("[[{key}]]\n{entries_str}\n");
    parse(txt.as_str()).children_with_tokens().collect()
}
