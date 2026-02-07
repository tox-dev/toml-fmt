use tombi_syntax::SyntaxKind::{
    ARRAY_OF_TABLE, BASIC_STRING, COMMA, KEY_VALUE, KEYS, LINE_BREAK, LITERAL_STRING, MULTI_LINE_BASIC_STRING, TABLE,
    WHITESPACE,
};

use crate::create::{
    make_array, make_array_entry, make_comma, make_empty_newline, make_entry_of_string,
    make_entry_with_array_of_inline_tables, make_key, make_literal_string_node, make_multiline_string_node,
    make_newline, make_string_node, make_table_array_entry, make_table_array_with_entries, make_table_entry,
    make_whitespace_n,
};

#[test]
fn test_make_string_node_simple() {
    let node = make_string_node("hello");
    assert_eq!(node.kind(), BASIC_STRING);
    assert_eq!(node.to_string(), "\"hello\"");
}

#[test]
fn test_make_string_node_with_quotes() {
    let node = make_string_node("hello \"world\"");
    assert_eq!(node.kind(), BASIC_STRING);
    assert_eq!(node.to_string(), "\"hello \\\"world\\\"\"");
}

#[test]
fn test_make_string_node_empty() {
    let node = make_string_node("");
    assert_eq!(node.kind(), BASIC_STRING);
    assert_eq!(node.to_string(), "\"\"");
}

#[test]
fn test_make_empty_newline() {
    let nodes = make_empty_newline();
    assert_eq!(nodes.len(), 2);
    for node in &nodes {
        assert_eq!(node.kind(), LINE_BREAK);
        assert_eq!(node.to_string(), "\n");
    }
}

#[test]
fn test_make_newline() {
    let node = make_newline();
    assert_eq!(node.kind(), LINE_BREAK);
    assert_eq!(node.to_string(), "\n");
}

#[test]
fn test_make_comma() {
    let node = make_comma();
    assert_eq!(node.kind(), COMMA);
    assert_eq!(node.to_string(), ",");
}

#[test]
fn test_make_key_simple() {
    let node = make_key("name");
    assert_eq!(node.kind(), KEYS);
    assert_eq!(node.to_string(), "name");
}

#[test]
fn test_make_key_dotted() {
    let node = make_key("tool.black");
    assert_eq!(node.kind(), KEYS);
    assert_eq!(node.to_string(), "tool.black");
}

#[test]
fn test_make_array() {
    let node = make_array("dependencies");
    assert_eq!(node.kind(), KEY_VALUE);
    assert!(node.to_string().contains("dependencies"));
    assert!(node.to_string().contains("[]"));
}

#[test]
fn test_make_array_entry() {
    let node = make_array_entry("pytest");
    assert_eq!(node.kind(), BASIC_STRING);
    assert!(node.to_string().contains("pytest"));
}

#[test]
fn test_make_entry_of_string() {
    let key = String::from("name");
    let value = String::from("my-package");
    let node = make_entry_of_string(&key, &value);
    assert_eq!(node.kind(), KEY_VALUE);
    let txt = node.to_string();
    assert!(txt.contains("name"));
    assert!(txt.contains("my-package"));
}

#[test]
fn test_make_table_entry() {
    let entries = make_table_entry("project");
    assert!(!entries.is_empty());

    let mut has_table = false;
    for entry in &entries {
        if entry.to_string().contains("[project]") {
            has_table = true;
        }
    }
    assert!(has_table);
}

#[test]
fn test_make_table_entry_dotted() {
    let entries = make_table_entry("tool.black");
    assert!(!entries.is_empty());

    let mut has_table = false;
    for entry in &entries {
        if entry.to_string().contains("[tool.black]") {
            has_table = true;
        }
    }
    assert!(has_table);
}

#[test]
fn test_make_string_node_with_backslash() {
    let node = make_string_node("path\\to\\file");
    assert_eq!(node.kind(), BASIC_STRING);
    assert_eq!(node.to_string(), "\"path\\\\to\\\\file\"");
}

#[test]
fn test_make_string_node_with_newline() {
    let node = make_string_node("hello\nworld");
    assert_eq!(node.kind(), BASIC_STRING);
    assert_eq!(node.to_string(), "\"hello\\nworld\"");
}

#[test]
fn test_make_literal_string_node_simple() {
    let node = make_literal_string_node("hello");
    assert_eq!(node.kind(), LITERAL_STRING);
    assert_eq!(node.to_string(), "'hello'");
}

#[test]
fn test_make_literal_string_node_with_backslash() {
    let node = make_literal_string_node("path\\to\\file");
    assert_eq!(node.kind(), LITERAL_STRING);
    assert_eq!(node.to_string(), "'path\\to\\file'");
}

