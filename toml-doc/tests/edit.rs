//! Editing a document and writing it back.

use toml_doc::{Document, Piece, Quoting, Repr, Value};

fn parse(source: &str) -> Document<'_> {
    toml_doc::parse(source).expect("valid source")
}

#[test]
fn reordering_sections_carries_their_comments() {
    let mut document = parse("# about b\n[b]\nx = 1\n\n# about a\n[a]\ny = 2\n");
    document.sections.reverse();

    // the blank line led section a, so it travels with it to the top
    assert_eq!(document.to_string(), "\n# about a\n[a]\ny = 2\n# about b\n[b]\nx = 1\n");
}

#[test]
fn limiting_blank_runs_keeps_comments() {
    let mut document = parse("a = 1\n\n\n\n# note\n\n\nb = 2\n");
    for entry in &mut document.root {
        entry.lead.limit_blank_runs(1);
    }

    assert_eq!(document.to_string(), "a = 1\n\n# note\n\nb = 2\n");
}

#[test]
fn string_bodies_drop_their_delimiters() {
    let document = parse("a = \"one\"\nb = 'two'\nc = \"\"\"three\"\"\"\nd = '''four'''\ne = 12\n");
    let bodies: Vec<&str> = document
        .root
        .iter()
        .map(|entry| match &entry.key_value.value {
            Value::Scalar(repr) => repr.body(),
            other => panic!("expected a scalar, got {other}"),
        })
        .collect();

    assert_eq!(bodies, ["one", "two", "three", "four", "12"]);
}

#[test]
fn a_comment_after_a_comma_closes_the_member_it_follows() {
    let document = parse("a = [\n  1, # one\n  2,\n]\n");
    let Value::Array(array) = &document.root[0].key_value.value else {
        panic!("expected an array");
    };

    assert_eq!(array.members[0].after.to_string(), " # one");
    assert_eq!(array.members[1].lead.to_string(), "\n  ");
}

#[test]
fn renaming_a_key_leaves_the_rest_of_the_line_alone() {
    let mut document = parse("[table]\nold   =   1 # keep\n");
    document.sections[0].entries[0].key_value.key.parts_mut()[0].set_name("new");

    assert_eq!(document.to_string(), "[table]\nnew   =   1 # keep\n");
}

#[test]
fn trailing_comments_stay_with_the_document() {
    let document = parse("a = 1\n\n# dangling\n");

    assert!(matches!(
        document.trailing.pieces(),
        [Piece::Blank { .. }, Piece::Comment { .. }]
    ));
}

#[test]
fn truncated_sources_are_rejected() {
    let rejected: Vec<bool> = ["a.", "a =", "[a", "a = [1,", "a = {b ="]
        .iter()
        .map(|source| toml_doc::parse(source).is_err())
        .collect();

    assert_eq!(rejected, [true; 5]);
}

#[test]
fn an_error_reports_where_it_was_found() {
    let errors = toml_doc::parse("a = 1\nb =\n").expect_err("missing value");

    assert_eq!(errors[0].to_string(), "string values must be quoted at byte 9");
}

#[test]
fn trivia_lines_can_be_rewritten_in_place() {
    let mut document = parse("# keep\na = 1\n");
    document.root[0].lead.pieces_mut().clear();

    assert_eq!(document.to_string(), "a = 1\n");
}

#[test]
fn array_padding_can_be_rewritten_in_place() {
    let mut document = parse("a = [ 1 ]\n");
    let Value::Array(array) = &mut document.root[0].key_value.value else {
        panic!("expected an array");
    };
    array.members[0].lead.parts_mut().clear();

    assert_eq!(document.to_string(), "a = [1 ]\n");
}

#[test]
fn padding_reports_what_it_holds() {
    let document = parse("a = [\n  1,  # one\n  2,\n]\nb = [ 1, 2 ]\n");
    let Value::Array(broken) = &document.root[0].key_value.value else {
        panic!("the source holds an array");
    };
    let Value::Array(inline) = &document.root[1].key_value.value else {
        panic!("the source holds an array");
    };

    assert!(broken.members[0].lead.is_multiline());
    assert!(broken.members[0].after.has_comment());
    assert!(!broken.members[1].after.has_comment());
    assert!(!inline.members[0].lead.is_multiline());
    assert_eq!(inline.members[0].lead.parts().len(), 1);
}

