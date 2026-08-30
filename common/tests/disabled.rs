//! Uncommenting a disabled key for the formatting pass, then commenting it back.

use common::disabled::{MARKER, try_with_disabled_keys};

/// The formatter the pass brackets, standing in for the real one: it only reports what it saw.
fn round_trip(source: &str) -> (String, String) {
    let mut seen = String::new();
    let out = held(source, |document| seen = document.to_string());
    (seen, out)
}

/// Run a formatter over the source the way the formatters do, from the document they already read.
fn held(source: &str, format: impl FnOnce(&mut toml_doc::Document<'_>)) -> String {
    tried(source, |document| {
        format(document);
        Ok(())
    })
    .expect("the pass wrote a document")
}

/// The same, for a pass that may reject what it was handed.
fn tried(
    source: &str,
    format: impl FnOnce(&mut toml_doc::Document<'_>) -> Result<(), String>,
) -> Result<String, String> {
    let mut document = toml_doc::parse(source).expect("valid source");
    try_with_disabled_keys(&mut document, source, format)
}

#[test]
fn a_commented_key_reaches_the_formatter_uncommented() {
    let (seen, out) = round_trip("# default = true\n");

    assert_eq!(seen, format!("default = true  # {MARKER}\n"));
    assert_eq!(out, "# default = true\n");
}

#[test]
fn prose_that_is_not_a_key_stays_a_comment() {
    assert_eq!(
        round_trip("# just a note\n"),
        ("# just a note\n".to_owned(), "# just a note\n".to_owned())
    );
}

#[test]
fn a_comment_holding_two_keys_stays_a_comment() {
    assert_eq!(round_trip("# a = 1\n# b = 2\n").1, "# a = 1\n# b = 2\n");
}

#[test]
fn a_commented_table_header_holds_its_keys_back() {
    let (seen, out) = round_trip("# [tool.x]\n# a = 1\n");

    assert_eq!(seen, "# [tool.x]\n# a = 1\n");
    assert_eq!(out, "# [tool.x]\n# a = 1\n");
}

#[test]
fn a_table_header_inside_a_run_ends_it() {
    let (seen, _) = round_trip("# a = [\n# [tool.x]\n");

    assert_eq!(seen, "# a = [\n# [tool.x]\n");
}

#[test]
fn a_value_spread_over_several_lines_comes_back_commented() {
    let source = "# a = [\n#   1,\n# ]\n";
    let (seen, out) = round_trip(source);

    assert_eq!(seen, format!("a = [\n  1,\n]  # {MARKER}\n"));
    assert_eq!(out, source);
}

#[test]
fn a_rejected_pass_restores_nothing() {
    let mut document = toml_doc::parse("# a = 1\n").expect("valid source");
    let error = try_with_disabled_keys(&mut document, "# a = 1\n", |_| Err(String::from("rejected")));

    assert_eq!(error, Err(String::from("rejected")));
}

/// Turning a key back on can leave the same name written twice, which no reader reads. The lines
/// still come back commented, and the caller is the one that reports on what was written.
#[test]
fn a_formatter_leaving_a_name_written_twice_still_gets_its_lines_back() {
    let out = held("# a = 1\na = 2\n", |document| {
        document.root.swap(0, 1);
    });

    assert_eq!(out, "a = 2\n# a = 1\n");
}

#[test]
fn a_key_written_before_any_table_comes_back_commented() {
    assert_eq!(round_trip("a = 1\n# b = 2\n").1, "a = 1\n# b = 2\n");
}

#[test]
fn an_empty_comment_line_stays_a_comment() {
    assert_eq!(round_trip("#\n").1, "#\n");
}

#[test]
fn a_disabled_key_that_already_ends_in_a_comment_gains_only_the_marker() {
    let (seen, out) = round_trip("# a = 1  # why\n");

    assert_eq!(seen, format!("a = 1  # why {MARKER}-kept\n"));
    assert_eq!(out, "# a = 1  # why\n");
}

#[test]
fn a_disabled_key_under_a_table_comes_back_where_it_was() {
    assert_eq!(round_trip("[tool.x]\na = 1\n# b = 2\n").1, "[tool.x]\na = 1\n# b = 2\n");
}

#[test]
fn a_source_without_a_closing_line_break_keeps_it_that_way() {
    assert_eq!(round_trip("# a = 1").1, "# a = 1");
}

/// A file is free to hold the marker's text itself. Only the comment the pass wrote gives a key
/// back, so nothing the file says is read as one.
#[test]
fn a_value_holding_the_marker_text_is_left_alone() {
    let holding = [
        format!("a = \"{MARKER}\"\n"),
        format!("a = 1  # {MARKER}\n"),
        format!("# {MARKER}\na = 1\n"),
        format!("\"{MARKER}\" = 1\n"),
        format!("a = \"\"\"\n{MARKER}\n\"\"\"\n"),
    ];
    for source in holding {
        assert_eq!(held(&source, |_| ()), source, "{source}");
    }
}

#[test]
fn a_disabled_key_still_comes_back_when_the_file_holds_the_marker_text() {
    let source = format!("a = \"{MARKER}\"\n# b = 2\n");

    assert_eq!(held(&source, |_| ()), source);
}

/// A `#` inside a value is part of what that value says, so nothing there is read as a disabled key.
#[test]
fn a_comment_inside_a_value_is_left_as_prose() {
    let source = "a = [\n  # b = 1\n  \"x\",\n]\n";

    let mut document = toml_doc::parse(source).expect("valid source");
    let read = try_with_disabled_keys(&mut document, source, |_| Ok(()));

    assert_eq!(
        (round_trip(source), read),
        ((String::from(source), String::from(source)), Ok(String::from(source)))
    );
}

/// A pass that would split, drop or merge an entry asks whether the pass here speaks for it, since
/// what says the entry is disabled is the comment beside it.
#[test]
fn an_entry_this_pass_turned_on_says_so() {
    let asked = |source: &str| {
        let mut seen = Vec::new();
        held(source, |document| {
            seen = document
                .root
                .iter()
                .map(common::disabled::is_enabled_here)
                .collect::<Vec<bool>>();
        });
        seen
    };

    assert_eq!(asked("# a = 1\nb = 2\n"), [true, false]);
    // a key the file already wrote a comment beside keeps it, with the marker written after it
    assert_eq!(asked("# a = 1  # why\nb = 2\n"), [true, false]);
}

/// The comment beside a disabled key is the file's own, so it comes back whole, hashes and all.
#[test]
fn a_disabled_key_keeps_the_comment_written_beside_it() {
    let source = "# a = 1  # why #\n";

    assert_eq!(round_trip(source).1, source);
}

/// What stands above a disabled key is a comment of the file's own, so it keeps every layer it was
/// written with.
#[test]
fn prose_above_a_disabled_key_keeps_its_hashes() {
    let heading = "# # heading\n# a = 1\n";
    let empty = "#\n# a = 1\n";

    assert_eq!(
        (round_trip(heading).1, round_trip(empty).1),
        (String::from(heading), String::from(empty))
    );
}

/// The marker names the pass that wrote it, so a comment the file wrote is ordinary configuration
/// however closely it spells one.
#[test]
fn a_comment_the_file_wrote_is_not_a_disabled_key() {
    let wrote = |comment: String| {
        let source = format!("[tool.x]\nsub.a = 1  # {comment}\nheld = \"{MARKER}\"\n");
        held(&source, |document| common::nesting::expand(document, "tool.x"))
    };

    // the file already holds the marker text, so the pass runs with a longer one
    assert_eq!(
        wrote(String::from(MARKER)),
        format!("[tool.x]\nheld = \"{MARKER}\"\n[tool.x.sub]\na = 1  # {MARKER}\n")
    );
    assert_eq!(
        wrote(format!("{MARKER}-kept")),
        format!("[tool.x]\nheld = \"{MARKER}\"\n[tool.x.sub]\na = 1  # {MARKER}-kept\n")
    );
    assert_eq!(
        wrote(format!("see {MARKER} for why")),
        format!("[tool.x]\nheld = \"{MARKER}\"\n[tool.x.sub]\na = 1  # see {MARKER} for why\n")
    );
}

/// A file already holding the marker text is formatted with a longer one, which the guards that
/// leave a disabled key where it is still read as the marker it is.
#[test]
fn a_longer_marker_still_says_the_key_is_disabled() {
    let source = format!("[tool.x]\nheld = \"{MARKER}\"\n# sub.a = 1\n");

    let formatted = held(&source, |document| common::nesting::expand(document, "tool.x"));

    assert_eq!(formatted, source);
}

/// A value spread over several lines holds whatever it says, a line shaped like a table header
/// included, and it is the value that decides where the run ends.
#[test]
fn a_header_inside_a_disabled_value_is_what_the_value_says() {
    let source = "# script = \"\"\"\n# [tool.lookalike]\n# \"\"\"\n";
    let (seen, out) = round_trip(source);

    assert_eq!(seen, format!("script = \"\"\"\n[tool.lookalike]\n\"\"\"  # {MARKER}\n"));
    assert_eq!(out, source);
}

/// A commented header that opens no value still ends the run, since the keys below it belong to
/// the table it names.
#[test]
fn a_header_beside_a_disabled_key_still_ends_the_run() {
    let source = "# a = 1\n# [tool.x]\n# b = 2\n";

    assert_eq!(round_trip(source).1, source);
}

/// A value the lines hold open carries a header-shaped line, while a run with nothing open ends at
/// one: the keys below it belong to the table it names.
#[test]
fn a_run_a_value_does_not_hold_open_ends_at_a_header() {
    let (seen, _) = round_trip("# a = \"\"\"x\n# [tool.y]\n# \"\"\"\n");
    assert_eq!(seen, format!("a = \"\"\"x\n[tool.y]\n\"\"\"  # {MARKER}\n"));

    for source in [
        "# a =\n# [tool.x]\n# b = 2\n",
        "# a = [ [1],\n# [tool.z]\n# ]\n",
        "# a = [\"x\",\n# [tool.z]\n# ]\n",
    ] {
        assert_eq!(round_trip(source).1, source, "{source}");
    }
}

/// A comment inside a disabled value runs to the end of its line, so a bracket it holds closes
/// nothing and the lines below it are still part of the value.
#[test]
fn a_comment_inside_a_disabled_value_closes_nothing() {
    let (seen, _) = round_trip("# a = [ # not ]\n# 1,\n# ]\n");

    assert_eq!(seen, format!("a = [ # not ]\n1,\n]  # {MARKER}\n"));
}

/// A string written with one quote closes on the line it opened, so a run whose first line leaves
/// one open reads no further.
#[test]
fn a_quote_left_open_closes_with_its_line() {
    let source = "# a = \"one\n# b = 2\n";

    assert_eq!(round_trip(source).1, source);
}

/// Every marker is the base one followed by some number of `x`, so the file's longest run after it
/// says how long this one has to be.
#[test]
fn a_marker_is_longer_than_every_run_the_file_holds() {
    let source = format!("a = \"{MARKER}x {MARKER}xxx\"\n# b = 2\n");
    let mut seen = String::new();

    let out = held(&source, |document| seen = document.to_string());

    assert!(seen.contains(&format!("b = 2  # {MARKER}xxxx")), "{seen}");
    assert_eq!(out, source);
}

/// A byte-order mark opens a document and says nothing inside one, so a comment holding one is
/// prose however much the rest of it reads like a key. The keys beside it are unaffected.
#[test]
fn a_comment_holding_a_byte_order_mark_stays_prose() {
    let source = "[tool.x]\n# \u{feff}a = 1\n# b = 2\n";
    let (seen, out) = round_trip(source);

    assert_eq!(seen, format!("[tool.x]\n# \u{feff}a = 1\nb = 2  # {MARKER}\n"));
    assert_eq!(out, source);
}

/// A value one line opens and never closes says nothing about the lines below it, so a key written
/// under one is still a key of its own.
#[test]
fn a_key_below_an_unclosed_value_is_still_read() {
    let (seen, out) = round_trip("# x = [\n# y = 1\n");

    assert_eq!(seen, format!("# x = [\ny = 1  # {MARKER}\n"));
    assert_eq!(out, "# x = [\n# y = 1\n");
}

/// A run holding a string written over lines is read line by line, since a value opened inside one
/// starts reading it afresh.
#[test]
fn a_run_holding_a_string_over_lines_is_read_line_by_line() {
    let (seen, out) = round_trip("# a = \"\"\"\n# held\n# \"\"\"\n# b = 1\n");

    assert_eq!(
        seen,
        format!("a = \"\"\"\nheld\n\"\"\"  # {MARKER}\nb = 1  # {MARKER}\n")
    );
    assert_eq!(out, "# a = \"\"\"\n# held\n# \"\"\"\n# b = 1\n");
}

/// A string a run opens and never closes leaves no key to read, and the lines stay as the file
/// wrote them.
#[test]
fn a_string_a_run_never_closes_holds_no_key() {
    let source = "# a = \"\"\"\n# held\n";

    assert_eq!(round_trip(source), (String::from(source), String::from(source)));
}

/// The entry point both formatters call: read the source, run the rules over its disabled keys as
/// the keys they spell, and hand back what that wrote once it reads as a document again.
#[test]
fn the_entry_point_runs_a_pass_over_a_document_read_from_the_source() {
    let written = common::formatted("# a = 1\nb = 2\n", |document| {
        document.root.reverse();
        Ok(())
    });

    assert_eq!(written, Ok(String::from("b = 2\n# a = 1\n")));
}

/// A pass that rejects its input says so, and nothing it did reaches the caller.
#[test]
fn a_rejected_pass_says_why() {
    let written = common::formatted("a = 1\n", |_| Err(String::from("nope")));

    assert_eq!(written, Err(String::from("nope")));
}

/// A source that is not a document is reported where it stops being one.
#[test]
fn a_source_that_is_not_a_document_is_reported() {
    let written = common::formatted("a =\n", |_| Ok(()));

    assert!(written.is_err(), "{written:?}");
}

/// A commented key whose value opens and never closes reaches a line the file wrote as a key, and
/// a key is not a comment, so the run says nothing and the comment stays prose.
#[test]
fn a_commented_value_a_real_key_cuts_short_stays_a_comment() {
    let source = "# a = [\nb = 2\n";
    let (seen, out) = round_trip(source);

    assert_eq!(seen, source);
    assert_eq!(out, source);
}

/// A commented key holding a string written over lines is read line by line, and a line the file
/// wrote as a key ends the run: a key is not a comment, so there is nothing left to turn back on.
#[test]
fn a_commented_multiline_string_a_key_cuts_short_stays_a_comment() {
    let source = "# a = \"\"\"\n# text\nb = 2\n";
    let (seen, out) = round_trip(source);

    assert_eq!(seen, source);
    assert_eq!(out, source);
}
