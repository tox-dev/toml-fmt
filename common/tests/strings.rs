//! Rewriting string values and choosing the form they are written in.

use common::strings::{self, Wrap};
use toml_doc::Document;

fn parse(source: &str) -> Document<'_> {
    toml_doc::parse(source).expect("valid source")
}

fn rewrite(source: &str, transform: impl Fn(&str) -> String) -> String {
    let mut document = parse(source);
    strings::update(&mut document.root[0].key_value.value, transform);
    document.to_string()
}

#[test]
fn a_rewritten_value_keeps_its_quotes() {
    assert_eq!(
        rewrite("a = \"one\"\n", |text| text.replace("one", "two")),
        "a = \"two\"\n"
    );
}

#[test]
fn text_holding_a_quote_becomes_a_literal_string() {
    assert_eq!(
        rewrite("a = \"plain\"\n", |_| "say \"hi\"".to_owned()),
        "a = 'say \"hi\"'\n"
    );
}

#[test]
fn an_untouched_multiline_string_keeps_its_lines() {
    let source = "a = \"\"\"\nkept\n\"\"\"\n";

    assert_eq!(rewrite(source, ToOwned::to_owned), source);
}

#[test]
fn a_long_value_wraps_with_continuations() {
    let mut document = parse("a = \"Programming Language :: Python :: 3 :: Only and more words here\"\n");
    strings::update_wrapped(
        &mut document.root[0].key_value.value,
        ToOwned::to_owned,
        Wrap {
            column_width: 40,
            indent: "  ",
            prefix: 0,
            inline_table: false,
        },
    );

    let written = document.to_string();
    assert!(written.starts_with("a = \"\"\"\\\n"), "{written}");
    assert!(written.lines().all(|line| line.len() <= 40), "{written}");
}

#[test]
fn a_value_inside_an_inline_table_is_never_wrapped() {
    let mut document = parse("a = { b = \"a very long string that would otherwise be wrapped for width\" }\n");
    let toml_doc::Value::InlineTable(table) = &mut document.root[0].key_value.value else {
        panic!("expected an inline table");
    };
    strings::update_wrapped(
        &mut table.members[0].item.value,
        ToOwned::to_owned,
        Wrap {
            column_width: 20,
            indent: "  ",
            prefix: 0,
            inline_table: true,
        },
    );

    assert!(!document.to_string().contains("\"\"\""));
}

#[test]
fn keys_lose_quotes_they_do_not_need() {
    let mut document = parse("[\"tool\".'ruff']\n\"plain\" = 1\n\"needs quotes\" = 2\n");
    strings::normalize_key_quotes(&mut document);

    assert_eq!(document.to_string(), "[tool.ruff]\nplain = 1\n\"needs quotes\" = 2\n");
}

#[test]
fn a_non_string_value_is_left_alone() {
    assert_eq!(rewrite("a = 12\n", |_| "changed".to_owned()), "a = 12\n");
}

/// A literal string cannot hold a control character, so the rewrite falls back to escaping rather
/// than dropping the change.
#[test]
fn text_holding_a_quote_and_a_control_character_is_escaped() {
    assert_eq!(
        rewrite("a = \"plain\"\n", |_| "say \"hi\"\u{7}".to_owned()),
        "a = \"say \\\"hi\\\"\\u0007\"\n"
    );
}

/// A rewrite is written out escaped: dropping the decoded text between `"""` would read a
/// backslash it holds as the start of an escape.
#[test]
fn a_rewritten_multiline_string_is_written_out_escaped() {
    assert_eq!(
        rewrite("a = \"\"\"one\ntwo\"\"\"\n", |text| text.replace("one", "ONE")),
        "a = \"ONE\\ntwo\"\n"
    );
    assert_eq!(
        rewrite("a = \"\"\"\n\none\"\"\"\n", |text| text.replace("one", "ONE")),
        "a = \"\\nONE\"\n"
    );
}

#[test]
fn a_rewritten_multiline_string_holding_a_backslash_stays_valid() {
    let written = rewrite("a = \'\'\'A and B\nC:\\path\'\'\'\n", |text| text.replace("and", "AND"));

    assert!(toml_doc::parse(&written).is_ok(), "{written}");
    assert_eq!(written, "a = \"A AND B\\nC:\\\\path\"\n");
}

