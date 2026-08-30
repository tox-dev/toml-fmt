//! Rewriting the text a value holds, and picking the form it is written in.
//!
//! A value that gains a quote is better off literal, one that keeps its newlines stays a multi-line
//! string, and one that outgrows the column is wrapped with line continuations. Choosing between
//! those is the whole job here; the document holds the result as its own source text.

use toml_doc::{Document, Quoting, Repr, Value};

/// What `"""\` takes, which is what a wrapped value opens its first line with.
const OPENER: usize = 4;

/// Where a value sits, which decides whether it may be wrapped and how far its continuations are
/// pushed in.
#[derive(Debug, Clone, Copy)]
pub struct Wrap<'a> {
    pub column_width: usize,
    pub indent: &'a str,
    /// What the layout writes on the line before the value: the key and its `= `, or the indent a
    /// nested value is pushed in by.
    pub prefix: usize,
    /// A value inside `{ }` cannot break across lines.
    pub inline_table: bool,
}

/// The characters a string value holds, or `None` when the value is not a string.
#[must_use]
pub fn text_of(value: &Value<'_>) -> Option<String> {
    match value {
        Value::Scalar(repr) if repr.quoting().is_some() => toml_doc::decode(repr).ok(),
        _ => None,
    }
}

/// Whether a rule is writing the value out afresh, or the value is only being laid out.
#[derive(Clone, Copy, PartialEq)]
enum Written {
    Afresh,
    AsTheFileHasIt,
}

/// Rewrite a string value through `transform`, leaving the written form alone.
pub fn update<F>(value: &mut Value<'_>, transform: F)
where
    F: Fn(&str) -> String,
{
    rewrite(value, transform, None, Written::Afresh);
}

/// Rewrite a string value through `transform` and wrap it when it outgrows the column.
pub fn update_wrapped<F>(value: &mut Value<'_>, transform: F, wrap: Wrap<'_>)
where
    F: Fn(&str) -> String,
{
    rewrite(value, transform, Some(wrap), Written::Afresh);
}

fn rewrite<F>(value: &mut Value<'_>, transform: F, wrap: Option<Wrap<'_>>, written: Written)
where
    F: Fn(&str) -> String,
{
    let Value::Scalar(repr) = value else { return };
    let Some(quoting) = repr.quoting() else { return };
    let current = repr.decoded();
    let text = transform(&current);
    if let Some(form) = form(&text, quoting, text == current, written, wrap) {
        *repr = form;
    }
}

/// The form the text should be written in, or `None` to leave what is already there.
fn form(
    text: &str,
    quoting: Quoting,
    unchanged: bool,
    written: Written,
    wrap: Option<Wrap<'_>>,
) -> Option<Repr<'static>> {
    // a string the file already spread over lines is left as it wrote it
    if unchanged && quoting.is_multiline() && text.contains('\n') {
        return None;
    }
    // a value the key ahead of it pushes past the column is broken up too, as long as what opens a
    // multi-line string still fits after that key: a key already over the column on its own cannot
    // be brought back by rewriting its value. This runs before picking a one-line form, which would
    // otherwise hold a value open past the column it was asked to fit.
    if let Some(wrap) = wrap
        && !wrap.inline_table
        && wrap.prefix + crate::width::columns(&toml_doc::encode_basic(text)) > wrap.column_width
        && wrap.prefix + OPENER <= wrap.column_width
        && let Some(broken) = wrap_with_continuations(text, wrap.column_width, wrap.indent)
    {
        let written = multiline_repr(&broken);
        // a continuation eats the line break and the whitespace after it, so a value whose own
        // whitespace would go with them is left as the file wrote it
        if toml_doc::decode(&written).is_ok_and(|read| read == text) {
            return Some(written);
        }
        return None;
    }
    // a quote is carried plainly by a literal string
    if text.contains('"') && toml_doc::fits_literal(text) {
        return Some(Repr::literal_string(text));
    }
    // laying a value out leaves what the file wrote alone: its escapes already read back as the
    // text it holds, and a backslash it wrote plainly would gain one to move into double quotes.
    // A rule that rewrote the value has picked new text, which is written in the canonical form.
    if written == Written::AsTheFileHasIt && (quoting == Quoting::Basic || text.contains('\\')) {
        return None;
    }
    // a rewrite is written out escaped, whatever form the file used: dropping the decoded text
    // between `"""` would read a backslash it holds as the start of an escape
    Some(Repr::basic_string(text))
}

