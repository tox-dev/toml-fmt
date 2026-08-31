//! A comment whose body is one valid key-value (`# default = true`) is a disabled field rather than prose. The pass
//! uncomments it so the formatter sorts it with its table, then comments it back; otherwise it would drift to the next
//! table and never get ordered.
//!
//! A value can span several comment lines. `# x = [` alone is invalid, yet `# x = [` / `#   1,` / `# ]` parses once the
//! run is uncommented together, so enabling works on whole runs. That also keeps the round-trip stable when the
//! formatter wraps a value across lines.

use std::collections::HashSet;

use toml_doc::Document;

/// Tags a disabled key's trailing comment so the pass can find it again after the formatter has reordered and
/// re-parsed everything. [`restore_disabled_keys`] strips it.
///
/// A file is free to hold this text itself, in a value or a comment of its own, so the pass works with a marker built
/// from this one that the file does not already contain.
pub const MARKER: &str = "__toml_fmt_disabled__";

thread_local! {
    /// The marker the pass running on this thread chose. A guard outside a pass has none, so
    /// nothing the file wrote is read as one.
    static IN_USE: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

/// Holds the marker for as long as the pass runs, and takes it back however the pass ends.
struct InUse;

impl InUse {
    fn hold(marker: &str) -> Self {
        IN_USE.with_borrow_mut(|held| *held = Some(marker.to_owned()));
        Self
    }
}

impl Drop for InUse {
    fn drop(&mut self) {
        IN_USE.with_borrow_mut(|held| *held = None);
    }
}

/// Whether the entry is one this pass turned back on, which is what the marker beside it says.
///
/// A pass that would split, drop or merge such an entry has to leave it alone: what says the entry
/// is disabled is the comment beside it, and none of those rewrites can say it of the entries they
/// leave behind.
#[must_use]
pub fn is_enabled_here(entry: &toml_doc::Entry<'_>) -> bool {
    let Some(comment) = entry.trail.comment.as_ref() else {
        return false;
    };
    IN_USE.with_borrow(|held| {
        held.as_deref()
            .is_some_and(|marker| marks_a_disabled_key(comment, marker))
    })
}

/// Whether the comment is the one [`enabled_form`] wrote for this pass: a comment holding nothing
/// but the marker, or the marker written at the end of a comment the file already had.
fn marks_a_disabled_key(comment: &str, marker: &str) -> bool {
    let body = comment.trim_start_matches('#').trim();
    body == marker
        || body
            .rsplit(char::is_whitespace)
            .next()
            .is_some_and(|last| last == kept_marker(marker))
}

/// A marker the source does not already hold, so nothing the file says can be read as one.
///
/// Every marker this can pick is the base one followed by some number of `x`, so the longest run
/// the file already writes after it says how many the marker needs to be one of its own.
fn fresh_marker(source: &str) -> String {
    let longest = source
        .match_indices(MARKER)
        .map(|(at, _)| {
            source[at + MARKER.len()..]
                .bytes()
                .take_while(|held| *held == b'x')
                .count()
        })
        .max();
    let mut marker = String::from(MARKER);
    if let Some(longest) = longest {
        marker.extend(std::iter::repeat_n('x', longest + 1));
    }
    marker
}

/// The one entry point formatters call, so enable and restore always bracket the pass as a pair.
///
/// Uncommenting a disabled key can put the same name in the document twice, which no reader can
/// read. So a document with a key turned back on is read with [`toml_doc::parse_syntax`], and the
/// caller is the one that says whether the source it was handed is a document at all. A key stands
/// only where the file wrote a comment on a line of its own, so turning one back on leaves a
/// document that still reads. A file holding no disabled key is formatted where it already is.
///
/// # Errors
///
/// Propagates whatever `format` rejected the document with. A rejected pass restores nothing.
pub fn try_with_disabled_keys(
    document: &mut Document<'_>,
    source: &str,
    format: impl FnOnce(&mut Document<'_>) -> Result<(), String>,
) -> Result<String, String> {
    let marker = fresh_marker(source);
    let Some(enabled) = enable_disabled_keys(document, source, &marker) else {
        // nothing was turned on, so nothing carries a marker to take back off
        format(document)?;
        return Ok(document.to_string());
    };
    let mut turned_on = toml_doc::parse_syntax(&enabled).expect("a comment stood where a key may stand");
    let formatted = {
        let _in_use = InUse::hold(&marker);
        format(&mut turned_on)?;
        turned_on.to_string()
    };
    Ok(restore_disabled_keys(&turned_on, &formatted, &marker))
}

/// The lines the file wrote a comment on where a key could have stood: what leads an entry or a
/// header, and what closes the document.
///
/// A `#` anywhere else is part of what a value says: inside a multi-line string, inside an array, or
/// beside a value, no key can stand there and uncommenting one would rewrite the value.
fn standing_comments(document: &Document<'_>) -> HashSet<usize> {
    let mut standing = HashSet::new();
    let mut line = 0;
    for entry in &document.root {
        take_comments(&entry.lead, &mut line, &mut standing);
        line += breaks(&entry.to_string()) - breaks(&entry.lead.to_string());
    }
    for section in &document.sections {
        take_comments(&section.header.lead, &mut line, &mut standing);
        line += breaks(&section.header.to_string()) - breaks(&section.header.lead.to_string());
        for entry in &section.entries {
            take_comments(&entry.lead, &mut line, &mut standing);
            line += breaks(&entry.to_string()) - breaks(&entry.lead.to_string());
        }
    }
    take_comments(&document.trailing, &mut line, &mut standing);
    standing
}

/// The comment lines a run of trivia holds, counted from `line`, which it moves past that run.
fn take_comments(trivia: &toml_doc::Trivia<'_>, line: &mut usize, standing: &mut HashSet<usize>) {
    for piece in trivia.pieces() {
        if !piece.is_blank() {
            standing.insert(*line);
        }
        *line += 1;
    }
}

fn breaks(written: &str) -> usize {
    written.matches('\n').count()
}

/// A commented table header ends enabling for the rest of its run, since the keys under it would otherwise leave
/// the table they belong to.
pub(crate) fn enable_disabled_keys(document: &Document<'_>, source: &str, marker: &str) -> Option<String> {
    let standing = standing_comments(document);
    let lines: Vec<&str> = source.lines().collect();
    let closes_at = closing_lines(&lines);
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut turned_on = false;
    let mut at = 0;
    while at < lines.len() {
        let Some((indent, body)) = split_comment(lines[at]).filter(|_| standing.contains(&at)) else {
            out.push(lines[at].to_string());
            at += 1;
            continue;
        };
        if is_table_header(body) {
            out.push(lines[at].to_string());
            at += 1;
            while at < lines.len() && split_comment(lines[at]).is_some() {
                out.push(lines[at].to_string());
                at += 1;
            }
            continue;
        }
        match enable_block(&lines, at, &closes_at, marker) {
            Some((end, enabled)) => {
                out.push(format!("{indent}{enabled}"));
                turned_on = true;
                at = end + 1;
            }
            None => {
                out.push(lines[at].to_string());
                at += 1;
            }
        }
    }
    turned_on.then(|| join_like(source, out))
}

/// A nested table header ends the run, since the keys under it are a separate value.
fn enable_block(lines: &[&str], start: usize, closes_at: &[Close], marker: &str) -> Option<(usize, String)> {
    let end = match closes_at.get(start) {
        Some(Close::At(end)) => *end,
        Some(Close::Never) | None => return None,
        // a run holding a string written over lines is read line by line, since a candidate inside
        // one starts reading it afresh
        Some(Close::Scan) => scan_to_close(lines, start)?,
    };
    // every line of a run is one the file wrote a comment on, which is what a run is
    let bodies: Vec<&str> = lines[start..=end]
        .iter()
        .map(|line| split_comment(line).expect("the run holds comments").1)
        .collect();
    enabled_form(&bodies.join("\n"), marker).map(|enabled| (end, enabled))
}

/// Read the run from `start` until the value it opens closes, which is where a string written over
/// lines makes the depth alone say too little.
fn scan_to_close(lines: &[&str], start: usize) -> Option<usize> {
    let mut open = OpenValue::default();
    for (end, line) in lines.iter().enumerate().skip(start) {
        let (_, body) = split_comment(line)?;
        open.read(body);
        if !open.is_open() {
            return Some(end);
        }
    }
    None
}

/// Where the value opened on a line first closes.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Close {
    /// On this line.
    At(usize),
    /// On no line of the run it stands in.
    Never,
    /// Not from the depth alone: the run holds a string written over lines.
    Scan,
}

/// Where the value opened on each line of a comment run first closes, read in one pass so a run is
/// measured once rather than once for every line in it.
///
/// A line leaves the depth it found one bracket deeper for each it opened, so a value opened on one
/// line closes on the first line the depth comes back to where that line started.
fn closing_lines(lines: &[&str]) -> Vec<Close> {
    let mut closes: Vec<Close> = vec![Close::Never; lines.len()];
    let mut ends: Vec<i64> = vec![0; lines.len()];
    let mut waiting: Vec<usize> = Vec::new();
    let mut run_start = 0;
    let mut depth: i64 = 0;
    let mut spans_lines = false;
    for (at, line) in lines.iter().enumerate() {
        // a line the file wrote no comment on ends the run, and so nothing before it is still open
        let Some((_, body)) = split_comment(line) else {
            read_line_by_line(&mut closes[run_start..at], spans_lines);
            waiting.clear();
            run_start = at + 1;
            depth = 0;
            spans_lines = false;
            continue;
        };
        let mut open = OpenValue::default();
        open.read(body);
        spans_lines = spans_lines || open.inside.is_some();
        depth += open.net;
        ends[at] = depth;
        waiting.push(at);
        while waiting
            .last()
            .is_some_and(|held| before(&ends, *held, run_start) >= depth)
        {
            let held = waiting.pop().expect("the stack was just read");
            closes[held] = Close::At(at);
        }
    }
    read_line_by_line(&mut closes[run_start..], spans_lines);
    closes
}

/// A run holding a string written over lines says too little through its brackets alone, so every
/// candidate in it is read from its own line.
fn read_line_by_line(closes: &mut [Close], spans_lines: bool) {
    if !spans_lines {
        return;
    }
    for held in closes {
        *held = Close::Scan;
    }
}

/// The depth the line before this one left behind, which is where a value opened here starts from.
fn before(ends: &[i64], at: usize, run_start: usize) -> i64 {
    if at == run_start { 0 } else { ends[at - 1] }
}

/// How much of a value the lines read so far have left open: a multi-line string, an array or an
/// inline table. A line shaped like a header inside one of those is text the value holds rather
/// than a table of its own.
#[derive(Default)]
struct OpenValue {
    inside: Option<&'static str>,
    depth: usize,
    /// The brackets the line opened less the ones it closed, which is what a line adds to the depth
    /// a value already stands at.
    net: i64,
}

impl OpenValue {
    /// Read one more line of the value.
    fn read(&mut self, body: &str) {
        let mut rest = body;
        while !rest.is_empty() {
            if let Some(delimiter) = self.inside {
                if let Some(after) = rest.strip_prefix(delimiter) {
                    self.inside = None;
                    rest = after;
                    continue;
                }
                // only a basic string reads a backslash as opening an escape
                let held = usize::from(delimiter.starts_with('"') && rest.starts_with('\\')) + 1;
                rest = past(rest, held);
                continue;
            }
            if let Some(delimiter) = ["\"\"\"", "'''", "\"", "'"]
                .into_iter()
                .find(|open| rest.starts_with(open))
            {
                self.inside = Some(delimiter);
                rest = &rest[delimiter.len()..];
                continue;
            }
            match rest.as_bytes()[0] {
                b'[' | b'{' => {
                    self.depth += 1;
                    self.net += 1;
                }
                b']' | b'}' => {
                    self.depth = self.depth.saturating_sub(1);
                    self.net -= 1;
                }
                // a comment runs to the end of its line, so what it holds says nothing about the value
                b'#' => break,
                _ => {}
            }
            rest = past(rest, 1);
        }
        // a string written with one quote closes on the line it opened
        if self.inside.is_some_and(|delimiter| delimiter.len() == 1) {
            self.inside = None;
        }
    }

    fn is_open(&self) -> bool {
        self.inside.is_some() || self.depth > 0
    }
}

fn past(text: &str, count: usize) -> &str {
    let mut chars = text.chars();
    for _ in 0..count {
        chars.next();
    }
    chars.as_str()
}

/// `None` unless `body` is exactly one key-value. The marker extends a comment already on the last line, so the value
/// never ends up with two trailing comments.
fn enabled_form(body: &str, marker: &str) -> Option<String> {
    // the comment stands somewhere inside the file, so the body is read there rather than at the
    // start of one: a byte-order mark opens a document and says nothing anywhere else
    let read = format!("{AHEAD}{body}");
    let document = toml_doc::parse_syntax(&read).ok()?;
    // one key-value and nothing else: a nested `set_env = { A = "1" }` is still one key. It has to
    // open the body too, since what stands above it is a comment of the file's own
    if document.root.len() != 2 || !document.sections.is_empty() || !document.root[1].lead.pieces().is_empty() {
        return None;
    }
    // the marker says which comment it is written in, since only the one it opened comes off again
    Some(if document.root[1].trail.comment.is_some() {
        format!("{body} {}", kept_marker(marker))
    } else {
        format!("{body}  # {marker}")
    })
}

/// A key-value written above the body being read, which puts it where the file wrote it: inside a
/// document rather than at the start of one.
const AHEAD: &str = "held = 0\n";

/// The marker as it is written inside a comment the file wrote, which stays where it is.
fn kept_marker(marker: &str) -> String {
    format!("{marker}-kept")
}

/// Drops one space after `#` to mirror how [`comment_disabled_line`] writes it back, keeping the round-trip stable.
fn split_comment(line: &str) -> Option<(&str, &str)> {
    let trimmed = line.trim_start();
    let rest = trimmed.strip_prefix('#')?;
    let indent = &line[..line.len() - trimmed.len()];
    Some((indent, rest.strip_prefix(' ').unwrap_or(rest)))
}

fn is_table_header(body: &str) -> bool {
    toml_doc::parse_syntax(body).is_ok_and(|document| !document.sections.is_empty())
}

/// The line without the marker, and without the comment the marker opened; a comment the file wrote
/// stays as it was written.
fn without_marker<'a>(line: &'a str, marker: &str) -> &'a str {
    let kept = kept_marker(marker);
    if let Some(at) = line.rfind(&kept) {
        return line[..at].trim_end();
    }
    let at = line.rfind(marker).expect("the line a span ends on carries the marker");
    let before = line[..at].trim_end();
    before.strip_suffix('#').map_or(before, str::trim_end)
}

