//! Moving entries between a table and the sub-tables written under it.
//!
//! `[tool.x] a.b = 1` and `[tool.x.a] b = 1` describe the same document, so a formatter can pick
//! either. Collapsing folds a sub-table into its parent as dotted keys; expanding writes the
//! dotted keys back out as their own header.

use std::collections::HashSet;

use toml_doc::{
    Array, Comment, Document, Entry, Header, InlineTable, Key, KeyPart, KeyValue, LineEnding, Pad, Padding, Piece,
    Section, SectionKind, Trail, Trivia, Value,
};

/// Fold every `[name.sub]` into `[name]` as `sub.key` entries, and every `[[name.sub]]` into
/// `sub = [ { ... } ]`.
///
/// A repeated header, or a sub-table that still has tables of its own beneath it, stays where it
/// is: neither survives the move.
pub fn collapse(document: &mut Document<'_>, name: &str) {
    collapse_where(document, name, &|_| true, Width::unbounded());
}

/// How wide a line may run and how far a nested one is pushed in, which is what a folded table has
/// to fit.
#[derive(Debug, Clone, Copy)]
pub struct Width {
    pub column: usize,
    pub indent: usize,
}

impl Width {
    /// A width nothing outgrows, for a fold whose shape is not in question.
    #[must_use]
    pub const fn unbounded() -> Self {
        Self {
            column: usize::MAX,
            indent: 2,
        }
    }
}

/// [`collapse`], folding in only the sub-tables `wanted` accepts by name, so one table can be held
/// out while the rest of its siblings fold in.
pub fn collapse_where(document: &mut Document<'_>, name: &str, wanted: &dyn Fn(&[String]) -> bool, width: Width) {
    let root: Vec<String> = name.split('.').map(str::to_owned).collect();
    collapse_of(document, &root, wanted, width);
}

/// [`collapse_where`] for a name read out of the document, whose segments may hold anything.
pub fn collapse_of(document: &mut Document<'_>, root: &[String], wanted: &dyn Fn(&[String]) -> bool, width: Width) {
    // fold the deepest table first, so `[a.b.c]` reaches `[a]` as `b.c.key` however many levels
    // were written out and whichever of them the file skipped
    let mut left_alone: Vec<Vec<String>> = Vec::new();
    // the tables this fold can touch are read once and worked through in that order, so a file
    // holding thousands of them does not walk them all over again for each one it moves
    let mut index = Under::new(document, root);
    let mut pending = deepest_first(document, &index, root, &left_alone);
    while let Some(sub) = pending.pop() {
        // the parent is this table minus its last segment, taken from the segments themselves so a
        // name holding a dot, like `plugins."poetry.application.plugin"`, is not cut in half
        let parent = &sub[..sub.len() - 1];
        if is_array_of_tables(document, &index, &sub) {
            if wanted(&sub) {
                collapse_array_of_tables_under(document, &sub, width);
                index = Under::new(document, root);
            }
            left_alone.push(sub);
            continue;
        }
        // a parent written more than once has no one place to fold into: the keys belong to the
        // element the file wrote them under, and `[a.b]` after the second `[[a]]` is not the first
        if !movable(document, &index, &sub) || !can_hold(document, &index, parent) || !wanted(&sub) {
            left_alone.push(sub);
            continue;
        }
        let at = index_of(document, &index, &sub).expect("the table is written");
        // a table with nothing in it still holds whatever was written under it, and folding it into
        // `leaf = {}` would leave those tables with no table of their own to belong to
        if document.sections[at].entries.is_empty() && has_tables_below(document, &index, &sub) {
            left_alone.push(sub);
            continue;
        }
        // a table whose every key is disabled is one the file wrote empty, and folding those keys
        // into the parent as comments would leave nothing saying the table is there at all
        let entries = &document.sections[at].entries;
        if !entries.is_empty() && entries.iter().all(crate::disabled::is_enabled_here) {
            left_alone.push(sub);
            continue;
        }
        let held = document.sections.len();
        ensure_exists(document, &index, parent);
        // writing the parent out puts a table under `root` that was not there to read before
        if document.sections.len() > held {
            index = Under::new(document, root);
            pending = deepest_first(document, &index, root, &left_alone);
            pending.retain(|name| name != &sub);
        }
        let at = index_of(document, &index, &sub).expect("the table is written");
        let depth = document.sections[at].header.key.parts().len() - 1;
        let mut section = document.sections.remove(at);
        index.gone(at);
        // the header's own segments carry over, so a quoted name stays quoted
        let leaf_parts: Vec<KeyPart<'_>> = section.header.key.parts()[depth..].to_vec();
        let parent_at = index_of(document, &index, parent).expect("the parent was just written out");
        if section.entries.is_empty() {
            let leaf = leaf_parts.first().cloned().expect("a table sits under a name");
            document.sections[parent_at]
                .entries
                .push(empty_table_entry(leaf, &section));
            continue;
        }
        for entry in &mut section.entries {
            entry.key_value.key.prepend_parts(leaf_parts.clone());
        }
        // the comments above and beside the header now lead the first key it brought along; the
        // blank lines set the header apart from the table above it, and that gap is gone
        let lead = std::mem::take(&mut section.header.lead);
        let mut kept: Vec<_> = lead
            .pieces()
            .iter()
            .filter(|piece| !piece.is_blank())
            .cloned()
            .collect();
        if let Some(text) = section.header.trail.comment.take() {
            kept.push(Piece::Comment {
                indent: "".into(),
                text,
                ending: section.header.trail.ending,
            });
        }
        section.entries[0].lead.pieces_mut().splice(0..0, kept);
        document.sections[parent_at].entries.append(&mut section.entries);
    }
}

