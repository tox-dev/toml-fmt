use common::disabled::MARKER;
use indoc::indoc;

use super::assert_valid_toml;
use _pyproject_fmt::{format_toml, Settings};

fn settings() -> Settings {
    Settings {
        column_width: 120,
        indent: 2,
        keep_full_version: false,
        max_supported_python: (3, 9),
        min_supported_python: (3, 9),
        generate_python_version_classifiers: false,
        table_format: String::from("short"),
        sub_table_spacing: String::new(),
        separate_root_table: String::from("\n"),
        expand_tables: vec![],
        collapse_tables: vec![],
        skip_wrap_for_keys: vec![],
    }
}

fn evaluate(start: &str) -> String {
    let result = format_toml(start, &settings()).unwrap();
    assert_valid_toml(&result);
    assert!(
        !result.contains(MARKER),
        "internal marker leaked into output:\n{result}"
    );
    result
}

#[test]
fn test_disabled_keys_stay_anchored_to_their_entry() {
    let start = indoc! {r#"
        [[tool.uv.index]]
        name = "pypi"
        url = "https://pypi.org/simple"
        authenticate = "never"
        # TODO: once ticket XYZ is complete
        #  to prioritize those over pypi
        # default = true

        # These definitions will be used as priority over the ones specified in uv.toml

        [[tool.uv.index]]
        name = "company-master"
        url = "https://dl.cloudsmith.io/basic/company/master/python/simple"
        authenticate = "always"
        # ignore-error-codes = [400, 401, 403]  # turn on for debugging
    "#};
    insta::assert_snapshot!(evaluate(start), @r#"
    [[tool.uv.index]]
    name = "pypi"
    url = "https://pypi.org/simple"
    authenticate = "never"
    # TODO: once ticket XYZ is complete
    #  to prioritize those over pypi
    # default = true

    # These definitions will be used as priority over the ones specified in uv.toml
    [[tool.uv.index]]
    name = "company-master"
    url = "https://dl.cloudsmith.io/basic/company/master/python/simple"
    authenticate = "always"
    # ignore-error-codes = [ 400, 401, 403 ]  # turn on for debugging
    "#);
}

#[test]
fn test_disabled_key_output_is_idempotent() {
    let start = indoc! {r#"
        [[tool.uv.index]]
        name = "pypi"
        url = "https://pypi.org/simple"
        # default = true
    "#};
    let once = evaluate(start);
    assert_eq!(evaluate(&once), once, "second pass must be stable");
}

#[test]
fn test_prose_comment_is_left_untouched() {
    let start = indoc! {r#"
        [project]
        name = "foo"
        # this is just a note
        version = "1.0"
    "#};
    let result = evaluate(start);
    assert!(
        result.contains("# this is just a note"),
        "prose comment must survive:\n{result}"
    );
}

/// Turning the alternative back on would say `name` twice, which no reader can read. The file the
/// caller wrote says it once, so it formats like any other.
#[test]
fn test_a_disabled_alternative_beside_the_active_key() {
    let start = indoc! {r#"
        [project]
        name = "active"
        # name = "alternative"
        requires-python = ">=3.9"
        dependencies = ["B", "a"]
    "#};
    insta::assert_snapshot!(evaluate(start), @r#"
    [project]
    name = "active"
    # name = "alternative"
    requires-python = ">=3.9"
    dependencies = [ "a", "b" ]
    "#);
}

/// The disabled key names a table that a header below writes out, so turning it on would say the
/// same table two ways.
#[test]
fn test_a_disabled_dotted_key_that_a_header_below_also_names() {
    let start = indoc! {r#"
        [tool.uv]
        # sources.mine = { path = "." }
        package = true

        [tool.uv.sources]
        mine = { workspace = true }
    "#};
    insta::assert_snapshot!(evaluate(start), @r#"
    [tool.uv]
    # sources.mine = { path = "." }
    sources.mine = { workspace = true }
    package = true
    "#);
}

#[test]
fn test_formatting_a_disabled_alternative_twice_changes_nothing() {
    let start = indoc! {r#"
        [project]
        name = "active"
        # name = "alternative"
    "#};
    let once = evaluate(start);

    assert_eq!(evaluate(&once), once);
}

/// A comment inside an open value is prose about the value: a key-value written where it sits would
/// close nothing and open nothing, so the pass leaves the file to the formatter as written.
#[test]
fn test_a_comment_inside_an_array_is_left_as_prose() {
    let start = indoc! {r#"
        [project]
        name = "example"
        dependencies = [
          # b = 1
          "x",
        ]
    "#};
    insta::assert_snapshot!(evaluate(start), @r#"
    [project]
    name = "example"
    dependencies = [
      # b = 1
      "x",
    ]
    "#);
}

/// A rewrite that would split one entry into several has no key to leave the comment on, so the
/// disabled entry is left as it was written.
#[test]
fn test_a_disabled_inline_table_is_not_split_into_keys() {
    let start = indoc! {r#"
        [project]
        name = "x"
        # entry-points.group = { first = "pkg:first", second = "pkg:second" }
    "#};
    let once = evaluate(start);

    insta::assert_snapshot!(once, @r#"
    [project]
    name = "x"
    # entry-points.group = { first = "pkg:first", second = "pkg:second" }
    "#);
    assert_eq!(evaluate(&once), once);
}

/// Folding an array of tables writes each element as an inline table, where a member has no line of
/// its own for the comment that says the key is disabled.
#[test]
fn test_a_disabled_key_holds_an_array_of_tables_open() {
    let start = indoc! {r#"
        [project]
        name = "x"

        [[project.authors]]
        # name = "Alice"
        email = "a@example.com"
    "#};
    let once = evaluate(start);

    insta::assert_snapshot!(once, @r#"
    [project]
    name = "x"

    [[project.authors]]
    # name = "Alice"
    email = "a@example.com"
    "#);
    assert_eq!(evaluate(&once), once);
}

/// A `#` inside a value is part of what that value says, so nothing there is read as a disabled key.
#[test]
fn test_a_comment_inside_a_value_is_not_a_disabled_key() {
    let inside_a_string = indoc! {r#"
        [project]
        name = "x"
        keywords = ["a"]
        # keywords = ["b"]
    "#};
    let inside_an_array = indoc! {r#"
        [tool.black]
        target-version = ["py311"]
        values = [
          1,
          # key = 2
        ]

        [tool.other]
        z = 1
    "#};

    insta::assert_snapshot!(evaluate(inside_a_string), @r#"
    [project]
    name = "x"
    keywords = [ "a" ]
    # keywords = [ "b" ]
    "#);
    insta::assert_snapshot!(evaluate(inside_an_array), @r#"
    [tool.black]
    target-version = [ "py311" ]
    values = [
      1,
      # key = 2
    ]

    [tool.other]
    z = 1
    "#);
}

/// A key the file wrote as a comment says nothing about what the project supports, so the
/// classifiers come from the one it wrote.
#[test]
fn test_a_disabled_requires_python_says_nothing() {
    let start = indoc! {r#"
        [project]
        name = "x"
        requires-python = ">=3.12"
        # requires-python = ">=3.9"
    "#};
    let settings = Settings {
        generate_python_version_classifiers: true,
        max_supported_python: (3, 13),
        ..settings()
    };
    let result = format_toml(start, &settings).unwrap();

    insta::assert_snapshot!(result, @r#"
    [project]
    name = "x"
    requires-python = ">=3.12"
    # requires-python = ">=3.9"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.12",
      "Programming Language :: Python :: 3.13",
    ]
    "#);
}
