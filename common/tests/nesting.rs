//! Folding sub-tables into their parent and writing them back out.

use common::nesting;
use toml_doc::Document;

fn parse(source: &str) -> Document<'_> {
    toml_doc::parse(source).expect("valid source")
}

#[test]
fn a_sub_table_folds_into_its_parent_as_dotted_keys() {
    let mut document = parse("[tool.x]\na = 1\n[tool.x.sub]\nb = 2\nc = 3\n");
    nesting::collapse(&mut document, "tool.x");

    assert_eq!(document.to_string(), "[tool.x]\na = 1\nsub.b = 2\nsub.c = 3\n");
}

#[test]
fn an_emptied_sub_table_is_kept_as_an_inline_table() {
    let mut document = parse("[tool.x]\na = 1\n[tool.x.sub]\n");
    nesting::collapse(&mut document, "tool.x");

    assert_eq!(document.to_string(), "[tool.x]\na = 1\nsub = {}\n");
}

#[test]
fn an_array_of_tables_folds_into_an_array_of_inline_tables() {
    let mut document = parse("[tool.x]\na = 1\n[[tool.x.sub]]\nb = 2\n[[tool.x.sub]]\nb = 3\n");
    nesting::collapse(&mut document, "tool.x");

    assert_eq!(document.to_string(), "[tool.x]\na = 1\nsub = [{b = 2},{b = 3}]\n");
}

#[test]
fn tables_fold_all_the_way_up_however_deep_they_go() {
    let mut document = parse("[tool.x]\n[tool.x.sub]\nb = 2\n[tool.x.sub.deeper]\nc = 3\n");
    nesting::collapse(&mut document, "tool.x");

    assert_eq!(document.to_string(), "[tool.x]\nsub.b = 2\nsub.deeper.c = 3\n");
}

#[test]
fn dotted_keys_are_written_back_out_as_headers() {
    let mut document = parse("[tool.x]\nplain = 0\nsub.b = 2\nsub.c = 3\n");
    nesting::expand(&mut document, "tool.x");

    assert_eq!(
        document.to_string(),
        "[tool.x]\nplain = 0\n[tool.x.sub]\nb = 2\nc = 3\n"
    );
}

#[test]
fn expanding_then_collapsing_returns_the_document() {
    let source = "[tool.x]\na = 1\nsub.b = 2\n";
    let mut document = parse(source);
    nesting::expand(&mut document, "tool.x");
    nesting::collapse(&mut document, "tool.x");

    assert_eq!(document.to_string(), source);
}

#[test]
fn folding_an_array_of_tables_carries_the_comments_of_its_first_key() {
    let mut document = parse("[tool.x]\n\n# why\n[[tool.x.sub]]\nb = 2  # beside\n");
    nesting::collapse(&mut document, "tool.x");

    assert_eq!(document.to_string(), "[tool.x]\nsub = [# why\n# beside\n{b = 2}]\n");
}

#[test]
fn an_array_of_tables_held_back_by_name_stays_written_out() {
    let mut document = parse("[tool.x]\n[[tool.x.sub]]\nb = 2\n");
    nesting::collapse_where(
        &mut document,
        "tool.x",
        &|name| name != ["tool", "x", "sub"],
        nesting::Width { column: 120, indent: 2 },
    );

    assert_eq!(document.to_string(), "[tool.x]\n[[tool.x.sub]]\nb = 2\n");
}

#[test]
fn a_table_the_file_never_wrote_is_written_out_to_hold_the_fold() {
    let mut document = parse("# about the leaf\n[tool.x.sub]\nb = 2\n");
    nesting::collapse(&mut document, "tool.x");

    assert_eq!(document.to_string(), "[tool.x]\n# about the leaf\nsub.b = 2\n");
}

#[test]
fn an_array_of_tables_with_nothing_in_it_stays_written_out() {
    let mut document = parse("[tool.x]\n[[tool.x.sub]]\n");
    nesting::collapse(&mut document, "tool.x");

    assert_eq!(document.to_string(), "[tool.x]\n[[tool.x.sub]]\n");
}