/// A wrapped value carries the marker on its last line only, so the whole span gets commented. The span starts at
/// the key rather than the node, which also owns the leading comments and blank lines before it that stay put.
pub(crate) fn restore_disabled_keys(document: &Document<'_>, formatted: &str, marker: &str) -> String {
    let lines: Vec<&str> = formatted.lines().collect();
    // the marker closes the last line of its entry, so only that line gives it back
    let mut plan: Vec<Option<(usize, bool)>> = vec![None; lines.len()];
    for (start, end) in marked_spans(document, marker) {
        let first = lines[start];
        let indent = first.len() - first.trim_start().len();
        for (at, slot) in plan.iter_mut().enumerate().take(end + 1).skip(start) {
            *slot = Some((indent, at == end));
        }
    }
    let restored = lines
        .iter()
        .zip(plan)
        .map(|(line, plan)| {
            plan.map_or_else(
                || (*line).to_string(),
                |(indent, strips)| comment_disabled_line(line, indent, strips.then_some(marker)),
            )
        })
        .collect();
    join_like(formatted, restored)
}

/// `base` is the key's own indent, so the `#` lands at its column and the value's deeper indentation survives.
fn comment_disabled_line(line: &str, base: usize, marker: Option<&str>) -> String {
    let cleaned = match marker {
        Some(marker) => without_marker(line, marker),
        None => line,
    };
    let cut = base.min(cleaned.len());
    format!("{}# {}", &cleaned[..cut], &cleaned[cut..])
}

