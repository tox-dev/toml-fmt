use super::evaluate_full;
use std::fs::read_to_string;

use super::data_dir;

use insta::assert_snapshot;

fn evaluate(start: &str) -> String {
    evaluate_full(start)
}

#[test]
fn test_order_ruff() {
    let data = data_dir();
    let start = read_to_string(data.join("ruff-order.toml")).unwrap();
    let result = evaluate(&start);
    assert_snapshot!(result);
}

#[test]
fn test_ruff_comment_21() {
    let start = indoc::indoc! {r#"
    [tool.ruff.lint]
    select = ["ALL"]

    ignore = [
        # Missing type annotation for **{name}.
        "ANN003",
    ]

    # Do not automatically remove commented out code.
    # We comment out code during development, and with VSCode auto-save, this code
    # is sometimes annoyingly removed.
    unfixable = ["ERA001"]
    "#};
    let result = evaluate(start);
    assert_snapshot!(result, @r#"
    [tool.ruff]
    lint.select = [ "ALL" ]
    lint.ignore = [
      # Missing type annotation for **{name}.
      "ANN003",
    ]
    # Do not automatically remove commented out code.
    # We comment out code during development, and with VSCode auto-save, this code
    # is sometimes annoyingly removed.
    lint.unfixable = [ "ERA001" ]
    "#);
}

#[test]
fn test_ruff_inline_comments_preserved() {
    let start = indoc::indoc! {r#"
    [tool.ruff.lint]
    ignore = [
      "COM812",  # Conflict with formatter
      "CPY",  # No copyright statements
      "D203",  # Blank line before class
    ]
    "#};
    let result = evaluate(start);
    assert_snapshot!(result, @r#"
    [tool.ruff]
    lint.ignore = [
      "COM812", # Conflict with formatter
      "CPY",    # No copyright statements
      "D203",   # Blank line before class
    ]
    "#);
}

#[test]
fn test_ruff_per_file_ignores() {
    let start = indoc::indoc! {r#"
    [tool.ruff]
    lint.per-file-ignores."tests/**/*.py" = ["S101", "D103", "ARG001"]
    "#};
    let result = evaluate(start);
    assert_snapshot!(result, @r#"
    [tool.ruff]
    lint.per-file-ignores."tests/**/*.py" = [ "ARG001", "D103", "S101" ]
    "#);
}

#[test]
fn test_ruff_extend_per_file_ignores() {
    let start = indoc::indoc! {r#"
    [tool.ruff]
    lint.extend-per-file-ignores."docs/*.py" = ["E501", "D100"]
    "#};
    let result = evaluate(start);
    assert_snapshot!(result, @r#"
    [tool.ruff]
    lint.extend-per-file-ignores."docs/*.py" = [ "D100", "E501" ]
    "#);
}

/// Ruff writes one auxiliary import block per forced-separate group, in the order the list gives
/// them, so the list decides where the imports land.
#[test]
fn test_ruff_forced_separate_keeps_its_order() {
    let start = indoc::indoc! {r#"
    [tool.ruff]
    lint.isort.forced-separate = ["vendor_z", "vendor_a"]
    lint.isort.extra-standard-library = ["zeta", "alpha"]
    "#};
    let result = evaluate(start);

    insta::assert_snapshot!(result, @r#"
    [tool.ruff]
    lint.isort.extra-standard-library = [ "alpha", "zeta" ]
    lint.isort.forced-separate = [ "vendor_z", "vendor_a" ]
    "#);
}
