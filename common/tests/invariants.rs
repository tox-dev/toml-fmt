//! Every pass leaves behind a document a parser still accepts.
//!
//! A pass that wrote something only a later pass could repair would make itself unusable on its
//! own, so each one is run alone here and its output read back.

use common::layout::Layout;
use common::{arrays, layout, nesting, sections, spacing, strings};
use toml_doc::{Document, LineEnding, Value};

/// Documents holding the shapes a pass can trip over: comments in every slot, quoted names, empty
/// tables, arrays of tables, nesting deep enough to fold more than once, and files that run out
/// before their last line ends, where a moved item would otherwise take the end of the file along.
const SOURCES: &[&str] = &[
    "",
    "a = 1\n",
    "# lead\n[tool.x]\n\n# why\nb = 2  # beside\n\n# trailing\n",
    "[tool.x]\n[[tool.x.sub]]\nb = 2\n[[tool.x.sub]]\nc = 3\n",
    "[tool.x]\n\n# why\n[[tool.x.sub]]\nb = 2  # beside\n",
    "[tool.x.sub.deeper]\nc = 3\n[tool.x.sub]\nb = 2\n",
    "[tool.x]\na = [\n  # above\n  1,  # beside\n  2,\n]\n",
    "[tool.x]\na = [ \"keep\", \"drop\" ]\n",
    "[tool.x]\na = [\n  \"drop\", # first\n  \"keep\",\n]\n",
    "[tool.x]\na = [\n  \"keep\",\n  \"drop\" # before the comma\n  ,\n]\n",
    "[tool.x]\na = { b = 1, c = 2 }\n",
    "[tool.x]\na = {\n  # about b\n  b = 1, # beside b\n  c = 2,\n}\n",
    "[tool.x.\"a.b\"]\nk = 1\n",
    "[[tool.x.\"a.b\"]]\nk = 1\n",
    "[tool.x]\n[tool.x.\"a.b\"]\n",
    "[tool.x]\nlong = \"once upon a time there was a very long string indeed to wrap\"\n",
    "[tool.x]\n# Group: one\nz = 1\ny = 2\n# Group: two\nb = 3\na = 4\n",
    "a = 1  # beside",
    "[tool.x]\nb = 2\na = 1",
    "[tool.a]\nx = 1\n\n[tool.z]",
    "[tool.x]\nsub.a = 1\nplain = 2",
    "[tool.x]\n[other]\no = 1\n[tool.x.sub]\na = 2",
    concat!(
        "[[fruit]]\nname = \"apple\"\n",
        "[fruit.physical]\ncolor = \"red\"\n",
        "[[fruit.variety]]\nname = \"red delicious\"\n",
        "[[fruit]]\nname = \"banana\"\n",
        "[fruit.physical]\ncolor = \"yellow\"\n",
    ),
];

fn parse(source: &str) -> Document<'_> {
    toml_doc::parse(source).expect("valid source")
}

fn check(source: &str, pass: &str, act: impl FnOnce(&mut Document<'_>)) {
    let mut document = parse(source);
    act(&mut document);
    let written = document.to_string();
    assert!(
        toml_doc::parse(&written).is_ok(),
        "{pass} left something no parser reads back, from {source:?}:\n{written}"
    );
    // an independent parser catches what is well formed yet says something no TOML file can say
    assert!(
        written.parse::<toml::Table>().is_ok(),
        "{pass} left something no TOML document can say, from {source:?}:\n{written}"
    );
}

fn each(pass: &str, act: impl Fn(&mut Document<'_>) + Copy) {
    for source in SOURCES {
        check(source, pass, act);
    }
}

/// Passes that move things around rather than rewrite them have to leave the document saying the
/// same thing. Reading the output back is not enough: a child table can end up attached to the
/// wrong array element and still parse.
fn keeps_its_meaning(pass: &str, act: impl Fn(&mut Document<'_>) + Copy) {
    for source in SOURCES {
        check(source, pass, act);
        let mut document = parse(source);
        act(&mut document);
        let written = document.to_string();
        assert_eq!(
            written.parse::<toml::Table>().expect("valid output"),
            source.parse::<toml::Table>().expect("valid source"),
            "{pass} changed what the document says, from {source:?}:\n{written}"
        );
    }
}

#[test]
fn collapsing_leaves_a_document_that_reads_back() {
    keeps_its_meaning("collapse", |document| nesting::collapse(document, "tool.x"));
}

#[test]
fn collapsing_one_table_of_several_leaves_a_document_that_reads_back() {
    keeps_its_meaning("collapse_where", |document| {
        nesting::collapse_where(
            document,
            "tool.x",
            &|name| name.last().is_none_or(|leaf| leaf != "sub"),
            nesting::Width { column: 40, indent: 2 },
        );
    });
}

#[test]
fn expanding_leaves_a_document_that_reads_back() {
    keeps_its_meaning("expand", |document| nesting::expand(document, "tool.x"));
}

#[test]
fn laying_out_leaves_a_document_that_reads_back() {
    keeps_its_meaning("layout", |document| {
        Layout {
            column_width: 40,
            indent: 2,
            ending: LineEnding::Lf,
        }
        .apply(document);
        layout::align_array_comments(document);
    });
}

#[test]
fn spacing_leaves_a_document_that_reads_back() {
    keeps_its_meaning("spacing", |document| {
        spacing::Spacing {
            between_groups: 1,
            within_group: Some(0),
            nested_prefixes: &["tool"],
            ending: LineEnding::Lf,
        }
        .apply(document);
    });
}

#[test]
fn ordering_leaves_a_document_that_reads_back() {
    keeps_its_meaning("reorder", |document| {
        sections::reorder_within(document, &["tool.x"], &["tool"], &|_| None);
        for section in &mut document.sections {
            sections::reorder_keys(&mut section.entries, &["b"]);
        }
    });
}

#[test]
fn rewriting_values_leaves_a_document_that_reads_back() {
    each("strings", |document| {
        strings::normalize_key_quotes(document);
        strings::wrap_long_strings(document, 20, 2, &[]);
    });
}

#[test]
fn reordering_members_leaves_a_document_that_reads_back() {
    each("arrays", |document| {
        for section in &mut document.sections {
            for entry in &mut section.entries {
                let Value::Array(array) = &mut entry.key_value.value else {
                    continue;
                };
                arrays::sort_strings(array, &str::to_owned, &str::cmp);
                arrays::dedupe_strings(array, &str::to_owned);
                arrays::retain_strings(array, |text| text != "drop");
                arrays::map_strings(array, str::to_lowercase);
            }
        }
    });
}

/// The file the caller holds is the last valid one until the formatter returns, so what it wrote is
/// read back before it goes anywhere.
#[test]
fn what_the_formatter_writes_reads_back_as_a_document() {
    assert_eq!(
        common::written_document("a = [1,\n]\n"),
        Ok(String::from("a = [1,\n]\n"))
    );
    assert!(common::written_document("a = [1,\n").is_err());
}