#[test]
fn a_quoting_says_whether_it_spans_lines() {
    let document = parse("a = \"one\"\nb = '''two'''\n");
    let quoting = |at: usize| match &document.root[at].key_value.value {
        Value::Scalar(repr) => repr.quoting().expect("a string carries its quoting"),
        other => panic!("expected a scalar, got {other}"),
    };

    assert!(!quoting(0).is_multiline());
    assert!(quoting(1).is_multiline());
}

#[test]
fn commas_move_to_where_the_members_now_are() {
    let mut document = parse("a = [ 1, 2, 3 ]\n");
    let Value::Array(array) = &mut document.root[0].key_value.value else {
        panic!("the source holds an array");
    };
    assert!(!array.trailing_comma);
    array.members.remove(2);
    array.trailing_comma = true;

    assert_eq!(document.to_string(), "a = [ 1, 2, ]\n");
}

#[test]
fn an_inline_table_closes_the_same_way() {
    let mut document = parse("a = { x = 1, y = 2, }\n");
    let Value::InlineTable(table) = &mut document.root[0].key_value.value else {
        panic!("the source holds an inline table");
    };
    assert!(table.trailing_comma);
    table.trailing_comma = false;

    assert_eq!(document.to_string(), "a = { x = 1, y = 2 }\n");
}

#[test]
fn empty_lines_inside_a_container_are_capped() {
    let mut document = parse("a = [\n  1,\n\n\n\n\n  2,\n]\n");
    let Value::Array(array) = &mut document.root[0].key_value.value else {
        panic!("the source holds an array");
    };
    for member in &mut array.members {
        member.lead.limit_blank_runs(1);
    }

    assert_eq!(document.to_string(), "a = [\n  1,\n\n  2,\n]\n");
}

/// A comment closes the run, since the line it opens is not empty, and the spacing that indents
/// what comes next stays.
#[test]
fn a_comment_closes_a_run_of_empty_lines() {
    let mut document = parse("a = [\n  1,\n\n\n\n  # note\n\n\n\n  2,\n]\n");
    let Value::Array(array) = &mut document.root[0].key_value.value else {
        panic!("the source holds an array");
    };
    for member in &mut array.members {
        member.lead.limit_blank_runs(1);
    }

    assert_eq!(document.to_string(), "a = [\n  1,\n\n  # note\n\n  2,\n]\n");
}

#[test]
fn a_run_within_the_limit_is_left_alone() {
    let source = "a = [\n  1,\n\n  2,\n]\n";
    let mut document = parse(source);
    let Value::Array(array) = &mut document.root[0].key_value.value else {
        panic!("the source holds an array");
    };
    for member in &mut array.members {
        member.lead.limit_blank_runs(2);
    }

    assert_eq!(document.to_string(), source);
}

/// Spacing that only pads a line being dropped goes with that line.
#[test]
fn the_spacing_on_a_dropped_empty_line_goes_with_it() {
    let mut document = parse("a = [\n  1,\n   \n   \n   \n  2,\n]\n");
    let Value::Array(array) = &mut document.root[0].key_value.value else {
        panic!("the source holds an array");
    };
    for member in &mut array.members {
        member.lead.limit_blank_runs(1);
    }

    assert_eq!(document.to_string(), "a = [\n  1,\n   \n  2,\n]\n");
}

/// A key names something, and the ways of changing one all hold to that, so a document cannot be
/// left saying ` = 1`.
#[test]
fn a_key_cannot_be_left_naming_nothing() {
    let mut document = parse("one.two = 1\n");
    let taken = document.root[0].key_value.key.take_leading(1);

    assert_eq!(
        (
            taken.len(),
            document.root[0].key_value.key.parts().len(),
            document.to_string()
        ),
        (1, 1, String::from("two = 1\n"))
    );
}

#[test]
#[should_panic(expected = "a key names at least one segment")]
fn taking_every_segment_of_a_key_is_refused() {
    let mut document = parse("one.two = 1\n");
    let _ = document.root[0].key_value.key.take_leading(2);
}

#[test]
#[should_panic(expected = "a key names at least one segment")]
fn a_key_built_from_no_parts_is_refused() {
    let _ = toml_doc::Key::from_parts(Vec::new());
}

/// What a caller can write into a value is a value, so what comes back out reads back.
#[test]
fn a_rewritten_key_and_value_read_back() {
    let mut document = parse("one = 1\n");
    document.root[0].key_value.key.parts_mut()[0].set_name("a name");
    document.root[0].key_value.value = toml_doc::Value::Scalar(toml_doc::Repr::basic_string("said"));

    let written = document.to_string();

    assert_eq!(
        (written.as_str(), toml_doc::parse(&written).is_ok()),
        ("\"a name\" = \"said\"\n", true)
    );
}

