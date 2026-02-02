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
fn test_load_text_escape_sequences() {
    // Test that escape sequences are properly unescaped using taplo's unescape
    let result = load_text(r#""hello\nworld""#, STRING);
    assert_eq!(result, "hello\nworld"); // \n should become actual newline

    let result = load_text(r#""hello\tworld""#, STRING);
    assert_eq!(result, "hello\tworld"); // \t should become actual tab

    let result = load_text(r#""hello\\world""#, STRING);
    assert_eq!(result, "hello\\world"); // \\ should become single backslash

    let result = load_text(r#""quote:\"text\"""#, STRING);
    assert_eq!(result, r#"quote:"text""#); // \" should become quote
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
fn test_update_content_string_literal_normalized() {
    let toml = "name = 'foo'";
    let root_ast = parse(toml).into_syntax().clone_for_update();

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == ENTRY {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if child.kind() == VALUE {
                    update_content(child.as_node().unwrap(), |s| s.to_string());
                }
            }
        }
    }

    // String literal should be normalized to basic string (prefer "" over '')
    let result = root_ast.to_string();
    assert!(result.contains("\"foo\""), "Got: {}", result);
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

#[test]
fn test_issue_22_backslash_uses_basic_string() {
    let toml = r#"regex = 'MPL-2\.0'"#;
    let root_ast = parse(toml).into_syntax().clone_for_update();

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == ENTRY {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if child.kind() == VALUE {
                    update_content(child.as_node().unwrap(), |s| s.to_string());
                }
            }
        }
    }

    let result = root_ast.to_string();
    assert!(result.contains(r#""MPL-2\\.0""#), "Got: {}", result);
}

#[rstest]
#[case::backslash_b(r"'\b'", r#""\\b""#)]
#[case::backslash_t(r"'\t'", r#""\\t""#)]
#[case::backslash_n(r"'\n'", r#""\\n""#)]
#[case::backslash_f(r"'\f'", r#""\\f""#)]
#[case::backslash_r(r"'\r'", r#""\\r""#)]
#[case::backslash_backslash(r"'\\'", r#""\\\\""#)]
#[case::unicode_4(r"'\u0041'", r#""\\u0041""#)]
#[case::unicode_8(r"'\U00000041'", r#""\\U00000041""#)]
#[case::backslash_dot(r"'\.'", r#""\\.""#)]
#[case::backslash_s(r"'\s'", r#""\\s""#)]
#[case::unicode_short(r"'\u04'", r#""\\u04""#)]
#[case::unicode_8_short(r"'\U0000004'", r#""\\U0000004""#)]
#[case::no_backslash(r"'hello'", r#""hello""#)]
fn test_literal_string_escape_handling(#[case] input: &str, #[case] expected: &str) {
    let toml = format!("key = {input}");
    let root_ast = parse(&toml).into_syntax().clone_for_update();

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == ENTRY {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if child.kind() == VALUE {
                    update_content(child.as_node().unwrap(), |s| s.to_string());
                }
            }
        }
    }

    let result = root_ast.to_string();
    assert!(result.contains(expected), "Expected {expected}, got: {result}");
}

#[test]
fn test_issue_36_line_continuation() {
    let toml = "desc = \"\"\"\\\n    hello\\\n\"\"\"";
    let root_ast = parse(toml).into_syntax().clone_for_update();

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == ENTRY {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if child.kind() == VALUE {
                    update_content(child.as_node().unwrap(), |s| s.to_string());
                }
            }
        }
    }

    let result = root_ast.to_string();
    assert!(result.contains("\"hello\""), "Got: {}", result);
}

#[test]
fn test_line_continuation_with_crlf() {
    let toml = "desc = \"\"\"\\\r\n    hello\"\"\"";
    let root_ast = parse(toml).into_syntax().clone_for_update();

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == ENTRY {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if child.kind() == VALUE {
                    update_content(child.as_node().unwrap(), |s| s.to_string());
                }
            }
        }
    }

    let result = root_ast.to_string();
    assert!(result.contains("\"hello\""), "Got: {}", result);
}