/// Where the tables one fold works over are written: the ones at or under the name being folded
/// into, and the ones on the way down to it, whose own keys could already write it out. Nothing
/// else can hold or name a table this fold moves, so this is what its questions are asked of.
struct Under {
    at: Vec<usize>,
}

impl Under {
    fn new(document: &Document<'_>, root: &[String]) -> Self {
        let at = document
            .sections
            .iter()
            .enumerate()
            // the name a header holds is read segment by segment here, since most of them are under
            // some other table and building each one to find that out is the bulk of the work
            .filter(|(_, section)| {
                let key = &section.header.key;
                key.opens_with(root) || (key.parts().len() < root.len() && key.opens_with(&root[..key.parts().len()]))
            })
            .map(|(at, _)| at)
            .collect();
        Self { at }
    }

    /// Take the table written at `at` out, which moves everything below it up one.
    fn gone(&mut self, at: usize) {
        self.at.retain(|held| *held != at);
        for held in &mut self.at {
            if *held > at {
                *held -= 1;
            }
        }
    }

    fn sections<'a, 'src>(
        &'a self,
        document: &'a Document<'src>,
    ) -> impl DoubleEndedIterator<Item = &'a Section<'src>> {
        self.at.iter().map(|at| &document.sections[*at])
    }
}

/// The tables written below `root`, deepest last so folding pops them off the end. Tables at the
/// same depth fold in the order they were written, which is the order they end up in.
fn deepest_first(document: &Document<'_>, index: &Under, root: &[String], skip: &[Vec<String>]) -> Vec<Vec<String>> {
    let mut names: Vec<Vec<String>> = index
        .sections(document)
        .rev()
        .filter(|section| section.header.key.parts().len() > root.len())
        .map(|section| section.header.key.segments())
        .filter(|segments| !skip.contains(segments))
        .collect();
    let mut seen: HashSet<Vec<String>> = HashSet::new();
    names.retain(|name| seen.insert(name.clone()));
    names.sort_by_key(Vec::len);
    names
}

/// Write the dotted keys of `[name]` back out as `[name.sub]` headers.
pub fn expand(document: &mut Document<'_>, name: &str) {
    expand_of(document, &name.split('.').map(str::to_owned).collect::<Vec<String>>());
}