#[test]
fn a_pattern_longer_than_the_key_skips_it() {
    let mut document = parse("a = \"once upon a time there was a very long string indeed\"\n");
    strings::wrap_long_strings(&mut document, 20, 2, &[String::from("a.b.c")]);
    assert_eq!(
        document.to_string(),
        "a = \"\"\"\\\n  once upon a time \\\n  there was a very \\\n  long string \\\n  indeed\\\n  \"\"\"\n"
    );
}

#[test]
fn text_holding_a_backslash_is_written_with_double_quotes() {
    assert_eq!(rewrite("a = '*\\dir'\n", |text| text.to_owned()), "a = \"*\\\\dir\"\n");
    assert_eq!(
        rewrite("a = 'path\\\\to\\\\file'\n", |text| text.to_owned()),
        "a = \"path\\\\\\\\to\\\\\\\\file\"\n"
    );
}

#[test]
fn the_characters_a_string_holds_read_back_decoded() {
    let document = parse("a = \"one\"\nb = 2\n");

    assert_eq!(
        strings::text_of(&document.root[0].key_value.value),
        Some(String::from("one"))
    );
    assert_eq!(strings::text_of(&document.root[1].key_value.value), None);
}

#[test]
fn keys_are_written_in_their_plainest_form() {
    let mut document = parse("'a' = 1\n\n['b'.'c d']\n'e' = { 'f' = [ { 'g' = 2 } ] }\n");
    strings::normalize_key_quotes(&mut document);

    assert_eq!(
        document.to_string(),
        "a = 1\n\n[b.\"c d\"]\ne = { f = [ { g = 2 } ] }\n"
    );
}

#[test]
fn a_long_string_under_a_table_wraps_unless_its_key_is_skipped() {
    let mut document = parse(concat!(
        "[tool.a]\n",
        "long = \"once upon a time there was a very long string\"\n",
        "skip_me = \"once upon a time there was a very long string\"\n",
    ));
    strings::wrap_long_strings(&mut document, 20, 2, &[String::from("tool.*.skip_me")]);

    assert_eq!(
        document.to_string(),
        concat!(
            "[tool.a]\n",
            "long = \"\"\"\\\n  once upon a time \\\n  there was a very \\\n  long string\\\n  \"\"\"\n",
            "skip_me = \"once upon a time there was a very long string\"\n",
        )
    );
}

#[test]
fn a_long_string_inside_an_array_wraps_too() {
    let mut document = parse("a = [ \"once upon a time there was a very long string indeed\" ]\n");
    strings::wrap_long_strings(&mut document, 20, 2, &[]);

    assert_eq!(
        document.to_string(),
        "a = [ \"\"\"\\\n  once upon a time \\\n  there was a very \\\n  long string \\\n  indeed\\\n  \"\"\" ]\n"
    );
}

#[test]
fn a_long_string_inside_an_inline_table_stays_on_its_line() {
    let mut document = parse("a = { b = \"once upon a time there was a very long string indeed\" }\n");
    strings::wrap_long_strings(&mut document, 20, 2, &[]);

    assert_eq!(
        document.to_string(),
        "a = { b = \"once upon a time there was a very long string indeed\" }\n"
    );
}

/// A line is measured in the columns it takes, and breaks between characters: a wide one takes two
/// columns, and a character written out of several scalars is still one character.
#[test]
fn wrapping_breaks_between_characters_rather_than_inside_one() {
    let held = |source: &str| {
        let mut document = parse(source);
        strings::wrap_long_strings(&mut document, 20, 2, &[]);
        document.to_string()
    };

    // seven wide characters take fourteen columns, which the width has room for
    assert_eq!(held("a = \"界界界界界界界\"\n"), "a = \"界界界界界界界\"\n");
    // a wide value breaks into lines the width has room for
    let wide = held("a = \"界界界界界界界界界界界界\"\n");
    assert!(toml_doc::parse(&wide).is_ok(), "{wide}");
    for line in wide.lines().filter(|line| line.starts_with("  ")) {
        assert!(common::width::columns(line) <= 20, "{line}");
    }

    // a character written out of several scalars stays one character
    let accented = held("a = \"aaaaaaaaaaaaaaaaaaaaaaaaaaaae\u{301}\"\n");
    let joined = held("a = \"xxxxxxxxxxxxxxxxxxxxxxxxxxxxx👩\u{200d}💻\"\n");
    assert!(accented.contains("e\u{301}"), "{accented}");
    assert!(joined.contains("👩\u{200d}💻"), "{joined}");
}

