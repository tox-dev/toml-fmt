use std::cell::RefCell;

use tombi_config::TomlVersion;
use tombi_syntax::SyntaxKind::{BASIC_STRING, KEY_VALUE, KEYS};

use crate::util::{find_first, iter, limit_blank_lines};

fn parse(source: &str) -> tombi_syntax::SyntaxNode {
    tombi_parser::parse(source, TomlVersion::default())
        .syntax_node()
        .clone_for_update()
}

#[test]
fn test_iter_single_level() {
    let toml = r#"name = "foo""#;
    let root_ast = parse(toml);

    let found_keys = RefCell::new(Vec::new());
    iter(&root_ast, &[KEY_VALUE, KEYS], &|node| {
        found_keys.borrow_mut().push(node.text().to_string());
    });

    let keys = found_keys.borrow();
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].trim(), "name");
}

#[test]
fn test_iter_nested_path() {
    let toml = r#"name = "foo""#;
    let root_ast = parse(toml);

    let found_values = RefCell::new(Vec::new());
    iter(&root_ast, &[KEY_VALUE, BASIC_STRING], &|node| {
        found_values.borrow_mut().push(node.text().to_string());
    });

    let values = found_values.borrow();
    assert_eq!(values.len(), 1);
    assert!(values[0].contains("foo"));
}

#[test]
fn test_iter_no_match() {
    let toml = r#"name = "foo""#;
    let root_ast = parse(toml);

    let count = RefCell::new(0);
    iter(&root_ast, &[BASIC_STRING], &|_node| {
        *count.borrow_mut() += 1;
    });

    assert_eq!(*count.borrow(), 0);
}

#[test]
fn test_find_first_existing() {
    let toml = r#"name = "foo""#;
    let root_ast = parse(toml);

    let result = find_first(&root_ast, &[KEY_VALUE], &|elem| elem.to_string());

    assert!(result.is_some());
    assert!(result.unwrap().contains("name"));
}

#[test]
fn test_find_first_non_existing() {
    let toml = r#"name = "foo""#;
    let root_ast = parse(toml);

    let result = find_first(&root_ast, &[BASIC_STRING], &|elem| elem.to_string());

    assert!(result.is_none());
}

#[test]
fn test_find_first_nested_path() {
    let toml = r#"name = "foo""#;
    let root_ast = parse(toml);

    let result = find_first(&root_ast, &[KEY_VALUE, BASIC_STRING], &|elem| elem.to_string());
    assert!(result.is_some());
    assert!(result.unwrap().contains("foo"));
}

#[test]
fn test_limit_blank_lines_no_excess() {
    let input = "line1\nline2\n\nline3\n";
    let result = limit_blank_lines(input, 2);
    assert_eq!(result, input);
}

#[test]
fn test_limit_blank_lines_removes_excess() {
    let input = "line1\n\n\n\nline2\n";
    let expected = "line1\n\n\nline2\n";
    let result = limit_blank_lines(input, 2);
    assert_eq!(result, expected);
}

#[test]
fn test_limit_blank_lines_multiple_sections() {
    let input = "section1\n\n\n\nsection2\n\n\n\nsection3\n";
    let expected = "section1\n\n\nsection2\n\n\nsection3\n";
    let result = limit_blank_lines(input, 2);
    assert_eq!(result, expected);
}

#[test]
fn test_limit_blank_lines_preserves_trailing_newline() {
    let input = "line1\n\n\n\nline2\n";
    let result = limit_blank_lines(input, 1);
    assert!(result.ends_with('\n'));
}

#[test]
fn test_limit_blank_lines_no_trailing_newline() {
    let input = "line1\n\n\n\nline2";
    let result = limit_blank_lines(input, 1);
    assert!(!result.ends_with('\n'));
}

#[test]
fn test_limit_blank_lines_zero_max() {
    let input = "line1\n\n\nline2\n";
    let expected = "line1\nline2\n";
    let result = limit_blank_lines(input, 0);
    assert_eq!(result, expected);
}
