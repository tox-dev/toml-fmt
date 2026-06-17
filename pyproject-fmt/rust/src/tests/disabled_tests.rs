use common::disabled::MARKER;
use indoc::indoc;

use super::assert_valid_toml;
use crate::{format_toml, Settings};

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
    let result = format_toml(start, &settings());
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