#[test]
fn wrapping_keeps_an_escape_whole() {
    let mut document = parse("a = \"\\u0001\\u0001\\u0001\\u0001\"\n");
    strings::wrap_long_strings(&mut document, 20, 2, &[]);

    let written = document.to_string();
    assert!(toml_doc::parse(&written).is_ok(), "{written}");
    assert_eq!(
        written,
        "a = \"\"\"\\\n  \\u0001\\u0001\\\n  \\u0001\\u0001\\\n  \"\"\"\n"
    );
}

#[test]
fn wrapping_keeps_a_combining_mark_with_its_character() {
    let mut document = parse("a = \"éééééééééé\"\n");
    strings::wrap_long_strings(&mut document, 20, 2, &[]);

    let written = document.to_string();
    assert!(toml_doc::parse(&written).is_ok(), "{written}");
    assert_eq!(
        strings::text_of(&toml_doc::parse(&written).expect("valid").root[0].key_value.value),
        Some(String::from("éééééééééé"))
    );
}

/// Leaving the written form alone is for text the rewrite did not touch. A value the rewrite
/// changed is written out afresh, whatever escaping that takes.
#[test]
fn a_rewrite_that_changes_the_text_is_written_out() {
    assert_eq!(
        rewrite("a = '  C:\\path  '\n", |text| text.trim().to_owned()),
        "a = \"C:\\\\path\"\n"
    );
    assert_eq!(
        rewrite("a = '''  old \"it's\"  '''\n", |text| text.trim().to_owned()),
        "a = \"old \\\"it's\\\"\"\n"
    );
    assert_eq!(
        rewrite("a = \"  keep  \"\n", |text| text.trim().to_owned()),
        "a = \"keep\"\n"
    );
}

#[test]
fn a_rewrite_that_changes_nothing_leaves_the_written_form_alone() {
    assert_eq!(
        rewrite("a = 'say \"hi\"'\n", |text| text.trim().to_owned()),
        "a = 'say \"hi\"'\n"
    );
    assert_eq!(
        rewrite("a = \"*\\\\dir\"\n", |text| text.trim().to_owned()),
        "a = \"*\\\\dir\"\n"
    );
    assert_eq!(
        rewrite("a = '''one\ntwo'''\n", |text| text.trim().to_owned()),
        "a = '''one\ntwo'''\n"
    );
}

/// A pattern opening with `*` names a key wherever it is written, which is what an environment
/// name in the path needs.
#[test]
fn a_leading_wildcard_reaches_a_key_at_any_depth() {
    let skipped = |source: &str, pattern: &str| {
        let mut document = parse(source);
        strings::wrap_long_strings(&mut document, 20, 2, &[String::from(pattern)]);
        !document.to_string().contains("\"\"\"")
    };
    let long = "a very long command that must otherwise wrap";

    assert!(skipped(&format!("[env.test]\ncommands = \"{long}\"\n"), "*.commands"));
    assert!(skipped(&format!("[env]\ncommands = \"{long}\"\n"), "*.commands"));
    assert!(skipped(&format!("[a.b.c.d]\ncommands = \"{long}\"\n"), "*.commands"));
    assert!(skipped(&format!("commands = \"{long}\"\n"), "commands"));
    assert!(skipped(
        &format!("[env.test]\ncommands = \"{long}\"\n"),
        "env.*.commands"
    ));
    assert!(skipped(&format!("[env.test]\ncommands = \"{long}\"\n"), "env.*.*"));
    assert!(!skipped(&format!("[env.test]\ncommands = \"{long}\"\n"), "env"));
    assert!(!skipped(&format!("[env.test]\ndeps = \"{long}\"\n"), "*.commands"));
    assert!(!skipped(&format!("[env.test]\ncommands = \"{long}\"\n"), "*.deps"));
    assert!(!skipped(&format!("commands = \"{long}\"\n"), "*.commands"));
}

