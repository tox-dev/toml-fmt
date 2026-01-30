//! Syntax node creation utilities using the parse-and-extract pattern.
//!
//! This module provides functions to create TOML syntax nodes by parsing valid TOML
//! and extracting the desired elements. While this approach involves parsing overhead,
//! it ensures:
//!
//! 1. **Correctness**: All created nodes are guaranteed to be valid TOML syntax
//! 2. **Proper escaping**: Taplo's parser handles all TOML escape sequences correctly
//! 3. **Simplicity**: Straightforward code that's easy to understand and maintain
//!

use taplo::parser::parse;
use taplo::syntax::SyntaxElement;
use taplo::syntax::SyntaxKind::{ARRAY, COMMA, ENTRY, KEY, NEWLINE, STRING, VALUE};

/// Create a STRING syntax element by parsing valid TOML.
///
/// This function ensures proper TOML escaping by using taplo's parser to handle
/// quote escaping, backslash escaping, and unicode sequences.
pub fn make_string_node(text: &str) -> SyntaxElement {
    let expr = &format!("a = \"{}\"", text.replace('"', "\\\""));
    parse(expr)
        .into_syntax()
        .clone_for_update()
        .first_child()
        .expect("parsed TOML has a child")
        .children_with_tokens()
        .find(|n| n.kind() == VALUE)
        .expect("entry has VALUE")
        .as_node()
        .expect("VALUE is a node")
        .children_with_tokens()
        .find(|n| n.kind() == STRING)
        .expect("VALUE contains STRING")
}

/// Create a NEWLINE token with a blank line (two newlines).
///
/// Used for adding vertical spacing between sections in TOML files.
pub fn make_empty_newline() -> SyntaxElement {
    parse("\n\n")
        .into_syntax()
        .clone_for_update()
        .children_with_tokens()
        .find(|n| n.kind() == NEWLINE)
        .expect("parsed newlines contain NEWLINE")
}

/// Create a single NEWLINE token.
pub fn make_newline() -> SyntaxElement {
    parse("\n")
        .into_syntax()
        .clone_for_update()
        .children_with_tokens()
        .find(|n| n.kind() == NEWLINE)
        .expect("parsed newline contains NEWLINE")
}

/// Create a COMMA token for use in arrays.
pub fn make_comma() -> SyntaxElement {
    parse("a=[1,2]")
        .into_syntax()
        .clone_for_update()
        .children_with_tokens()
        .find(|n| n.kind() == ENTRY)
        .expect("parsed TOML has ENTRY")
        .as_node()
        .expect("ENTRY is a node")
        .children_with_tokens()
        .find(|n| n.kind() == VALUE)
        .expect("ENTRY has VALUE")
        .as_node()
        .expect("VALUE is a node")
        .children_with_tokens()
        .find(|n| n.kind() == ARRAY)
        .expect("VALUE contains ARRAY")
        .as_node()
        .expect("ARRAY is a node")
        .children_with_tokens()
        .find(|n| n.kind() == COMMA)
        .expect("ARRAY contains COMMA")
}

/// Create a KEY token with the given text.
///
/// Supports dotted keys like `"foo.bar"`.
pub fn make_key(text: &str) -> SyntaxElement {
    parse(format!("{text}=1").as_str())
        .into_syntax()
        .clone_for_update()
        .children_with_tokens()
        .find(|n| n.kind() == ENTRY)
        .expect("parsed TOML has ENTRY")
        .as_node()
        .expect("ENTRY is a node")
        .children_with_tokens()
        .find(|n| n.kind() == KEY)
        .expect("ENTRY has KEY")
}

pub fn make_array(key: &str) -> SyntaxElement {
    let txt = format!("{key} = []");
    parse(txt.as_str())
        .into_syntax()
        .clone_for_update()
        .children_with_tokens()
        .find(|n| n.kind() == ENTRY)
        .expect("parsed array has ENTRY")
}

pub fn make_array_entry(key: &str) -> SyntaxElement {
    let txt = format!("a = [\"{key}\"]");
    parse(txt.as_str())
        .into_syntax()
        .clone_for_update()
        .children_with_tokens()
        .find(|n| n.kind() == ENTRY)
        .expect("parsed TOML has ENTRY")
        .as_node()
        .expect("ENTRY is a node")
        .children_with_tokens()
        .find(|n| n.kind() == VALUE)
        .expect("ENTRY has VALUE")
        .as_node()
        .expect("VALUE is a node")
        .children_with_tokens()
        .find(|n| n.kind() == ARRAY)
        .expect("VALUE contains ARRAY")
        .as_node()
        .expect("ARRAY is a node")
        .children_with_tokens()
        .find(|n| n.kind() == VALUE)
        .expect("ARRAY contains VALUE")
}

pub fn make_entry_of_string(key: &String, value: &String) -> SyntaxElement {
    let txt = format!("{key} = \"{value}\"\n");
    parse(txt.as_str())
        .into_syntax()
        .clone_for_update()
        .children_with_tokens()
        .find(|n| n.kind() == ENTRY)
        .expect("parsed entry has ENTRY")
}

pub fn make_table_entry(key: &str) -> Vec<SyntaxElement> {
    let txt = format!("[{key}]\n");
    parse(txt.as_str())
        .into_syntax()
        .clone_for_update()
        .children_with_tokens()
        .collect()
}

pub fn make_entry_with_array_of_inline_tables(key: &str, inline_tables: &[String]) -> SyntaxElement {
    let tables_str = inline_tables.join(", ");
    let txt = format!("{key} = [{tables_str}]\n");
    parse(txt.as_str())
        .into_syntax()
        .clone_for_update()
        .children_with_tokens()
        .find(|n| n.kind() == ENTRY)
        .expect("parsed entry has ENTRY")
}

/// Create an array of tables entry (e.g., `[[project.authors]]`)
pub fn make_table_array_entry(key: &str) -> Vec<SyntaxElement> {
    let txt = format!("[[{key}]]\n");
    parse(txt.as_str())
        .into_syntax()
        .clone_for_update()
        .children_with_tokens()
        .collect()
}

/// Create a table array with entries (e.g., `[[project.authors]]\nname = "John"\nemail = "john@example.com"`)
pub fn make_table_array_with_entries(key: &str, entries: &[(String, String)]) -> Vec<SyntaxElement> {
    let entries_str = entries
        .iter()
        .map(|(k, v)| format!("{k} = {v}"))
        .collect::<Vec<_>>()
        .join("\n");
    let txt = format!("[[{key}]]\n{entries_str}\n");
    parse(txt.as_str())
        .into_syntax()
        .clone_for_update()
        .children_with_tokens()
        .collect()
}
