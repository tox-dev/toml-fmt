use rstest::rstest;
use taplo::parser::parse;
use taplo::syntax::SyntaxKind::{
    ENTRY, IDENT, MULTI_LINE_STRING, MULTI_LINE_STRING_LITERAL, STRING, STRING_LITERAL, VALUE,
};

use crate::string::{load_text, update_content};

#[rstest]
#[case::basic_string("\"hello\"", STRING, "hello")]
#[case::string_with_escaped_quote("\"hello \\\"world\\\"\"", STRING, "hello \"world\"")]
#[case::string_literal("'hello'", STRING_LITERAL, "hello")]
#[case::multi_line_string("\"\"\"hello\"\"\"", MULTI_LINE_STRING, "hello")]
#[case::multi_line_literal("'''hello'''", MULTI_LINE_STRING_LITERAL, "hello")]
#[case::ident("name", IDENT, "name")]
fn test_load_text(#[case] input: &str, #[case] kind: taplo::syntax::SyntaxKind, #[case] expected: &str) {
    let result = load_text(input, kind);
    assert_eq!(result, expected);
}

#[test]
fn test_load_text_empty_string() {
    let result = load_text("\"\"", STRING);
    assert_eq!(result, "");
}

#[test]
fn test_update_content_basic() {
    let toml = r#"name = "foo""#;
    let root_ast = parse(toml).into_syntax().clone_for_update();

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == ENTRY {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if child.kind() == VALUE {
                    update_content(child.as_node().unwrap(), |s| s.to_uppercase());
                }
            }
        }
    }

    let result = root_ast.to_string();
    assert!(result.contains("FOO"));
}

#[test]
fn test_update_content_no_change() {
    let toml = r#"name = "foo""#;
    let root_ast = parse(toml).into_syntax().clone_for_update();

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == ENTRY {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if child.kind() == VALUE {
                    // Return same string - should not trigger update
                    update_content(child.as_node().unwrap(), |s| s.to_string());
                }
            }
        }
    }

    let result = root_ast.to_string();
    assert!(result.contains("foo"));
}

#[test]
fn test_update_content_string_literal_to_string() {
    let toml = "name = 'foo'";
    let root_ast = parse(toml).into_syntax().clone_for_update();

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == ENTRY {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if child.kind() == VALUE {
                    // Same content but different quote style triggers update
                    update_content(child.as_node().unwrap(), |s| s.to_string());
                }
            }
        }
    }

    // String literal should be converted to regular string
    let result = root_ast.to_string();
    assert!(result.contains("\"foo\""));
}

#[test]
fn test_update_content_multi_line_string() {
    let toml = "desc = \"\"\"multi\nline\"\"\"";
    let root_ast = parse(toml).into_syntax().clone_for_update();

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == ENTRY {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if child.kind() == VALUE {
                    update_content(child.as_node().unwrap(), |s| s.replace('\n', " "));
                }
            }
        }
    }

    let result = root_ast.to_string();
    // Multi-line should be converted to single-line string
    assert!(result.contains("\"multi line\""));
}
