use crate::disabled::{MARKER, enable_disabled_keys, restore_disabled_keys, with_disabled_keys};

#[test]
fn test_with_disabled_keys_brackets_the_format_pass() {
    let out = with_disabled_keys("# x = 1\n", |enabled| {
        assert!(enabled.contains(MARKER), "the format pass sees the enabled key");
        enabled.to_string()
    });
    assert_eq!(out, "# x = 1\n");
}

#[test]
fn test_enable_promotes_valid_key() {
    assert_eq!(
        enable_disabled_keys("# default = true\n"),
        format!("default = true  # {MARKER}\n")
    );
}

#[test]
fn test_enable_promotes_inline_table_key() {
    assert_eq!(
        enable_disabled_keys("# set_env = {A = \"1\"}\n"),
        format!("set_env = {{A = \"1\"}}  # {MARKER}\n")
    );
}

#[test]
fn test_enable_leaves_two_keys_on_one_line() {
    let source = "# a = 1 b = 2\n";
    assert_eq!(enable_disabled_keys(source), source);
}

#[test]
fn test_enable_preserves_indentation() {
    assert_eq!(enable_disabled_keys("  # x = 1\n"), format!("  x = 1  # {MARKER}\n"));
}

#[test]
fn test_enable_appends_to_existing_trailing_comment() {
    assert_eq!(
        enable_disabled_keys("# x = 1  # note\n"),
        format!("x = 1  # note {MARKER}\n")
    );
}

#[test]
fn test_enable_leaves_prose_comment() {
    let source = "# just a note, not a key\n";
    assert_eq!(enable_disabled_keys(source), source);
}

#[test]
fn test_enable_leaves_commented_table_header() {
    let source = "# [tool.foo]\n";
    assert_eq!(enable_disabled_keys(source), source);
}

#[test]
fn test_enable_leaves_non_comment_lines() {
    let source = "enabled = true\n";
    assert_eq!(enable_disabled_keys(source), source);
}

#[test]
fn test_enable_without_trailing_newline() {
    assert_eq!(enable_disabled_keys("# x = 1"), format!("x = 1  # {MARKER}"));
}

#[test]
fn test_enable_promotes_multiline_value() {
    assert_eq!(
        enable_disabled_keys("# x = [\n#   1,\n# ]\n"),
        format!("x = [\n  1,\n]  # {MARKER}\n")
    );
}

#[test]
fn test_enable_leaves_incomplete_value() {
    let source = "# x = [\n# still going\n";
    assert_eq!(enable_disabled_keys(source), source);
}

#[test]
fn test_enable_leaves_empty_comment() {
    let source = "#\n";
    assert_eq!(enable_disabled_keys(source), source);
}

#[test]
fn test_enable_leaves_value_cut_off_by_table_header() {
    let source = "# x = [\n# [tool.x]\n";
    assert_eq!(enable_disabled_keys(source), source);
}

#[test]
fn test_enable_promotes_key_next_to_prose() {
    assert_eq!(
        enable_disabled_keys("# a note\n# a = 1\n"),
        format!("# a note\na = 1  # {MARKER}\n")
    );
}

#[test]
fn test_enable_stops_at_commented_table_header() {
    let source = "# a = 1\n# [tool.foo]\n# b = 2\n";
    assert_eq!(
        enable_disabled_keys(source),
        format!("a = 1  # {MARKER}\n# [tool.foo]\n# b = 2\n")
    );
}

#[test]
fn test_restore_marker_only_comment() {
    assert_eq!(
        restore_disabled_keys(&format!("default = true  # {MARKER}\n")),
        "# default = true\n"
    );
}

#[test]
fn test_restore_keeps_original_trailing_comment() {
    assert_eq!(
        restore_disabled_keys(&format!("x = 1  # note {MARKER}\n")),
        "# x = 1  # note\n"
    );
}

#[test]
fn test_restore_preserves_indentation() {
    assert_eq!(restore_disabled_keys(&format!("  x = 1  # {MARKER}\n")), "  # x = 1\n");
}

#[test]
fn test_restore_leaves_unmarked_lines() {
    let source = "x = 1  # real comment\n";
    assert_eq!(restore_disabled_keys(source), source);
}

#[test]
fn test_restore_without_trailing_newline() {
    assert_eq!(restore_disabled_keys(&format!("x = 1  # {MARKER}")), "# x = 1");
}

