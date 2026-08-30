//! Deciding the whitespace of a document: what goes around `=`, inside brackets, and where an
//! array breaks across lines.
//!
//! Every rule writes into the slots the document already carries, so laying out is a walk that sets
//! fields rather than a pass that re-prints the file.

use toml_doc::{Array, Document, Entry, InlineTable, LineEnding, Pad, Padding, Section, Value};

/// How wide a line may run, and how far a continuation line is pushed in.
#[derive(Debug, Clone, Copy)]
pub struct Layout {
    pub column_width: usize,
    pub indent: usize,
    pub ending: LineEnding,
}

impl Layout {
    /// Lay out every line of the document.
    pub fn apply(self, document: &mut Document<'_>) {
        for entry in &mut document.root {
            self.entry(entry, 0);
        }
        for section in &mut document.sections {
            self.section(section);
        }
        self.close_last_line(document);
    }

    fn section(self, section: &mut Section<'_>) {
        self.indent_lead(&mut section.header.lead, 0);
        section.header.indent = "".into();
        section.header.pre_key = "".into();
        section.header.post_key = "".into();
        tighten_key(&mut section.header.key);
        self.trail_of(&mut section.header.trail);
        for entry in &mut section.entries {
            self.entry(entry, 0);
        }
    }

    fn entry(self, entry: &mut Entry<'_>, depth: usize) {
        self.indent_lead(&mut entry.lead, depth);
        entry.indent = self.pad(depth).into();
        tighten_key(&mut entry.key_value.key);
        entry.key_value.pre_eq = " ".into();
        entry.key_value.post_eq = " ".into();
        let prefix = crate::width::columns(&entry.indent) + crate::width::columns(&entry.key_value.key.to_string()) + 3;
        // a comment closing the line is part of how wide that line runs
        let suffix = entry
            .trail
            .comment
            .as_ref()
            .map_or(0, |text| crate::width::columns(text) + 2);
        self.value(&mut entry.key_value.value, depth, prefix, suffix);
        self.trail_of(&mut entry.trail);
    }

    /// A line that ran out without a break gets one, since folding and reordering can leave any
    /// line with another below it.
    fn trail_of(self, trail: &mut toml_doc::Trail<'_>) {
        // a comment closing a line sits two spaces past the value, which is the column the array
        // alignment pass widens from
        trail.ws = if trail.comment.is_some() {
            "  ".into()
        } else {
            "".into()
        };
        trail.ending = self.ending;
    }

    fn value(self, value: &mut Value<'_>, depth: usize, prefix: usize, suffix: usize) -> Written {
        match value {
            Value::Scalar(repr) => {
                normalize_quotes(repr);
                Written::text(&repr.to_string())
            }
            Value::Array(array) => self.array(array, depth, prefix, suffix),
            Value::InlineTable(table) => self.inline_table(table, depth, prefix),
        }
    }

    fn array(self, array: &mut Array<'_>, depth: usize, prefix: usize, suffix: usize) -> Written {
        let was_open = array.members.iter().any(|member| member.lead.is_multiline());
        // a comma follows a member wherever the array ends up, so it is part of how wide the line
        // that member closes on runs
        let held: Vec<Written> = array
            .members
            .iter_mut()
            .map(|member| self.value(&mut member.item, depth + 1, self.indent * (depth + 1), 1))
            .collect();
        let one_line = one_line_written(&held);
        if !self.array_breaks(array, depth, prefix + suffix + one_line.columns) {
            self.inline_array(array);
            return one_line;
        }
        // an array too wide on its own is being broken up, which a trailing comma then holds open;
        // one that only overruns because of the key or the indent before it is merely wrapped, and
        // a comma there would say something about the file that the file does not say
        let outgrew = one_line
            .columns
            .saturating_sub(usize::from(!array.members.is_empty()) * 2)
            > self.column_width;
        let trailing = array.trailing_comma || (!was_open && outgrew);
        self.explode_array(array, depth, trailing);
        written_members(&array.members, &held, array.trailing_comma, &array.trailing)
    }