/// A pattern naming no wildcard names the one key it spells, so what is written below that key is
/// wrapped like anything else.
#[test]
fn a_pattern_without_a_wildcard_reaches_only_the_key_it_names() {
    let skipped = |source: &str, pattern: &str| {
        let mut document = parse(source);
        // wide enough that a nested key still leaves room for what opens a multi-line string
        strings::wrap_long_strings(&mut document, 40, 2, &[String::from(pattern)]);
        !document.to_string().contains("\"\"\"")
    };
    let long = "a very long command that must otherwise wrap";

    assert!(skipped(&format!("[project]\nurls = \"{long}\"\n"), "project.urls"));
    assert!(!skipped(
        &format!("[project]\nurls.Homepage.description = \"{long}\"\n"),
        "project.urls"
    ));
    assert!(skipped(
        &format!("[project]\nurls.Homepage.description = \"{long}\"\n"),
        "project.urls.*.*"
    ));
}

/// A quoted segment holding a dot is one segment on both sides, so a pattern spells a name the way
/// the file does.
#[test]
fn a_pattern_reads_a_quoted_segment_as_one_segment() {
    let skipped = |source: &str, pattern: &str| {
        let mut document = parse(source);
        strings::wrap_long_strings(&mut document, 20, 2, &[String::from(pattern)]);
        !document.to_string().contains("\"\"\"")
    };
    let long = "a very long command that must otherwise wrap";

    assert!(skipped(
        &format!("[tool.\"a.b\"]\ncommands = \"{long}\"\n"),
        "tool.\"a.b\".commands"
    ));
    assert!(skipped(
        &format!("[tool.\"a.b\"]\ncommands = \"{long}\"\n"),
        "*.commands"
    ));
    assert!(skipped(&format!("\"a.b\" = \"{long}\"\n"), "\"a.b\""));
    assert!(skipped(
        &format!("[tool.\"a\\\"b\"]\ncommands = \"{long}\"\n"),
        "tool.\"a\\\"b\".commands"
    ));
    assert!(!skipped(
        &format!("[tool.\"a.b\"]\ncommands = \"{long}\"\n"),
        "tool.a.b.commands"
    ));
}

/// A pattern TOML cannot read as a key names itself, so the components a reader sees are the ones
/// written between the dots.
#[test]
fn a_pattern_that_is_not_a_key_names_itself() {
    let skipped = |source: &str, pattern: &str| {
        let mut document = parse(source);
        strings::wrap_long_strings(&mut document, 20, 2, &[String::from(pattern)]);
        !document.to_string().contains("\"\"\"")
    };
    let long = "a very long command that must otherwise wrap";

    assert!(skipped(&format!("[tool]\n\"a b\" = \"{long}\"\n"), "tool.a b"));
    assert!(!skipped(&format!("[tool]\ncommands = \"{long}\"\n"), "tool.\"commands"));
    assert!(!skipped(&format!("[tool]\ncommands = \"{long}\"\n"), "tool.\"a.b"));
}

/// Choosing a one-line form before measuring would hold a value open past the column it was asked
/// to fit.
#[test]
fn a_long_value_holding_a_quote_wraps_rather_than_going_literal() {
    let mut document = parse("a = \"one \\\"two\\\" three four five six seven eight nine\"\n");
    strings::wrap_long_strings(&mut document, 20, 2, &[]);

    let written = document.to_string();
    assert!(toml_doc::parse(&written).is_ok(), "{written}");
    assert!(written.contains("\"\"\"\\"), "{written}");
}

