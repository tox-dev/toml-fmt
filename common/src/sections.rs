//! Finding tables by name and putting their keys in order.
//!
//! A document already groups entries under the header they were written below, so a section is the
//! unit that moves. Ordering keys is a sort over that section's entries: an entry carries the
//! comments written above it, so nothing has to be spliced back into place.

use std::collections::HashMap;

use toml_doc::{
    Document, Entry, InlineTable, Key, KeyPart, KeyValue, LineEnding, Member, Piece, Section, SectionKind, Trivia,
    Value,
};

use crate::group::{base_segments, is_group_marker};

/// The sections written under `name`, in document order.
///
/// `[[tool.x]]` may repeat, so a name can name several sections.
pub fn named<'d, 'a>(document: &'d mut Document<'a>, name: &str) -> Vec<&'d mut Section<'a>> {
    document
        .sections
        .iter_mut()
        .filter(|section| section.header.key.is_path(name))
        .collect()
}

/// The first section written under `name`.
pub fn first<'d, 'a>(document: &'d mut Document<'a>, name: &str) -> Option<&'d mut Section<'a>> {
    document
        .sections
        .iter_mut()
        .find(|section| section.header.key.is_path(name))
}

/// Run `visit` over every key and value in the section.
///
/// The name a rule matches on is [`dispatch_name`], so a key the file quoted whole never reaches a
/// rule written for the dotted path that reads the same.
pub fn for_entries<F>(section: &mut Section<'_>, mut visit: F)
where
    F: FnMut(&str, &mut Value<'_>),
{
    for entry in active(&mut section.entries) {
        visit(&dispatch_name(&entry.key_value.key), &mut entry.key_value.value);
    }
}

/// Run `visit` over every key written under `path`, however the file split that path between its
/// headers, its dotted keys and the tables it wrote as values.
///
/// TOML gives every spelling of a table the same name, so a rule reads the same keys whichever one
/// the file chose. The name each key is visited by is the rest of its path below `path`, spelled the
/// way [`dispatch_name`] spells one.
pub fn for_keys_under<F>(document: &mut Document<'_>, path: &[String], mut visit: F)
where
    F: FnMut(&str, &mut Value<'_>),
{
    for_key_values(document, path, &mut |key_value, under| {
        let named = named_by(under, &key_value.key);
        if let Some(rest) = named.strip_prefix(path)
            && !rest.is_empty()
        {
            visit(&dotted_name(rest), &mut key_value.value);
        }
    });
}

/// Run `visit` over the key of everything written under `path`, with the rest of its path below
/// `path`, so a rule can rename what a key names wherever the file wrote it.
pub fn for_names_under<F>(document: &mut Document<'_>, path: &[String], mut visit: F)
where
    F: FnMut(&[String], &mut Key<'_>),
{
    for_key_values(document, path, &mut |key_value, under| {
        let named = named_by(under, &key_value.key);
        if let Some(rest) = named.strip_prefix(path)
            && !rest.is_empty()
        {
            let rest = rest.to_vec();
            visit(&rest, &mut key_value.key);
        }
    });
}

/// Run `visit` over every key written under `path`, with the whole path it names.
///
/// A caller reading several tables at once walks the file once this way rather than once for each
/// of them.
pub fn for_key_paths_under<F>(document: &mut Document<'_>, path: &[String], mut visit: F)
where
    F: FnMut(&[String], &mut KeyValue<'_>),
{
    for_key_values(document, path, &mut |key_value, under| {
        let named = named_by(under, &key_value.key);
        if named.starts_with(path) && named.len() > path.len() {
            visit(&named, key_value);
        }
    });
}

/// Put the keys of the tables `of` names in order, reading each container the file wrote once
/// however many tables it holds.
pub fn reorder_tables_of(
    document: &mut Document<'_>,
    of: &dyn Fn(&[String]) -> Option<Vec<String>>,
    order: &[&str],
    keep_order: &[&str],
) {
    let held = Ordering {
        path: Which::Of(of),
        order,
        keep_order,
    };
    held.entries(&mut document.root, &[]);
    for section in &mut document.sections {
        let under = section.header.key.segments();
        held.entries(&mut section.entries, &under);
    }
}

/// What a key names below the table it belongs to, which the caller has already picked it for.
fn below(under: &[String], key: &Key<'_>, table: &[String]) -> Vec<String> {
    named_by(under, key)[table.len()..].to_vec()
}

/// The whole path a key names, which is the table it sits under followed by its own segments.
fn named_by(under: &[String], key: &Key<'_>) -> Vec<String> {
    under.iter().chain(&key.segments()).cloned().collect()
}

/// Run `visit` over every key-value written on the way to `path` or under it, with the table it
/// sits under, descending into the tables the file wrote as values.
fn for_key_values<F>(document: &mut Document<'_>, path: &[String], visit: &mut F)
where
    F: FnMut(&mut KeyValue<'_>, &[String]),
{
    for entry in active(&mut document.root) {
        take_key_value(&mut entry.key_value, &[], path, visit);
    }
    for section in &mut document.sections {
        let under = section.header.key.segments();
        for entry in active(&mut section.entries) {
            take_key_value(&mut entry.key_value, &under, path, visit);
        }
    }
}

fn take_key_value<F>(key_value: &mut KeyValue<'_>, under: &[String], path: &[String], visit: &mut F)
where
    F: FnMut(&mut KeyValue<'_>, &[String]),
{
    let named = named_by(under, &key_value.key);
    // a key on the way to the table says nothing about it, but what it holds may
    if !named.starts_with(path) && !path.starts_with(&named) {
        return;
    }
    visit(key_value, under);
    if let Value::InlineTable(table) = &mut key_value.value {
        for member in &mut table.members {
            take_key_value(&mut member.item, &named, path, visit);
        }
    }
}

/// Run `visit` over every run of entries that writes keys under `path`, with the table its header
/// names, so a rule that adds, splits or renames entries works on the container the file wrote.
pub fn for_entry_runs<F>(document: &mut Document<'_>, path: &[String], mut visit: F)
where
    F: FnMut(&mut Vec<Entry<'_>>, &[String]),
{
    visit(&mut document.root, &[]);
    for section in &mut document.sections {
        let under = section.header.key.segments();
        if under.starts_with(path) || path.starts_with(&under) {
            visit(&mut section.entries, &under);
        }
    }
}

/// Run `visit` over the value written at `path`, wherever the file wrote it.
pub fn for_value_at<F>(document: &mut Document<'_>, path: &[String], mut visit: F)
where
    F: FnMut(&mut Value<'_>),
{
    for_key_values(document, path, &mut |key_value, under| {
        if named_by(under, &key_value.key) == path {
            visit(&mut key_value.value);
        }
    });
}

/// Run `visit` over the table written at `path`, where the file wrote one as a value.
pub fn for_table_at<F>(document: &mut Document<'_>, path: &[String], mut visit: F)
where
    F: FnMut(&mut InlineTable<'_>),
{
    for_value_at(document, path, |value| {
        if let Value::InlineTable(table) = value {
            visit(table);
        }
    });
}

/// Put the keys of every table written inside the array at `path` in order.
///
/// The path already says which table the array holds, so what is inside one does not have to name
/// itself for its keys to be ordered.
pub fn reorder_array_tables_at(document: &mut Document<'_>, path: &[String], order: &[&str]) {
    for_value_at(document, path, |value| {
        let Value::Array(array) = value else {
            return;
        };
        for member in &mut array.members {
            if let Value::InlineTable(table) = &mut member.item {
                sort_members(&mut table.members, |item| {
                    let key = dispatch_name(&item.key);
                    (rank(&key, order), key.to_lowercase())
                });
            }
        }
    });
}

/// Put the keys written under `path` in order, inside whichever container the file wrote them in.
///
/// A key written as part of a longer path sits among keys that say something else, so it moves only
/// past the ones under `path` and leaves the rest where the file put them.
pub fn reorder_under(document: &mut Document<'_>, path: &[String], order: &[&str]) {
    reorder_under_keeping(document, path, order, &[]);
}

/// [`reorder_under`], where the keys written under a name in `keep_order` hold the order the file
/// gave them, since where each one sits among the others is part of what it says.
pub fn reorder_under_keeping(document: &mut Document<'_>, path: &[String], order: &[&str], keep_order: &[&str]) {
    let held = Ordering {
        path: Which::At(path),
        order,
        keep_order,
    };
    held.entries(&mut document.root, &[]);
    for section in &mut document.sections {
        let under = section.header.key.segments();
        held.entries(&mut section.entries, &under);
    }
}

/// Which table is being ordered: the one a caller named, or whichever one each key belongs to.
#[derive(Clone, Copy)]
enum Which<'a> {
    At(&'a [String]),
    Of(&'a dyn Fn(&[String]) -> Option<Vec<String>>),
}

impl Which<'_> {
    /// The table a key belongs to, where it belongs to one being ordered.
    fn table(self, under: &[String], key: &Key<'_>) -> Option<Vec<String>> {
        let named = named_by(under, key);
        let path = match self {
            Self::At(path) => path.to_vec(),
            Self::Of(of) => of(&named)?,
        };
        (named.len() > path.len() && named.starts_with(&path)).then_some(path)
    }

    /// Whether a container can hold a key of the table, which only a named one can rule out.
    fn reaches(self, under: &[String]) -> bool {
        match self {
            Self::At(path) => under.starts_with(path) || path.starts_with(under),
            Self::Of(_) => true,
        }
    }
}

/// What ordering a table asks of every container that holds a key of it.
#[derive(Clone, Copy)]
struct Ordering<'a> {
    path: Which<'a>,
    order: &'a [&'a str],
    keep_order: &'a [&'a str],
}

impl Ordering<'_> {
    /// Where a key sits among the ones it is sorted against.
    fn ranked(self, tail: &[String], key: &Key<'_>) -> (usize, String) {
        let name = dotted_name(tail);
        // a sort that keeps equal keys where they were is what holds a run in place, so the keys of
        // an ordered name are all given the same one
        let within = if self.keep_order.iter().any(|kept| is_named(&name, kept)) {
            String::new()
        } else {
            written_key(key).to_lowercase()
        };
        (rank(&name, self.order), within)
    }

    /// Whether this ordering speaks for the keys of the table `under` names.
    ///
    /// A table below the one being ordered has keys of its own, and the order speaks for them only
    /// where it names one: `lint.select` says where `select` sits inside `lint`, while `authors`
    /// says where the authors sit and nothing about what one holds.
    fn speaks_for(self, under: &[String]) -> bool {
        let held = match self.path {
            Which::At(path) => path.to_vec(),
            Which::Of(of) => match of(under) {
                Some(path) => path,
                None => return true,
            },
        };
        let Some(tail) = under.strip_prefix(&held[..]) else {
            return true;
        };
        if tail.is_empty() {
            return true;
        }
        let named = format!("{}.", dotted_name(tail));
        self.order.iter().any(|wanted| wanted.starts_with(&named))
    }

    /// The slots of one run that hold a key being ordered, gathered under the table each belongs to.
    fn by_table<'k>(
        self,
        under: &[String],
        slots: std::ops::Range<usize>,
        key_of: impl Fn(usize) -> &'k Key<'k>,
    ) -> Vec<(Vec<String>, Vec<usize>)> {
        let mut held: Vec<(Vec<String>, Vec<usize>)> = Vec::new();
        for at in slots {
            let Some(table) = self.path.table(under, key_of(at)) else {
                continue;
            };
            match held.iter_mut().find(|(named, _)| *named == table) {
                Some((_, found)) => found.push(at),
                None => held.push((table, vec![at])),
            }
        }
        held
    }

    /// Order the entries of one run, and the tables written inside their values.
    fn entries(self, entries: &mut Vec<Entry<'_>>, under: &[String]) {
        for entry in entries.iter_mut() {
            let named = named_by(under, &entry.key_value.key);
            self.members(&mut entry.key_value.value, &named);
        }
        if !self.speaks_for(under) {
            return;
        }
        // a marker names the group it opens, so the entries of one group sort among themselves
        for group in groups(entries) {
            // a key the file wrote as a comment is here to be ordered with its table, so it sorts
            // with the keys around it, and one table's keys never sort against another's
            let held = self.by_table(under, group.clone(), |at| &entries[at].key_value.key);
            if held.is_empty() {
                continue;
            }
            // the marker names the group, not the entry it was written above, so it stays on top
            let opens = held.iter().any(|(_, slots)| slots[0] == group.start);
            let marker = opens.then(|| take_marker(&mut entries[group.start].lead));
            for (table, slots) in &held {
                sort_slots(entries, slots, |entry| {
                    self.ranked(&below(under, &entry.key_value.key, table), &entry.key_value.key)
                });
            }
            if let Some(marker) = marker {
                entries[group.start].lead.pieces_mut().splice(0..0, marker);
            }
            // reordering breaks up whatever grouping the empty lines marked, so they go and the
            // comments that belong to an entry travel with it. A disabled key is a comment the file
            // wrote, and the lines around it are part of what that comment says
            for at in held.into_iter().flat_map(|(_, slots)| slots) {
                if !crate::disabled::is_enabled_here(&entries[at]) {
                    entries[at].lead.pieces_mut().retain(|piece| !piece.is_blank());
                }
            }
        }
    }

    /// Order the members of a table the file wrote as a value, and of the tables inside it.
    fn members(self, value: &mut Value<'_>, under: &[String]) {
        let Value::InlineTable(table) = value else {
            return;
        };
        // a table neither on the way to the one being ordered nor under it holds none of its keys,
        // and an order saying nothing about a table says nothing about the tables inside it either
        if !self.path.reaches(under) {
            return;
        }
        if !self.speaks_for(under) {
            return;
        }
        for member in &mut table.members {
            let named = named_by(under, &member.item.key);
            self.members(&mut member.item.value, &named);
        }
        for group in crate::group::member_ranges(&table.members) {
            let held = self.by_table(under, group, |at| &table.members[at].item.key);
            for (named, slots) in &held {
                sort_slots(&mut table.members, slots, |member| {
                    self.ranked(&below(under, &member.item.key, named), &member.item.key)
                });
            }
        }
    }
}

/// Sort what sits at `held` among itself, leaving everything around it where the file wrote it.
fn sort_slots<T, K: Ord>(items: &mut Vec<T>, held: &[usize], key_of: impl FnMut(&T) -> K) {
    if held.len() < 2 {
        return;
    }
    // the run is rebuilt in one pass: taking each item out of the middle and putting it back would
    // shift everything after it, once for every slot being sorted
    let total = items.len();
    let mut wanted = held.iter().peekable();
    let mut picked: Vec<T> = Vec::with_capacity(held.len());
    let mut rest: Vec<T> = Vec::with_capacity(total - held.len());
    for (at, item) in std::mem::take(items).into_iter().enumerate() {
        if wanted.next_if(|held| **held == at).is_some() {
            picked.push(item);
        } else {
            rest.push(item);
        }
    }
    picked.sort_by_cached_key(key_of);
    let mut wanted = held.iter().peekable();
    let mut picked = picked.into_iter();
    let mut rest = rest.into_iter();
    items.extend((0..total).map(|at| {
        if wanted.next_if(|held| **held == at).is_some() {
            picked.next()
        } else {
            rest.next()
        }
        .expect("every slot holds the item that was read out of it")
    }));
}

/// The entries a rule reads.
///
/// A key the file wrote as a comment says nothing to a rule reading what the file says: it is here
/// to be ordered with its table, not to be read or rewritten.
pub fn active<'e, 'a>(entries: &'e mut [Entry<'a>]) -> impl Iterator<Item = &'e mut Entry<'a>> {
    entries
        .iter_mut()
        .filter(|entry| !crate::disabled::is_enabled_here(entry))
}

/// The name a rule matches an entry by.
///
/// `a.b` is the dotted path of two plain names. A segment TOML cannot read bare is quoted, so it
/// reads as the one name the file gave it and no rule written against a bare name can match it.
/// Build a name to compare against with [`quoted_segment`], so both sides spell it the same way.
#[must_use]
pub fn dispatch_name(key: &Key<'_>) -> String {
    dotted_name(&key.segments())
}

/// The segments spelled as one dotted name, quoting each one TOML cannot read bare, so the name
/// reads back as the segments it was built from.
#[must_use]
pub fn dotted_name(segments: &[String]) -> String {
    segments
        .iter()
        .map(|segment| quoted_segment(segment))
        .collect::<Vec<String>>()
        .join(".")
}

/// One segment as it appears in a [`dispatch_name`]: bare where TOML reads it as a key, quoted
/// where it does not.
#[must_use]
pub fn quoted_segment(name: &str) -> String {
    toml_doc::encode_key(name)
}

/// Put the entries in `order`, holding anything unnamed after them in alphabetical order.
///
/// A name in `order` also claims the dotted keys beneath it, so `lint` pulls `lint.select` along.
/// Entries never cross a `# Group:` marker.
pub fn reorder_keys(entries: &mut [Entry<'_>], order: &[&str]) {
    reorder_keys_within(entries, order, &[]);
}

/// [`reorder_keys`], where the keys written under a name in `keep_order` hold the order the file
/// gave them, since where each one sits among the others is part of what it says.
pub fn reorder_keys_within(entries: &mut [Entry<'_>], order: &[&str], keep_order: &[&str]) {
    for group in groups(entries) {
        if group.is_empty() {
            continue;
        }
        let start = group.start;
        // the marker names the group, not the entry it was written above, so it stays on top of it
        let marker = take_marker(&mut entries[start].lead);
        entries[group].sort_by_cached_key(|entry| {
            // the order names keys the way a dispatch name spells them, while what falls outside it
            // sorts the way the file spells them, quotes and all
            let name = dispatch_name(&entry.key_value.key);
            // a sort that keeps equal keys where they were is what holds a run in place, so the keys
            // of an ordered name are all given the same one
            let held = keep_order.iter().any(|kept| is_named(&name, kept));
            let within = if held {
                String::new()
            } else {
                written_key(&entry.key_value.key).to_lowercase()
            };
            (rank(&name, order), within)
        });
        let lead = entries[start].lead.pieces_mut();
        lead.splice(0..0, marker);
    }
    // reordering breaks up whatever grouping the empty lines marked, so they go and the comments
    // that belong to an entry travel with it. A disabled key is a comment the file wrote, and the
    // lines around it are part of what that comment says
    for entry in entries.iter_mut() {
        if crate::disabled::is_enabled_here(entry) {
            continue;
        }
        entry.lead.pieces_mut().retain(|piece| !piece.is_blank());
    }
}

/// Take the lines up to and including the group marker off the trivia.
fn take_marker<'a>(lead: &mut Trivia<'a>) -> Vec<Piece<'a>> {
    let pieces = lead.pieces_mut();
    let last = pieces.iter().rposition(|piece| match piece {
        Piece::Comment { text, .. } => is_group_marker(text),
        Piece::Blank { .. } => false,
    });
    last.map_or_else(Vec::new, |index| pieces.drain(..=index).collect())
}

/// Rewrite the entry keys that appear in `aliases`, leaving the rest alone, and hand back the
/// renames it made so a caller can follow what named them.
///
/// A file that already spells a key the canonical way keeps both as written: renaming the older
/// spelling on top of it would say the same key twice, which no TOML document can.
pub fn rename_keys(entries: &mut [Entry<'_>], aliases: &[(&str, &str)]) -> Vec<(String, String)> {
    // a key the file wrote as a comment reserves no name: the comment comes back over it, and what
    // is left is the keys the file wrote
    let mut taken: Vec<String> = entries
        .iter()
        .filter(|entry| !crate::disabled::is_enabled_here(entry))
        .map(|entry| dispatch_name(&entry.key_value.key))
        .collect();
    let mut renamed = Vec::new();
    for entry in active(entries) {
        let key = dispatch_name(&entry.key_value.key);
        let Some((_, replacement)) = aliases.iter().find(|(from, _)| *from == key) else {
            continue;
        };
        if taken.iter().any(|held| held == replacement) {
            continue;
        }
        entry.key_value.key = Key::new(replacement.split('.'));
        taken.push((*replacement).to_owned());
        renamed.push((key, (*replacement).to_owned()));
    }
    renamed
}

/// Rewrite the names written under `path` that `aliases` moves, wherever the file wrote them, and
/// hand back the renames it made so a caller can follow what named them.
///
/// A file that already spells a key the canonical way keeps both as written: renaming the older
/// spelling on top of it would say the same key twice, which no TOML document can.
pub fn rename_under(document: &mut Document<'_>, path: &[String], aliases: &[(&str, &str)]) -> Vec<(String, String)> {
    let mut taken: Vec<String> = Vec::new();
    for_names_under(document, path, |tail, _| taken.push(dotted_name(tail)));
    let mut renamed = Vec::new();
    for_names_under(document, path, |tail, key| {
        // an alias names one key of the table, so a name written below one is not the key it moves
        let [name] = tail else {
            return;
        };
        let Some((_, to)) = aliases.iter().find(|(from, _)| from == name) else {
            return;
        };
        if taken.iter().any(|held| held == to) {
            return;
        }
        key.parts_mut()
            .last_mut()
            .expect("a key names at least one segment")
            .set_name(to);
        taken.push((*to).to_owned());
        renamed.push((name.clone(), (*to).to_owned()));
    });
    renamed
}

/// [`rename_under`], for every table `of` names at once, reading the file once however many tables
/// it holds.
pub fn rename_tables_of(
    document: &mut Document<'_>,
    of: &dyn Fn(&[String]) -> Option<Vec<String>>,
    aliases: &[(&str, &str)],
) -> Vec<(Vec<String>, String, String)> {
    let mut taken: Vec<Vec<String>> = Vec::new();
    for_key_paths_under(document, &[], |named, _| taken.push(named.to_vec()));
    let mut renamed = Vec::new();
    for_key_paths_under(document, &[], |named, key_value| {
        let Some(table) = of(named) else {
            return;
        };
        // an alias names one key of the table, so a name written below one is not the key it moves
        let [name] = &named[table.len()..] else {
            return;
        };
        let Some((_, to)) = aliases.iter().find(|(from, _)| from == name) else {
            return;
        };
        let written: Vec<String> = table.iter().cloned().chain([(*to).to_owned()]).collect();
        if taken.contains(&written) {
            return;
        }
        key_value
            .key
            .parts_mut()
            .last_mut()
            .expect("a key names at least one segment")
            .set_name(to);
        renamed.push((table, name.clone(), (*to).to_owned()));
        taken.push(written);
    });
    renamed
}

/// Sort the names each value of the table at `name` holds, and put its keys in order.
///
/// A table written out as a header of its own is a run of names paired with lists of names, and
/// nothing ranks one name above another, so both sides sort.
pub fn sort_names_under(document: &mut Document<'_>, name: &str) {
    let Some(section) = first(document, name) else {
        return;
    };
    for_entries(section, |_key, value| crate::arrays::sort_names_in(value));
    reorder_keys(&mut section.entries, &[]);
}

/// The value written for `key`, if the section holds it.
pub fn find<'e, 'a>(entries: &'e mut [Entry<'a>], key: &str) -> Option<&'e mut Value<'a>> {
    entries
        .iter_mut()
        .find(|entry| dispatch_name(&entry.key_value.key) == key)
        .map(|entry| &mut entry.key_value.value)
}

/// A key as it was written, quotes included, which is what decides where it sorts: `"Source Code"`
/// leads `Changelog` because the quote does.
fn written_key(key: &Key<'_>) -> String {
    key.parts()
        .iter()
        .map(KeyPart::written)
        .collect::<Vec<&str>>()
        .join(".")
}

/// Where `wanted` ends inside the path, when the path names it or something written under it, so a
/// name below something else is still the name it is.
fn run_end(path: &str, wanted: &str) -> Option<usize> {
    if path == wanted || path.starts_with(&format!("{wanted}.")) {
        return Some(wanted.len());
    }
    let held = format!(".{wanted}");
    let at = path.find(&held)?;
    let end = at + held.len();
    (path.len() == end || path[end..].starts_with('.')).then_some(end)
}

/// Whether the name is `wanted` or a key written under it.
fn is_named(name: &str, wanted: &str) -> bool {
    name == wanted || name.strip_prefix(wanted).is_some_and(|rest| rest.starts_with('.'))
}

/// Where a key sits in `order`. A key the order does not name sorts after every one it does.
fn rank(key: &str, order: &[&str]) -> usize {
    order.iter().position(|name| is_named(key, name)).unwrap_or(order.len())
}

/// The ranges the `# Group:` markers split the entries into.
fn groups(entries: &[Entry<'_>]) -> Vec<std::ops::Range<usize>> {
    let mut starts = vec![0];
    for (index, entry) in entries.iter().enumerate().skip(1) {
        let marked = entry.lead.pieces().iter().any(|piece| match piece {
            Piece::Comment { text, .. } => is_group_marker(text),
            Piece::Blank { .. } => false,
        });
        if marked {
            starts.push(index);
        }
    }
    starts.push(entries.len());
    starts.windows(2).map(|pair| pair[0]..pair[1]).collect()
}

/// Put the members of a table written as a value in order, one authored group at a time.
pub fn sort_members<T, K: Ord>(members: &mut Vec<Member<'_, T>>, mut key_of: impl FnMut(&T) -> K) {
    for group in crate::group::member_ranges(members) {
        let held: Vec<usize> = group.collect();
        sort_slots(members, &held, |member| key_of(&member.item));
    }
}

/// An inline table is recognized by a key only it carries, which then fixes the order of the rest.
#[derive(Debug, Clone, Copy)]
pub struct InlineSchema<'a> {
    pub discriminator: &'a str,
    pub key_order: &'a [&'a str],
}

/// Order the keys of every inline table a schema recognizes, among the values written under `path`.
///
/// A discriminator names a key one tool writes, not one no other tool may: the table a rule belongs
/// to is what says the rule is about it. An empty path is the whole document, which is what a file
/// written for one tool alone is.
pub fn reorder_inline_tables(document: &mut Document<'_>, path: &[String], schemas: &[InlineSchema<'_>]) {
    for entry in active(&mut document.root) {
        if entry.key_value.key.segments().starts_with(path) {
            order_within(&mut entry.key_value.value, schemas);
        }
    }
    for section in &mut document.sections {
        let header = section.header.key.segments();
        for entry in active(&mut section.entries) {
            let mut named = header.clone();
            named.extend(entry.key_value.key.segments());
            if named.starts_with(path) {
                order_within(&mut entry.key_value.value, schemas);
            }
        }
    }
}

fn order_within(value: &mut Value<'_>, schemas: &[InlineSchema<'_>]) {
    match value {
        Value::Scalar(_) => {}
        Value::Array(array) => {
            for member in &mut array.members {
                order_within(&mut member.item, schemas);
            }
        }
        Value::InlineTable(table) => {
            for member in &mut table.members {
                order_within(&mut member.item.value, schemas);
            }
            let Some(schema) = schemas.iter().find(|schema| {
                table
                    .members
                    .iter()
                    .any(|member| member.item.key.is_path(schema.discriminator))
            }) else {
                return;
            };
            sort_members(&mut table.members, |item| {
                let key = dispatch_name(&item.key);
                (rank(&key, schema.key_order), key.to_lowercase())
            });
        }
    }
}

/// Apply one rule to every element of an array of tables, however the file writes it: as the
/// `[[path]]` headers it was written with, or as the inline tables the short format folds them
/// into. `visit` sees every key of every element and the value it holds, and each element's keys
/// end up in `key_order`.
pub fn for_array_elements(
    document: &mut Document<'_>,
    path: &[String],
    key_order: &[&str],
    visit: &mut dyn FnMut(&str, &mut Value<'_>),
) {
    for section in &mut document.sections {
        if section.header.kind == SectionKind::ArrayOfTables && section.header.key.segments() == path {
            for entry in active(&mut section.entries) {
                let name = dispatch_name(&entry.key_value.key);
                visit(&name, &mut entry.key_value.value);
            }
            reorder_keys(&mut section.entries, key_order);
        }
    }
    for_value_at(document, path, |value| {
        let Value::Array(array) = value else {
            return;
        };
        for member in &mut array.members {
            let Value::InlineTable(table) = &mut member.item else {
                continue;
            };
            for held in &mut table.members {
                let name = dispatch_name(&held.item.key);
                visit(&name, &mut held.item.value);
            }
            sort_members(&mut table.members, |item| {
                let name = dispatch_name(&item.key);
                (rank(&name, key_order), written_key(&item.key).to_lowercase())
            });
        }
    });
}

/// Every value the document holds, in the order it was written.
pub fn every_value<'d, 'a>(document: &'d mut Document<'a>) -> Vec<&'d mut Value<'a>> {
    let root = document.root.iter_mut().map(|entry| &mut entry.key_value.value);
    let held = document
        .sections
        .iter_mut()
        .flat_map(|section| section.entries.iter_mut().map(|entry| &mut entry.key_value.value));
    root.chain(held).collect()
}

/// Put the sections in `order`, holding the ones it does not name after them alphabetically.
///
/// A name in `order` claims the tables written beneath it, so `tool.ruff` keeps `tool.ruff.lint`
/// with it, and sections that share a name hold the order they were written in. `key_order` places
/// the sub-tables of a table by that table's own key order, so `[tool.coverage.run]` sits where
/// `run` would sit among the keys of `[tool.coverage]`.
pub fn reorder_within(
    document: &mut Document<'_>,
    order: &[&str],
    nested_prefixes: &[&str],
    key_order: &dyn Fn(&[String]) -> Option<Vec<String>>,
) {
    reorder_within_keeping(document, order, nested_prefixes, key_order, &|_| Vec::new());
}

/// [`reorder_within`], where a table written under one of the names `keep_order` gives for its base
/// holds the place the file gave it among the tables beside it.
pub fn reorder_within_keeping(
    document: &mut Document<'_>,
    order: &[&str],
    nested_prefixes: &[&str],
    key_order: &dyn Fn(&[String]) -> Option<Vec<String>>,
    keep_order: &dyn Fn(&[String]) -> Vec<String>,
) {
    let blocks = blocks(document);
    // a table the order does not name keeps the place its group was first written in, so a file
    // using tools this formatter has no policy for is left as its author arranged it
    let mut seen: HashMap<Vec<String>, usize> = HashMap::new();
    for block in &blocks {
        let base = base_segments(&block[0].header.key.segments(), nested_prefixes);
        let next = seen.len();
        seen.entry(base).or_insert(next);
    }
    // where the order names each table, read once rather than looked up again for every table
    let mut placed_at: HashMap<&str, usize> = HashMap::new();
    for (at, name) in order.iter().enumerate() {
        placed_at.entry(*name).or_insert(at);
    }
    let mut sorted: Vec<Vec<Section<'_>>> = Vec::with_capacity(blocks.len());
    for mut partition in partitions(blocks) {
        // the marker names the group, not the table written under it, so it stays on top of it
        let marker = take_marker(&mut partition[0][0].header.lead);
        partition.sort_by_cached_key(|block| {
            let segments = block[0].header.key.segments();
            let head = base_segments(&segments, nested_prefixes);
            let base = dotted_name(&head);
            let leaf = dotted_name(&segments[head.len()..]);
            let within = key_order(&head).map_or(0, |names| {
                let refs: Vec<&str> = names.iter().map(String::as_str).collect();
                rank(&leaf, &refs)
            });
            // a table is placed by the name it was given, not by a shorter name it happens to start
            // with: `env_base.test` is its own table rather than part of `env_base`. One the order
            // does not name still belongs among its own kind, so an unknown tool stays with the
            // tools rather than falling in among the tables that are nobody's tool.
            let placed = placed_at.get(base.as_str()).copied().unwrap_or_else(|| {
                // the namespace is the first segment of the name, so a table merely spelled like one,
                // as `toolbox` is like `tool`, is its own table
                let nested = nested_prefixes.iter().any(|prefix| head[0] == *prefix);
                order.len() + usize::from(!nested)
            });
            let group = seen.get(&head).copied().unwrap_or(seen.len());
            // a sort that keeps equal keys where they were is what holds a run of tables in place,
            // so the ones written under an ordered name are read only as far as that name
            let last = keep_order(&head)
                .iter()
                .find_map(|kept| run_end(&leaf, kept))
                .map_or_else(|| leaf.to_lowercase(), |end| leaf[..end].to_lowercase());
            // the table itself leads the tables written beneath it
            (placed, group, usize::from(!leaf.is_empty()), within, last)
        });
        // the marker opens the group, so nothing the file left above the table it now leads stays
        let lead = partition[0][0].header.lead.pieces_mut();
        let kept = lead.iter().position(|piece| !piece.is_blank()).unwrap_or(lead.len());
        lead.drain(..kept);
        lead.splice(0..0, marker);
        sorted.extend(partition);
    }
    document.sections = sorted.into_iter().flatten().collect();
    // reordering breaks whatever spacing the file had, so the tables land one line apart; the
    // spacing pass then widens or closes the gaps it cares about
    let opens_next: Vec<bool> = document
        .sections
        .windows(2)
        .map(|pair| pair[0].entries.is_empty() && pair[0].header.key.segments() != pair[1].header.key.segments())
        .collect();
    for (index, section) in document.sections.iter_mut().enumerate().skip(1) {
        let pieces = section.header.lead.pieces_mut();
        pieces.retain(|piece| !piece.is_blank());
        // a header with nothing under it reads as the opening of what follows, so it keeps it close
        if opens_next[index - 1] {
            continue;
        }
        pieces.insert(
            0,
            Piece::Blank {
                indent: "".into(),
                ending: LineEnding::Lf,
            },
        );
    }
}

/// The runs of sections that have to move together.
///
/// A `[[name]]` header opens one element of an array, and a `[name.child]` written anywhere below
/// it belongs to that element rather than to the name. The two are not required to sit next to
/// each other, so ownership is tracked per array path and survives unrelated tables in between.
/// Every other header names its table in full, so it moves on its own.
fn blocks<'a>(document: &mut Document<'a>) -> Vec<Vec<Section<'a>>> {
    let mut blocks: Vec<Vec<Section<'a>>> = Vec::new();
    let mut owners: Vec<(Vec<String>, usize)> = Vec::new();
    for section in std::mem::take(&mut document.sections) {
        let segments = section.header.key.segments();
        // the innermost element the header falls under owns it; a longer path is the closer one
        let owner = owners
            .iter()
            .filter(|(path, _)| segments.len() > path.len() && segments.starts_with(path))
            .max_by_key(|(path, _)| path.len())
            .map(|(_, block)| *block);
        let at = match owner {
            Some(block) => {
                blocks[block].push(section);
                block
            }
            None => {
                blocks.push(vec![section]);
                blocks.len() - 1
            }
        };
        if blocks[at].last().expect("just pushed").header.kind == SectionKind::ArrayOfTables {
            // a new element starts its own scope, so what the previous one owned is out of reach
            owners.retain(|(path, _)| !(path.len() >= segments.len() && path.starts_with(&segments)));
            owners.push((segments, at));
        }
    }
    blocks
}

/// The runs of blocks a `# Group:` marker splits the document into, which sorting must not cross.
fn partitions<'a>(blocks: Vec<Vec<Section<'a>>>) -> Vec<Vec<Vec<Section<'a>>>> {
    let mut partitions: Vec<Vec<Vec<Section<'a>>>> = Vec::new();
    for block in blocks {
        let marked = block[0].header.lead.pieces().iter().any(|piece| match piece {
            Piece::Comment { text, .. } => is_group_marker(text),
            Piece::Blank { .. } => false,
        });
        match partitions.last_mut() {
            Some(open) if !marked => open.push(block),
            _ => partitions.push(vec![block]),
        }
    }
    partitions
}

/// Run `visit` over the entries of every table called `name`, the empty name included: a file is
/// free to write `[""]`. The keys written before the first header are [`with_root_entries`].
pub fn with_entries<F>(document: &mut Document<'_>, name: &str, mut visit: F)
where
    F: FnMut(&mut Vec<Entry<'_>>),
{
    for section in &mut document.sections {
        if section.header.key.is_path(name) {
            visit(&mut section.entries);
        }
    }
}

/// [`named`] for a name read out of the document rather than written by hand.
pub fn named_of<'d, 'a>(document: &'d mut Document<'a>, name: &[String]) -> Vec<&'d mut Section<'a>> {
    document
        .sections
        .iter_mut()
        .filter(|section| section.header.key.segments() == name)
        .collect()
}

/// [`first`] for a name read out of the document rather than written by hand.
pub fn first_of<'d, 'a>(document: &'d mut Document<'a>, name: &[String]) -> Option<&'d mut Section<'a>> {
    document
        .sections
        .iter_mut()
        .find(|section| section.header.key.segments() == name)
}

/// A configured table name read as the segments it names, so `a."b.c"` is two and `a.b.c` three.
///
/// A name TOML cannot read as a key falls back to splitting on its dots, which is what a setting
/// written without quotes means.
#[must_use]
pub fn parse_name(name: &str) -> Vec<String> {
    read_name(name).expect("the name is one this repository writes")
}

/// The segments the text names, read the way TOML reads a key.
///
/// # Errors
///
/// Returns why the text is not a key path, so a caller reading one from a user can say so.
pub fn read_name(name: &str) -> Result<Vec<String>, String> {
    let read = format!("{name} = 0\n");
    let document = toml_doc::parse(&read).map_err(|errors| errors[0].to_string())?;
    // the name has to be the whole of what was read: text carrying its own key, comment or table
    // would make the value below it read as part of something else
    let [entry] = document.root.as_slice() else {
        return Err(String::from("it names more than one key"));
    };
    if !document.sections.is_empty()
        || !entry.lead.pieces().is_empty()
        || !document.trailing.pieces().is_empty()
        || entry.trail.comment.is_some()
    {
        return Err(String::from("it says more than a name"));
    }
    Ok(entry.key_value.key.segments())
}

/// The names written one level below `prefix`, each the one segment the file gave it.
#[must_use]
pub fn headers_below(document: &Document<'_>, prefix: &[&str]) -> Vec<String> {
    let mut names: Vec<String> = document
        .sections
        .iter()
        .map(|section| section.header.key.segments())
        .filter(|segments| {
            segments.len() > prefix.len() && segments.iter().zip(prefix).all(|(held, want)| held == want)
        })
        .map(|segments| segments[prefix.len()].clone())
        .collect();
    names.sort();
    names.dedup();
    names
}

/// [`headers_below`] for the dotted keys of one table, which say the same thing folded up.
#[must_use]
pub fn keys_below(entries: &[Entry<'_>], prefix: &[&str]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for entry in entries {
        let segments = entry.key_value.key.segments();
        if segments.len() <= prefix.len() || !segments.iter().zip(prefix).all(|(held, want)| held == want) {
            continue;
        }
        let name = segments[prefix.len()].clone();
        if !names.contains(&name) {
            names.push(name);
        }
    }
    names
}

/// Run `visit` over the keys written before the first header, which is where a file that names no
/// table writes what it has to say.
pub fn with_root_entries<F>(document: &mut Document<'_>, mut visit: F)
where
    F: FnMut(&mut Vec<Entry<'_>>),
{
    visit(&mut document.root);
}

/// [`with_entries`] for a name read out of the document rather than written by hand, where a
/// segment may hold anything at all, a dot included.
pub fn with_entries_of<F>(document: &mut Document<'_>, name: &[String], mut visit: F)
where
    F: FnMut(&mut Vec<Entry<'_>>),
{
    for section in &mut document.sections {
        if section.header.key.segments() == name {
            visit(&mut section.entries);
        }
    }
}

/// The names of every table in the document, in the order they were written.
#[must_use]
pub fn names(document: &Document<'_>) -> Vec<String> {
    document
        .sections
        .iter()
        .map(|section| dispatch_name(&section.header.key))
        .collect()
}
