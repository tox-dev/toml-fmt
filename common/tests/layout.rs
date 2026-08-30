//! The whitespace rules a formatted document follows.

use common::layout::Layout;
use toml_doc::LineEnding;

fn lay_out(source: &str, column_width: usize) -> String {
    let mut document = toml_doc::parse(source).expect("valid source");
    Layout {
        column_width,
        indent: 2,
        ending: LineEnding::Lf,
    }
    .apply(&mut document);
    document.to_string()
}

#[test]
fn spacing_around_the_equals_sign_is_normalized() {
    assert_eq!(lay_out("a=1\nb   =    2\n", 120), "a = 1\nb = 2\n");
}

#[test]
fn a_short_array_stays_on_one_line() {
    assert_eq!(
        lay_out("keywords=[\"python\",\"toml\"]\n", 120),
        "keywords = [ \"python\", \"toml\" ]\n"
    );
}

#[test]
fn an_array_wider_than_the_column_breaks_apart() {
    let written = lay_out("keywords = [\"web\", \"toml\", \"pyproject\", \"formatting\"]\n", 30);

    assert_eq!(
        written,
        "keywords = [\n  \"web\",\n  \"toml\",\n  \"pyproject\",\n  \"formatting\",\n]\n"
    );
}

#[test]
fn a_trailing_comma_keeps_an_array_open() {
    assert_eq!(lay_out("a = [\"one\",]\n", 120), "a = [\n  \"one\",\n]\n");
}

#[test]
fn a_comment_keeps_an_array_open() {
    let written = lay_out("ignore = [\"E501\",  # too long\n\"E701\"]\n", 120);

    assert_eq!(written, "ignore = [\n  \"E501\",  # too long\n  \"E701\"\n]\n");
}

#[test]
fn an_inline_table_is_padded_inside_its_braces() {
    assert_eq!(lay_out("a={b=1,c=2}\n", 120), "a = { b = 1, c = 2 }\n");
}

#[test]
fn a_table_header_loses_the_spacing_inside_its_brackets() {
    assert_eq!(lay_out("[ tool . ruff ]\nx=1\n", 120), "[tool.ruff]\nx = 1\n");
}

#[test]
fn a_built_entry_is_laid_out_like_any_other() {
    let mut document = toml_doc::parse("[project]\n").expect("valid source");
    document.sections[0]
        .entries
        .push(common::build::string_entry("name", "demo"));
    document.sections[0].entries.push(common::build::entry(
        "classifiers",
        common::build::array([common::build::string("Programming Language :: Python")]),
    ));
    Layout {
        column_width: 120,
        indent: 2,
        ending: LineEnding::Lf,
    }
    .apply(&mut document);

    assert_eq!(
        document.to_string(),
        "[project]\nname = \"demo\"\nclassifiers = [ \"Programming Language :: Python\" ]\n"
    );
}

#[test]
fn an_empty_line_between_members_holds_them_apart() {
    assert_eq!(
        lay_out("a = [\n  \"one\",\n\n  \"two\",\n]\n", 120),
        "a = [\n  \"one\",\n\n  \"two\",\n]\n"
    );
}

#[test]
fn comments_line_up_one_column_past_the_widest_member() {
    let mut document =
        toml_doc::parse("a = [\n  \"one\",  # first\n  \"three\",  # second\n]\n").expect("valid source");
    Layout {
        column_width: 120,
        indent: 2,
        ending: LineEnding::Lf,
    }
    .apply(&mut document);
    common::layout::align_array_comments(&mut document);

    assert_eq!(
        document.to_string(),
        "a = [\n  \"one\",   # first\n  \"three\", # second\n]\n"
    );
}

#[test]
fn alignment_reaches_arrays_written_inside_a_table() {
    let mut document = toml_doc::parse("[tool.a]\nb = { c = [\n  \"one\",  # first\n  \"three\",  # second\n] }\n")
        .expect("valid source");
    Layout {
        column_width: 120,
        indent: 2,
        ending: LineEnding::Lf,
    }
    .apply(&mut document);
    common::layout::align_array_comments(&mut document);

    assert_eq!(
        document.to_string(),
        "[tool.a]\nb = { c = [\n  \"one\",   # first\n  \"three\", # second\n] }\n"
    );
}