    /// A trailing comma, a comment, or a line that would run past `column_width` each force the
    /// array open. Without one of those it closes back up, however it was written.
    ///
    /// An array whose indent already fills the column gains nothing by opening: every line it wrote
    /// would start past the column it was asked to fit, so it stays on one line unless the file says
    /// otherwise.
    fn array_breaks(self, array: &Array<'_>, depth: usize, width: usize) -> bool {
        let commented = array
            .members
            .iter()
            .any(|member| member.lead.has_comment() || member.trail.has_comment() || member.after.has_comment())
            || array.trailing.has_comment();
        let room = self.indent.saturating_mul(depth + 1) < self.column_width;
        array.trailing_comma || commented || (room && width > self.column_width)
    }

    /// A one-line array is written `[ a, b ]`, with a space inside each bracket. An empty one keeps
    /// its brackets together.
    fn inline_array(self, array: &mut Array<'_>) {
        // an array holding a comment is written out over lines, so nothing here carries one
        for member in &mut array.members {
            member.lead = space(1);
            member.after = Padding::default();
            member.trail = Padding::default();
        }
        array.trailing_comma = false;
        array.trailing = space(usize::from(!array.members.is_empty()));
    }

    fn explode_array(self, array: &mut Array<'_>, depth: usize, trailing: bool) {
        for member in &mut array.members {
            let comments = comments_of(&member.lead);
            let blank = leads_with_blank(&member.lead);
            member.lead = self.line_break(depth + 1, depth + 1, &comments);
            if blank {
                member.lead.parts_mut().insert(0, Pad::Newline(self.ending));
            }
            // a comment can sit either side of the comma, and each one runs to the end of its own
            // line, so only the first closes this one
            member.after = self.closing_comments(depth + 1, &member.trail, &member.after);
            member.trail = Padding::default();
        }
        array.trailing_comma = trailing;
        let comments = comments_of(&array.trailing);
        array.trailing = self.line_break(depth, depth + 1, &comments);
    }

    /// An inline table is written on one line, `{ a = 1, b = 2 }`.
    ///
    /// TOML 1.1 lets one span several lines and hold comments, and no single-line form can keep a
    /// comment. Such a table keeps the spacing the file gave it; only what sits around `=` and the
    /// values themselves are laid out.
    fn inline_table(self, table: &mut InlineTable<'_>, depth: usize, prefix: usize) -> Written {
        let commented = table
            .members
            .iter()
            .any(|member| member.lead.has_comment() || member.trail.has_comment() || member.after.has_comment())
            || table.trailing.has_comment();
        // members share a line, so each one starts where the ones before it left off, and a member
        // the file wrote on a line of its own starts where that line's indent leaves it
        let mut column = prefix + 2;
        let mut held: Vec<Written> = Vec::with_capacity(table.members.len());
        for member in &mut table.members {
            if let Some(indent) = opens_a_line(&member.lead) {
                column = indent;
            }
            tighten_key(&mut member.item.key);
            let key_width = crate::width::columns(&member.item.key.to_string()) + 3;
            let written = self.value(&mut member.item.value, depth, column + key_width, 0);
            column += key_width + written.last_line + 2;
            held.push(Written::of(key_width).then(written));
            member.item.pre_eq = " ".into();
            member.item.post_eq = " ".into();
            if commented {
                continue;
            }
            member.lead = space(1);
            member.trail = Padding::default();
            member.after = Padding::default();
        }
        if commented {
            // the file's own spacing stands, and the members were measured as they were laid out
            return written_members(&table.members, &held, table.trailing_comma, &table.trailing);
        }
        table.trailing_comma = false;
        table.trailing = space(usize::from(!table.members.is_empty()));
        one_line_written(&held)
    }