/// A value inside `{ }` cannot break across lines, so it stays on one however wide it is.
#[test]
fn a_long_value_inside_an_inline_table_stays_on_one_line() {
    let mut document = parse("a = { b = \"one \\\"two\\\" three four five six seven eight\" }\n");
    strings::wrap_long_strings(&mut document, 20, 2, &[]);

    assert_eq!(
        document.to_string(),
        "a = { b = 'one \"two\" three four five six seven eight' }\n"
    );
}

/// Laying a value out writes it in the plainest form, except where a backslash the file wrote
/// plainly would have to gain an escape to move into double quotes.
#[test]
fn laying_out_leaves_a_literal_that_holds_a_backslash() {
    let mut document = parse("a = 'plain'\nb = '*\\dir'\nc = \"Jos\\u00E9\"\n");
    strings::wrap_long_strings(&mut document, 120, 2, &[]);

    assert_eq!(
        document.to_string(),
        "a = \"plain\"\nb = '*\\dir'\nc = \"Jos\\u00E9\"\n"
    );
}

/// A wildcard the pattern opens with stands for the path above it and one it closes with for the
/// path below. Anywhere else it names one segment, so the pattern reaches the key it spells rather
/// than what is written under it.
#[test]
fn a_wildcard_in_the_middle_names_one_segment() {
    let skipped = |source: &str, pattern: &str| {
        let mut document = parse(source);
        strings::wrap_long_strings(&mut document, 20, 2, &[String::from(pattern)]);
        !document.to_string().contains("\"\"\"")
    };
    let long = "once upon a time there was a very long string indeed";

    assert!(skipped(
        &format!("[tool.ruff]\ncommands = \"{long}\"\n"),
        "tool.*.commands"
    ));
    assert!(!skipped(
        &format!("[tool.ruff.commands]\nchild = \"{long}\"\n"),
        "tool.*.commands"
    ));
}

/// A quoted star names the key spelled that way, which is a key TOML reads and a pattern can reach.
#[test]
fn a_quoted_star_names_the_key_it_spells() {
    let skipped = |source: &str, pattern: &str| {
        let mut document = parse(source);
        strings::wrap_long_strings(&mut document, 20, 2, &[String::from(pattern)]);
        !document.to_string().contains("\"\"\"")
    };
    let long = "once upon a time there was a very long string indeed";

    assert!(skipped(&format!("[\"*\"]\ncommands = \"{long}\"\n"), "\"*\".commands"));
    assert!(!skipped(&format!("[tool]\ncommands = \"{long}\"\n"), "\"*\".commands"));
}

/// A continuation eats the line break and the whitespace after it, so a value whose own whitespace
/// would go with them is left as the file wrote it.
#[test]
fn a_value_wrapping_would_change_is_left_as_written() {
    let source = "a = \"\"\"\n    one two three four five six seven \\\n    eight nine ten eleven\\\n    \"\"\"\n";
    let mut document = parse(source);
    strings::wrap_long_strings(&mut document, 40, 2, &[]);

    assert_eq!(document.to_string(), source);
}

/// A line runs from the start of its key, so what the layout writes ahead of a value counts toward
/// the column it has to fit.
#[test]
fn a_key_pushing_its_value_past_the_column_wraps_it() {
    let mut document = parse("a_longer_name = \"one two three four\"\n");
    strings::wrap_long_strings(&mut document, 30, 2, &[]);

    assert_eq!(
        document.to_string(),
        "a_longer_name = \"\"\"\\\n  one two three four\\\n  \"\"\"\n"
    );
}

/// A key already over the column cannot be brought back by rewriting its value, and wrapping it
/// would only cost the file a line.
#[test]
fn a_key_wider_than_the_column_leaves_its_value_alone() {
    let source = "a_name_wider_than_the_whole_column = \"one two three four\"\n";
    let mut document = parse(source);
    strings::wrap_long_strings(&mut document, 30, 2, &[]);

    assert_eq!(document.to_string(), source);
}