/// [`expand`] for a name read out of the document, whose segments may hold anything.
pub fn expand_of(document: &mut Document<'_>, name: &[String]) {
    expand_where(document, name, &|_| true);
}

/// [`expand_of`] for a table whose children are written out one by one, where `wanted` says which
/// of them a header of its own is meant for.
pub fn expand_where(document: &mut Document<'_>, name: &[String], wanted: &dyn Fn(&[String]) -> bool) {
    let Some(parent) = document.sections.iter().position(|section| named(section, name)) else {
        return;
    };
    let mut moved: Vec<(Vec<String>, Vec<KeyPart<'_>>, Vec<Entry<'_>>)> = Vec::new();
    let mut kept = Vec::new();
    for mut entry in std::mem::take(&mut document.sections[parent].entries) {
        // a disabled key is one the comment beside it speaks for, and a header written for it would
        // carry none of that
        if crate::disabled::is_enabled_here(&entry) {
            kept.push(entry);
            continue;
        }
        let Some(head) = leading_segments(&entry, name, wanted) else {
            kept.push(entry);
            continue;
        };
        // the parts carry their own quoting, so a name holding a dot stays the one segment it is
        let leaf = entry.key_value.key.take_leading(head.len());
        match moved.iter_mut().find(|(existing, _, _)| *existing == head) {
            Some((_, _, bucket)) => bucket.push(entry),
            None => moved.push((head, leaf, vec![entry])),
        }
    }
    document.sections[parent].entries = kept;

    let mut at = parent + 1;
    for (_, leaf, entries) in moved {
        let mut key = document.sections[parent].header.key.clone();
        key.extend_parts(leaf);
        let header = header_for(key, &document.sections[parent]);
        document.sections.insert(at, Section { header, entries });
        at += 1;
    }
}

/// The segments of a dotted key that become a header of their own: the shortest run `wanted` names,
/// so a table stays where it is written until something asks for it, and the name that asks for one
/// below it is the one that gets it.
fn leading_segments(entry: &Entry<'_>, name: &[String], wanted: &dyn Fn(&[String]) -> bool) -> Option<Vec<String>> {
    let segments = entry.key_value.key.segments();
    (1..segments.len()).find_map(|width| {
        let mut child = name.to_vec();
        child.extend_from_slice(&segments[..width]);
        wanted(&child).then(|| segments[..width].to_vec())
    })
}

/// Write out `[name]` when the file only ever named tables below it.
fn ensure_exists(document: &mut Document<'_>, index: &Under, name: &[String]) {
    if index_of(document, index, name).is_some() {
        return;
    }
    let at = *index
        .at
        .iter()
        .find(|at| is_below(&document.sections[**at].header.key.segments(), name))
        .expect("a table below the one being written out");
    // the parent's own segments come from the table below it, so a quoted name stays one segment
    let key = Key::from_parts(document.sections[at].header.key.parts()[..name.len()].to_vec());
    // a header the file never wrote carries nothing of its own: what led the table below it, and
    // whatever was written beside that header, stay with the table they were written for
    let header = header_for(key, &document.sections[at]);
    document.sections.insert(
        at,
        Section {
            header,
            entries: Vec::new(),
        },
    );
}

fn is_array_of_tables(document: &Document<'_>, index: &Under, name: &[String]) -> bool {
    index
        .sections(document)
        .any(|section| section.header.kind == SectionKind::ArrayOfTables && named(section, name))
}

/// Whether a table folded under `name` would land where the file put it: either the name is not
/// written yet, so writing it out creates the one place, or it names a single plain table.
fn can_hold(document: &Document<'_>, index: &Under, name: &[String]) -> bool {
    let mut matches = index.sections(document).filter(|section| named(section, name));
    match matches.next() {
        None => !written_by_a_key(document, index, name),
        Some(section) => section.header.kind == SectionKind::Table && matches.next().is_none(),
    }
}

