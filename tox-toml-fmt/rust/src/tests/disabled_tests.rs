use common::disabled::MARKER;
use indoc::indoc;

use super::assert_valid_toml;
use crate::{format_toml, Settings};

fn settings() -> Settings {
    Settings {
        column_width: 80,
        indent: 2,
        table_format: String::from("short"),
        sub_table_spacing: String::new(),
        separate_root_table: String::from("\n"),
        expand_tables: vec![],
        collapse_tables: vec![],
        skip_wrap_for_keys: vec![],
        pin_envs: vec![],
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
fn test_disabled_key_stays_with_its_env_table() {
    let start = indoc! {r#"
        [env_run_base]
        description = "run the tests"
        # set_env = {A = "1"}

        [env.type]
        description = "type check"
    "#};
    insta::assert_snapshot!(evaluate(start), @r#"
    [env_run_base]
    description = "run the tests"
    # set_env = { A = "1" }

    [env.type]
    description = "type check"
    "#);
}

#[test]
fn test_prose_comment_is_left_untouched() {
    let start = indoc! {r#"
        [env_run_base]
        # run under every interpreter
        description = "run the tests"
    "#};
    let result = evaluate(start);
    assert!(
        result.contains("# run under every interpreter"),
        "prose comment must survive:\n{result}"
    );
}