#[test]
fn a_quoted_key_is_written_the_way_a_string_would_be() {
    assert_eq!(lay_out("['a b'.c] \nd = 1\n", 120), "[\"a b\".c]\nd = 1\n");
}

#[test]
fn a_key_holding_a_quote_falls_back_to_single_quotes() {
    assert_eq!(lay_out("[\"a\\\"b\"]\nd = 1\n", 120), "['a\"b']\nd = 1\n");
}

#[test]
fn a_number_keeps_the_form_it_was_written_in() {
    assert_eq!(lay_out("a = 0x1F\n", 120), "a = 0x1F\n");
}

#[test]
fn a_comment_closing_a_line_sits_two_spaces_past_the_value() {
    assert_eq!(lay_out("a = 1 # why\n", 120), "a = 1  # why\n");
}

#[test]
fn a_multiline_string_keeps_the_quotes_it_was_written_with() {
    assert_eq!(lay_out("a = '''one'''\n", 120), "a = '''one'''\n");
}

#[test]
fn a_string_holding_a_quote_falls_back_to_single_quotes() {
    assert_eq!(lay_out("a = \"say \\\"hi\\\"\"\n", 120), "a = 'say \"hi\"'\n");
}

#[test]
fn aligning_an_array_with_nothing_in_it_leaves_it_alone() {
    let mut document = toml_doc::parse("a = []\n").expect("valid source");
    common::layout::align_array_comments(&mut document);

    assert_eq!(document.to_string(), "a = []\n");
}

#[test]
fn a_member_that_is_not_a_scalar_sets_the_column_by_how_it_is_written() {
    let mut document = toml_doc::parse("a = [\n  { b = 1 },  # first\n  \"x\",  # second\n]\n").expect("valid source");
    Layout {
        column_width: 120,
        indent: 2,
        ending: LineEnding::Lf,
    }
    .apply(&mut document);
    common::layout::align_array_comments(&mut document);

    assert_eq!(
        document.to_string(),
        "a = [\n  { b = 1 }, # first\n  \"x\",       # second\n]\n"
    );
}

#[test]
fn a_line_that_ran_out_without_a_break_gets_one() {
    assert_eq!(lay_out("a = 1", 120), "a = 1\n");
    assert_eq!(lay_out("[tool.a]", 120), "[tool.a]\n");
    assert_eq!(lay_out("a = 1\n# tail", 120), "a = 1\n# tail\n");
    assert_eq!(lay_out("", 120), "");
}

/// Folding moves an entry that closed the file into the middle of one, where a missing line break
/// would run the next line onto it.
#[test]
fn an_entry_that_closed_the_file_still_gets_a_break_once_it_moves() {
    let mut document =
        toml_doc::parse("[tool.a.report]\nskip = true\n[tool.a.run]\nbranch = true").expect("valid source");
    common::nesting::collapse(&mut document, "tool.a");
    Layout {
        column_width: 120,
        indent: 2,
        ending: LineEnding::Lf,
    }
    .apply(&mut document);

    assert_eq!(
        document.to_string(),
        "[tool.a]\nreport.skip = true\nrun.branch = true\n"
    );
}

#[test]
fn a_comment_leading_an_item_moves_to_that_item_s_column() {
    assert_eq!(
        lay_out("  # above\n[tool.a]\n  # inside\n  x = 1\n  # trailing\n", 120),
        "# above\n[tool.a]\n# inside\nx = 1\n# trailing\n"
    );
}

#[test]
fn an_empty_line_carries_no_whitespace_of_its_own() {
    assert_eq!(lay_out("a = 1\n   \nb = 2\n", 120), "a = 1\n\nb = 2\n");
}

/// TOML 1.1 lets an inline table span several lines and hold comments, and no single-line form can
/// keep one.
#[test]
fn an_inline_table_holding_a_comment_stays_as_the_file_wrote_it() {
    assert_eq!(
        lay_out("a = {\n  # about b\n  b = 1, # beside b\n  c   =  2,\n}\n", 120),
        "a = {\n  # about b\n  b = 1, # beside b\n  c = 2,\n}\n"
    );
}

