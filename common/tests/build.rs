//! Making entries, arrays and sections that were never in the source.

use common::build;
use toml_doc::{Document, SectionKind};

#[test]
fn an_entry_carries_its_key_and_value() {
    let mut document = Document::default();
    document.root.push(build::string_entry("name", "example"));
    document
        .root
        .push(build::entry("tags", build::array([build::string("one")])));

    assert_eq!(document.to_string(), "name = \"example\"\ntags = [\"one\"]\n");
}

#[test]
fn a_section_holds_the_entries_it_was_given() {
    let mut document = Document::default();
    document.sections.push(build::section(
        "tool.demo",
        SectionKind::Table,
        vec![build::string_entry("kind", "table")],
    ));
    document.sections.push(build::section(
        "tool.demo.item",
        SectionKind::ArrayOfTables,
        vec![build::string_entry("kind", "array")],
    ));

    assert_eq!(
        document.to_string(),
        "[tool.demo]\nkind = \"table\"\n[[tool.demo.item]]\nkind = \"array\"\n"
    );
}

#[test]
fn a_key_holding_a_dot_is_quoted_where_it_needs_to_be() {
    let mut document = Document::default();
    document
        .sections
        .push(build::section("tool.demo", SectionKind::Table, vec![]));
    document.root.push(build::string_entry("a b", "spaced"));

    assert_eq!(document.to_string(), "\"a b\" = \"spaced\"\n[tool.demo]\n");
}

/// A table written as a value holds key-values of its own, which are the same key-values a section
/// holds without the lines around them.
#[test]
fn a_key_value_is_built_from_its_name_and_what_it_holds() {
    let held = build::key_value("a.b", build::string("held"));

    assert_eq!(held.to_string(), "a.b = \"held\"");
}