    /// Rebuild the run before a member: each comment keeps the line it had, and the member lands
    /// on a fresh one.
    fn line_break(self, depth: usize, comment_depth: usize, comments: &[String]) -> Padding<'static> {
        let mut padding = Padding::default();
        padding.parts_mut().push(Pad::Newline(self.ending));
        for comment in comments {
            padding.parts_mut().push(Pad::Space(self.pad(comment_depth).into()));
            padding.parts_mut().push(Pad::Comment(comment.clone().into()));
            padding.parts_mut().push(Pad::Newline(self.ending));
        }
        if depth > 0 {
            padding.parts_mut().push(Pad::Space(self.pad(depth).into()));
        }
        padding
    }

    /// A comment leading an item sits at the item's own column, wherever the file had put it.
    fn indent_lead(self, lead: &mut toml_doc::Trivia<'_>, depth: usize) {
        for piece in lead.pieces_mut() {
            match piece {
                toml_doc::Piece::Blank { indent, .. } => *indent = "".into(),
                toml_doc::Piece::Comment { indent, .. } => *indent = self.pad(depth).into(),
            }
        }
    }

    /// Trailing lines get the same treatment as the rest, so a file never runs out mid-line.
    /// A formatted file ends where a line ends, whether or not the source it came from did.
    fn close_last_line(self, document: &mut Document<'_>) {
        self.indent_lead(&mut document.trailing, 0);
        if let Some(piece) = document.trailing.pieces_mut().last_mut() {
            match piece {
                toml_doc::Piece::Blank { ending, .. } | toml_doc::Piece::Comment { ending, .. } => {
                    *ending = self.ending;
                }
            }
        }
        document.ends_without_newline = false;
    }

    /// What closes a member's line. The first comment shares the line; each one after it takes a
    /// line of its own, since a comment runs to the end of the line it opens.
    fn closing_comments(self, depth: usize, before: &Padding<'_>, after: &Padding<'_>) -> Padding<'static> {
        let mut out = Padding::default();
        for (index, comment) in comments_of(before).into_iter().chain(comments_of(after)).enumerate() {
            if index > 0 {
                out.parts_mut().push(Pad::Newline(self.ending));
                out.parts_mut().push(Pad::Space(self.pad(depth).into()));
            } else {
                out.parts_mut().push(Pad::Space("  ".into()));
            }
            out.parts_mut().push(Pad::Comment(comment.into()));
        }
        out
    }

    fn pad(self, depth: usize) -> String {
        " ".repeat(self.indent * depth)
    }
}

/// A dotted key reads as one name, so no spacing sits around its dots. A quoted segment keeps its
/// quotes, written the same way as any other string.
fn tighten_key(key: &mut toml_doc::Key<'_>) {
    let names = key.segments();
    for (part, name) in key.parts_mut().iter_mut().zip(names) {
        part.lead = "".into();
        part.trail = "".into();
        if part.is_quoted()
            && let Some(written) = double_quoted(&name)
        {
            part.set_quoted(written);
        }
    }
}

/// A single-line string is written with double quotes, unless it holds one, in which case single
/// quotes save it from being escaped.
fn normalize_quotes(repr: &mut toml_doc::Repr<'_>) {
    let Some(quoting) = repr.quoting() else { return };
    if quoting.is_multiline() {
        return;
    }
    if let Some(written) = double_quoted(&repr.decoded()) {
        *repr = written;
    }
}

/// The text as a double-quoted string, unless writing it that way would mean adding escapes the
/// form it was written in does not need.
fn double_quoted(text: &str) -> Option<toml_doc::Repr<'static>> {
    if text.contains('"') {
        return toml_doc::fits_literal(text).then(|| toml_doc::Repr::literal_string(text));
    }
    // a backslash the file wrote plainly would gain an escape on the way into double quotes, and a
    // value nothing else touched is not worth spelling at more length
    (!text.contains('\\')).then(|| toml_doc::Repr::basic_string(text))
}

fn space(width: usize) -> Padding<'static> {
    let mut padding = Padding::default();
    if width > 0 {
        padding.parts_mut().push(Pad::Space(" ".repeat(width).into()));
    }
    padding
}

/// The comments a run holds, in the order they were written.
fn comments_of(padding: &Padding<'_>) -> Vec<String> {
    padding
        .parts()
        .iter()
        .filter_map(|part| match part {
            Pad::Comment(text) => Some(text.to_string()),
            Pad::Space(_) | Pad::Newline(_) => None,
        })
        .collect()
}

/// Line up the trailing comments inside each array, one space past the widest member.
///
/// The widest member sets the column, so the comments read as a block rather than stepping in and
/// out with the values they follow.
pub fn align_array_comments(document: &mut Document<'_>) {
    for entry in &mut document.root {
        align_in_value(&mut entry.key_value.value);
    }
    for section in &mut document.sections {
        for entry in &mut section.entries {
            align_in_value(&mut entry.key_value.value);
        }
    }
}

