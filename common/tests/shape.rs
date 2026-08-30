//! The shape a formatted file takes: which tables fold into their parents, and what the layout
//! writes once every rule has run.

use common::shape::{Tables, Written};

fn named(name: &str) -> Vec<String> {
    common::sections::parse_name(name)
}

/// The closest setting that names a table or one above it decides whether it folds, so a setting
/// on a child outranks one on its parent.
#[test]
fn the_closest_setting_says_whether_a_table_folds() {
    let held = Tables::new(
        "short",
        &[String::from("tool.a"), String::from("tool.b.c")],
        &[String::from("tool.a.keep")],
    );

    assert!(held.should_collapse(&named("tool.other")));
    assert!(!held.should_collapse(&named("tool.a")));
    assert!(!held.should_collapse(&named("tool.a.child")));
    assert!(held.should_collapse(&named("tool.a.keep")));
    assert!(!held.should_collapse(&named("tool.b.c")));
    assert!(held.should_collapse(&named("tool.b")));
}

/// A name TOML quotes is one segment, so a setting cannot cut one holding a dot in half.
#[test]
fn a_quoted_name_is_the_one_segment_the_file_wrote() {
    let held = Tables::new("long", &[], &[String::from("tool.\"a.b\"")]);

    assert!(held.should_collapse(&named("tool.\"a.b\"")));
    assert!(!held.should_collapse(&named("tool.a.b")));
    assert!(!held.should_collapse(&named("tool.a")));
}

/// What the layout writes is the same for every formatter: the wrapping, the lines, the comment
/// columns and the blank lines between tables.
#[test]
fn what_the_layout_writes_is_written_once() {
    let mut document = toml_doc::parse("[tool.a]\nheld=[1,2]\n[tool.b]\nc=1\n").expect("valid source");
    Written {
        column_width: 120,
        indent: 2,
        separate_root_table: "\n",
        sub_table_spacing: "",
        table_format: "short",
        skip_wrap_for_keys: &[],
        nested_prefixes: &["tool"],
    }
    .apply(&mut document);

    assert_eq!(document.to_string(), "[tool.a]\nheld = [ 1, 2 ]\n\n[tool.b]\nc = 1\n");
}

/// A long value is broken up where the column has no room for it, and the keys a caller names are
/// left as the file wrote them.
#[test]
fn a_skipped_key_keeps_the_value_the_file_wrote() {
    let source = "[tool.a]\nheld = \"one two three four five six\"\n";
    let written = |skip: &[String]| {
        let mut document = toml_doc::parse(source).expect("valid source");
        Written {
            column_width: 30,
            indent: 2,
            separate_root_table: "\n",
            sub_table_spacing: "\n",
            table_format: "long",
            skip_wrap_for_keys: skip,
            nested_prefixes: &["tool"],
        }
        .apply(&mut document);
        document.to_string()
    };

    assert!(written(&[]).contains("\"\"\"\\"), "{}", written(&[]));
    assert_eq!(written(&[String::from("tool.a.held")]), source);
}