#[test]
fn an_array_of_tables_holding_a_comment_past_its_first_key_stays_written_out() {
    let mut document = parse("[tool.x]\n[[tool.x.sub]]\nb = 2\nc = 3  # why\n");
    nesting::collapse(&mut document, "tool.x");

    assert_eq!(document.to_string(), "[tool.x]\n[[tool.x.sub]]\nb = 2\nc = 3  # why\n");
}

#[test]
fn an_array_of_tables_too_wide_for_a_line_stays_written_out() {
    let mut document = parse("[tool.x]\n[[tool.x.sub]]\nb = \"a rather long value indeed\"\n");
    nesting::collapse_where(
        &mut document,
        "tool.x",
        &|_| true,
        nesting::Width { column: 20, indent: 2 },
    );

    assert_eq!(
        document.to_string(),
        "[tool.x]\n[[tool.x.sub]]\nb = \"a rather long value indeed\"\n"
    );
}

/// A line is measured in the columns it takes, so a wide character costs two of them here as it
/// does everywhere else the column is checked.
#[test]
fn an_array_of_tables_of_wide_characters_is_measured_in_columns() {
    let collapsed = |column: usize| {
        let mut document = parse(
            "[tool.x]\n[[tool.x.sub]]\nb = \"\u{754c}\u{754c}\u{754c}\u{754c}\u{754c}\u{754c}\u{754c}\u{754c}\"\n",
        );
        nesting::collapse_where(&mut document, "tool.x", &|_| true, nesting::Width { column, indent: 2 });
        document.to_string()
    };

    assert_eq!(
        collapsed(30),
        "[tool.x]\n[[tool.x.sub]]\nb = \"\u{754c}\u{754c}\u{754c}\u{754c}\u{754c}\u{754c}\u{754c}\u{754c}\"\n"
    );
    assert_eq!(
        collapsed(40),
        "[tool.x]\nsub = [{b = \"\u{754c}\u{754c}\u{754c}\u{754c}\u{754c}\u{754c}\u{754c}\u{754c}\"}]\n"
    );
}

#[test]
fn a_table_held_back_by_name_stays_written_out() {
    let mut document = parse("[tool.x]\n[tool.x.sub]\nb = 2\n");
    nesting::collapse_where(
        &mut document,
        "tool.x",
        &|name| name != ["tool", "x", "sub"],
        nesting::Width { column: 120, indent: 2 },
    );

    assert_eq!(document.to_string(), "[tool.x]\n[tool.x.sub]\nb = 2\n");
}

/// The same header under two array elements names two tables, and neither has one place to fold
/// into.
#[test]
fn a_table_written_once_per_element_stays_written_out() {
    let source = "[[tool.x]]\n[tool.x.sub]\nb = 2\n[[tool.x]]\n[tool.x.sub]\nc = 3\n";
    let mut document = parse(source);
    nesting::collapse(&mut document, "tool.x");

    assert_eq!(document.to_string(), source);
}

#[test]
fn a_table_with_nothing_in_it_but_tables_below_stays_written_out() {
    let mut document = parse("[tool.x]\n[tool.x.sub]\n[[tool.x.sub.deeper]]\n");
    nesting::collapse(&mut document, "tool.x");

    assert_eq!(document.to_string(), "[tool.x]\n[tool.x.sub]\n[[tool.x.sub.deeper]]\n");
}

/// An element with nothing in it is still an element, and `{}` says exactly that.
#[test]
fn an_empty_element_keeps_its_place_in_a_folded_array_of_tables() {
    let fold = |source: &str| {
        let mut document = parse(source);
        nesting::collapse(&mut document, "tool.x");
        document.to_string()
    };

    assert_eq!(
        fold("[tool.x]\n[[tool.x.sub]]\nb = 1\n[[tool.x.sub]]\n[[tool.x.sub]]\nb = 3\n"),
        "[tool.x]\nsub = [{b = 1},{},{b = 3}]\n"
    );
    assert_eq!(
        fold("[tool.x]\n[[tool.x.sub]]\n[[tool.x.sub]]\nb = 2\n"),
        "[tool.x]\nsub = [{},{b = 2}]\n"
    );
    assert_eq!(
        fold("[tool.x]\n[[tool.x.sub]]\nb = 1\n[[tool.x.sub]]\n"),
        "[tool.x]\nsub = [{b = 1},{}]\n"
    );
}

