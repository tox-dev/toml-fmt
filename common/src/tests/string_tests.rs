use tombi_config::TomlVersion;
use tombi_syntax::SyntaxKind::{
    BARE_KEY, BASIC_STRING, KEY_VALUE, LITERAL_STRING, MULTI_LINE_BASIC_STRING, MULTI_LINE_LITERAL_STRING,
};

use crate::string::{load_text, strip_quotes, update_content, update_content_wrapped, wrap_all_long_strings};

fn parse(source: &str) -> tombi_syntax::SyntaxNode {
    tombi_parser::parse(source, TomlVersion::default())
        .syntax_node()
        .clone_for_update()
}

fn is_string_kind(kind: tombi_syntax::SyntaxKind) -> bool {
    matches!(
        kind,
        BASIC_STRING | LITERAL_STRING | MULTI_LINE_BASIC_STRING | MULTI_LINE_LITERAL_STRING
    )
}

fn load_text_helper(input: &str, kind: tombi_syntax::SyntaxKind) -> String {
    load_text(input, kind)
}

fn literal_string_escape_helper(input: &str) -> String {
    let toml = format!("key = {input}");
    let root_ast = parse(&toml);

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == KEY_VALUE {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if is_string_kind(child.kind()) {
                    update_content(child.as_node().unwrap(), |s| s.to_string());
                }
            }
        }
    }

    root_ast.to_string()
}

#[test]
fn test_load_text_case_basic_string() {
    let result = load_text_helper("\"hello\"", BASIC_STRING);
    insta::assert_snapshot!(result, @"hello");
}