/// Folding a table into its parent writes the parent's name ahead of every key it brought along,
/// and writing one out again takes the same names back off.
#[test]
fn a_key_can_gain_segments_at_either_end() {
    let mut document = parse("[table]\nkey = 1\n");
    let leading = toml_doc::Key::new(["tool", "x"]).parts().to_vec();
    let trailing = toml_doc::Key::new(["deep"]).parts().to_vec();

    let key = &mut document.sections[0].entries[0].key_value.key;
    key.prepend_parts(leading);
    key.extend_parts(trailing);
    let taken = toml_doc::Key::from_parts(key.take_leading(2));

    assert_eq!(
        (document.to_string(), taken.to_string()),
        (String::from("[table]\nkey.deep = 1\n"), String::from("tool.x"))
    );
}

/// A file that ran out before its last line ended is written back the same way, whatever its lines
/// end with and whatever moved inside it.
#[test]
fn a_file_that_ends_without_a_break_is_written_back_without_one() {
    let sources = [
        "a = 1",
        "a = 1\r\nb = 2",
        "[tool.x]\nb = 2\na = 1",
        "a = 1\n# tail",
        "a = 1\n",
        "",
    ];

    let written: Vec<String> = sources
        .iter()
        .map(|source| toml_doc::parse(source).expect("valid source").to_string())
        .collect();

    assert_eq!(written, sources);
}

/// The ending the last line was holding goes with the file, not with the entry, so an entry that
/// closed the file still ends a line once something follows it.
#[test]
fn the_end_of_the_file_stays_with_the_file() {
    let mut document = parse("[tool.x]\nb = 2\na = 1");
    document.sections[0].entries.reverse();

    assert_eq!(document.to_string(), "[tool.x]\na = 1\nb = 2");
}

/// A comment runs to the end of its line, and the line break that closed it belongs to whatever was
/// written next. Moving a member takes the comment away from that break, so the writer puts one
/// back rather than letting the comment swallow the comma or the bracket.
#[test]
fn a_comment_never_swallows_what_closes_the_container() {
    let mut document = parse("a = [1, # one\n2]\n");
    let Value::Array(array) = &mut document.root[0].key_value.value else {
        panic!("the source holds an array");
    };
    array.members.reverse();
    let written = document.to_string();

    assert_eq!(written, "a = [\n2,1 # one\n]\n");
    assert!(toml_doc::parse(&written).is_ok(), "{written}");
}

#[test]
fn a_comment_never_swallows_the_closing_brace() {
    let mut document = parse("a = {x = 1, # one\ny = 2}\n");
    let Value::InlineTable(table) = &mut document.root[0].key_value.value else {
        panic!("the source holds an inline table");
    };
    table.members.reverse();
    let written = document.to_string();

    assert_eq!(written, "a = {\ny = 2,x = 1 # one\n}\n");
    assert!(toml_doc::parse(&written).is_ok(), "{written}");
}

/// A key says whether it opens with the segments given, so a caller can tell what a header holds
/// without building the name to find out.
#[test]
fn a_key_says_whether_it_opens_with_the_segments_given() {
    let document = parse("[tool.\"a.b\".sub]\nx = 1\n");
    let key = &document.sections[0].header.key;
    let named = |segments: &[&str]| {
        let held: Vec<String> = segments.iter().map(|held| (*held).to_owned()).collect();
        key.opens_with(&held)
    };

    assert!(named(&["tool"]));
    assert!(named(&["tool", "a.b"]));
    assert!(!named(&["tool", "a", "b"]));
    assert!(!named(&["tool", "other"]));
    assert!(!named(&["tool", "a.b", "sub", "deeper"]));
}

/// A key and a value read different grammars, so a token spelled for one is not a token for the
/// other. Only the model can hold that: a document whose key says `1979-05-27T07:32:00Z` writes
/// text no reader takes back.
#[test]
#[should_panic(expected = "a key segment holds a name written on one line")]
fn a_bare_value_token_cannot_be_written_where_a_name_stands() {
    let mut document = parse("held = 1979-05-27T07:32:00Z\n");
    let Value::Scalar(repr) = document.root[0].key_value.value.clone() else {
        panic!("the value is a scalar");
    };
    document.root[0].key_value.key.parts_mut()[0].set_quoted(repr);
}