/// Whether a dotted key elsewhere already writes the table out, in which case a header for it would
/// say the same table twice. The sections at or below the name are the ones being folded away, so
/// what they hold is what the header comes to stand for.
fn written_by_a_key(document: &Document<'_>, index: &Under, name: &[String]) -> bool {
    let below = |path: &[String]| path.len() > name.len() && path.starts_with(name);
    document.root.iter().any(|entry| below(&entry.key_value.key.segments()))
        || index
            .sections(document)
            .filter(|section| !section.header.key.segments().starts_with(name))
            .any(|section| {
                let header = section.header.key.segments();
                section.entries.iter().any(|entry| {
                    let mut path = header.clone();
                    path.extend(entry.key_value.key.segments());
                    below(&path)
                })
            })
}

/// Whether the name is written once, as a plain table.
fn movable(document: &Document<'_>, index: &Under, name: &[String]) -> bool {
    let mut matches = index.sections(document).filter(|section| named(section, name));
    matches
        .next()
        .is_some_and(|section| section.header.kind == SectionKind::Table)
        && matches.next().is_none()
}

fn has_tables_below(document: &Document<'_>, index: &Under, name: &[String]) -> bool {
    index
        .sections(document)
        .any(|section| is_below(&section.header.key.segments(), name))
}

/// Whether the key names a table under `wanted`, compared segment by segment so a quoted name
/// holding a dot counts as the one segment it is.
fn is_below(segments: &[String], wanted: &[impl AsRef<str>]) -> bool {
    segments.len() > wanted.len()
        && segments
            .iter()
            .zip(wanted)
            .all(|(held, want)| held.as_str() == want.as_ref())
}

fn index_of(document: &Document<'_>, index: &Under, name: &[String]) -> Option<usize> {
    index.at.iter().copied().find(|at| named(&document.sections[*at], name))
}

/// Whether the header names exactly these segments, so a quoted name holding a dot is the one
/// segment the file wrote rather than the two a dotted path would read.
fn named(section: &Section<'_>, name: &[String]) -> bool {
    section.header.key.parts().len() == name.len() && section.header.key.opens_with(name)
}

/// An emptied sub-table still has to say it was there, as `sub = {}`.
fn empty_table_entry<'a>(leaf: KeyPart<'a>, section: &Section<'a>) -> Entry<'a> {
    Entry {
        lead: section.header.lead.clone(),
        indent: "".into(),
        key_value: KeyValue {
            key: Key::from_parts(vec![leaf]),
            pre_eq: " ".into(),
            post_eq: " ".into(),
            value: Value::InlineTable(InlineTable::default()),
        },
        trail: section.header.trail.clone(),
    }
}

fn header_for<'a>(key: Key<'a>, sibling: &Section<'a>) -> Header<'a> {
    Header {
        lead: Trivia::default(),
        indent: "".into(),
        kind: SectionKind::Table,
        pre_key: "".into(),
        key,
        post_key: "".into(),
        trail: Trail {
            ws: "".into(),
            comment: None,
            ending: sibling.header.trail.ending,
        },
    }
}

/// Fold every `[[name.field]]` back into `name` as `field = [ { ... } ]`.
///
/// This is the short table format's half of [`expand`]'s work on arrays of tables.
pub fn collapse_array_of_tables(document: &mut Document<'_>, full_name: &str, width: Width) {
    let name: Vec<String> = full_name.split('.').map(str::to_owned).collect();
    collapse_array_of_tables_under(document, &name, width);
}