#[test]
fn test_multiline_with_regular_escapes() {
    let toml = "desc = \"\"\"hello\\nworld\"\"\"";
    let root_ast = parse(toml).into_syntax().clone_for_update();

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == ENTRY {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if child.kind() == VALUE {
                    update_content(child.as_node().unwrap(), |s| s.to_string());
                }
            }
        }
    }

    let result = root_ast.to_string();
    assert!(result.contains("\"hello\\nworld\""), "Got: {}", result);
}

#[test]
fn test_multiline_with_tabs_after_continuation() {
    let toml = "desc = \"\"\"\\\n\t\thello\"\"\"";
    let root_ast = parse(toml).into_syntax().clone_for_update();

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == ENTRY {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if child.kind() == VALUE {
                    update_content(child.as_node().unwrap(), |s| s.to_string());
                }
            }
        }
    }

    let result = root_ast.to_string();
    assert!(result.contains("\"hello\""), "Got: {}", result);
}

#[test]
fn test_basic_string_stays_basic() {
    let toml = r#"name = "hello""#;
    let root_ast = parse(toml).into_syntax().clone_for_update();

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == ENTRY {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if child.kind() == VALUE {
                    update_content(child.as_node().unwrap(), |s| s.to_string());
                }
            }
        }
    }

    let result = root_ast.to_string();
    assert!(result.contains("\"hello\""), "Got: {}", result);
}

#[test]
fn test_literal_with_single_quote_uses_basic_string() {
    let toml = r#"name = "it's""#;
    let root_ast = parse(toml).into_syntax().clone_for_update();

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == ENTRY {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if child.kind() == VALUE {
                    update_content(child.as_node().unwrap(), |s| s.to_string());
                }
            }
        }
    }

    let result = root_ast.to_string();
    assert!(result.contains("\"it's\""), "Got: {}", result);
}

#[test]
fn test_basic_string_with_backslash_stays_basic() {
    let toml = r#"regex = "MPL-2\\.0""#;
    let root_ast = parse(toml).into_syntax().clone_for_update();

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == ENTRY {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if child.kind() == VALUE {
                    update_content(child.as_node().unwrap(), |s| s.to_string());
                }
            }
        }
    }

    let result = root_ast.to_string();
    assert!(result.contains(r#""MPL-2\\.0""#), "Got: {}", result);
}

#[test]
fn test_issue_150_prefer_double_quotes() {
    let toml = "name = 'simple-string'";
    let root_ast = parse(toml).into_syntax().clone_for_update();

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == ENTRY {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if child.kind() == VALUE {
                    update_content(child.as_node().unwrap(), |s| s.to_string());
                }
            }
        }
    }

    let result = root_ast.to_string();
    assert!(
        result.contains("\"simple-string\""),
        "Expected double-quoted string, got: {}",
        result
    );
}

#[test]
fn test_issue_150_backslash_uses_basic_string() {
    let toml = r#"regex = 'path\\to\\file'"#;
    let root_ast = parse(toml).into_syntax().clone_for_update();

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == ENTRY {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if child.kind() == VALUE {
                    update_content(child.as_node().unwrap(), |s| s.to_string());
                }
            }
        }
    }

    let result = root_ast.to_string();
    assert!(
        result.contains(r#""path\\\\to\\\\file""#),
        "Expected basic string (prefer \"\"), got: {}",
        result
    );
}

#[test]
fn test_string_with_both_quotes_uses_basic_with_escaping() {
    let toml = r#"msg = "it's a \"test\"""#;
    let root_ast = parse(toml).into_syntax().clone_for_update();

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == ENTRY {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if child.kind() == VALUE {
                    update_content(child.as_node().unwrap(), |s| s.to_string());
                }
            }
        }
    }

    let result = root_ast.to_string();
    assert!(
        result.contains(r#""it's a \"test\"""#),
        "Expected basic string with escaped quotes (can't use literal due to '), got: {}",
        result
    );
}

#[test]
fn test_string_with_double_quote_uses_literal() {
    let toml = r#"msg = "say \"hello\"""#;
    let root_ast = parse(toml).into_syntax().clone_for_update();

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == ENTRY {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if child.kind() == VALUE {
                    update_content(child.as_node().unwrap(), |s| s.to_string());
                }
            }
        }
    }

    let result = root_ast.to_string();
    assert!(
        result.contains(r#"'say "hello"'"#),
        "Expected literal string (no escaping needed for \" in ''), got: {}",
        result
    );
}
