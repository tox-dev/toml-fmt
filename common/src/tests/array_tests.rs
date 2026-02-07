use indoc::indoc;
use tombi_config::TomlVersion;
use tombi_syntax::SyntaxKind::{ARRAY, KEY_VALUE};

use crate::array::{
    align_array_comments, dedupe_strings, ensure_all_arrays_multiline, ensure_trailing_comma, sort, sort_strings,
    transform,
};
use crate::pep508::Requirement;
use crate::tests::format_toml;

fn apply_to_arrays<F>(source: &str, mut f: F) -> String
where
    F: FnMut(&tombi_syntax::SyntaxNode),
{
    let root_ast = tombi_parser::parse(source, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    for children in root_ast.children_with_tokens() {
        if children.kind() == KEY_VALUE {
            for entry in children.as_node().unwrap().children_with_tokens() {
                if entry.kind() == ARRAY {
                    f(entry.as_node().unwrap());
                }
            }
        }
    }
    let formatted = format_toml(&root_ast, 120);
    let formatted_ast = tombi_parser::parse(&formatted, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    crate::array::align_array_comments(&formatted_ast);
    formatted_ast.to_string()
}

fn normalize_requirement_helper(start: &str, keep_full_version: bool) -> String {
    apply_to_arrays(start, |array| {
        transform(array, &|s| {
            Requirement::new(s).unwrap().normalize(keep_full_version).to_string()
        });
    })
}

fn sort_array_helper(start: &str) -> String {
    apply_to_arrays(start, |array| {
        sort_strings::<String, _, _>(array, |s| s.to_lowercase(), &|lhs, rhs| lhs.cmp(rhs));
    })
}

fn dedupe_strings_helper(start: &str) -> String {
    apply_to_arrays(start, |array| {
        dedupe_strings(array, |s| s.to_lowercase());
    })
}

#[test]
fn test_normalize_requirement_strip_micro_no_keep() {
    let start = indoc! {r#"
    a=["maturin >= 1.5.0"]
    "#};
    let res = normalize_requirement_helper(start, false);
    insta::assert_snapshot!(res, @r#"a = [ "maturin>=1.5" ]"#);
}

#[test]
fn test_normalize_requirement_strip_micro_keep() {
    let start = indoc! {r#"
    a=["maturin >= 1.5.0"]
    "#};
    let res = normalize_requirement_helper(start, true);
    insta::assert_snapshot!(res, @r#"a = [ "maturin>=1.5.0" ]"#);
}

#[test]
fn test_normalize_requirement_no_change() {
    let start = indoc! {r#"
    a = [
    "maturin>=1.5.3",# comment here
    # a comment afterwards
    ]
    "#};
    let res = normalize_requirement_helper(start, false);
    insta::assert_snapshot!(res, @r#"
    a = [
      "maturin>=1.5.3", # comment here
      # a comment afterwards
    ]
    "#);
}

#[test]
fn test_normalize_requirement_ignore_non_string() {
    let start = indoc! {r#"
    a=[{key="maturin>=1.5.0"}]
    "#};
    let res = normalize_requirement_helper(start, false);
    insta::assert_snapshot!(res, @r#"a = [ { key = "maturin>=1.5.0" } ]"#);
}

#[test]
fn test_normalize_requirement_has_double_quote() {
    let start = indoc! {r#"
    a=['importlib-metadata>=7.0.0;python_version<"3.8"']
    "#};
    let res = normalize_requirement_helper(start, false);
    insta::assert_snapshot!(res, @r#"a = [ "importlib-metadata>=7; python_version<'3.8'" ]"#);
}

#[test]
fn test_order_array_empty() {
    let start = indoc! {r"
    a = []
    "};
    let res = sort_array_helper(start);
    insta::assert_snapshot!(res, @"a = [  ]");
}

#[test]
fn test_order_array_single() {
    let start = indoc! {r#"
    a = ["A"]
    "#};
    let res = sort_array_helper(start);
    insta::assert_snapshot!(res, @r#"a = [ "A" ]"#);
}

#[test]
fn test_order_array_newline_single() {
    let start = indoc! {r#"
    a = ["A"]
    "#};
    let res = sort_array_helper(start);
    insta::assert_snapshot!(res, @r#"a = [ "A" ]"#);
}

#[test]
fn test_order_array_newline_single_comment() {
    let start = indoc! {r#"
    a = [ # comment
      "A"
    ]
    "#};
    let res = sort_array_helper(start);
    insta::assert_snapshot!(res, @r#"
    a = [
      # comment
      "A"
    ]
    "#);
}

#[test]
fn test_order_array_double() {
    let start = indoc! {r#"
    a = ["A", "B"]
    "#};
    let res = sort_array_helper(start);
    insta::assert_snapshot!(res, @r#"a = [ "A", "B" ]"#);
}

#[test]
fn test_order_array_with_comments_skips_sorting() {
    let start = indoc! {r#"
    a=["B", "D",
       # C comment
       "C", # C trailing
       # A comment
       "A" # A trailing
      # extra
    ] # array comment
    "#};
    let res = sort_array_helper(start);
    insta::assert_snapshot!(res, @r#"
    a = [
      "B",
      "D",
      # C comment
      "C", # C trailing
      # A comment
      "A", # A trailing
      # extra
    ]  # array comment
    "#);
}

#[test]
fn test_reorder_no_trailing_comma_reorder_no_trailing_comma() {
    let start = indoc! {r#"a=["B","A"]"#};
    let root_ast = tombi_parser::parse(start, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    for children in root_ast.children_with_tokens() {
        if children.kind() == KEY_VALUE {
            for entry in children.as_node().unwrap().children_with_tokens() {
                if entry.kind() == ARRAY {
                    sort_strings::<String, _, _>(entry.as_node().unwrap(), |s| s.to_lowercase(), &|lhs, rhs| {
                        lhs.cmp(rhs)
                    });
                }
            }
        }
    }
    let mut res = root_ast.to_string();
    res.retain(|x| !x.is_whitespace());
    insta::assert_snapshot!(res, @r#"a=["A","B"]"#);
}

#[test]
fn test_reorder_no_trailing_comma_single_element_no_comma() {
    let start = indoc! {r#"a=["A"]"#};
    let root_ast = tombi_parser::parse(start, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    for children in root_ast.children_with_tokens() {
        if children.kind() == KEY_VALUE {
            for entry in children.as_node().unwrap().children_with_tokens() {
                if entry.kind() == ARRAY {
                    sort_strings::<String, _, _>(entry.as_node().unwrap(), |s| s.to_lowercase(), &|lhs, rhs| {
                        lhs.cmp(rhs)
                    });
                }
            }
        }
    }
    let mut res = root_ast.to_string();
    res.retain(|x| !x.is_whitespace());
    insta::assert_snapshot!(res, @r#"a=["A"]"#);
}

#[test]
fn test_reorder_no_trailing_comma_empty_array() {
    let start = indoc! {r#"a=[]"#};
    let root_ast = tombi_parser::parse(start, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    for children in root_ast.children_with_tokens() {
        if children.kind() == KEY_VALUE {
            for entry in children.as_node().unwrap().children_with_tokens() {
                if entry.kind() == ARRAY {
                    sort_strings::<String, _, _>(entry.as_node().unwrap(), |s| s.to_lowercase(), &|lhs, rhs| {
                        lhs.cmp(rhs)
                    });
                }
            }
        }
    }
    let mut res = root_ast.to_string();
    res.retain(|x| !x.is_whitespace());
    insta::assert_snapshot!(res, @"a=[]");
}

#[test]
fn test_sort_empty_array_direct() {
    let start = r#"a=[]"#;
    let root_ast = tombi_parser::parse(start, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    for children in root_ast.children_with_tokens() {
        if children.kind() == KEY_VALUE {
            for entry in children.as_node().unwrap().children_with_tokens() {
                if entry.kind() == ARRAY {
                    sort::<String, _, _>(entry.as_node().unwrap(), |_| None, &|lhs, rhs| lhs.cmp(rhs));
                }
            }
        }
    }
    let res = root_ast.to_string();
    assert_eq!(res, "a=[]");
}

#[test]
fn test_dedupe_strings_no_duplicates() {
    let start = indoc! {r#"
    a = ["A", "B", "C"]
    "#};
    let res = dedupe_strings_helper(start);
    insta::assert_snapshot!(res, @r#"a = [ "A", "B", "C" ]"#);
}

#[test]
fn test_dedupe_strings_basic_duplicates() {
    let start = indoc! {r#"
    a = ["A", "a", "B", "A"]
    "#};
    let res = dedupe_strings_helper(start);
    insta::assert_snapshot!(res, @r#"a = [ "A", "B" ]"#);
}

#[test]
fn test_dedupe_strings_empty_array() {
    let start = indoc! {r"
    a = []
    "};
    let res = dedupe_strings_helper(start);
    insta::assert_snapshot!(res, @"a = [  ]");
}

#[test]
fn test_dedupe_strings_multiline_with_trailing_duplicate() {
    let start = indoc! {r#"
    a = [
      "A",
      "B",
      "a",
    ]
    "#};
    let res = dedupe_strings_helper(start);
    insta::assert_snapshot!(res, @r#"a = [ "A", "B" ]"#);
}

#[test]
fn test_dedupe_strings_duplicate_at_end_no_trailing_comma() {
    let start = indoc! {r#"
    a = ["A", "B", "a"]
    "#};
    let res = dedupe_strings_helper(start);
    insta::assert_snapshot!(res, @r#"a = [ "A", "B" ]"#);
}

#[test]
fn test_sort_with_duplicate_keys() {
    let start = indoc! {r#"
        a = [
            "pkg; marker1",
            "other",
            "pkg; marker2",
        ]
    "#};
    let expected = indoc! {r#"
        a = [
          "other",
          "pkg; marker1",
          "pkg; marker2",
        ]
    "#};
    let root_ast = tombi_parser::parse(start, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    for children in root_ast.children_with_tokens() {
        if children.kind() == KEY_VALUE {
            for entry in children.as_node().unwrap().children_with_tokens() {
                if entry.kind() == ARRAY {
                    sort_strings::<String, _, _>(
                        entry.as_node().unwrap(),
                        |s| s.split(';').next().unwrap_or(&s).trim().to_lowercase(),
                        &|lhs, rhs| lhs.cmp(rhs),
                    );
                }
            }
        }
    }
    let res = format_toml(&root_ast, 120);
    assert_eq!(res, expected);
}

#[test]
fn test_ensure_trailing_comma() {
    let start = r#"a = ["x", "y"]"#;
    let expected_raw = indoc! {r#"a = [
        "x", "y",
        ]"#};
    let root_ast = tombi_parser::parse(start, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    for children in root_ast.children_with_tokens() {
        if children.kind() == KEY_VALUE {
            for entry in children.as_node().unwrap().children_with_tokens() {
                if entry.kind() == ARRAY {
                    ensure_trailing_comma(entry.as_node().unwrap());
                }
            }
        }
    }
    assert_eq!(root_ast.to_string(), expected_raw);
}

#[test]
fn test_trailing_comma_prevents_collapse() {
    let start = r#"a = ["x", "y"]"#;
    let expected = indoc! {r#"
        a = [
          "x",
          "y",
        ]
    "#};
    let root_ast = tombi_parser::parse(start, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    for children in root_ast.children_with_tokens() {
        if children.kind() == KEY_VALUE {
            for entry in children.as_node().unwrap().children_with_tokens() {
                if entry.kind() == ARRAY {
                    ensure_trailing_comma(entry.as_node().unwrap());
                }
            }
        }
    }
    let res = format_toml(&root_ast, 120);
    assert_eq!(res, expected);
}

#[test]
fn test_ensure_all_arrays_multiline_no_duplicate() {
    let input = r#"[build-system]
build-backend = "backend"
requires = ["c", "d"]
"#;
    let root_ast = tombi_parser::parse(input, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    ensure_all_arrays_multiline(&root_ast);
    let result = root_ast.to_string();

    let count = result.matches("requires").count();
    assert_eq!(count, 1, "requires should appear exactly once, but got:\n{}", result);
}

#[test]
fn test_ensure_all_arrays_multiline_empty_array() {
    let input = r#"a = []"#;
    let root_ast = tombi_parser::parse(input, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    ensure_all_arrays_multiline(&root_ast);
    let result = root_ast.to_string();
    assert_eq!(result, r#"a = []"#);
}

#[test]
fn test_ensure_all_arrays_multiline_already_multiline() {
    let input = "a = [\n  \"x\",\n]";
    let root_ast = tombi_parser::parse(input, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    ensure_all_arrays_multiline(&root_ast);
    let result = root_ast.to_string();
    assert!(result.contains("\n"), "Should remain multiline");
    assert!(result.ends_with(",\n]"), "Should have trailing comma");
}

#[test]
fn test_ensure_all_arrays_multiline_has_trailing_but_no_newline() {
    let input = r#"a = ["x",]"#;
    let root_ast = tombi_parser::parse(input, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    ensure_all_arrays_multiline(&root_ast);
    let result = root_ast.to_string();
    assert!(result.contains("\n"), "Should add newlines, got: {}", result);
}

#[test]
fn test_ensure_all_arrays_multiline_nested_arrays() {
    let input = r#"a = [["x", "y"], ["z"]]"#;
    let root_ast = tombi_parser::parse(input, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    ensure_all_arrays_multiline(&root_ast);
    let result = root_ast.to_string();
    assert!(result.contains("\n"), "Should add newlines to nested arrays");
}

#[test]
fn test_align_simple() {
    let start = indoc! {r#"
    a = [
      "COM812",  # Comment 1
      "CPY",  # Comment 2
    ]
    "#};
    let root_ast = tombi_parser::parse(start, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    crate::array::align_array_comments(&root_ast);
    let result = root_ast.to_string();

    assert!(result.contains("\"COM812\", # Comment 1"));
    assert!(result.contains("\"CPY\",    # Comment 2"));
}

#[test]
fn test_align_multiple_arrays() {
    let start = indoc! {r#"
    a = [
      "ABC", # Short
      "XY", # Shorter
    ]
    b = [
      "VERYLONGVALUE", # Comment
      "S", # Short
    ]
    "#};
    let root_ast = tombi_parser::parse(start, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    crate::array::align_array_comments(&root_ast);
    let result = root_ast.to_string();
    insta::assert_snapshot!(result, @r#"
    a = [
      "ABC", # Short
      "XY",  # Shorter
    ]
    b = [
      "VERYLONGVALUE", # Comment
      "S",             # Short
    ]
    "#);
}

#[test]
fn test_align_mixed_comments() {
    let start = indoc! {r#"
    a = [
      "A", # Has comment
      "B",
      "C", # Another comment
    ]
    "#};
    let root_ast = tombi_parser::parse(start, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    crate::array::align_array_comments(&root_ast);
    let result = root_ast.to_string();
    insta::assert_snapshot!(result, @r#"
    a = [
      "A", # Has comment
      "B",
      "C", # Another comment
    ]
    "#);
}

#[test]
fn test_align_no_comments() {
    let start = indoc! {r#"
    a = ["A", "B", "C"]
    "#};
    let root_ast = tombi_parser::parse(start, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    crate::array::align_array_comments(&root_ast);
    let result = root_ast.to_string();
    insta::assert_snapshot!(result, @r#"a = ["A", "B", "C"]"#);
}

#[test]
fn test_align_very_long_value() {
    let start = indoc! {r#"
    a = [
      "AVERYLONGVALUETHATEXCEEDSTYPICALWIDTH", # Comment
      "SHORT", # Another
    ]
    "#};
    let root_ast = tombi_parser::parse(start, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    crate::array::align_array_comments(&root_ast);
    let result = root_ast.to_string();
    insta::assert_snapshot!(result, @r#"
    a = [
      "AVERYLONGVALUETHATEXCEEDSTYPICALWIDTH", # Comment
      "SHORT",                                 # Another
    ]
    "#);
}

#[test]
fn test_align_single_item_with_comment() {
    let start = indoc! {r#"
    a = ["ITEM", # Comment]
    "#};
    let root_ast = tombi_parser::parse(start, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    crate::array::align_array_comments(&root_ast);
    let result = root_ast.to_string();
    insta::assert_snapshot!(result, @r#"a = ["ITEM", # Comment]"#);
}

#[test]
fn test_align_nested_structure() {
    let start = indoc! {r#"
    [section]
    items = [
      "A", # First
      "BB", # Second
    ]
    others = [
      "XXX", # Comment
      "Y", # Short
    ]
    "#};
    let root_ast = tombi_parser::parse(start, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    crate::array::align_array_comments(&root_ast);
    let result = root_ast.to_string();
    insta::assert_snapshot!(result, @r#"
    [section]
    items = [
      "A",  # First
      "BB", # Second
    ]
    others = [
      "XXX", # Comment
      "Y",   # Short
    ]
    "#);
}

#[test]
fn test_ensure_trailing_comma_empty_array() {
    let start = r#"a = []"#;
    let root_ast = tombi_parser::parse(start, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    for children in root_ast.children_with_tokens() {
        if children.kind() == KEY_VALUE {
            for entry in children.as_node().unwrap().children_with_tokens() {
                if entry.kind() == ARRAY {
                    ensure_trailing_comma(entry.as_node().unwrap());
                }
            }
        }
    }
    insta::assert_snapshot!(root_ast.to_string(), @"a = []");
}

#[test]
fn test_ensure_trailing_comma_already_multiline_with_comma() {
    let start = "a = [\n  \"x\",\n]";
    let root_ast = tombi_parser::parse(start, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    for children in root_ast.children_with_tokens() {
        if children.kind() == KEY_VALUE {
            for entry in children.as_node().unwrap().children_with_tokens() {
                if entry.kind() == ARRAY {
                    ensure_trailing_comma(entry.as_node().unwrap());
                }
            }
        }
    }
    let result = root_ast.to_string();
    insta::assert_snapshot!(result, @r#"
    a = [
      "x",
    ]
    "#);
}

#[test]
fn test_dedupe_strings_all_duplicates() {
    let start = indoc! {r#"
    a = ["A", "a", "A", "a"]
    "#};
    let res = dedupe_strings_helper(start);
    insta::assert_snapshot!(res, @r#"a = [ "A" ]"#);
}

#[test]
fn test_dedupe_with_inline_table() {
    let start = indoc! {r#"
    a = ["pkg", {key="value"}, "pkg"]
    "#};
    let res = dedupe_strings_helper(start);
    insta::assert_snapshot!(res, @r#"a = [ "pkg", { key = "value" } ]"#);
}

#[test]
fn test_sort_with_none_key() {
    let start = r#"a = [42, "B", "A"]"#;
    let root_ast = tombi_parser::parse(start, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    for children in root_ast.children_with_tokens() {
        if children.kind() == KEY_VALUE {
            for entry in children.as_node().unwrap().children_with_tokens() {
                if entry.kind() == ARRAY {
                    sort::<String, _, _>(entry.as_node().unwrap(), |_| None, &|lhs, rhs| lhs.cmp(rhs));
                }
            }
        }
    }
    let res = root_ast.to_string();
    insta::assert_snapshot!(res, @r#"a = [42, "B", "A"]"#);
}

#[test]
fn test_transform_with_literal_string() {
    let start = indoc! {r"
    a = ['hello', 'world']
    "};
    let res = apply_to_arrays(start, |array| {
        transform(array, &|s| s.to_uppercase());
    });
    insta::assert_snapshot!(res, @r#"a = [ "HELLO", "WORLD" ]"#);
}

#[test]
fn test_align_empty_array() {
    let start = r#"a = []"#;
    let root_ast = tombi_parser::parse(start, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    align_array_comments(&root_ast);
    insta::assert_snapshot!(root_ast.to_string(), @"a = []");
}

#[test]
fn test_align_array_no_string_values() {
    let start = r#"a = [1, 2, 3]"#;
    let root_ast = tombi_parser::parse(start, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    align_array_comments(&root_ast);
    insta::assert_snapshot!(root_ast.to_string(), @"a = [1, 2, 3]");
}

#[test]
fn test_dedupe_with_value_wrapper() {
    let start = indoc! {r#"
    [section]
    items = [
      "foo",
      "FOO",
      "bar",
    ]
    "#};
    let root_ast = tombi_parser::parse(start, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    for descendant in root_ast.descendants() {
        if descendant.kind() == ARRAY {
            dedupe_strings(&descendant, |s| s.to_lowercase());
        }
    }
    let res = format_toml(&root_ast, 120);
    insta::assert_snapshot!(res, @r#"
    [section]
    items = [
      "foo",
      "bar",
    ]
    "#);
}

#[test]
fn test_sort_multiline_no_trailing_comma() {
    let start = "a = [\n  \"B\",\n  \"A\"\n]";
    let root_ast = tombi_parser::parse(start, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    for children in root_ast.children_with_tokens() {
        if children.kind() == KEY_VALUE {
            for entry in children.as_node().unwrap().children_with_tokens() {
                if entry.kind() == ARRAY {
                    sort_strings::<String, _, _>(entry.as_node().unwrap(), |s| s.to_lowercase(), &|lhs, rhs| {
                        lhs.cmp(rhs)
                    });
                }
            }
        }
    }
    let res = root_ast.to_string();
    insta::assert_snapshot!(res, @r#"
    a = [
      "A",
    "B"
    ]
    "#);
}

#[test]
fn test_align_comment_after_whitespace() {
    let start = indoc! {r#"
    a = [
      "short",   # comment
      "verylongstring",   # another
    ]
    "#};
    let root_ast = tombi_parser::parse(start, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    align_array_comments(&root_ast);
    let result = root_ast.to_string();
    insta::assert_snapshot!(result, @r#"
    a = [
      "short",          # comment
      "verylongstring", # another
    ]
    "#);
}

#[test]
fn test_ensure_trailing_comma_single_item_no_comma() {
    let start = r#"a = ["only"]"#;
    let root_ast = tombi_parser::parse(start, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    for children in root_ast.children_with_tokens() {
        if children.kind() == KEY_VALUE {
            for entry in children.as_node().unwrap().children_with_tokens() {
                if entry.kind() == ARRAY {
                    ensure_trailing_comma(entry.as_node().unwrap());
                }
            }
        }
    }
    let result = root_ast.to_string();
    insta::assert_snapshot!(result, @r#"
    a = [
    "only",
    ]
    "#);
}

#[test]
fn test_transform_empty_array() {
    let start = r#"a = []"#;
    let res = apply_to_arrays(start, |array| {
        transform(array, &|s| s.to_uppercase());
    });
    insta::assert_snapshot!(res, @"a = [  ]");
}

#[test]
fn test_dedupe_consecutive_duplicates() {
    let start = indoc! {r#"
    a = ["X", "X", "X", "Y", "Y"]
    "#};
    let res = dedupe_strings_helper(start);
    insta::assert_snapshot!(res, @r#"a = [ "X", "Y" ]"#);
}

#[test]
fn test_dedupe_array_with_non_string_values() {
    let start = r#"a = [1, "foo", 2, "FOO", 3]"#;
    let root_ast = tombi_parser::parse(start, TomlVersion::default())
        .syntax_node()
        .clone_for_update();
    for children in root_ast.children_with_tokens() {
        if children.kind() == KEY_VALUE {
            for entry in children.as_node().unwrap().children_with_tokens() {
                if entry.kind() == ARRAY {
                    dedupe_strings(entry.as_node().unwrap(), |s| s.to_lowercase());
                }
            }
        }
    }
    let res = root_ast.to_string();
    insta::assert_snapshot!(res, @r#"a = [1, "foo", 2, 3]"#);
}