#[test]
fn test_restore_recomments_value_reflowed_across_lines() {
    let formatted = concat!(
        "x = [\n",
        "  { path = \"README.rst\", start-after = \".. begin\" }\n",
        "]  # __toml_fmt_disabled__\n",
    );
    let restored = restore_disabled_keys(formatted);
    assert_eq!(
        restored,
        concat!(
            "# x = [\n",
            "#   { path = \"README.rst\", start-after = \".. begin\" }\n",
            "# ]\n",
        )
    );
    assert_eq!(
        restore_disabled_keys(&restored),
        restored,
        "an already-disabled block is stable"
    );
}

#[test]
fn test_restore_recomments_value_reflowed_when_indented() {
    let formatted = concat!("  x = [\n", "    1,\n", "    2\n", "  ]  # __toml_fmt_disabled__\n");
    assert_eq!(
        restore_disabled_keys(formatted),
        concat!("  # x = [\n", "  #   1,\n", "  #   2\n", "  # ]\n")
    );
}

#[test]
fn test_restore_leaves_adjacent_real_key_and_comment_untouched() {
    let formatted = concat!(
        "version.source = \"vcs\"\n",
        "# TODO: keep me\n",
        "metadata.x = [\n",
        "  { a = 1 }\n",
        "]  # __toml_fmt_disabled__\n",
    );
    assert_eq!(
        restore_disabled_keys(formatted),
        concat!(
            "version.source = \"vcs\"\n",
            "# TODO: keep me\n",
            "# metadata.x = [\n",
            "#   { a = 1 }\n",
            "# ]\n",
        )
    );
}

#[test]
fn test_round_trip_is_identity_for_disabled_key() {
    let source = "# default = true\n";
    assert_eq!(restore_disabled_keys(&enable_disabled_keys(source)), source);
}

// Guards against another #390: every value shape that fits on one comment line must round-trip unchanged, beyond
// the scalars the feature started with.
#[test]
fn test_disabled_value_constructs_round_trip() {
    let constructs = [
        "# n = 1\n",
        "# f = 1.5\n",
        "# b = true\n",
        "# s = \"hello\"\n",
        "# s = 'C:\\path'\n",
        "# items = [ \"a\", \"b\" ]\n",
        "# matrix = [ [ 1, 2 ], [ 3, 4 ] ]\n",
        "# opts = { deep = true, name = \"x\" }\n",
        "# rows = [ { a = 1 }, { b = 2 } ]\n",
        "# tool.x.y = [ { a = 1 } ]\n",
    ];
    for source in constructs {
        let out = with_disabled_keys(source, |enabled| {
            assert!(enabled.contains(MARKER), "{source:?} is enabled for the pass");
            crate::test_util::format_toml_str(enabled, 120)
        });
        assert_eq!(out, source, "{source:?} restored to its original comment");
    }
}

// The bug in #390: a value the formatter wraps must come back as a valid, stable comment block.
#[test]
fn test_with_disabled_keys_recomments_reflowed_array_block() {
    let source = "# data = [ \"alpha\", \"beta\", \"gamma\", \"delta\", \"epsilon\", \"zeta\", \"eta\" ]\n";
    let out = with_disabled_keys(source, |enabled| crate::test_util::format_toml_str(enabled, 30));
    assert!(
        out.contains('\n') && out.lines().count() > 1,
        "the value reflowed across lines"
    );
    assert!(
        out.lines().all(|line| line.trim_start().starts_with('#')),
        "every reflowed line stays commented, got:\n{out}"
    );
    assert!(!out.contains(MARKER), "the marker never reaches output");
    let again = with_disabled_keys(&out, |enabled| crate::test_util::format_toml_str(enabled, 30));
    assert_eq!(again, out, "the reflowed disabled block is stable");
}

#[test]
fn test_commented_table_section_stays_commented() {
    let source = concat!(
        "[tool.real]\n",
        "# [tool.disabled]\n",
        "# key = [\n",
        "#   1,\n",
        "# ]\n"
    );
    let out = with_disabled_keys(source, |enabled| crate::test_util::format_toml_str(enabled, 120));
    assert_eq!(
        out, source,
        "a commented table header and its keys stay verbatim comments"
    );
}
