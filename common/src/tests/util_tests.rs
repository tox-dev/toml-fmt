use std::cell::RefCell;
use taplo::parser::parse;
use taplo::syntax::SyntaxKind::{ENTRY, KEY, STRING, VALUE};

use crate::util::{find_first, iter};

#[test]
fn test_iter_single_level() {
    let toml = r#"name = "foo""#;
    let root_ast = parse(toml).into_syntax().clone_for_update();

    let found_keys = RefCell::new(Vec::new());
    iter(&root_ast, &[ENTRY, KEY], &|node| {
        found_keys.borrow_mut().push(node.text().to_string());
    });

    let keys = found_keys.borrow();
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].trim(), "name");
}

#[test]
fn test_iter_nested_path() {
    let toml = r#"name = "foo""#;
    let root_ast = parse(toml).into_syntax().clone_for_update();

    let found_values = RefCell::new(Vec::new());
    iter(&root_ast, &[ENTRY, VALUE], &|node| {
        found_values.borrow_mut().push(node.text().to_string());
    });

    let values = found_values.borrow();
    assert_eq!(values.len(), 1);
    assert!(values[0].contains("foo"));
}

#[test]
fn test_iter_no_match() {
    let toml = r#"name = "foo""#;
    let root_ast = parse(toml).into_syntax().clone_for_update();

    let count = RefCell::new(0);
    iter(&root_ast, &[STRING], &|_node| {
        *count.borrow_mut() += 1;
    });

    assert_eq!(*count.borrow(), 0);
}

#[test]
fn test_find_first_existing() {
    let toml = r#"name = "foo""#;
    let root_ast = parse(toml).into_syntax().clone_for_update();

    let result = find_first(&root_ast, &[ENTRY], &|elem| elem.to_string());

    assert!(result.is_some());
    assert!(result.unwrap().contains("name"));
}

#[test]
fn test_find_first_non_existing() {
    let toml = r#"name = "foo""#;
    let root_ast = parse(toml).into_syntax().clone_for_update();

    let result = find_first(&root_ast, &[STRING], &|elem| elem.to_string());

    assert!(result.is_none());
}

#[test]
fn test_find_first_nested_path() {
    let toml = r#"name = "foo""#;
    let root_ast = parse(toml).into_syntax().clone_for_update();

    let result = find_first(&root_ast, &[ENTRY, VALUE], &|elem| elem.to_string());
    assert!(result.is_none());
}