fn align_in_value(value: &mut Value<'_>) -> Written {
    match value {
        Value::Scalar(repr) => Written::text(&repr.to_string()),
        Value::InlineTable(table) => {
            let held: Vec<Written> = table
                .members
                .iter_mut()
                .map(|member| {
                    let key = crate::width::columns(&member.item.key.to_string())
                        + crate::width::columns(&member.item.pre_eq)
                        + crate::width::columns(&member.item.post_eq)
                        + 1;
                    Written::of(key).then(align_in_value(&mut member.item.value))
                })
                .collect();
            written_members(&table.members, &held, table.trailing_comma, &table.trailing)
        }
        Value::Array(array) => {
            let held: Vec<Written> = array
                .members
                .iter_mut()
                .map(|member| align_in_value(&mut member.item))
                .collect();
            align_members(array, &held);
            written_members(&array.members, &held, array.trailing_comma, &array.trailing)
        }
    }
}

fn align_members(array: &mut Array<'_>, held: &[Written]) {
    // each member is measured as it is written, comma included
    let widths: Vec<usize> = held.iter().map(|written| written.last_line + 1).collect();
    let Some(widest) = widths.iter().copied().max() else {
        return;
    };
    // the layout writes a comment as spacing then the comment itself, so widening the spacing it
    // opens with is what moves the comment to the column
    for (index, width) in widths.into_iter().enumerate() {
        if let Some(Pad::Space(spacing)) = array.members[index].after.parts_mut().first_mut() {
            *spacing = " ".repeat(widest - width + 1).into();
        }
    }
}

/// Where a member begins on its line, when what leads it opened a new one.
fn opens_a_line(padding: &Padding<'_>) -> Option<usize> {
    let written = padding.to_string();
    written
        .rsplit_once('\n')
        .map(|(_, indent)| crate::width::columns(indent))
}

/// How far along the line the member ends.
///
/// What moves a comment along is the text written before it, escapes and quotes included, and a
/// value that closes on a later line carries only that line with it.
/// How wide a value is written: the columns it takes in all, and the columns of the line it ends
/// on, since a value the layout broke over lines only ends on the last of them.
#[derive(Debug, Clone, Copy, Default)]
struct Written {
    columns: usize,
    last_line: usize,
    broken: bool,
}

impl Written {
    /// What the text takes, read once rather than measured again by every line above it.
    fn text(written: &str) -> Self {
        Self {
            columns: crate::width::columns(written),
            last_line: crate::width::columns(written.rsplit('\n').next().unwrap_or(written)),
            broken: written.contains('\n'),
        }
    }

    /// A run of `columns` on one line.
    const fn of(columns: usize) -> Self {
        Self {
            columns,
            last_line: columns,
            broken: false,
        }
    }

    /// What the two take written one after the other.
    fn then(self, next: Self) -> Self {
        Self {
            columns: self.columns + next.columns,
            last_line: if next.broken {
                next.last_line
            } else {
                self.last_line + next.last_line
            },
            broken: self.broken || next.broken,
        }
    }
}

/// What the members take written on one line, brackets and separators included: `[ a, b ]` puts a
/// space before each member, a comma between them, and a space before the closing bracket.
///
/// A member the file wrote over lines still runs over them here, so what the container closes on is
/// the line that member ended, not the sum of all of them.
fn one_line_written(held: &[Written]) -> Written {
    let last = held.len().saturating_sub(1);
    let mut written = Written::of(1);
    for (index, member) in held.iter().enumerate() {
        written = written.then(Written::of(1)).then(*member);
        if index < last {
            written = written.then(Written::of(1));
        }
    }
    written
        .then(Written::of(usize::from(!held.is_empty())))
        .then(Written::of(1))
}

/// What the members take once the layout has written them out over several lines.
fn written_members<T>(
    members: &[toml_doc::Member<'_, T>],
    held: &[Written],
    trailing_comma: bool,
    trailing: &Padding<'_>,
) -> Written {
    let last = members.len().saturating_sub(1);
    let mut written = Written::of(1);
    for (index, member) in members.iter().enumerate() {
        written = written
            .then(Written::text(&member.lead.to_string()))
            .then(held[index])
            .then(Written::text(&member.trail.to_string()));
        if index < last || trailing_comma {
            written = written.then(Written::of(1));
        }
        written = written.then(Written::text(&member.after.to_string()));
    }
    written.then(Written::text(&trailing.to_string())).then(Written::of(1))
}

/// Whether an empty line was written above the member, which the file used to set it apart from
/// the ones before it.
fn leads_with_blank(padding: &Padding<'_>) -> bool {
    padding
        .parts()
        .iter()
        .take_while(|part| !matches!(part, Pad::Comment(_)))
        .filter(|part| matches!(part, Pad::Newline(_)))
        .count()
        > 1
}
