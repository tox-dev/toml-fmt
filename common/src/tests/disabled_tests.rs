use crate::disabled::{MARKER, enable_disabled_keys, restore_disabled_keys, with_disabled_keys};

#[test]
fn test_with_disabled_keys_brackets_the_format_pass() {
    let out = with_disabled_keys("# x = 1\n", 120, |enabled| {
        assert!(enabled.contains(MARKER), "the format pass sees the enabled key");
        enabled.to_string()
    });
    assert_eq!(out, "# x = 1\n");
}

#[test]
fn test_enable_promotes_valid_key() {
    let out = enable_disabled_keys("# default = true\n", 120);
    assert_eq!(out, format!("default = true  # {MARKER}\n"));
}

#[test]
fn test_enable_promotes_inline_table_key() {
    let out = enable_disabled_keys("# set_env = {A = \"1\"}\n", 120);
    assert_eq!(out, format!("set_env = {{A = \"1\"}}  # {MARKER}\n"));
}

#[test]
fn test_enable_leaves_two_keys_on_one_line() {
    let source = "# a = 1 b = 2\n";
    assert_eq!(enable_disabled_keys(source, 120), source);
}

#[test]
fn test_enable_preserves_indentation() {
    let out = enable_disabled_keys("  # x = 1\n", 120);
    assert_eq!(out, format!("  x = 1  # {MARKER}\n"));
}

#[test]
fn test_enable_appends_to_existing_trailing_comment() {
    let out = enable_disabled_keys("# x = 1  # note\n", 120);
    assert_eq!(out, format!("x = 1  # note {MARKER}\n"));
}

#[test]
fn test_enable_leaves_prose_comment() {
    let source = "# just a note, not a key\n";
    assert_eq!(enable_disabled_keys(source, 120), source);
}

#[test]
fn test_enable_leaves_commented_table_header() {
    let source = "# [tool.foo]\n";
    assert_eq!(enable_disabled_keys(source, 120), source);
}

#[test]
fn test_enable_leaves_non_comment_lines() {
    let source = "enabled = true\n";
    assert_eq!(enable_disabled_keys(source, 120), source);
}

#[test]
fn test_enable_skips_when_line_exceeds_width() {
    let source = "# x = 1\n";
    assert_eq!(enable_disabled_keys(source, 3), source);
}

#[test]
fn test_enable_without_trailing_newline() {
    let out = enable_disabled_keys("# x = 1", 120);
    assert_eq!(out, format!("x = 1  # {MARKER}"));
}

#[test]
fn test_restore_marker_only_comment() {
    let line = format!("default = true  # {MARKER}\n");
    assert_eq!(restore_disabled_keys(&line), "# default = true\n");
}

#[test]
fn test_restore_keeps_original_trailing_comment() {
    let line = format!("x = 1  # note {MARKER}\n");
    assert_eq!(restore_disabled_keys(&line), "# x = 1  # note\n");
}

#[test]
fn test_restore_preserves_indentation() {
    let line = format!("  x = 1  # {MARKER}\n");
    assert_eq!(restore_disabled_keys(&line), "  # x = 1\n");
}

#[test]
fn test_restore_leaves_unmarked_lines() {
    let source = "x = 1  # real comment\n";
    assert_eq!(restore_disabled_keys(source), source);
}

#[test]
fn test_restore_without_trailing_newline() {
    let line = format!("x = 1  # {MARKER}");
    assert_eq!(restore_disabled_keys(&line), "# x = 1");
}

#[test]
fn test_round_trip_is_identity_for_disabled_key() {
    let source = "# default = true\n";
    let enabled = enable_disabled_keys(source, 120);
    assert_eq!(restore_disabled_keys(&enabled), source);
}