/// A member of an array starts its line at the indent the layout pushes it in by, however the file
/// happened to write it.
#[test]
fn an_array_member_is_measured_from_the_indent_it_is_written_at() {
    let written = |source: &str| {
        let mut document = parse(source);
        strings::wrap_long_strings(&mut document, 24, 2, &[]);
        document.to_string()
    };

    assert_eq!(written("a = [ \"one two three\" ]\n"), "a = [ \"one two three\" ]\n");
    assert_eq!(
        written("a = [\n        \"one two three\",\n]\n"),
        "a = [\n        \"one two three\",\n]\n"
    );
    assert_eq!(
        written("a = [ \"one two three four five\" ]\n"),
        "a = [ \"\"\"\\\n  one two three four \\\n  five\\\n  \"\"\" ]\n"
    );
    assert_eq!(
        written("a = [\n        \"one two three four five\",\n]\n"),
        "a = [\n        \"\"\"\\\n  one two three four \\\n  five\\\n  \"\"\",\n]\n"
    );
}

/// Wrapping is decided once: what it writes the first time is what a second pass sees, and the
/// string a reader gets back is the one the file started with.
#[test]
fn wrapping_settles_on_its_first_pass() {
    let source = "a_longer_name = \"one two three four\"\n";
    let mut document = parse(source);
    strings::wrap_long_strings(&mut document, 30, 2, &[]);
    let once = document.to_string();

    let mut again = parse(&once);
    strings::wrap_long_strings(&mut again, 30, 2, &[]);

    assert_eq!(again.to_string(), once);
    assert_eq!(
        strings::text_of(&parse(&once).root[0].key_value.value),
        strings::text_of(&parse(source).root[0].key_value.value)
    );
}

/// The line is measured as the layout will write the key, not as the file spaced it out, so two
/// files saying the same thing are wrapped the same way.
#[test]
fn a_key_is_measured_in_the_form_the_layout_writes_it() {
    let laid_out = |source: &str| {
        let mut document = parse(source);
        strings::normalize_key_quotes(&mut document);
        strings::wrap_long_strings(&mut document, 28, 2, &[]);
        common::layout::Layout {
            column_width: 28,
            indent: 2,
            ending: toml_doc::LineEnding::Lf,
        }
        .apply(&mut document);
        document.to_string()
    };

    let spaced = laid_out("a    .    long = \"one two three\"\n");
    assert_eq!(spaced, laid_out("a.long = \"one two three\"\n"));
    assert_eq!(spaced, laid_out(&spaced));
    assert_eq!(
        strings::text_of(&parse(&spaced).root[0].key_value.value),
        Some(String::from("one two three"))
    );
}

/// A column with no room for the indent, one character and the continuation after it cannot be
/// wrapped to, so the value is left as the file wrote it rather than broken into longer lines.
#[test]
fn a_column_too_narrow_to_wrap_to_leaves_the_value_alone() {
    let written = |column_width: usize, indent: usize| {
        let mut document = parse("a = \"one two three four\"\n");
        strings::wrap_long_strings(&mut document, column_width, indent, &[]);
        document.to_string()
    };

    // a narrow column still wraps, to lines it has room for
    let narrow = written(10, 2);
    assert!(
        narrow.lines().skip(1).all(|line| common::width::columns(line) <= 10),
        "{narrow}"
    );
    // an indent as wide as the column leaves no room for what a continuation line holds
    assert_eq!(written(10, 10), "a = \"one two three four\"\n");
    // a wide character takes two columns, which the narrowed line has no room for
    let mut document = parse("a = \"\u{754c}\u{754c}\u{754c}\u{754c}\u{754c}\u{754c}\"\n");
    strings::wrap_long_strings(&mut document, 9, 7, &[]);
    assert_eq!(
        document.to_string(),
        "a = \"\u{754c}\u{754c}\u{754c}\u{754c}\u{754c}\u{754c}\"\n"
    );
}

/// A rewrite reads the text a value holds, and a value holding no text holds none to rewrite.
#[test]
fn a_value_that_is_not_a_string_is_left_as_written() {
    let mut document = parse("a = [ 1 ]\n");
    let value = &mut document.root[0].key_value.value;
    strings::update_wrapped(
        value,
        |text| text.to_uppercase(),
        Wrap {
            column_width: 40,
            indent: "  ",
            prefix: 0,
            inline_table: false,
        },
    );

    assert_eq!(document.to_string(), "a = [ 1 ]\n");
}