#[test]
fn a_comment_before_the_closing_brace_holds_the_table_open() {
    assert_eq!(
        lay_out("a = {\n  b = 1\n  # last word\n}\n", 120),
        "a = {\n  b = 1\n  # last word\n}\n"
    );
}

#[test]
fn a_comment_inside_a_nested_inline_table_survives() {
    assert_eq!(
        lay_out("a = { b = {\n  # inner\n  c = 1,\n} }\n", 120),
        "a = { b = {\n  # inner\n  c = 1,\n} }\n"
    );
}

#[test]
fn an_inline_table_without_comments_still_closes_up() {
    assert_eq!(lay_out("a = {\n  b = 1,\n  c = 2,\n}\n", 120), "a = { b = 1, c = 2 }\n");
}

/// TOML 1.1 lets a comment sit either side of a member's comma. Each runs to the end of its own
/// line, so writing both on one line would fold the second into the first.
#[test]
fn a_comment_on_each_side_of_the_comma_keeps_its_own_line() {
    assert_eq!(
        lay_out("a = [\n  1 # before\n  , # after\n  2,\n]\n", 120),
        "a = [\n  1,  # before\n  # after\n  2,\n]\n"
    );
}

#[test]
fn the_same_shape_inside_a_nested_array_keeps_its_lines() {
    let written = lay_out("a = [\n  [\n    1 # before\n    , # after\n  ],\n]\n", 120);

    assert!(toml_doc::parse(&written).is_ok(), "{written}");
    assert_eq!(written.matches('#').count(), 2, "{written}");
}

/// A bare scalar carries no quotes, so measuring one as though it did would step its comment two
/// columns left of the strings around it.
#[test]
fn comments_line_up_whatever_the_members_are() {
    let mut document = toml_doc::parse(concat!(
        "a = [\n",
        "  \"x\", # string\n",
        "  1, # integer\n",
        "  true, # boolean\n",
        "  1979-05-27, # date\n",
        "  { b = 1 }, # table\n",
        "]\n",
    ))
    .expect("valid source");
    Layout {
        column_width: 120,
        indent: 2,
        ending: LineEnding::Lf,
    }
    .apply(&mut document);
    common::layout::align_array_comments(&mut document);

    assert_eq!(
        document.to_string(),
        concat!(
            "a = [\n",
            "  \"x\",        # string\n",
            "  1,          # integer\n",
            "  true,       # boolean\n",
            "  1979-05-27, # date\n",
            "  { b = 1 },  # table\n",
            "]\n",
        )
    );
}

/// What moves a comment along is the text written before it, so an escape counts the columns it
/// takes and a value that closes on a later line carries only that line.
#[test]
fn a_comment_is_aligned_by_what_the_line_holds() {
    let mut document =
        toml_doc::parse("a = [\n  \"x\\ny\", # escaped\n  \"abc\", # plain\n  [\n    1,\n  ],\n  3, # scalar\n]\n")
            .expect("valid source");
    Layout {
        column_width: 120,
        indent: 2,
        ending: LineEnding::Lf,
    }
    .apply(&mut document);
    common::layout::align_array_comments(&mut document);

    assert_eq!(
        document.to_string(),
        "a = [\n  \"x\\ny\", # escaped\n  \"abc\",  # plain\n  [\n    1,\n  ],\n  3,      # scalar\n]\n"
    );
}

/// A comma follows a member wherever the array ends up, so a nested one is measured with it and no
/// line runs past the column.
#[test]
fn a_nested_array_leaves_room_for_the_comma_after_it() {
    let written = lay_out("a = [ [ \"123456789012\" ], 1 ]\n", 20);

    assert_eq!(written, "a = [\n  [\n    \"123456789012\"\n  ],\n  1,\n]\n");
    assert!(written.lines().all(|line| line.len() <= 20), "{written}");
}