fn collapse_array_of_tables_under(document: &mut Document<'_>, name: &[String], width: Width) {
    let (field, parent) = name.split_last().expect("an array of tables sits under a table");
    let index = Under::new(document, parent);
    // a table written under one of the elements has no inline form to move into, and the array
    // would leave its header naming a value that no dotted key can extend
    if has_tables_below(document, &index, name) || written_by_a_key(document, &index, parent) {
        return;
    }
    // which element of the parent array a child belongs to is what the order it is written in says,
    // and one array holding every child would say it belongs to the first
    if index
        .sections(document)
        .filter(|section| named(section, parent))
        .count()
        > 1
    {
        return;
    }
    let written: Vec<&Section<'_>> = index
        .sections(document)
        .filter(|section| section.header.kind == SectionKind::ArrayOfTables && named(section, name))
        .collect();
    // an array of nothing but empty elements says no more as `[ {}, {} ]` than it does written out
    if written.iter().all(|section| section.entries.is_empty()) {
        return;
    }
    // a comment past the first key would end up inside the braces, where the rest of the line after
    // it would be swallowed. A disabled key is one the comment beside it speaks for, wherever it
    // sits, and a member of an inline table has no line of its own to carry that
    if written.iter().any(|section| {
        section.entries.iter().skip(1).any(has_comment) || section.entries.iter().any(crate::disabled::is_enabled_here)
    }) {
        return;
    }
    let leaf = index
        .sections(document)
        .find(|section| named(section, name))
        .map(|section| section.header.key.parts()[name.len() - 1].clone())
        .expect("the array of tables is written");
    let mut members = Vec::with_capacity(written.len());
    for section in written {
        let table = InlineTable {
            members: section
                .entries
                .iter()
                .map(|entry| crate::build::member(entry.key_value.clone()))
                .collect(),
            trailing_comma: false,
            trailing: Padding::default(),
        };
        let mut member = crate::build::member(Value::InlineTable(table));
        // folding a table too wide for one line would bury it, so it stays written out. What it
        // takes is read from the way the layout will write it, since the spacing the file happened
        // to use is not what the folded table ends up with
        if !fits_one_line(&leaf, &member.item, width) {
            return;
        }
        member.lead = lead_comments(section);
        members.push(member);
    }
    // the parent has to be there before the sections holding the members are dropped
    ensure_exists(document, &index, parent);
    document.sections.retain(|section| !named(section, name));
    let index = Under::new(document, parent);
    let at = index_of(document, &index, parent).expect("the parent was just written out");
    let array = Array {
        members,
        trailing_comma: false,
        trailing: Padding::default(),
    };
    let mut entry = crate::build::entry(field, Value::Array(array));
    // the field's own segment carries over, so a quoted name holding a dot stays one key
    entry.key_value.key = Key::from_parts(vec![leaf]);
    document.sections[at].entries.push(entry);
}

/// Whether the value, written under the key that will hold it, stays on one line the column has
/// room for.
fn fits_one_line(leaf: &KeyPart<'_>, value: &Value<'_>, width: Width) -> bool {
    let mut document = Document::default();
    let mut entry = crate::build::entry("x", value.clone());
    entry.key_value.key = Key::from_parts(vec![leaf.clone()]);
    document.root.push(entry);
    crate::layout::Layout {
        column_width: width.column,
        indent: width.indent,
        ending: LineEnding::Lf,
    }
    .apply(&mut document);
    let written = document.to_string();
    let mut lines = written.lines();
    lines
        .next()
        .is_some_and(|line| crate::width::columns(line) <= width.column)
        && lines.next().is_none()
}

fn has_comment(entry: &Entry<'_>) -> bool {
    entry.trail.comment.is_some() || entry.lead.pieces().iter().any(|piece| !piece.is_blank())
}

/// The comments above the header and above or beside its first key, moved to where they lead the
/// inline table the table folds into.
fn lead_comments<'a>(section: &Section<'a>) -> Padding<'a> {
    let mut lead = Padding::default();
    let first = section.entries.first();
    let texts = comments_in(&section.header.lead)
        .chain(section.header.trail.comment.clone())
        .chain(first.into_iter().flat_map(|entry| comments_in(&entry.lead)))
        .chain(first.and_then(|entry| entry.trail.comment.clone()));
    for text in texts {
        lead.parts_mut().push(Pad::Comment(text));
        // a comment runs to the end of its line, so the value has to start on the next one
        lead.parts_mut().push(Pad::Newline(LineEnding::Lf));
    }
    lead
}

fn comments_in<'a>(trivia: &Trivia<'a>) -> impl Iterator<Item = Comment<'a>> {
    trivia.pieces().iter().filter_map(|piece| match piece {
        Piece::Comment { text, .. } => Some(text.clone()),
        Piece::Blank { .. } => None,
    })
}
