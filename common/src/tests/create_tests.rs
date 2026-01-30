use taplo::syntax::SyntaxKind::{COMMA, ENTRY, KEY, NEWLINE, STRING, VALUE};

use crate::create::{
    make_array, make_array_entry, make_comma, make_empty_newline, make_entry_of_string, make_key, make_newline,
    make_string_node, make_table_entry,
};

#[test]
fn test_make_string_node_simple() {
    let node = make_string_node("hello");
    assert_eq!(node.kind(), STRING);
    assert_eq!(node.to_string(), "\"hello\"");
}

#[test]
fn test_make_string_node_with_quotes() {
    let node = make_string_node("hello \"world\"");
    assert_eq!(node.kind(), STRING);
    assert_eq!(node.to_string(), "\"hello \\\"world\\\"\"");
}

#[test]
fn test_make_string_node_empty() {
    let node = make_string_node("");
    assert_eq!(node.kind(), STRING);
    assert_eq!(node.to_string(), "\"\"");
}

#[test]
fn test_make_empty_newline() {
    let node = make_empty_newline();
    assert_eq!(node.kind(), NEWLINE);
    assert_eq!(node.to_string(), "\n\n");
}

#[test]
fn test_make_newline() {
    let node = make_newline();
    assert_eq!(node.kind(), NEWLINE);
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
    assert_eq!(node.kind(), KEY);
    assert_eq!(node.to_string(), "name");
}

#[test]
fn test_make_key_dotted() {
    let node = make_key("tool.black");
    assert_eq!(node.kind(), KEY);
    assert_eq!(node.to_string(), "tool.black");
}

#[test]
fn test_make_array() {
    let node = make_array("dependencies");
    assert_eq!(node.kind(), ENTRY);
    assert!(node.to_string().contains("dependencies"));
    assert!(node.to_string().contains("[]"));
}

#[test]
fn test_make_array_entry() {
    let node = make_array_entry("pytest");
    assert_eq!(node.kind(), VALUE);

    let node_as_node = node.as_node().unwrap();
    let mut has_string = false;
    for child in node_as_node.children_with_tokens() {
        if child.kind() == STRING {
            has_string = true;
            assert!(child.to_string().contains("pytest"));
        }
    }
    assert!(has_string);
}

#[test]
fn test_make_entry_of_string() {
    let key = String::from("name");
    let value = String::from("my-package");
    let node = make_entry_of_string(&key, &value);
    assert_eq!(node.kind(), ENTRY);
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
