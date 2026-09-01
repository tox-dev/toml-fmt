//! Empty lines between tables.

use common::spacing::Spacing;
use toml_doc::LineEnding;

#[test]
fn a_different_tool_gets_an_empty_line_above_it() {
    assert_eq!(
        space("[tool.a]\nx = 1\n[tool.b]\ny = 2\n", None),
        "[tool.a]\nx = 1\n\n[tool.b]\ny = 2\n"
    );
}

#[test]
fn tables_of_one_tool_stay_together() {
    let source = "[tool.a]\nx = 1\n[tool.a.sub]\ny = 2\n";

    assert_eq!(space(source, Some(0)), source);
}

#[test]
fn the_gap_sits_above_the_comments_that_lead_a_table() {
    let written = space("[tool.a]\nx = 1\n# about b\n[tool.b]\ny = 2\n", None);

    assert_eq!(written, "[tool.a]\nx = 1\n\n# about b\n[tool.b]\ny = 2\n");
}

#[test]
fn repeated_array_of_tables_entries_keep_their_own_spacing() {
    let source = "[[tool.a]]\nx = 1\n[[tool.a]]\nx = 2\n";

    assert_eq!(space(source, Some(1)), source);
}

#[test]
fn a_wider_gap_can_be_asked_for_within_a_group() {
    let written = space("[tool.a]\nx = 1\n[tool.a.sub]\ny = 2\n", Some(1));

    assert_eq!(written, "[tool.a]\nx = 1\n\n[tool.a.sub]\ny = 2\n");
}

#[test]
fn the_first_table_is_held_off_the_keys_above_it() {
    assert_eq!(space("a = 1\n[tool.b]\ny = 2\n", None), "a = 1\n\n[tool.b]\ny = 2\n");
}

#[test]
fn a_document_that_opens_with_empty_lines_loses_them() {
    assert_eq!(space("\n\na = 1\n", None), "a = 1\n");
}

#[test]
fn a_table_outside_the_nested_prefixes_groups_by_its_first_name() {
    assert_eq!(
        space("[demo.a]\nx = 1\n[demo.b]\ny = 2\n", None),
        "[demo.a]\nx = 1\n[demo.b]\ny = 2\n"
    );
}

#[test]
fn a_document_holding_nothing_is_left_as_it_is() {
    assert_eq!(space("", None), "");
}

/// The passes above set the gaps they know about; the limit covers the rest, including tables no
/// rule recognizes and the end of the file.
#[test]
fn every_run_of_empty_lines_is_capped() {
    let mut document = toml_doc::parse(concat!(
        "a = 1\n\n\n\n\nb = 2\n\n\n\n",
        "[unknown]\nx = 1\n\n\n\n# note\n\n\n\ny = 2\n\n\n\n\n",
    ))
    .expect("valid source");
    common::spacing::limit_blank_runs(&mut document, 2);

    assert_eq!(
        document.to_string(),
        concat!(
            "a = 1\n\n\nb = 2\n\n\n",
            "[unknown]\nx = 1\n\n\n# note\n\n\ny = 2\n\n\n",
        )
    );
}

/// The pass walks the document's own trivia, so empty lines a value holds are not formatting.
#[test]
fn empty_lines_inside_a_multiline_string_are_left_alone() {
    let source = "a = \"\"\"one\n\n\n\n\ntwo\"\"\"\n";
    let mut document = toml_doc::parse(source).expect("valid source");
    common::spacing::limit_blank_runs(&mut document, 2);

    assert_eq!(document.to_string(), source);
}

/// A commented inline table keeps the spacing the file gave it, so the limit is what reaches the
/// empty lines it holds.
#[test]
fn empty_lines_inside_a_value_are_capped_too() {
    let mut document = toml_doc::parse(concat!(
        "value = {\n  one = 1,\n\n\n\n\n  # still in the table\n  two = 2,\n}\n",
        "list = [\n  1,\n\n\n\n\n  # still in the array\n  2,\n]\n",
    ))
    .expect("valid source");
    common::spacing::limit_blank_runs(&mut document, 2);

    assert_eq!(
        document.to_string(),
        concat!(
            "value = {\n  one = 1,\n\n\n  # still in the table\n  two = 2,\n}\n",
            "list = [\n  1,\n\n\n  # still in the array\n  2,\n]\n",
        )
    );
}

#[test]
fn a_nested_value_is_reached_as_well() {
    let mut document = toml_doc::parse("a = { b = [ {\n  c = 1,\n\n\n\n\n  d = 2,\n} ] }\n").expect("valid source");
    common::spacing::limit_blank_runs(&mut document, 2);

    assert_eq!(document.to_string(), "a = { b = [ {\n  c = 1,\n\n\n  d = 2,\n} ] }\n");
}

/// The walk stops at a scalar, so what a multiline string says is not formatting.
#[test]
fn empty_lines_inside_a_value_of_its_own_are_left_alone() {
    let source = "a = [ \"\"\"one\n\n\n\n\ntwo\"\"\", '''three\n\n\n\n\nfour''' ]\n";
    let mut document = toml_doc::parse(source).expect("valid source");
    common::spacing::limit_blank_runs(&mut document, 2);

    assert_eq!(document.to_string(), source);
}

fn space(source: &str, within_group: Option<usize>) -> String {
    let mut document = toml_doc::parse(source).expect("valid source");
    Spacing {
        between_groups: 1,
        within_group,
        nested_prefixes: &["tool"],
        ending: LineEnding::Lf,
    }
    .apply(&mut document);
    document.to_string()
}