/// A name runs to the end of the line the key is written on, so a string that spans several lines
/// is not one however it is quoted.
#[test]
#[should_panic(expected = "a key segment holds a name written on one line")]
fn a_name_cannot_be_written_across_several_lines() {
    let mut document = parse("held = 1\n");
    let written = Repr::string("\"\"\"a\nb\"\"\"", Quoting::MlBasic).expect("a multi-line string");
    document.root[0].key_value.key.parts_mut()[0].set_quoted(written);
}

/// Spacing spaces two tokens apart and says nothing itself, so text that says something is not
/// spacing.
#[test]
#[should_panic(expected = "spacing is written with spaces and tabs")]
fn text_that_is_not_spacing_cannot_be_written_where_spacing_goes() {
    let mut document = parse("held = 1\n");
    document.root[0].key_value.pre_eq = String::from("held").into();
}

/// Spacing is what a formatter writes to line values up, so what it spells goes where it says.
#[test]
fn spacing_a_formatter_spells_lines_a_value_up() {
    let mut document = parse("held = 1\n");
    document.root[0].key_value.pre_eq = "  ".into();

    assert_eq!(document.to_string(), "held  = 1\n");
}

/// A comment opens with `#` and runs to the end of its line, so text that never opens one is not a
/// comment.
#[test]
#[should_panic(expected = "a comment opens with # and runs to the end of its line")]
fn text_without_a_hash_cannot_be_written_where_a_comment_goes() {
    let mut document = parse("held = 1\n");
    document.root[0].trail.comment = Some(String::from("a note").into());
}

/// A comment runs to the end of its line, so text closing that line is not one comment.
#[test]
#[should_panic(expected = "a comment opens with # and runs to the end of its line")]
fn a_comment_cannot_close_the_line_it_runs_to() {
    let mut document = parse("held = 1\n");
    document.root[0].trail.comment = Some(String::from("# a note\n# another").into());
}

/// A quoted name and a single-line string are written the same way, so a caller that has spelled
/// one hands it over rather than spelling the name again.
#[test]
fn a_quoted_name_can_be_written_from_a_string_a_caller_spelled() {
    let mut document = parse("[tool.'a b']\nheld = 1\n");
    let part = &mut document.sections[0].header.key.parts_mut()[1];

    assert!(part.is_quoted());
    assert_eq!(part.written(), "'a b'");

    part.set_quoted(Repr::basic_string("a b"));

    assert_eq!(document.to_string(), "[tool.\"a b\"]\nheld = 1\n");
}

/// Every part of a document writes the text it was read from, so a caller measuring one part reads
/// the same characters the whole file holds.
#[test]
fn each_part_of_a_document_writes_the_text_it_holds() {
    let source = "# lead\n[tool.\"a b\"]  # beside\nheld = [ 1, { x = 2 } ]  # why\n";
    let document = parse(source);
    let section = &document.sections[0];
    let entry = &section.entries[0];
    let Value::Array(array) = &entry.key_value.value else {
        panic!("the value is an array");
    };
    let Value::InlineTable(table) = &array.members[1].item else {
        panic!("the member is a table");
    };

    assert_eq!(document.to_string(), source);
    assert_eq!(section.to_string(), source);
    assert_eq!(section.header.to_string(), "# lead\n[tool.\"a b\"]  # beside\n");
    assert_eq!(section.header.lead.to_string(), "# lead\n");
    assert_eq!(section.header.trail.to_string(), "  # beside\n");
    assert_eq!(section.header.key.to_string(), "tool.\"a b\"");
    assert_eq!(entry.to_string(), "held = [ 1, { x = 2 } ]  # why\n");
    assert_eq!(entry.key_value.to_string(), "held = [ 1, { x = 2 } ]");
    assert_eq!(entry.key_value.value.to_string(), "[ 1, { x = 2 } ]");
    assert_eq!(array.to_string(), "[ 1, { x = 2 } ]");
    assert_eq!(table.to_string(), "{ x = 2 }");
    assert_eq!(array.members[0].lead.to_string(), " ");
}

/// A source that stops in the middle of a key or a value leaves the reader with nothing to read,
/// which is a document that does not parse rather than a panic.
#[test]
fn a_source_that_stops_mid_token_is_rejected() {
    for source in ["a.", "a.b", "a =", "a = ", "[a.", "a.b =", "a = [", "a = {"] {
        assert!(toml_doc::parse(source).is_err(), "{source}");
        assert!(toml_doc::parse_syntax(source).is_err(), "{source}");
    }
}