/// The line spans of the entries the pass enabled, counted through the written document.
///
/// An entry is one of them when the marker closes its line, which is where [`enabled_form`] put it.
/// Text that merely holds the same characters, in a value or in a comment of its own, is left
/// alone. The span starts at the entry's own line rather than at its leading comments, which are
/// about something else, and ends on its last line, since a value wrapped over several lines
/// carries the marker there.
fn marked_spans(document: &Document<'_>, marker: &str) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut line = 0;
    for entry in &document.root {
        push_span(&mut spans, &mut line, entry, marker);
    }
    for section in &document.sections {
        line += section.header.to_string().matches('\n').count();
        for entry in &section.entries {
            push_span(&mut spans, &mut line, entry, marker);
        }
    }
    spans
}

fn push_span(spans: &mut Vec<(usize, usize)>, line: &mut usize, entry: &toml_doc::Entry<'_>, marker: &str) {
    let written = entry.to_string();
    let breaks = written.matches('\n').count();
    let enabled = entry
        .trail
        .comment
        .as_deref()
        .is_some_and(|comment| comment.contains(marker));
    if enabled {
        let start = *line + entry.lead.to_string().matches('\n').count();
        spans.push((start, *line + breaks - usize::from(written.ends_with('\n'))));
    }
    *line += breaks;
}

fn join_like(original: &str, lines: Vec<String>) -> String {
    let joined = lines.join("\n");
    if original.ends_with('\n') {
        format!("{joined}\n")
    } else {
        joined
    }
}