/// Once the array is a value, a `[name.child]` header has nothing left to extend.
#[test]
fn an_array_of_tables_with_a_table_under_it_stays_written_out() {
    let source = "[[tool.x.sub]]\nname = \"first\"\n[tool.x.sub.child]\nv = 1\n[[tool.x.sub]]\nname = \"second\"\n[tool.x.sub.child]\nv = 2\n";
    let mut document = parse(source);
    nesting::collapse(&mut document, "tool.x");

    assert_eq!(document.to_string(), source);
}

/// One child of one element has a place to go: the element the file wrote it under.
#[test]
fn a_table_under_the_only_element_folds_into_it() {
    let mut document = parse("[[tool.x.sub]]\nname = \"first\"\n[[tool.x.sub.child]]\nv = 1\n");
    nesting::collapse(&mut document, "tool.x");

    assert_eq!(
        document.to_string(),
        "[tool.x]\nsub = [{name = \"first\",child = [{v = 1}]}]\n"
    );
}

/// A comment beside a header is about the table it opens, so folding that table carries it along.
#[test]
fn folding_carries_the_comment_written_beside_the_header() {
    let fold = |source: &str| {
        let mut document = parse(source);
        nesting::collapse(&mut document, "tool.x");
        let written = document.to_string();
        assert!(toml_doc::parse(&written).is_ok(), "{written}");
        written
    };

    assert_eq!(
        fold("[tool.x]\n[tool.x.sub] # plain\nb = 1\n"),
        "[tool.x]\n# plain\nsub.b = 1\n"
    );
    assert_eq!(
        fold("[tool.x]\n[[tool.x.sub]] # array\nb = 1\n"),
        "[tool.x]\nsub = [# array\n{b = 1}]\n"
    );
    assert_eq!(fold("[tool.x]\n[tool.x.sub] # empty\n"), "[tool.x]\nsub = {} # empty\n");
}

/// A quoted segment holding a dot is one segment. Reading it back as two would rename the table.
#[test]
fn a_quoted_name_holding_a_dot_keeps_its_meaning_through_a_fold() {
    let round_trip = |source: &str| {
        let before: toml::Table = source.parse().expect("valid source");
        let mut document = parse(source);
        nesting::collapse(&mut document, "tool.x");
        let out = document.to_string();
        assert_eq!(out.parse::<toml::Table>().expect("valid output"), before, "{out}");
        out
    };

    assert_eq!(
        round_trip("[[tool.x.\"a.b\"]]\nk = 1\n"),
        "[tool.x]\n\"a.b\" = [{k = 1}]\n"
    );
    assert_eq!(round_trip("[tool.x]\n[tool.x.\"a.b\"]\n"), "[tool.x]\n\"a.b\" = {}\n");
    assert_eq!(round_trip("[tool.x.\"a.b\"]\nk = 1\n"), "[tool.x]\n\"a.b\".k = 1\n");
    assert_eq!(
        round_trip("[tool.x.\"a.b\".\"c.d\"]\nk = 1\n"),
        "[tool.x]\n\"a.b\".\"c.d\".k = 1\n"
    );
    assert_eq!(
        round_trip("[tool.x.\"a.b\"]\nk = 1\n[tool.x.\"a.b\".deeper]\nj = 2\n"),
        "[tool.x]\n\"a.b\".k = 1\n\"a.b\".deeper.j = 2\n"
    );
}

#[test]
fn a_quoted_name_holding_a_dot_keeps_its_meaning_through_an_expansion() {
    let source = "[tool.x]\n\"a.b\".k = 1\n";
    let before: toml::Table = source.parse().expect("valid source");
    let mut document = parse(source);
    nesting::expand(&mut document, "tool.x");
    let out = document.to_string();

    assert_eq!(out.parse::<toml::Table>().expect("valid output"), before, "{out}");
    assert_eq!(out, "[tool.x]\n[tool.x.\"a.b\"]\nk = 1\n");
}