#[test]
fn test_make_literal_string_node_with_regex() {
    let node = make_literal_string_node("MPL-2\\.0");
    assert_eq!(node.kind(), LITERAL_STRING);
    assert_eq!(node.to_string(), "'MPL-2\\.0'");
}

#[test]
fn test_make_whitespace_n_single() {
    let node = make_whitespace_n(1);
    assert_eq!(node.kind(), WHITESPACE);
    assert_eq!(node.to_string(), " ");
}

#[test]
fn test_make_whitespace_n_multiple() {
    let node = make_whitespace_n(4);
    assert_eq!(node.kind(), WHITESPACE);
    assert_eq!(node.to_string(), "    ");
}

#[test]
fn test_make_whitespace_n_zero_defaults_to_one() {
    let node = make_whitespace_n(0);
    assert_eq!(node.kind(), WHITESPACE);
    assert_eq!(node.to_string(), " ");
}

#[test]
fn test_make_multiline_string_node() {
    let node = make_multiline_string_node("\"\"\"\\\n  hello\\\n  \"\"\"");
    assert_eq!(node.kind(), MULTI_LINE_BASIC_STRING);
    assert!(node.to_string().starts_with("\"\"\""));
    assert!(node.to_string().ends_with("\"\"\""));
}

#[test]
fn test_make_entry_with_array_of_inline_tables_single() {
    let tables = vec![String::from("{ name = \"John\" }")];
    let node = make_entry_with_array_of_inline_tables("authors", &tables);
    assert_eq!(node.kind(), KEY_VALUE);
    let txt = node.to_string();
    assert!(txt.contains("authors"));
    assert!(txt.contains("{ name = \"John\" }"));
}

#[test]
fn test_make_entry_with_array_of_inline_tables_multiple() {
    let tables = vec![String::from("{ name = \"John\" }"), String::from("{ name = \"Jane\" }")];
    let node = make_entry_with_array_of_inline_tables("authors", &tables);
    assert_eq!(node.kind(), KEY_VALUE);
    let txt = node.to_string();
    assert!(txt.contains("authors"));
    assert!(txt.contains("John"));
    assert!(txt.contains("Jane"));
}

#[test]
fn test_make_entry_with_array_of_inline_tables_empty() {
    let tables: Vec<String> = vec![];
    let node = make_entry_with_array_of_inline_tables("authors", &tables);
    assert_eq!(node.kind(), KEY_VALUE);
    let txt = node.to_string();
    assert!(txt.contains("authors = []"));
}

#[test]
fn test_make_table_array_entry() {
    let entries = make_table_array_entry("project.authors");
    assert!(!entries.is_empty());

    let has_array_table = entries
        .iter()
        .any(|entry| entry.kind() == ARRAY_OF_TABLE || entry.to_string().contains("[[project.authors]]"));
    assert!(has_array_table);
}

#[test]
fn test_make_table_array_entry_simple() {
    let entries = make_table_array_entry("items");
    let combined: String = entries.iter().map(|e| e.to_string()).collect();
    assert!(combined.contains("[[items]]"));
}

#[test]
fn test_make_table_array_with_entries_single() {
    let data = vec![(String::from("name"), String::from("\"John\""))];
    let entries = make_table_array_with_entries("project.authors", &data);
    let combined: String = entries.iter().map(|e| e.to_string()).collect();
    assert!(combined.contains("[[project.authors]]"));
    assert!(combined.contains("name = \"John\""));
}

#[test]
fn test_make_table_array_with_entries_multiple() {
    let data = vec![
        (String::from("name"), String::from("\"John\"")),
        (String::from("email"), String::from("\"john@example.com\"")),
    ];
    let entries = make_table_array_with_entries("authors", &data);
    let combined: String = entries.iter().map(|e| e.to_string()).collect();
    assert!(combined.contains("[[authors]]"));
    assert!(combined.contains("name = \"John\""));
    assert!(combined.contains("email = \"john@example.com\""));
}

#[test]
fn test_make_table_array_with_entries_empty() {
    let data: Vec<(String, String)> = vec![];
    let entries = make_table_array_with_entries("empty", &data);
    let combined: String = entries.iter().map(|e| e.to_string()).collect();
    assert!(combined.contains("[[empty]]"));
}

#[test]
fn test_make_table_entry_returns_table_node() {
    let entries = make_table_entry("section");
    let has_table = entries
        .iter()
        .any(|e| e.kind() == TABLE || e.to_string().contains("[section]"));
    assert!(has_table);
}
