//! Transparent handling of commented-out (disabled) keys.
//!
//! A standalone comment whose body is itself a single valid key-value (for example
//! `# default = true`) is treated as a temporarily disabled field rather than free
//! text. It is enabled for the duration of the formatting pass so it is laid out and
//! ordered together with the table it belongs to, then disabled again on the way out.
//! This keeps a disabled key anchored to its entry instead of drifting to the next
//! table, and formats the line the same way the enabled key would be formatted.

use tombi_syntax::SyntaxKind::{ARRAY_OF_TABLE, COMMENT, KEY_VALUE, TABLE};

/// Marker appended to a disabled key's trailing comment so the pass can find the key
/// again after it has travelled through reordering and re-parsing. Inline trailing
/// comments stay attached to their key-value, so the marker rides along for free.
/// It never reaches the formatted output: [`restore_disabled_keys`] strips it.
pub const MARKER: &str = "__toml_fmt_disabled__";

/// Rewrite a comment body into its enabled key-value form, or `None` when the body is
/// not a single key-value (prose, a commented table header, multiple keys, ...).
fn enabled_form(body: &str) -> Option<String> {
    let parsed = tombi_parser::parse(body);
    if !parsed.errors.is_empty() {
        return None;
    }
    let root = parsed.syntax_node();
    let key_values = root.descendants().filter(|n| n.kind() == KEY_VALUE).count();
    let has_table = root.descendants().any(|n| matches!(n.kind(), TABLE | ARRAY_OF_TABLE));
    if key_values != 1 || has_table {
        return None;
    }
    let has_trailing_comment = root.descendants_with_tokens().any(|t| t.kind() == COMMENT);
    Some(if has_trailing_comment {
        format!("{body} {MARKER}")
    } else {
        format!("{body}  # {MARKER}")
    })
}

/// Pre-pass: turn each disabled key into a real key-value tagged with [`MARKER`].
///
/// Keys that would not fit on a single line within `column_width` are left as plain
/// comments, since enabling them could reflow the value across several lines and break
/// the single-line restore.
pub fn enable_disabled_keys(source: &str, column_width: usize) -> String {
    let mut out: Vec<String> = Vec::with_capacity(source.lines().count());
    for line in source.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix('#') {
            let indent = &line[..line.len() - trimmed.len()];
            let body = rest.trim_start();
            if indent.len() + body.len() <= column_width
                && let Some(enabled) = enabled_form(body)
            {
                out.push(format!("{indent}{enabled}"));
                continue;
            }
        }
        out.push(line.to_string());
    }
    join_like(source, out)
}

/// Post-pass: turn every marker-tagged key-value back into a comment, dropping the
/// marker. The key kept its surrounding comment (if any), which is restored verbatim.
pub fn restore_disabled_keys(formatted: &str) -> String {
    let mut out: Vec<String> = Vec::with_capacity(formatted.lines().count());
    for line in formatted.lines() {
        if let Some(idx) = line.find(MARKER) {
            let before = line[..idx].trim_end();
            let body = before.strip_suffix('#').map_or(before, str::trim_end);
            let indent_len = body.len() - body.trim_start().len();
            let (indent, content) = body.split_at(indent_len);
            out.push(format!("{indent}# {content}"));
        } else {
            out.push(line.to_string());
        }
    }
    join_like(formatted, out)
}

fn join_like(original: &str, lines: Vec<String>) -> String {
    let joined = lines.join("\n");
    if original.ends_with('\n') {
        format!("{joined}\n")
    } else {
        joined
    }
}