#[test]
fn one_array_of_tables_can_be_folded_by_name() {
    let mut document = parse("[tool.x]\na = 1\n[[tool.x.sub]]\nb = 2\n[[tool.x.other]]\nc = 3\n");
    nesting::collapse_array_of_tables(&mut document, "tool.x.sub", nesting::Width { column: 120, indent: 2 });

    assert_eq!(
        document.to_string(),
        "[tool.x]\na = 1\nsub = [{b = 2}]\n[[tool.x.other]]\nc = 3\n"
    );
}

/// A comment past the first key of an element would end up inside the braces, where the rest of the
/// line after it is swallowed, so the array stays written out.
#[test]
fn an_element_holding_a_comment_keeps_the_array_written_out() {
    let source = "[[tool.x.people]]\nname = \"a\"\n# who this is\nemail = \"a@example.com\"\n";
    let mut document = parse(source);
    nesting::collapse_array_of_tables(
        &mut document,
        "tool.x.people",
        nesting::Width { column: 120, indent: 2 },
    );

    assert_eq!(document.to_string(), source);
}

/// A disabled key is one the comment beside it speaks for, and a header written out for it would
/// carry none of that.
#[test]
fn a_disabled_key_is_not_written_out_into_a_header() {
    let source = "[tool.x]\n# sub.a = 1\nsub.b = 2\n";
    let mut document = parse(source);
    let formatted = common::disabled::try_with_disabled_keys(&mut document, source, |document| {
        nesting::expand(document, "tool.x");
        Ok(())
    })
    .expect("the pass wrote a document");

    assert_eq!(formatted, "[tool.x]\n# sub.a = 1\n[tool.x.sub]\nb = 2\n");
}

/// A dotted key at the root already writes the parent out, and a header synthesized beside it would
/// name the same table a second time, which no TOML reader accepts.
#[test]
fn a_parent_a_dotted_key_already_writes_keeps_its_sub_table() {
    let source = "tool.x.a = 1\n\n[tool.x.sub]\nb = 2\n";
    let mut document = parse(source);
    nesting::collapse(&mut document, "tool.x");

    assert_eq!(document.to_string(), source);
}

#[test]
fn a_parent_a_dotted_key_already_writes_keeps_its_array_of_tables() {
    let source = "tool.x.a = 1\n\n[[tool.x.sub]]\nb = 2\n";
    let mut document = parse(source);
    nesting::collapse_array_of_tables(&mut document, "tool.x.sub", nesting::Width { column: 120, indent: 2 });

    assert_eq!(document.to_string(), source);
}

/// The dotted key sits in a table of its own, where it still spells out the same path.
#[test]
fn a_parent_another_section_writes_with_a_dotted_key_keeps_its_sub_table() {
    let source = "[tool]\nx.a = 1\n\n[tool.x.sub]\nb = 2\n";
    let mut document = parse(source);
    nesting::collapse(&mut document, "tool.x");

    assert_eq!(document.to_string(), source);
}

/// Which element of the parent array a child belongs to is what the order it is written in says,
/// and one folded array would say every child belongs to the first element.
#[test]
fn children_of_several_array_elements_stay_written_out() {
    let source = concat!(
        "[[tool.x.groups]]\nname = \"one\"\n",
        "[[tool.x.groups.items]]\nvalue = 1\n",
        "[[tool.x.groups]]\nname = \"two\"\n",
        "[[tool.x.groups.items]]\nvalue = 2\n",
    );
    let mut document = parse(source);
    nesting::collapse_array_of_tables(
        &mut document,
        "tool.x.groups.items",
        nesting::Width { column: 120, indent: 2 },
    );
    nesting::collapse_array_of_tables(
        &mut document,
        "tool.x.groups",
        nesting::Width { column: 120, indent: 2 },
    );

    assert_eq!(document.to_string(), source);
}

/// A table whose every key is disabled is one the file wrote empty, and folding those keys into the
/// parent would leave nothing saying the table is there.
#[test]
fn a_table_of_only_disabled_keys_is_not_folded() {
    let source = "[tool.x]\na = 1\n[tool.x.sub]\n# b = 2\n";
    let mut document = parse(source);
    let formatted = common::disabled::try_with_disabled_keys(&mut document, source, |document| {
        nesting::collapse(document, "tool.x");
        Ok(())
    })
    .expect("the pass wrote a document");

    assert_eq!(formatted, source);
}
