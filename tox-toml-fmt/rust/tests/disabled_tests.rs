use common::disabled::MARKER;
use indoc::indoc;

use super::{assert_valid_toml, default_settings};
use _tox_toml_fmt::format_toml;

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

/// Turning the alternative back on would say `description` twice, which no reader can read. The
/// file the caller wrote says it once, so it formats like any other.
#[test]
fn test_a_disabled_alternative_beside_the_active_key() {
    let start = indoc! {r#"
        [env_run_base]
        description = "run the tests"
        # description = "run them differently"
        deps = ["b", "a"]
    "#};
    insta::assert_snapshot!(evaluate(start), @r#"
    [env_run_base]
    description = "run the tests"
    # description = "run them differently"
    deps = [ "a", "b" ]
    "#);
}

/// The disabled key names a table that a header below writes out, so turning it on would say the
/// same table two ways.
#[test]
fn test_a_disabled_dotted_key_that_a_header_below_also_names() {
    let start = indoc! {r#"
        [env_run_base]
        # set_env.A = "1"
        description = "run the tests"

        [env_run_base.set_env]
        B = "2"
    "#};
    insta::assert_snapshot!(evaluate(start), @r#"
    [env_run_base]
    description = "run the tests"
    # set_env.A = "1"
    set_env.B = "2"
    "#);
}

#[test]
fn test_formatting_a_disabled_alternative_twice_changes_nothing() {
    let start = indoc! {r#"
        [env_run_base]
        description = "run the tests"
        # description = "run them differently"
    "#};
    let once = evaluate(start);

    assert_eq!(evaluate(&once), once);
}

/// A comment inside an open value is prose about the value: a key-value written where it sits would
/// close nothing and open nothing, so the pass leaves the file to the formatter as written.
#[test]
fn test_a_comment_inside_an_array_is_left_as_prose() {
    let start = indoc! {r#"
        [env.test]
        deps = [
          # b = 1
          "x",
        ]
    "#};
    insta::assert_snapshot!(evaluate(start), @r#"
    [env.test]
    deps = [
      # b = 1
      "x",
    ]
    "#);
}

/// The migration reads the keys the file wrote: a commented `use_develop` says nothing to migrate,
/// and a commented `package` reserves no name for the migrated key to collide with.
#[test]
fn test_the_use_develop_migration_reads_the_keys_the_file_wrote() {
    let disabled_older = evaluate(indoc! {r#"
        [env.test]
        package = "wheel"
        # use_develop = true
    "#});
    let disabled_newer = evaluate(indoc! {r#"
        [env.test]
        use_develop = true
        # package = "wheel"
    "#});

    insta::assert_snapshot!(disabled_older, @r#"
    [env.test]
    package = "wheel"
    # use_develop = true
    "#);
    insta::assert_snapshot!(disabled_newer, @r#"
    [env.test]
    package = "editable"
    # package = "wheel"
    "#);
    assert_eq!(evaluate(&disabled_older), disabled_older);
    assert_eq!(evaluate(&disabled_newer), disabled_newer);
}

/// A key the file wrote as a comment reserves no name, so the alias beside it still migrates to the
/// spelling tox reads today.
#[test]
fn test_a_commented_key_holds_no_name_back_from_an_alias() {
    let beside_a_comment = evaluate(indoc! {r#"
        [env.test]
        setenv = { A = "1" }
        # set_env = { B = "2" }
    "#});

    insta::assert_snapshot!(beside_a_comment, @r#"
    [env.test]
    set_env = { A = "1" }
    # set_env = { B = "2" }
    "#);
    assert_eq!(evaluate(&beside_a_comment), beside_a_comment);
}

/// A list the file wrote as a comment names no environment tox runs, so the tables it names are
/// arranged the way an unlisted environment is.
#[test]
fn test_a_commented_env_list_orders_no_table() {
    let commented = evaluate(indoc! {r#"
        # env_list = ["b", "a"]

        [env.b]
        description = "b"

        [env.a]
        description = "a"
    "#});

    insta::assert_snapshot!(commented, @r#"
    # env_list = [ "b", "a" ]

    [env.a]
    description = "a"

    [env.b]
    description = "b"
    "#);
    assert_eq!(evaluate(&commented), commented);
}

fn evaluate(start: &str) -> String {
    let result = format_toml(start, &default_settings()).expect("the formatter reads its own output");
    assert_valid_toml(&result);
    assert!(
        !result.contains(MARKER),
        "internal marker leaked into output:\n{result}"
    );
    result
}