fn multiline_repr(written: &str) -> Repr<'static> {
    // the wrapper writes the escapes itself, so what it hands over is a string that reads back
    Repr::string(written, Quoting::MlBasic).expect("the wrapper writes a multi-line string")
}

/// Break the text across lines with `\` continuations, so the written form fits the column while
/// standing for one unbroken string.
///
/// `None` where the column has no room for the form: a width that cannot hold the indent, one
/// character and the continuation after it would only trade a long line for a longer one.
fn wrap_with_continuations(text: &str, column_width: usize, indent: &str) -> Option<String> {
    let escaped = escaped_body(text);
    let mut result = String::from("\"\"\"\\\n");
    let mut line_start = 0;
    // the continuation the line ends with takes a column of its own
    let effective_width = column_width.checked_sub(indent.len() + 1)?;

    while line_start < escaped.len() {
        let remaining = &escaped[line_start..];
        if crate::width::columns(remaining) + indent.len() < column_width {
            result.push_str(indent);
            result.push_str(remaining);
            result.push_str("\\\n");
            break;
        }
        let split_at = wrap_point(remaining, effective_width);
        result.push_str(indent);
        result.push_str(&remaining[..split_at]);
        result.push_str("\\\n");
        line_start += split_at;
    }
    result.push_str(indent);
    result.push_str("\"\"\"");
    // a character wider than what is left of the line has to go somewhere, and a line it runs past
    // the column is not the wrapping the caller asked for
    result
        .lines()
        .skip(1)
        .all(|line| crate::width::columns(line) <= column_width)
        .then_some(result)
}

/// Break after ` :: `, which separates the parts of a classifier, else after the last space, else
/// wherever the width runs out.
fn wrap_point(text: &str, max_len: usize) -> usize {
    let ends = crate::width::break_points(text, max_len);
    let head_end = ends
        .iter()
        .copied()
        .take_while(|end| crate::width::columns(&text[..*end]) <= max_len)
        .last()
        .unwrap_or(ends[0]);
    let head = &text[..head_end];
    if let Some(position) = head.rfind(" :: ") {
        return position + 4;
    }
    head.rfind(' ').map_or(head_end, |position| position + 1)
}

fn escaped_body(text: &str) -> String {
    let quoted = toml_doc::encode_basic(text);
    quoted[1..quoted.len() - 1].to_owned()
}

/// Write every key in its plainest form: bare where TOML allows it, double quoted otherwise.
pub fn normalize_key_quotes(document: &mut Document<'_>) {
    for entry in &mut document.root {
        normalize_key_value(&mut entry.key_value);
    }
    for section in &mut document.sections {
        normalize_key(&mut section.header.key);
        for entry in &mut section.entries {
            normalize_key_value(&mut entry.key_value);
        }
    }
}

fn normalize_key_value(key_value: &mut toml_doc::KeyValue<'_>) {
    normalize_key(&mut key_value.key);
    normalize_keys_in(&mut key_value.value);
}

/// Inline tables hold keys too, however deeply an array nests them.
fn normalize_keys_in(value: &mut Value<'_>) {
    match value {
        Value::Scalar(_) => {}
        Value::Array(array) => {
            for member in &mut array.members {
                normalize_keys_in(&mut member.item);
            }
        }
        Value::InlineTable(table) => {
            for member in &mut table.members {
                normalize_key_value(&mut member.item);
            }
        }
    }
}

fn normalize_key(key: &mut toml_doc::Key<'_>) {
    let names = key.segments();
    for (part, name) in key.parts_mut().iter_mut().zip(names) {
        part.set_name(&name);
    }
}

/// Wrap every string in the document that outgrows the column, apart from the keys named in
/// `skip`, whose values carry meaning that line breaks would obscure.
pub fn wrap_long_strings(document: &mut Document<'_>, column_width: usize, indent: usize, skip: &[String]) {
    let padding = " ".repeat(indent);
    let skip: Vec<Vec<Want>> = skip.iter().map(|pattern| pattern_segments(pattern)).collect();
    for entry in &mut document.root {
        wrap_entry(entry, &[], column_width, &padding, &skip);
    }
    for section in &mut document.sections {
        let table = section.header.key.segments();
        for entry in &mut section.entries {
            wrap_entry(entry, &table, column_width, &padding, &skip);
        }
    }
}

