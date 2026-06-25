//! Commented-out (disabled) keys, kept in step with the table they belong to.
//!
//! A comment whose body is one valid key-value (`# default = true`) is a disabled field, not
//! prose. The pass uncomments it so the formatter sorts it with its table, then comments it back;
//! otherwise it would drift to the next table and never get ordered.
//!
//! A value can span several comment lines. `# x = [` alone is invalid, yet `# x = [` / `#   1,` /
//! `# ]` parses once the run is uncommented together, so enabling works on whole runs. That also
//! keeps the round-trip stable when the formatter wraps a value across lines.

use tombi_syntax::SyntaxKind::{
    ARRAY_OF_TABLE, COMMENT, KEY_VALUE, KEY_VALUE_GROUP, KEYS, LINE_BREAK, TABLE, WHITESPACE,
};
use tombi_syntax::SyntaxNode;

/// Tags a disabled key's trailing comment so the pass can find it again after the formatter has
/// reordered and re-parsed everything. [`restore_disabled_keys`] strips it.
pub const MARKER: &str = "__toml_fmt_disabled__";

/// Top-level key-values only; a nested `set_env = { A = "1" }` is one key, not two.
fn top_level_key_values(root: &SyntaxNode) -> usize {
    root.children()
        .filter(|child| child.kind() == KEY_VALUE_GROUP)
        .map(|group| group.children().filter(|kv| kv.kind() == KEY_VALUE).count())
        .sum()
}

/// Uncomment `body` (one line or several) into a single marker-tagged key-value, or `None` when it
/// is not exactly one key-value. The marker extends a comment already on the last line so the
/// value never ends up with two trailing comments.
fn enabled_form(body: &str) -> Option<String> {
    let parsed = tombi_parser::parse(body);
    if !parsed.errors.is_empty() {
        return None;
    }
    let root = parsed.syntax_node();
    let has_table = root.children().any(|n| matches!(n.kind(), TABLE | ARRAY_OF_TABLE));
    if top_level_key_values(&root) != 1 || has_table {
        return None;
    }
    let ends_with_comment = root
        .descendants_with_tokens()
        .filter(|t| !matches!(t.kind(), WHITESPACE | LINE_BREAK))
        .last()
        .is_some_and(|t| t.kind() == COMMENT);
    Some(if ends_with_comment {
        format!("{body} {MARKER}")
    } else {
        format!("{body}  # {MARKER}")
    })
}

/// The one entry point formatters call, so enable and restore always bracket the pass as a pair.
pub fn with_disabled_keys(content: &str, format: impl FnOnce(&str) -> String) -> String {
    let enabled = enable_disabled_keys(content);
    restore_disabled_keys(&format(&enabled))
}

/// Indent before the `#` and the body after it, dropping one space to mirror how
/// [`comment_disabled_line`] writes a comment back. `None` for a non-comment line.
fn split_comment(line: &str) -> Option<(&str, &str)> {
    let trimmed = line.trim_start();
    let rest = trimmed.strip_prefix('#')?;
    let indent = &line[..line.len() - trimmed.len()];
    Some((indent, rest.strip_prefix(' ').unwrap_or(rest)))
}

fn is_table_header(body: &str) -> bool {
    tombi_parser::parse(body)
        .syntax_node()
        .children()
        .any(|n| matches!(n.kind(), TABLE | ARRAY_OF_TABLE))
}

/// Shortest run from `start` that uncomments to one key-value, with its last line index and the
/// tagged text. A nested table header ends the run, since the keys under it are a separate value.
fn enable_block(lines: &[&str], start: usize, run_end: usize) -> Option<(usize, String)> {
    let mut bodies: Vec<&str> = Vec::new();
    for (end, line) in lines.iter().enumerate().take(run_end).skip(start) {
        let (_, body) = split_comment(line)?;
        if end > start && is_table_header(body) {
            break;
        }
        bodies.push(body);
        if let Some(enabled) = enabled_form(&bodies.join("\n")) {
            return Some((end, enabled));
        }
    }
    None
}

/// Uncomment every disabled key-value, tagging each with [`MARKER`]; prose and incomplete fragments
/// stay commented. A commented table header ends enabling for the rest of its run, since the keys
/// under it would otherwise leave the table they belong to.
pub(crate) fn enable_disabled_keys(source: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut i = 0;
    while i < lines.len() {
        let Some((indent, body)) = split_comment(lines[i]) else {
            out.push(lines[i].to_string());
            i += 1;
            continue;
        };
        if is_table_header(body) {
            out.push(lines[i].to_string());
            i += 1;
            while i < lines.len() && split_comment(lines[i]).is_some() {
                out.push(lines[i].to_string());
                i += 1;
            }
            continue;
        }
        let run_end = (i..lines.len())
            .find(|&k| split_comment(lines[k]).is_none())
            .unwrap_or(lines.len());
        match enable_block(&lines, i, run_end) {
            Some((end, enabled)) => {
                out.push(format!("{indent}{enabled}"));
                i = end + 1;
            }
            None => {
                out.push(lines[i].to_string());
                i += 1;
            }
        }
    }
    join_like(source, out)
}

/// Comment every marker-tagged key-value back, dropping the marker. A wrapped value carries the
/// marker on its last line only, so the whole span gets commented. The span starts at the key, not
/// the node: the node also owns the leading comments and blank lines before it, which stay put.
pub(crate) fn restore_disabled_keys(formatted: &str) -> String {
    if !formatted.contains(MARKER) {
        return formatted.to_string();
    }
    let lines: Vec<&str> = formatted.lines().collect();
    let mut base_indent: Vec<Option<usize>> = vec![None; lines.len()];
    let root = tombi_parser::parse(formatted).syntax_node();
    for kv in root.descendants().filter(|n| n.kind() == KEY_VALUE) {
        if !kv.to_string().contains(MARKER) {
            continue;
        }
        let keys = kv.children().find(|c| c.kind() == KEYS).expect("a key-value has a key");
        let start = keys.range().start.line as usize;
        let end = kv.range().end.line as usize;
        let first = lines[start];
        let indent = first.len() - first.trim_start().len();
        for slot in base_indent.iter_mut().take(end + 1).skip(start) {
            *slot = Some(indent);
        }
    }
    let restored = lines
        .iter()
        .zip(base_indent)
        .map(|(line, indent)| indent.map_or_else(|| (*line).to_string(), |indent| comment_disabled_line(line, indent)))
        .collect();
    join_like(formatted, restored)
}

/// Comment one line of a disabled key-value, stripping the marker if present. `base` is the key's
/// own indent, so the `#` lands at its column and the value's deeper indentation survives after it.
fn comment_disabled_line(line: &str, base: usize) -> String {
    let cleaned = match line.find(MARKER) {
        Some(idx) => {
            let before = line[..idx].trim_end();
            before.strip_suffix('#').map_or(before, str::trim_end)
        }
        None => line,
    };
    let cut = base.min(cleaned.len());
    format!("{}# {}", &cleaned[..cut], &cleaned[cut..])
}

fn join_like(original: &str, lines: Vec<String>) -> String {
    let joined = lines.join("\n");
    if original.ends_with('\n') {
        format!("{joined}\n")
    } else {
        joined
    }
}