#[test]
fn test_load_text_case_string_with_escaped_quote() {
    let result = load_text_helper("\"hello \\\"world\\\"\"", BASIC_STRING);
    insta::assert_snapshot!(result, @r#"hello "world""#);
}

#[test]
fn test_load_text_case_string_literal() {
    let result = load_text_helper("'hello'", LITERAL_STRING);
    insta::assert_snapshot!(result, @"hello");
}

#[test]
fn test_load_text_case_multi_line_string() {
    let result = load_text_helper("\"\"\"hello\"\"\"", MULTI_LINE_BASIC_STRING);
    insta::assert_snapshot!(result, @"hello");
}

#[test]
fn test_load_text_case_multi_line_literal() {
    let result = load_text_helper("'''hello'''", MULTI_LINE_LITERAL_STRING);
    insta::assert_snapshot!(result, @"hello");
}

#[test]
fn test_load_text_case_ident() {
    let result = load_text_helper("name", BARE_KEY);
    insta::assert_snapshot!(result, @"name");
}

#[test]
fn test_load_text_empty_string() {
    let result = load_text("\"\"", BASIC_STRING);
    assert_eq!(result, "");
}

#[test]
fn test_load_text_escape_sequences() {
    let result = load_text(r#""hello\nworld""#, BASIC_STRING);
    assert_eq!(result, "hello\nworld");

    let result = load_text(r#""hello\tworld""#, BASIC_STRING);
    assert_eq!(result, "hello\tworld");

    let result = load_text(r#""hello\\world""#, BASIC_STRING);
    assert_eq!(result, "hello\\world");

    let result = load_text(r#""quote:\"text\"""#, BASIC_STRING);
    assert_eq!(result, r#"quote:"text""#);
}

#[test]
fn test_update_content_basic() {
    let toml = r#"name = "foo""#;
    let root_ast = parse(toml);

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == KEY_VALUE {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if is_string_kind(child.kind()) {
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
    let root_ast = parse(toml);

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == KEY_VALUE {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if is_string_kind(child.kind()) {
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
    let root_ast = parse(toml);

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == KEY_VALUE {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if is_string_kind(child.kind()) {
                    update_content(child.as_node().unwrap(), |s| s.to_string());
                }
            }
        }
    }

    let result = root_ast.to_string();
    assert!(result.contains("\"foo\""), "Got: {}", result);
}

#[test]
fn test_update_content_multi_line_string() {
    let toml = "desc = \"\"\"multi\nline\"\"\"";
    let root_ast = parse(toml);

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == KEY_VALUE {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if is_string_kind(child.kind()) {
                    update_content(child.as_node().unwrap(), |s| s.replace('\n', " "));
                }
            }
        }
    }

    let result = root_ast.to_string();
    assert!(result.contains("\"multi line\""));
}

#[test]
fn test_issue_22_backslash_uses_basic_string() {
    let toml = r#"regex = 'MPL-2\.0'"#;
    let root_ast = parse(toml);

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == KEY_VALUE {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if is_string_kind(child.kind()) {
                    update_content(child.as_node().unwrap(), |s| s.to_string());
                }
            }
        }
    }

    let result = root_ast.to_string();
    assert!(result.contains(r#""MPL-2\\.0""#), "Got: {}", result);
}

#[test]
fn test_literal_string_escape_handling_case_backslash_b() {
    let result = literal_string_escape_helper(r"'\b'");
    insta::assert_snapshot!(result, @r#"key = "\\b""#);
}

#[test]
fn test_literal_string_escape_handling_case_backslash_t() {
    let result = literal_string_escape_helper(r"'\t'");
    insta::assert_snapshot!(result, @r#"key = "\\t""#);
}

#[test]
fn test_literal_string_escape_handling_case_backslash_n() {
    let result = literal_string_escape_helper(r"'\n'");
    insta::assert_snapshot!(result, @r#"key = "\\n""#);
}

#[test]
fn test_literal_string_escape_handling_case_backslash_f() {
    let result = literal_string_escape_helper(r"'\f'");
    insta::assert_snapshot!(result, @r#"key = "\\f""#);
}

#[test]
fn test_literal_string_escape_handling_case_backslash_r() {
    let result = literal_string_escape_helper(r"'\r'");
    insta::assert_snapshot!(result, @r#"key = "\\r""#);
}

#[test]
fn test_literal_string_escape_handling_case_backslash_backslash() {
    let result = literal_string_escape_helper(r"'\\'");
    insta::assert_snapshot!(result, @r#"key = "\\\\""#);
}

#[test]
fn test_literal_string_escape_handling_case_unicode_4() {
    let result = literal_string_escape_helper(r"'\u0041'");
    insta::assert_snapshot!(result, @r#"key = "\\u0041""#);
}

#[test]
fn test_literal_string_escape_handling_case_unicode_8() {
    let result = literal_string_escape_helper(r"'\U00000041'");
    insta::assert_snapshot!(result, @r#"key = "\\U00000041""#);
}

#[test]
fn test_literal_string_escape_handling_case_backslash_dot() {
    let result = literal_string_escape_helper(r"'\.'");
    insta::assert_snapshot!(result, @r#"key = "\\.""#);
}

#[test]
fn test_literal_string_escape_handling_case_backslash_s() {
    let result = literal_string_escape_helper(r"'\s'");
    insta::assert_snapshot!(result, @r#"key = "\\s""#);
}

#[test]
fn test_literal_string_escape_handling_case_unicode_short() {
    let result = literal_string_escape_helper(r"'\u04'");
    insta::assert_snapshot!(result, @r#"key = "\\u04""#);
}

#[test]
fn test_literal_string_escape_handling_case_unicode_8_short() {
    let result = literal_string_escape_helper(r"'\U0000004'");
    insta::assert_snapshot!(result, @r#"key = "\\U0000004""#);
}

#[test]
fn test_literal_string_escape_handling_case_no_backslash() {
    let result = literal_string_escape_helper(r"'hello'");
    insta::assert_snapshot!(result, @r#"key = "hello""#);
}

#[test]
fn test_issue_36_line_continuation() {
    let toml = "desc = \"\"\"\\\n    hello\\\n\"\"\"";
    let root_ast = parse(toml);

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == KEY_VALUE {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if is_string_kind(child.kind()) {
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
    let root_ast = parse(toml);

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == KEY_VALUE {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if is_string_kind(child.kind()) {
                    update_content(child.as_node().unwrap(), |s| s.to_string());
                }
            }
        }
    }

    let result = root_ast.to_string();
    assert!(result.contains("\"hello\""), "Got: {}", result);
}

#[test]
fn test_wrap_long_string_is_idempotent() {
    use crate::string::wrap_all_long_strings;

    let original_text = "This is a long description string that needs to exceed the default column width of one hundred and twenty characters to trigger wrapping.";
    let toml = format!("[project]\ndescription = \"{}\"", original_text);

    let root_ast = parse(&toml);
    wrap_all_long_strings(&root_ast, 120, "  ");
    let after_first = root_ast.to_string();

    let root_ast2 = parse(&after_first);
    wrap_all_long_strings(&root_ast2, 120, "  ");
    let after_second = root_ast2.to_string();

    let root_ast3 = parse(&after_second);
    wrap_all_long_strings(&root_ast3, 120, "  ");
    let after_third = root_ast3.to_string();

    assert_eq!(after_first, after_second, "wrap_all_long_strings should be idempotent (first->second)");
    assert_eq!(after_second, after_third, "wrap_all_long_strings should be idempotent (second->third)");
}

#[test]
fn test_multiline_with_regular_escapes() {
    let toml = "desc = \"\"\"hello\\nworld\"\"\"";
    let root_ast = parse(toml);

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == KEY_VALUE {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if is_string_kind(child.kind()) {
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
    let root_ast = parse(toml);

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == KEY_VALUE {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if is_string_kind(child.kind()) {
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
    let root_ast = parse(toml);

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == KEY_VALUE {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if is_string_kind(child.kind()) {
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
    let root_ast = parse(toml);

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == KEY_VALUE {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if is_string_kind(child.kind()) {
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
    let root_ast = parse(toml);

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == KEY_VALUE {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if is_string_kind(child.kind()) {
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
    let root_ast = parse(toml);

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == KEY_VALUE {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if is_string_kind(child.kind()) {
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
    let root_ast = parse(toml);

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == KEY_VALUE {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if is_string_kind(child.kind()) {
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
    let root_ast = parse(toml);

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == KEY_VALUE {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if is_string_kind(child.kind()) {
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
    let root_ast = parse(toml);

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == KEY_VALUE {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if is_string_kind(child.kind()) {
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

#[test]
fn test_wrap_all_long_strings_wraps_long_string() {
    let toml = r#"description = "This is a very long description that exceeds the column width limit""#;
    let root_ast = parse(toml);
    wrap_all_long_strings(&root_ast, 50, "  ");
    let result = root_ast.to_string();
    assert!(result.contains(r#"""""#), "Expected multiline string, got: {}", result);
    assert!(result.contains(r#"\"#), "Expected line continuation, got: {}", result);
}

#[test]
fn test_wrap_all_long_strings_short_string_unchanged() {
    let toml = r#"name = "short""#;
    let root_ast = parse(toml);
    wrap_all_long_strings(&root_ast, 120, "  ");
    let result = root_ast.to_string();
    assert_eq!(result, r#"name = "short""#);
}

#[test]
fn test_wrap_all_long_strings_inline_table_not_wrapped() {
    let toml = r#"authors = [{ name = "A very long author name that would normally exceed column width" }]"#;
    let root_ast = parse(toml);
    wrap_all_long_strings(&root_ast, 50, "  ");
    let result = root_ast.to_string();
    assert!(
        !result.contains(r#"""""#),
        "Inline table strings should not be wrapped, got: {}",
        result
    );
}

#[test]
fn test_wrap_all_long_strings_converts_quote_style() {
    let toml = r#"msg = "say \"hello\"""#;
    let root_ast = parse(toml);
    wrap_all_long_strings(&root_ast, 120, "  ");
    let result = root_ast.to_string();
    assert!(
        result.contains(r#"'say "hello"'"#),
        "Expected literal string for double quotes, got: {}",
        result
    );
}

#[test]
fn test_wrap_all_long_strings_multiline_to_single() {
    let toml = "msg = \"\"\"\nno newlines here\"\"\"";
    let root_ast = parse(toml);
    wrap_all_long_strings(&root_ast, 120, "  ");
    let result = root_ast.to_string();
    assert!(
        result.contains(r#""no newlines here""#),
        "Expected single-line string, got: {}",
        result
    );
}

#[test]
fn test_wrap_string_very_long_multiple_wraps() {
    let toml = r#"description = "This is an extremely long description that will definitely need multiple line wraps to fit within the specified column width limit and should be properly wrapped with line continuations""#;
    let root_ast = parse(toml);
    wrap_all_long_strings(&root_ast, 60, "  ");
    let result = root_ast.to_string();
    insta::assert_snapshot!(result, @r#"
    description = """\
      This is an extremely long description that will \
      definitely need multiple line wraps to fit within the \
      specified column width limit and should be properly \
      wrapped with line continuations\
      """
    "#);
}

#[test]
fn test_wrap_string_with_spaces_at_break_points() {
    let toml = r#"description = "First part of the description and second part of the description""#;
    let root_ast = parse(toml);
    wrap_all_long_strings(&root_ast, 40, "  ");
    let result = root_ast.to_string();
    insta::assert_snapshot!(result, @r#"
    description = """\
      First part of the description and \
      second part of the description\
      """
    "#);
}

#[test]
fn test_wrap_string_with_indent_calculation() {
    let toml = r#"very_long_key_name = "This is a long string that needs wrapping""#;
    let root_ast = parse(toml);
    wrap_all_long_strings(&root_ast, 50, "    ");
    let result = root_ast.to_string();
    insta::assert_snapshot!(result, @r#"very_long_key_name = "This is a long string that needs wrapping""#);
}

#[test]
fn test_wrap_string_preserves_special_chars() {
    let toml = r#"msg = "String with \n newline \t tab and \\ backslash that is very long and needs wrapping""#;
    let root_ast = parse(toml);
    wrap_all_long_strings(&root_ast, 50, "  ");
    let result = root_ast.to_string();
    insta::assert_snapshot!(result, @r#"
    msg = """\
      String with \n newline \t tab and \\ backslash \
      that is very long and needs wrapping\
      """
    "#);
}

#[test]
fn test_wrap_string_with_unicode() {
    let toml = r#"description = "Unicode string with émojis 🎉 and special characters: αβγ that is quite long""#;
    let root_ast = parse(toml);
    wrap_all_long_strings(&root_ast, 50, "  ");
    let result = root_ast.to_string();
    insta::assert_snapshot!(result, @r#"
    description = """\
      Unicode string with émojis 🎉 and special \
      characters: αβγ that is quite long\
      """
    "#);
}

#[test]
fn test_wrap_string_exact_boundary() {
    let toml = r#"description = "Exactly eighty characters long to test boundary conditions for wrapping!!!!""#;
    let root_ast = parse(toml);
    wrap_all_long_strings(&root_ast, 80, "  ");
    let result = root_ast.to_string();
    insta::assert_snapshot!(result, @r#"description = "Exactly eighty characters long to test boundary conditions for wrapping!!!!""#);
}

#[test]
fn test_wrap_string_single_word_longer_than_width() {
    let toml = r#"url = "https://example.com/very/long/path/that/exceeds/column/width/limit/significantly""#;
    let root_ast = parse(toml);
    wrap_all_long_strings(&root_ast, 40, "  ");
    let result = root_ast.to_string();
    insta::assert_snapshot!(result, @r#"
    url = """\
      https://example.com/very/long/path/th\
      at/exceeds/column/width/limit/signifi\
      cantly\
      """
    "#);
}

#[test]
fn test_wrap_multiple_strings_in_document() {
    let toml = r#"
        description = "This is a very long description that needs wrapping"
        summary = "This is another very long summary that also needs wrapping"
        details = "And yet another long text field that requires wrapping"
    "#;
    let root_ast = parse(toml);
    wrap_all_long_strings(&root_ast, 50, "  ");
    let result = root_ast.to_string();
    insta::assert_snapshot!(result, @r#"

          description = """\
    This is a very long description that needs \
    wrapping\
    """
          summary = """\
    This is another very long summary that also \
    needs wrapping\
    """
          details = """\
    And yet another long text field that requires \
    wrapping\
    """
    "#);
}

#[test]
fn test_wrap_string_with_double_quotes_inside() {
    let toml = r#"msg = "Text with \"quotes\" inside that is very long and needs wrapping for sure""#;
    let root_ast = parse(toml);
    wrap_all_long_strings(&root_ast, 50, "  ");
    let result = root_ast.to_string();
    insta::assert_snapshot!(result, @r#"
    msg = """\
      Text with \"quotes\" inside that is very long \
      and needs wrapping for sure\
      """
    "#);
}

#[test]
fn test_wrap_nested_in_array() {
    let toml = r#"items = ["This is a very long string in an array that might need wrapping"]"#;
    let root_ast = parse(toml);
    wrap_all_long_strings(&root_ast, 50, "  ");
    let result = root_ast.to_string();
    insta::assert_snapshot!(result, @r#"
    items = ["""\
      This is a very long string in an array that \
      might need wrapping\
      """]
    "#);
}

#[test]
fn test_update_content_wrapped() {
    let toml = r#"name = "short""#;
    let root_ast = parse(toml);

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == KEY_VALUE {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if is_string_kind(child.kind()) {
                    update_content_wrapped(
                        child.as_node().unwrap(),
                        |s| format!("{} but now it's very long and will need wrapping at some point", s),
                        50,
                        "  ",
                    );
                }
            }
        }
    }

    let result = root_ast.to_string();
    insta::assert_snapshot!(result, @r#"
    name = """\
      short but now it's very long and will need \
      wrapping at some point\
      """
    "#);
}

#[test]
fn test_strip_quotes_double_quotes() {
    let result = strip_quotes("\"hello\"");
    insta::assert_snapshot!(result, @"hello");
}

#[test]
fn test_strip_quotes_single_quotes() {
    let result = strip_quotes("'hello'");
    insta::assert_snapshot!(result, @"hello");
}

#[test]
fn test_strip_quotes_no_quotes() {
    let result = strip_quotes("hello");
    insta::assert_snapshot!(result, @"hello");
}

#[test]
fn test_strip_quotes_empty_string() {
    let result = strip_quotes("\"\"");
    insta::assert_snapshot!(result, @"");
}

#[test]
fn test_strip_quotes_triple_quotes() {
    let result = strip_quotes("\"\"\"hello\"\"\"");
    insta::assert_snapshot!(result, @"hello");
}

#[test]
fn test_wrap_string_with_double_colon() {
    let toml = r#"classifier = "Programming Language :: Python :: 3 :: Only and more text here""#;
    let root_ast = parse(toml);
    wrap_all_long_strings(&root_ast, 50, "  ");
    let result = root_ast.to_string();
    insta::assert_snapshot!(result, @r#"
    classifier = """\
      Programming Language :: Python :: 3 :: \
      Only and more text here\
      """
    "#);
}

#[test]
fn test_wrap_string_no_spaces() {
    let toml = r#"url = "verylongurlwithoutanyspacesthatneedstobewrappedanyway""#;
    let root_ast = parse(toml);
    wrap_all_long_strings(&root_ast, 30, "  ");
    let result = root_ast.to_string();
    insta::assert_snapshot!(result, @r#"
    url = """\
      verylongurlwithoutanyspaces\
      thatneedstobewrappedanyway\
      """
    "#);
}

#[test]
fn test_wrap_with_control_characters() {
    let toml = "desc = \"has\\ttab\\nand newline that is quite long for wrapping\"";
    let root_ast = parse(toml);
    wrap_all_long_strings(&root_ast, 40, "  ");
    let result = root_ast.to_string();
    insta::assert_snapshot!(result, @r#"
    desc = """\
      has\ttab\nand newline that is quite \
      long for wrapping\
      """
    "#);
}

#[test]
fn test_wrap_short_string_no_wrap_needed() {
    let toml = r#"short = "hi""#;
    let root_ast = parse(toml);
    wrap_all_long_strings(&root_ast, 120, "  ");
    let result = root_ast.to_string();
    insta::assert_snapshot!(result, @r#"short = "hi""#);
}

#[test]
fn test_wrap_string_at_exact_column_boundary() {
    let toml = r#"x = "exactly""#;
    let root_ast = parse(toml);
    wrap_all_long_strings(&root_ast, 15, "  ");
    let result = root_ast.to_string();
    insta::assert_snapshot!(result, @r#"x = "exactly""#);
}

#[test]
fn test_load_text_multiline_with_leading_newline() {
    let result = load_text("\"\"\"\nhello\"\"\"", MULTI_LINE_BASIC_STRING);
    insta::assert_snapshot!(result, @"hello");
}

#[test]
fn test_load_text_multiline_with_crlf() {
    let result = load_text("\"\"\"\r\nhello\"\"\"", MULTI_LINE_BASIC_STRING);
    insta::assert_snapshot!(result, @"hello");
}

#[test]
fn test_update_content_with_very_long_key() {
    let toml = r#"this_is_a_very_long_key_name_that_takes_up_space = "short value""#;
    let root_ast = parse(toml);

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == KEY_VALUE {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if is_string_kind(child.kind()) {
                    update_content_wrapped(child.as_node().unwrap(), |s| s.to_string(), 80, "  ");
                }
            }
        }
    }

    let result = root_ast.to_string();
    insta::assert_snapshot!(result, @r#"this_is_a_very_long_key_name_that_takes_up_space = "short value""#);
}

#[test]
fn test_string_with_control_char_uses_basic() {
    let toml = "key = 'has\\ttab'";
    let root_ast = parse(toml);

    for entry in root_ast.children_with_tokens() {
        if entry.kind() == KEY_VALUE {
            for child in entry.as_node().unwrap().children_with_tokens() {
                if is_string_kind(child.kind()) {
                    update_content(child.as_node().unwrap(), |s| s.to_string());
                }
            }
        }
    }

    let result = root_ast.to_string();
    insta::assert_snapshot!(result, @r#"key = "has\\ttab""#);
}