fn wrap_entry(
    entry: &mut toml_doc::Entry<'_>,
    table: &[String],
    column_width: usize,
    indent: &str,
    skip: &[Vec<Want>],
) {
    // a pattern is written against the key's whole path, table included, so `*.skip_me` reaches a
    // key of that name under any table
    let mut path = table.to_vec();
    path.extend(entry.key_value.key.segments());
    if skip.iter().any(|pattern| matches_key(&path, pattern)) {
        return;
    }
    // the layout writes `key = ` ahead of the value, which is part of what the line runs to; the
    // key is measured as the layout will write it, not as the file spaced it out
    let prefix = canonical_key_columns(&entry.key_value.key) + 3;
    wrap_value(&mut entry.key_value.value, column_width, indent, prefix, 0, false);
}

/// How wide the key is once it is written in its plainest form, with nothing between its parts but
/// the dots that separate them.
fn canonical_key_columns(key: &toml_doc::Key<'_>) -> usize {
    let names = key.segments();
    let dots = names.len() - 1;
    names
        .iter()
        .map(|name| crate::width::columns(&toml_doc::encode_key(name)))
        .sum::<usize>()
        + dots
}

fn wrap_value(
    value: &mut Value<'_>,
    column_width: usize,
    indent: &str,
    prefix: usize,
    depth: usize,
    inline_table: bool,
) {
    match value {
        Value::Scalar(_) => rewrite(
            value,
            ToOwned::to_owned,
            Some(Wrap {
                column_width,
                indent,
                prefix,
                inline_table,
            }),
            Written::AsTheFileHasIt,
        ),
        Value::Array(array) => {
            // a member of an array the layout opens starts its line one level further in
            let inside = indent.len() * (depth + 1);
            for member in &mut array.members {
                wrap_value(&mut member.item, column_width, indent, inside, depth + 1, inline_table);
            }
        }
        Value::InlineTable(table) => {
            for member in &mut table.members {
                wrap_value(&mut member.item.value, column_width, indent, prefix, depth, true);
            }
        }
    }
}

/// A pattern matches segment by segment, with `*` standing for any one segment. A pattern opening
/// with `*` matches the tail of the path, so `*.commands` reaches `commands` under any table; one
/// ending in `*` matches from the head, so `tool.ruff.*` covers what is written under that table.
/// A pattern naming no wildcard names the one key it spells.
fn matches_key(path: &[String], pattern: &[Want]) -> bool {
    if pattern.len() > path.len() {
        return false;
    }
    let leads = pattern.first() == Some(&Want::Any);
    let ends = pattern.last() == Some(&Want::Any);
    // a wildcard the pattern opens with stands for the path above it and one it closes with for the
    // path below; anywhere else a wildcard names one segment, and so does every name, which is what
    // holds a pattern to the key it spells rather than to what is written under it
    if !leads && !ends && pattern.len() != path.len() {
        return false;
    }
    let from = if leads && pattern.len() > 1 {
        path.len() - pattern.len()
    } else {
        0
    };
    pattern.iter().zip(&path[from..]).all(|(want, have)| match want {
        Want::Any => true,
        Want::Named(name) => name == have,
    })
}

/// What one component of a pattern names: any one segment, or the one it spells.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Want {
    Any,
    Named(String),
}

/// The segments a pattern names, read the way TOML reads a key: a dot inside quotes belongs to the
/// segment around it, so `tool."a.b".commands` names three segments rather than four. `*` stands
/// for a segment of its own and is never a name, since TOML has no bare key of that spelling.
fn pattern_segments(pattern: &str) -> Vec<Want> {
    let mut segments: Vec<Want> = Vec::new();
    let mut start = 0;
    let mut quote: Option<char> = None;
    let mut escaped = false;
    for (at, held) in pattern.char_indices() {
        match (quote, held) {
            (Some('"'), '\\') => escaped = !escaped,
            (Some(open), held) if held == open && !escaped => quote = None,
            (Some(_), _) => escaped = false,
            (None, '"' | '\'') => quote = Some(held),
            (None, '.') => {
                segments.push(one_segment(&pattern[start..at]));
                start = at + 1;
            }
            (None, _) => {}
        }
    }
    segments.push(one_segment(&pattern[start..]));
    segments
}

/// What one component of a pattern names, with its quoting resolved. A bare `*` stands for any one
/// segment, while a quoted one names the key spelled that way. A component TOML cannot read as a
/// key names itself.
fn one_segment(component: &str) -> Want {
    if component == "*" {
        return Want::Any;
    }
    match crate::sections::read_name(component).as_deref() {
        Ok([only]) => Want::Named(only.clone()),
        _ => Want::Named(component.to_owned()),
    }
}