/// A member the file wrote on a line of its own starts where that line's indent leaves it, so what
/// stands above it does not decide whether its value fits.
#[test]
fn a_member_on_its_own_line_is_measured_from_that_line() {
    let source = concat!(
        "value = {\n",
        "  first = \"12345678901234567890\", # c\n",
        "  second = [1, 2],\n",
        "}\n",
    );

    assert_eq!(
        lay_out(source, 40),
        concat!(
            "value = {\n",
            "  first = \"12345678901234567890\", # c\n",
            "  second = [ 1, 2 ],\n",
            "}\n",
        )
    );
}

/// A value the file wrote over lines still runs over them where its container stays on one line, so
/// what that container closes on is the line the value ended rather than every line it took.
#[test]
fn a_member_written_over_lines_leaves_its_container_on_the_line_it_ended() {
    let source = "x = { a = [\n  [\n    1, # note\n    2,\n  ],\n], b = [\"1234567890\"] }\n";
    let written = |column_width: usize| {
        let mut document = toml_doc::parse(source).expect("valid source");
        Layout {
            column_width,
            indent: 2,
            ending: LineEnding::Lf,
        }
        .apply(&mut document);
        common::layout::align_array_comments(&mut document);
        document.to_string()
    };

    for column_width in [50, 58, 60, 62, 70] {
        let held = written(column_width);
        assert_eq!(
            held, "x = { a = [\n  [\n    1, # note\n    2,\n  ],\n], b = [ \"1234567890\" ] }\n",
            "{column_width}"
        );
        let mut again = toml_doc::parse(&held).expect("valid source");
        Layout {
            column_width,
            indent: 2,
            ending: LineEnding::Lf,
        }
        .apply(&mut again);
        common::layout::align_array_comments(&mut again);
        assert_eq!(again.to_string(), held, "{column_width}");
    }
}

/// An array whose indent already fills the column gains nothing by opening: every line it wrote
/// would start past the column it was asked to fit. So a value nested that deep stays on one line,
/// and what the layout writes grows with the column rather than with the depth.
#[test]
fn a_value_nested_past_the_column_stays_on_one_line() {
    let written = |depth: usize| {
        let source = format!("a = {}1{}\n", "[".repeat(depth), "]".repeat(depth));
        let mut document = toml_doc::parse(&source).expect("valid source");
        Layout {
            column_width: 120,
            indent: 2,
            ending: LineEnding::Lf,
        }
        .apply(&mut document);
        document.to_string()
    };

    // the layout opens an array only while the indent leaves room for one
    for depth in [60, 120, 256] {
        let held = written(depth);
        assert!(
            held.lines().count() <= 3 * (120 / 2) + 4,
            "{depth}: {}",
            held.lines().count()
        );
        assert!(toml_doc::parse(&held).is_ok(), "{depth}");
    }
}

/// A table the file wrote a comment inside stays over the lines it wrote, and what each of its
/// members takes was measured as it was laid out rather than written out again for every table
/// above it.
#[test]
fn a_commented_table_written_as_a_value_keeps_the_lines_the_file_gave_it() {
    let source = "a = { # why\n  b = { c = 1 }, d = 2 }\n";
    let mut document = toml_doc::parse(source).expect("valid source");
    Layout {
        column_width: 120,
        indent: 2,
        ending: LineEnding::Lf,
    }
    .apply(&mut document);

    assert_eq!(document.to_string(), "a = { # why\n  b = { c = 1 }, d = 2 }\n");
}

/// A file ends where a line ends, whichever kind of line the last one is.
#[test]
fn the_last_line_of_a_file_is_closed_whether_it_holds_a_comment_or_nothing() {
    for (name, source, written) in [
        ("comment", "a = 1\n# trailing", "a = 1\n# trailing\n"),
        ("blank", "a = 1\n   ", "a = 1\n\n"),
    ] {
        assert_eq!(lay_out(source, 120), written, "{name}");
    }
}

/// A value holding a backslash is written the way the file wrote it: double quotes would have to
/// escape it, and the form it is already in does not.
#[test]
fn a_value_holding_a_backslash_keeps_the_form_it_was_written_in() {
    assert_eq!(lay_out("a = 'x\\y'\n", 120), "a = 'x\\y'\n");
}
