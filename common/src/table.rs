use std::cell::{RefCell, RefMut};
use std::collections::{HashMap, HashSet};
use std::iter::zip;
use std::ops::Index;

use tombi_syntax::SyntaxKind::{
    ARRAY_OF_TABLE, BARE_KEY, BASIC_STRING, BRACE_START, BRACKET_END, BRACKET_START, COMMA, COMMENT,
    DANGLING_COMMENT_GROUP, DOUBLE_BRACKET_START, EQUAL, INLINE_TABLE, KEY_VALUE, KEY_VALUE_GROUP,
    KEY_VALUE_WITH_COMMA_GROUP, KEYS, LINE_BREAK, LITERAL_STRING, TABLE, WHITESPACE,
};
use tombi_syntax::{SyntaxElement, SyntaxKind, SyntaxNode};

fn is_value_kind(kind: SyntaxKind) -> bool {
    !matches!(kind, KEYS | EQUAL | WHITESPACE | LINE_BREAK | COMMENT)
}

fn get_key_text(element: &SyntaxElement) -> String {
    element
        .as_node()
        .unwrap()
        .children_with_tokens()
        .find(|c| c.kind() == KEYS)
        .map(|c| c.as_node().unwrap().text().to_string().trim().to_string())
        .unwrap_or_default()
}

fn get_value_text(element: &SyntaxElement) -> String {
    element
        .as_node()
        .unwrap()
        .children_with_tokens()
        .find(|c| is_value_kind(c.kind()))
        .map(|c| c.as_node().unwrap().text().to_string())
        .unwrap_or_default()
}

use crate::create::{make_empty_inline_table, make_empty_newline, make_key, make_newline, make_table_entry};

fn ensure_table_exists(tables: &mut Tables, name: &str) {
    if !tables.header_to_pos.contains_key(name) {
        tables
            .header_to_pos
            .insert(String::from(name), vec![tables.table_set.len()]);
        tables.table_set.push(RefCell::new(make_table_entry(name)));
    }
}

fn has_live_descendant_table(tables: &Tables, full_name: &str) -> bool {
    let prefix = format!("{full_name}.");
    tables.header_to_pos.iter().any(|(name, positions)| {
        name.starts_with(&prefix) && positions.iter().any(|&pos| !tables.table_set[pos].borrow().is_empty())
    })
}

fn filter_entries(table: &mut RefMut<Vec<SyntaxElement>>, entries_to_remove: &HashSet<usize>) {
    let mut new_elements = Vec::new();
    let mut entry_index = 0;

    for element in table.iter() {
        if element.kind() == KEY_VALUE {
            if !entries_to_remove.contains(&entry_index) {
                new_elements.push(element.clone());
            }
            entry_index += 1;
        } else {
            new_elements.push(element.clone());
        }
    }

    while new_elements.last().is_some_and(|e| e.kind() == LINE_BREAK) {
        new_elements.pop();
    }
    new_elements.push(make_newline());

    let table_len = table.len();
    table.splice(0..table_len, new_elements);
}
use crate::string::load_text;
use crate::util::is_group_marker;

fn split_leading_group_marker(kv: &SyntaxElement) -> Option<Vec<SyntaxElement>> {
    let node = kv.as_node()?;
    let children: Vec<SyntaxElement> = node.children_with_tokens().collect();
    let after = group_marker_prefix_len(&children)?;
    let header = children[..after].to_vec();
    let mut remaining = vec![make_newline()];
    remaining.extend_from_slice(&children[after..]);
    node.splice_children(0..children.len(), remaining);
    Some(header)
}

fn emit_key(to_insert: &mut Vec<SyntaxElement>, set: &[SyntaxElement]) {
    let first_has_leading = set.first().is_some_and(has_leading_newline);
    if first_has_leading {
        while to_insert.last().is_some_and(|e| e.kind() == LINE_BREAK) {
            to_insert.pop();
        }
    } else if !to_insert.is_empty() && to_insert.last().map(|e| e.kind()) != Some(LINE_BREAK) {
        to_insert.push(make_newline());
    }
    to_insert.extend(set.iter().cloned());
}

fn parse(source: &str) -> SyntaxNode {
    tombi_parser::parse(source).syntax_node().clone_for_update()
}

fn flatten_key_value_group(group: &SyntaxNode, entry_set: &RefCell<Vec<SyntaxElement>>) {
    for child in group.children_with_tokens() {
        entry_set.borrow_mut().push(child);
    }
}

fn collapse_consecutive_line_breaks(entries: &mut Vec<SyntaxElement>) {
    let mut collapsed = Vec::new();
    let mut prev_was_newline = false;
    for element in entries.iter() {
        if element.kind() == LINE_BREAK {
            if !prev_was_newline {
                collapsed.push(element.clone());
            }
            prev_was_newline = true;
        } else {
            prev_was_newline = false;
            collapsed.push(element.clone());
        }
    }
    entries.splice(0..entries.len(), collapsed);
}

fn has_leading_newline(element: &SyntaxElement) -> bool {
    element
        .as_node()
        .is_some_and(|n| n.children_with_tokens().next().is_some_and(|c| c.kind() == LINE_BREAK))
}

fn find_key_value_in_parsed(root: &SyntaxNode) -> Option<SyntaxElement> {
    for c in root.children_with_tokens() {
        if c.kind() == KEY_VALUE {
            return Some(c);
        }
        if c.kind() == KEY_VALUE_GROUP {
            for kv in c.as_node().unwrap().children_with_tokens() {
                if kv.kind() == KEY_VALUE {
                    return Some(kv);
                }
            }
        }
    }
    None
}

#[derive(Debug)]
pub struct Tables {
    pub header_to_pos: HashMap<String, Vec<usize>>,
    pub table_set: Vec<RefCell<Vec<SyntaxElement>>>,
}

impl Tables {
    pub fn get(&self, key: &str) -> Option<Vec<&RefCell<Vec<SyntaxElement>>>> {
        let positions = self.header_to_pos.get(key)?;
        let res: Vec<&RefCell<Vec<SyntaxElement>>> = positions
            .iter()
            .map(|pos| &self.table_set[*pos])
            .filter(|cell| !cell.borrow().is_empty())
            .collect();
        if res.is_empty() { None } else { Some(res) }
    }

    pub fn from_ast(root_ast: &SyntaxNode) -> Self {
        let mut header_to_pos = HashMap::<String, Vec<usize>>::new();
        let mut table_set = Vec::<RefCell<Vec<SyntaxElement>>>::new();
        let entry_set = RefCell::new(Vec::<SyntaxElement>::new());
        let mut table_kind = TABLE;
        let mut add_to_table_set = |kind, table_name: &str| {
            let mut entry_set_borrow = entry_set.borrow_mut();
            if !entry_set_borrow.is_empty() {
                let indexes = header_to_pos.entry(String::from(table_name)).or_default();
                if kind == ARRAY_OF_TABLE || (kind == TABLE && indexes.is_empty()) {
                    indexes.push(table_set.len());
                    table_set.push(RefCell::new(entry_set_borrow.clone()));
                } else if kind == TABLE && !indexes.is_empty() {
                    let pos = indexes.first().unwrap();
                    let mut res = table_set.index(*pos).borrow_mut();
                    let mut new = entry_set_borrow.clone();
                    if let Some(last_non_trailing_newline_index) = new.iter().rposition(|x| x.kind() != LINE_BREAK) {
                        new.truncate(last_non_trailing_newline_index + 1);
                    }
                    if res.last().unwrap().kind() != LINE_BREAK {
                        res.push(make_newline());
                    }
                    res.extend(new.into_iter().skip_while(|x| [LINE_BREAK, TABLE].contains(&x.kind())));
                }
                entry_set_borrow.clear();
            }
        };
        let mut current_table_name = String::new();
        for c in root_ast.children_with_tokens() {
            if [ARRAY_OF_TABLE, TABLE].contains(&c.kind()) {
                let mut borrow = entry_set.borrow_mut();

                let last_entry_pos = borrow.iter().rposition(|x| x.kind() == KEY_VALUE);

                let comments_start = match last_entry_pos {
                    Some(entry_pos) => borrow
                        .iter()
                        .skip(entry_pos + 1)
                        .position(|x| x.kind() == COMMENT)
                        .map_or(borrow.len(), |p| entry_pos + 1 + p),
                    None => borrow.iter().position(|x| x.kind() == COMMENT).unwrap_or(borrow.len()),
                };

                let comments_for_new_table: Vec<SyntaxElement> = borrow.drain(comments_start..).collect();

                // Trailing LINE_BREAKs are inter-table spacing; keep them out of table content.
                while let Some(last) = borrow.last() {
                    if last.kind() == LINE_BREAK {
                        borrow.pop();
                    } else {
                        break;
                    }
                }

                collapse_consecutive_line_breaks(&mut borrow);

                drop(borrow);

                add_to_table_set(table_kind, &current_table_name);
                table_kind = c.kind();
                current_table_name = get_table_name(&c);

                entry_set.borrow_mut().extend(comments_for_new_table);

                if let Some(table_node) = c.as_node() {
                    for child in table_node.children_with_tokens() {
                        if child.kind() == KEY_VALUE_GROUP || child.kind() == DANGLING_COMMENT_GROUP {
                            flatten_key_value_group(child.as_node().unwrap(), &entry_set);
                        } else {
                            entry_set.borrow_mut().push(child);
                        }
                    }
                }
            } else if c.kind() == KEY_VALUE_GROUP || c.kind() == DANGLING_COMMENT_GROUP {
                flatten_key_value_group(c.as_node().unwrap(), &entry_set);
            } else {
                entry_set.borrow_mut().push(c);
            }
        }
        collapse_consecutive_line_breaks(&mut entry_set.borrow_mut());
        add_to_table_set(table_kind, &current_table_name);
        Self {
            header_to_pos,
            table_set,
        }
    }

    pub fn reorder(
        &self,
        root_ast: &SyntaxNode,
        order: &[&str],
        multi_level_prefixes: &[&str],
        root_table_spacing: &str,
        sub_table_spacing: &str,
    ) {
        let root_breaks = root_table_spacing.chars().filter(|&c| c == '\n').count() + 1;
        let sub_breaks = sub_table_spacing.chars().filter(|&c| c == '\n').count() + 1;
        let mut to_insert = Vec::<SyntaxElement>::new();
        let order = calculate_order(&self.header_to_pos, &self.table_set, order, multi_level_prefixes);

        let pos_group = compute_pos_groups(&self.table_set);
        let mut group_marker: HashMap<usize, Vec<SyntaxElement>> = HashMap::new();
        for (pos, cell) in self.table_set.iter().enumerate() {
            if let Some(header) = take_leading_group_marker(&mut cell.borrow_mut()) {
                group_marker.insert(pos_group[pos], header);
            }
        }
        let group_of_name: HashMap<&String, usize> = self
            .header_to_pos
            .iter()
            .map(|(name, positions)| (name, pos_group[*positions.iter().min().unwrap()]))
            .collect();
        let mut emitted_groups = HashSet::<usize>::new();

        let mut next = order.clone();
        if !next.is_empty() {
            next.remove(0);
        }
        next.push(String::new());
        for (name, next_name) in zip(order.iter(), next.iter()) {
            let group = group_of_name[name];
            if emitted_groups.insert(group)
                && let Some(header) = group_marker.get(&group)
            {
                to_insert.extend(header.iter().cloned());
            }
            let entries_list = self.get(name).unwrap();
            let num_entries = entries_list.len();

            for (entry_idx, entries) in entries_list.iter().enumerate() {
                let got = entries.borrow_mut();
                if !got.is_empty() {
                    let mut add = got.clone();

                    let is_last_entry_of_this_table = entry_idx == num_entries - 1;

                    if is_last_entry_of_this_table {
                        if !next_name.is_empty() {
                            let breaks =
                                if get_key(name, multi_level_prefixes) != get_key(next_name, multi_level_prefixes) {
                                    root_breaks
                                } else {
                                    sub_breaks
                                };
                            while !add.is_empty() && add.last().unwrap().kind() == LINE_BREAK {
                                add.pop();
                            }
                            for _ in 0..breaks {
                                add.push(make_newline());
                            }
                        }
                    } else {
                        add.extend(make_empty_newline());
                    }

                    to_insert.extend(add);
                }
            }
        }

        root_ast.splice_children(0..root_ast.children_with_tokens().count(), to_insert);

        // from_ast flattened TABLE nodes into bare children, and splice_children put them back without TABLE wrappers.
        // Re-parse to rebuild the wrappers and parent chain that later traversal needs.
        let reparsed = parse(&root_ast.to_string());
        let new_children: Vec<SyntaxElement> = reparsed.children_with_tokens().collect();
        root_ast.splice_children(0..root_ast.children_with_tokens().count(), new_children);
    }
}

fn calculate_order(
    header_to_pos: &HashMap<String, Vec<usize>>,
    table_set: &[RefCell<Vec<SyntaxElement>>],
    ordering: &[&str],
    multi_level_prefixes: &[&str],
) -> Vec<String> {
    let key_to_pos = ordering
        .iter()
        .enumerate()
        .map(|(k, v)| (v, k * 2))
        .collect::<HashMap<&&str, usize>>();

    let pos_group = compute_pos_groups(table_set);

    let mut header_pos: Vec<(String, usize)> = header_to_pos
        .clone()
        .into_iter()
        .filter(|(_k, v)| v.iter().any(|p| !table_set.get(*p).unwrap().borrow().is_empty()))
        .map(|(k, v)| (k, *v.iter().min().unwrap()))
        .collect();

    let mut base_key_first_pos: HashMap<String, usize> = HashMap::new();
    for (k, file_pos) in &header_pos {
        let base = get_key(k, multi_level_prefixes);
        base_key_first_pos
            .entry(base)
            .and_modify(|p| *p = (*p).min(*file_pos))
            .or_insert(*file_pos);
    }

    header_pos.sort_by(|(k1, fp1), (k2, fp2)| {
        pos_group[*fp1].cmp(&pos_group[*fp2]).then_with(|| {
            let key1 = get_key(k1, multi_level_prefixes);
            let key2 = get_key(k2, multi_level_prefixes);
            let pos1 = key_to_pos.get(&key1.as_str());
            let pos2 = key_to_pos.get(&key2.as_str());

            match (pos1, pos2) {
                (Some(&p1), Some(&p2)) => {
                    let offset1 = usize::from(key1 != *k1);
                    let offset2 = usize::from(key2 != *k2);
                    (p1 + offset1)
                        .cmp(&(p2 + offset2))
                        .then_with(|| k1.to_lowercase().cmp(&k2.to_lowercase()))
                }
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => {
                    let base_pos1 = base_key_first_pos.get(&key1).unwrap_or(&usize::MAX);
                    let base_pos2 = base_key_first_pos.get(&key2).unwrap_or(&usize::MAX);
                    base_pos1
                        .cmp(base_pos2)
                        .then_with(|| k1.to_lowercase().cmp(&k2.to_lowercase()))
                }
            }
        })
    });
    header_pos.into_iter().map(|(k, _)| k).collect()
}

fn compute_pos_groups(table_set: &[RefCell<Vec<SyntaxElement>>]) -> Vec<usize> {
    let mut pos_group = vec![0_usize; table_set.len()];
    let mut group = 0;
    for (pos, cell) in table_set.iter().enumerate() {
        if group_marker_prefix_len(&cell.borrow()).is_some() {
            group += 1;
        }
        pos_group[pos] = group;
    }
    pos_group
}

fn group_marker_prefix_len(elements: &[SyntaxElement]) -> Option<usize> {
    let marker_at = elements
        .iter()
        .position(|e| !matches!(e.kind(), LINE_BREAK | WHITESPACE))?;
    if elements[marker_at].kind() != COMMENT || !is_group_marker(&elements[marker_at].to_string()) {
        return None;
    }
    let trailing_break = usize::from(elements.get(marker_at + 1).is_some_and(|e| e.kind() == LINE_BREAK));
    Some(marker_at + 1 + trailing_break)
}

fn take_leading_group_marker(content: &mut Vec<SyntaxElement>) -> Option<Vec<SyntaxElement>> {
    let after = group_marker_prefix_len(content)?;
    Some(content.drain(..after).collect())
}

fn get_key(k: &str, multi_level_prefixes: &[&str]) -> String {
    let parts: Vec<&str> = k.splitn(3, '.').collect();
    let is_multi_level = multi_level_prefixes.iter().any(|prefix| *prefix == parts[0]);
    if is_multi_level && parts.len() >= 2 {
        parts[0..2].join(".")
    } else {
        String::from(parts[0])
    }
}

/// Re-apply the table spacing the tombi format pass flattened. Tombi collapses every gap between
/// top-level tables to one blank line, so the counts [`Tables::reorder`] inserted are lost. Reset
/// each gap: `root_spacing` blank lines between different groups, `sub_spacing` between same-group
/// sub-tables, where each `\n` is one blank line. Pass `sub_spacing` as `None` to leave same-group
/// gaps as tombi left them (short format, where a force-expanded sub-table keeps its single blank
/// line). Grouping matches `reorder`, so pass the same `multi_level_prefixes`.
#[must_use]
pub fn normalize_table_spacing(
    content: &str,
    multi_level_prefixes: &[&str],
    root_spacing: &str,
    sub_spacing: Option<&str>,
) -> String {
    let root_blanks = root_spacing.matches('\n').count();
    let sub_blanks = sub_spacing.map(|s| s.matches('\n').count());
    let lines: Vec<&str> = content.lines().collect();

    let headers: Vec<(usize, String, String)> = lines
        .iter()
        .enumerate()
        .filter_map(|(i, line)| header_name(line).map(|name| (i, get_key(&name, multi_level_prefixes), name)))
        .collect();

    let mut gap_before: HashMap<usize, usize> = HashMap::new();
    for pair in headers.windows(2) {
        let (prev_pos, prev_group, prev_name) = &pair[0];
        let (pos, group, name) = &pair[1];
        // Repeated array-of-tables entries share a name and keep reorder's fixed single blank line.
        if name == prev_name {
            continue;
        }
        let blanks = if group == prev_group {
            let Some(blanks) = sub_blanks else { continue };
            blanks
        } else {
            root_blanks
        };
        // Leading comments belong to the following table, so the gap sits above them.
        let start = (prev_pos + 1..*pos)
            .rev()
            .take_while(|&line| lines[line].trim_start().starts_with('#'))
            .last()
            .unwrap_or(*pos);
        gap_before.insert(start, blanks);
    }

    let mut out: Vec<&str> = Vec::with_capacity(lines.len());
    for (i, line) in lines.iter().enumerate() {
        if let Some(&blanks) = gap_before.get(&i) {
            while out.last().is_some_and(|l| l.trim().is_empty()) {
                out.pop();
            }
            out.resize(out.len() + blanks, "");
        }
        out.push(line);
    }

    let mut result = out.join("\n");
    if content.ends_with('\n') {
        result.push('\n');
    }
    result
}

fn header_name(line: &str) -> Option<String> {
    if !line.starts_with('[') {
        return None;
    }
    let trimmed = line.trim_end();
    if let Some(rest) = trimmed.strip_prefix("[[") {
        rest.find("]]").map(|end| rest[..end].trim().to_string())
    } else {
        let rest = &trimmed[1..];
        rest.find(']').map(|end| rest[..end].trim().to_string())
    }
}

pub fn reorder_table_keys(table: &mut RefMut<Vec<SyntaxElement>>, order: &[&str]) {
    let (size, mut to_insert) = (table.len(), Vec::<SyntaxElement>::new());
    let (key_to_position, key_set) = load_keys(table);

    let mut headers: Vec<Vec<SyntaxElement>> = vec![Vec::new()];
    let mut pos_group = vec![0_usize; key_set.len()];
    let mut current_group = 0;
    for position in 0..key_set.len() {
        if let Some(first) = key_set[position].first()
            && let Some(header) = split_leading_group_marker(first)
        {
            headers.push(header);
            current_group = headers.len() - 1;
        }
        pos_group[position] = current_group;
    }

    let mut handled_positions = HashSet::<usize>::new();
    for (group, header) in headers.iter().enumerate() {
        if !header.is_empty() {
            let starts_with_break = header.first().map(SyntaxElement::kind) == Some(LINE_BREAK);
            if !starts_with_break && !to_insert.is_empty() && to_insert.last().map(|e| e.kind()) != Some(LINE_BREAK) {
                to_insert.push(make_newline());
            }
            to_insert.extend(header.iter().cloned());
        }
        for current_key in order {
            let mut matching_keys = key_to_position
                .iter()
                .filter(|(checked_key, position)| {
                    !handled_positions.contains(position)
                        && (current_key == checked_key
                            || (checked_key.starts_with(current_key)
                                && checked_key.len() > current_key.len()
                                && checked_key.chars().nth(current_key.len()).unwrap() == '.'))
                })
                .map(|(key, _)| key)
                .clone()
                .collect::<Vec<&String>>();
            matching_keys.sort_by_key(|key| key.to_lowercase());
            for key in matching_keys {
                let position = key_to_position[key];
                if pos_group[position] != group {
                    continue;
                }
                emit_key(&mut to_insert, &key_set[position]);
                handled_positions.insert(position);
            }
        }
        let mut unhandled: Vec<(String, usize)> = key_to_position
            .iter()
            .filter(|(_, position)| !handled_positions.contains(position) && pos_group[**position] == group)
            .map(|(key, position)| (key.clone(), *position))
            .collect();
        unhandled.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
        for (_, position) in unhandled {
            emit_key(&mut to_insert, &key_set[position]);
            handled_positions.insert(position);
        }
    }
    table.splice(0..size, to_insert);
}

fn load_keys(table: &[SyntaxElement]) -> (HashMap<String, usize>, Vec<Vec<SyntaxElement>>) {
    let table_clone = if table.last().unwrap().kind() == LINE_BREAK {
        &table[..table.len() - 1]
    } else {
        table
    };
    let mut key_to_pos = HashMap::<String, usize>::new();
    let mut key_set = Vec::<Vec<SyntaxElement>>::new();
    let entry_set = RefCell::new(Vec::<SyntaxElement>::new());
    let mut add_to_key_set = |k| {
        let mut entry_set_borrow = entry_set.borrow_mut();
        if !entry_set_borrow.is_empty() {
            key_to_pos.insert(k, key_set.len());
            key_set.push(entry_set_borrow.clone());
            entry_set_borrow.clear();
        }
    };
    let mut key = String::new();
    let mut cutoff = false;
    for element in table_clone {
        let kind = element.kind();
        if kind == KEY_VALUE {
            if cutoff {
                add_to_key_set(key.clone());
                cutoff = false;
            }
            if let Some(e) = element
                .as_node()
                .unwrap()
                .children_with_tokens()
                .find(|e| e.kind() == KEYS)
            {
                key = e.as_node().unwrap().text().to_string().trim().to_string();
            }
        }
        if [KEY_VALUE, TABLE, ARRAY_OF_TABLE, BRACKET_START, DOUBLE_BRACKET_START].contains(&kind) {
            cutoff = true;
        }
        entry_set.borrow_mut().push(element.clone());
        if cutoff && kind == LINE_BREAK {
            add_to_key_set(key.clone());
            cutoff = false;
        }
    }
    add_to_key_set(key);
    (key_to_pos, key_set)
}

pub fn get_table_name(entry: &SyntaxElement) -> String {
    if [TABLE, ARRAY_OF_TABLE].contains(&entry.kind()) {
        for child in entry.as_node().unwrap().children_with_tokens() {
            if child.kind() == KEYS {
                return child.as_node().unwrap().text().to_string().trim().to_string();
            }
        }
    }
    String::new()
}

pub fn for_entries<F>(table: &[SyntaxElement], f: &mut F)
where
    F: FnMut(String, &SyntaxNode),
{
    let mut key = String::new();
    for table_entry in table {
        if table_entry.kind() == KEY_VALUE {
            for entry in table_entry.as_node().unwrap().children_with_tokens() {
                if entry.kind() == KEYS {
                    key = entry.as_node().unwrap().text().to_string().trim().to_string();
                } else if is_value_kind(entry.kind()) {
                    f(key.clone(), entry.as_node().unwrap());
                }
            }
        }
    }
}

pub fn rename_keys(table: &mut RefMut<Vec<SyntaxElement>>, aliases: &[(&str, &str)]) {
    use crate::create::make_key;
    for entry in table.iter() {
        if entry.kind() != KEY_VALUE {
            continue;
        }
        let node = entry.as_node().unwrap();
        let keys_node = node
            .children_with_tokens()
            .find(|c| c.kind() == KEYS)
            .expect("KEY_VALUE must have KEYS child");
        let keys_node = keys_node.as_node().unwrap();
        let key_text = keys_node.text().to_string().trim().to_string();
        for &(old, new) in aliases {
            if key_text == old {
                let new_key = make_key(new);
                let count = keys_node.children_with_tokens().count();
                let new_children: Vec<SyntaxElement> = new_key.as_node().unwrap().children_with_tokens().collect();
                keys_node.splice_children(0..count, new_children);
                break;
            }
        }
    }
}

pub fn find_key(table: &SyntaxNode, key: &str) -> Option<SyntaxNode> {
    let mut current_key = String::new();
    for table_entry in table.children_with_tokens() {
        let kind = table_entry.kind();
        if kind == KEY_VALUE {
            for entry in table_entry.as_node().unwrap().children_with_tokens() {
                if entry.kind() == KEYS {
                    current_key = entry.as_node().unwrap().text().to_string().trim().to_string();
                } else if is_value_kind(entry.kind()) && current_key == key {
                    return Some(entry.as_node().unwrap().clone());
                }
            }
        } else if kind == KEY_VALUE_GROUP || kind == KEY_VALUE_WITH_COMMA_GROUP {
            for kv in table_entry.as_node().unwrap().children_with_tokens() {
                if kv.kind() == KEY_VALUE {
                    for entry in kv.as_node().unwrap().children_with_tokens() {
                        if entry.kind() == KEYS {
                            current_key = entry.as_node().unwrap().text().to_string().trim().to_string();
                        } else if is_value_kind(entry.kind()) && current_key == key {
                            return Some(entry.as_node().unwrap().clone());
                        }
                    }
                }
            }
        }
    }
    None
}

pub fn collapse_sub_tables(tables: &mut Tables, name: &str) {
    let h2p = tables.header_to_pos.clone();
    let sub_name_prefix = format!("{name}.");
    let sub_table_keys: Vec<&String> = h2p.keys().filter(|s| s.starts_with(sub_name_prefix.as_str())).collect();
    if sub_table_keys.is_empty() {
        return;
    }
    ensure_table_exists(tables, name);
    let main_positions = tables.header_to_pos[name].clone();
    if main_positions.len() != 1 {
        return;
    }
    let mut main = tables.table_set[*main_positions.first().unwrap()].borrow_mut();
    for key in sub_table_keys {
        let sub_positions = tables.header_to_pos[key].clone();
        if sub_positions.len() != 1 {
            continue;
        }
        let mut sub = tables.table_set[*sub_positions.first().unwrap()].borrow_mut();

        // Array-of-tables shows up as ARRAY_OF_TABLE (old parse) or DOUBLE_BRACKET_START (new parse).
        let is_array_table = sub
            .iter()
            .any(|child| child.kind() == ARRAY_OF_TABLE || child.kind() == DOUBLE_BRACKET_START);
        if is_array_table {
            continue;
        }

        let sub_name = key.strip_prefix(sub_name_prefix.as_str()).unwrap();

        let is_empty_table = !sub.iter().any(|child| child.kind() == KEY_VALUE);
        if is_empty_table {
            if has_live_descendant_table(tables, key) {
                continue;
            }
            if main.last().is_some_and(|e| e.kind() != LINE_BREAK) {
                main.push(make_newline());
            }
            main.push(make_empty_inline_table(sub_name));
            sub.clear();
            continue;
        }

        let mut in_header = false;
        let mut skip_next_line_break = false;
        for child in sub.iter() {
            let kind = child.kind();
            if kind == BRACKET_START || kind == TABLE {
                in_header = true;
                continue;
            }
            if in_header && (kind == KEYS || kind == BRACKET_END) {
                if kind == BRACKET_END {
                    in_header = false;
                    skip_next_line_break = true;
                }
                continue;
            }
            if skip_next_line_break && kind == LINE_BREAK {
                skip_next_line_break = false;
                continue;
            }
            if kind == KEY_VALUE {
                let mut to_insert = Vec::<SyntaxElement>::new();
                let child_node = child.as_node().unwrap();
                for mut entry in child_node.children_with_tokens() {
                    if entry.kind() == KEYS {
                        let mut key_parts = vec![String::from(sub_name)];
                        for array_entry_value in entry.as_node().unwrap().children_with_tokens() {
                            if array_entry_value.kind() == BARE_KEY {
                                let txt = load_text(&array_entry_value.to_string(), BARE_KEY);
                                key_parts.push(txt);
                            }
                        }
                        entry = make_key(&key_parts.join("."));
                    }
                    to_insert.push(entry);
                }
                child_node.splice_children(0..to_insert.len(), to_insert);
            }
            let kind = child.kind();
            if main.last().unwrap().kind() != LINE_BREAK && kind != LINE_BREAK && !has_leading_newline(child) {
                main.push(make_newline());
            }
            main.push(child.clone());
        }
        sub.clear();
    }
}

pub fn expand_sub_tables(tables: &mut Tables, name: &str) {
    let main_positions = match tables.header_to_pos.get(name) {
        Some(p) if !p.is_empty() => p.clone(),
        _ => return,
    };
    if main_positions.len() != 1 {
        return;
    }

    let mut groups: HashMap<String, Vec<(String, SyntaxElement)>> = HashMap::new();
    let mut entries_to_remove: HashSet<usize> = HashSet::new();

    {
        let main = tables.table_set[*main_positions.first().unwrap()].borrow();

        for (entry_index, element) in main.iter().filter(|e| e.kind() == KEY_VALUE).enumerate() {
            let key_text = get_key_text(element);

            if let Some(dot_pos) = key_text.find('.') {
                let prefix = &key_text[..dot_pos];
                let rest = &key_text[dot_pos + 1..];

                groups
                    .entry(String::from(prefix))
                    .or_default()
                    .push((String::from(rest), element.clone()));
                entries_to_remove.insert(entry_index);
            }
        }
    }

    if groups.is_empty() {
        return;
    }

    filter_entries(
        &mut tables.table_set[*main_positions.first().unwrap()].borrow_mut(),
        &entries_to_remove,
    );

    for (sub_name, entries) in groups {
        let full_name = format!("{name}.{sub_name}");

        let mut new_table = make_table_entry(&full_name);

        for (simple_key, original_entry) in entries {
            let value_text = get_value_text(&original_entry);

            let new_entry_text = format!("{simple_key} ={value_text}\n");
            let parsed_root = parse(&new_entry_text);
            if let Some(entry) = find_key_value_in_parsed(&parsed_root) {
                new_table.push(entry);
            }
        }

        let pos = tables.table_set.len();
        tables.table_set.push(RefCell::new(new_table));
        tables.header_to_pos.entry(full_name).or_default().push(pos);
    }
}

pub fn collapse_sub_table(tables: &mut Tables, parent_name: &str, sub_name: &str, column_width: usize) {
    let full_name = format!("{parent_name}.{sub_name}");
    let sub_positions = match tables.header_to_pos.get(&full_name) {
        Some(p) if !p.is_empty() => p.clone(),
        _ => return,
    };

    let first_sub = tables.table_set[*sub_positions.first().unwrap()].borrow();
    // Array-of-tables shows up as ARRAY_OF_TABLE (old parse) or DOUBLE_BRACKET_START (new parse).
    let is_array_table = first_sub
        .iter()
        .any(|child| child.kind() == ARRAY_OF_TABLE || child.kind() == DOUBLE_BRACKET_START);
    drop(first_sub);

    if is_array_table {
        collapse_array_of_tables(tables, parent_name, sub_name, &sub_positions, column_width);
        return;
    }

    let sub_is_empty = !tables.table_set[*sub_positions.first().unwrap()]
        .borrow()
        .iter()
        .any(|child| child.kind() == KEY_VALUE);
    if sub_is_empty && has_live_descendant_table(tables, &full_name) {
        return;
    }

    ensure_table_exists(tables, parent_name);
    let main_positions = tables.header_to_pos[parent_name].clone();
    if main_positions.len() != 1 {
        return;
    }
    let mut main = tables.table_set[*main_positions.first().unwrap()].borrow_mut();
    let mut sub = tables.table_set[*sub_positions.first().unwrap()].borrow_mut();

    let is_empty_table = !sub.iter().any(|child| child.kind() == KEY_VALUE);
    if is_empty_table {
        if main.last().is_some_and(|e| e.kind() != LINE_BREAK) {
            main.push(make_newline());
        }
        main.push(make_empty_inline_table(sub_name));
        sub.clear();
        return;
    }

    let mut in_header = false;
    let mut skip_next_line_break = false;
    for child in sub.iter() {
        let kind = child.kind();
        if kind == BRACKET_START || kind == TABLE {
            in_header = true;
            continue;
        }
        if in_header && (kind == KEYS || kind == BRACKET_END) {
            if kind == BRACKET_END {
                in_header = false;
                skip_next_line_break = true;
            }
            continue;
        }
        if skip_next_line_break && kind == LINE_BREAK {
            skip_next_line_break = false;
            continue;
        }
        if kind == KEY_VALUE {
            let mut to_insert = Vec::<SyntaxElement>::new();
            let child_node = child.as_node().unwrap();
            for mut entry in child_node.children_with_tokens() {
                if entry.kind() == KEYS {
                    let mut key_parts = vec![String::from(sub_name)];
                    for array_entry_value in entry.as_node().unwrap().children_with_tokens() {
                        let entry_kind = array_entry_value.kind();
                        if entry_kind == BARE_KEY {
                            let txt = load_text(&array_entry_value.to_string(), BARE_KEY);
                            key_parts.push(txt);
                        } else if entry_kind == BASIC_STRING || entry_kind == LITERAL_STRING {
                            key_parts.push(array_entry_value.to_string());
                        }
                    }
                    entry = make_key(&key_parts.join("."));
                }
                to_insert.push(entry);
            }
            child_node.splice_children(0..to_insert.len(), to_insert);
        }
        if main.last().unwrap().kind() != LINE_BREAK && kind != LINE_BREAK && !has_leading_newline(child) {
            main.push(make_newline());
        }
        main.push(child.clone());
    }
    sub.clear();
}

struct KeyValueWithComments {
    comments: Vec<String>,
    key: String,
    value: String,
}

fn extract_comments_from_key_value(element: &SyntaxElement) -> Vec<String> {
    element
        .as_node()
        .map(|node| {
            node.children_with_tokens()
                .filter(|c| c.kind() == COMMENT)
                .map(|c| c.to_string().trim().to_string())
                .collect()
        })
        .unwrap_or_default()
}

fn collapse_array_of_tables(
    tables: &mut Tables,
    parent_name: &str,
    sub_name: &str,
    sub_positions: &[usize],
    column_width: usize,
) {
    // A disabled key carries the marker in a trailing comment; collapsing the entry into an inline table would embed
    // that comment and produce invalid TOML, so keep it expanded.
    let has_disabled_key = sub_positions.iter().any(|pos| {
        tables.table_set[*pos]
            .borrow()
            .iter()
            .any(|e| e.to_string().contains(crate::disabled::MARKER))
    });
    if has_disabled_key {
        return;
    }

    let mut all_entries: Vec<Vec<KeyValueWithComments>> = Vec::new();

    for pos in sub_positions {
        let sub = tables.table_set[*pos].borrow();
        let mut pending_comments: Vec<String> = Vec::new();
        let mut entries_for_this_aot: Vec<KeyValueWithComments> = Vec::new();

        for child in sub.iter() {
            match child.kind() {
                KEY_VALUE => {
                    let mut comments = std::mem::take(&mut pending_comments);
                    comments.extend(extract_comments_from_key_value(child));
                    let key = get_key_text(child);
                    let value = get_value_text(child).trim().to_string();
                    if !key.is_empty() && !value.is_empty() {
                        entries_for_this_aot.push(KeyValueWithComments { comments, key, value });
                    }
                }
                COMMENT => {
                    pending_comments.push(child.to_string().trim().to_string());
                }
                _ => {}
            }
        }

        if !pending_comments.is_empty()
            && let Some(last) = entries_for_this_aot.last_mut()
        {
            last.comments.extend(pending_comments);
        }

        if !entries_for_this_aot.is_empty() {
            all_entries.push(entries_for_this_aot);
        }
    }

    if all_entries.is_empty() {
        return;
    }

    let has_comments_between_keys = all_entries
        .iter()
        .any(|aot_entries| aot_entries.iter().skip(1).any(|entry| !entry.comments.is_empty()));
    if has_comments_between_keys {
        return;
    }

    let array_value = {
        let has_leading_comments = all_entries
            .iter()
            .any(|aot_entries| aot_entries.first().is_some_and(|e| !e.comments.is_empty()));
        if has_leading_comments {
            let mut parts: Vec<String> = Vec::new();
            for aot_entries in &all_entries {
                if let Some(first) = aot_entries.first() {
                    for comment in &first.comments {
                        parts.push(format!("  {comment}"));
                    }
                }
                let kv_pairs: Vec<String> = aot_entries.iter().map(|e| format!("{} = {}", e.key, e.value)).collect();
                let inline_table = format!("{{ {} }}", kv_pairs.join(", "));
                if inline_table.len() > column_width {
                    return;
                }
                parts.push(format!("  {inline_table},"));
            }
            format!("[\n{}\n]", parts.join("\n"))
        } else {
            let mut inline_tables: Vec<String> = Vec::new();
            for aot_entries in &all_entries {
                let kv_pairs: Vec<String> = aot_entries.iter().map(|e| format!("{} = {}", e.key, e.value)).collect();
                let inline_table = format!("{{ {} }}", kv_pairs.join(", "));
                if inline_table.len() > column_width {
                    return;
                }
                inline_tables.push(inline_table);
            }
            format!("[{}]", inline_tables.join(", "))
        }
    };
    let entry_text = format!("{sub_name} = {array_value}\n");

    ensure_table_exists(tables, parent_name);
    let main_positions = tables.header_to_pos[parent_name].clone();
    if main_positions.len() != 1 {
        return;
    }
    let mut main = tables.table_set[*main_positions.first().unwrap()].borrow_mut();

    if main.last().is_some_and(|e| e.kind() != LINE_BREAK) {
        main.push(make_newline());
    }

    let parsed_root = parse(&entry_text);
    if let Some(entry) = find_key_value_in_parsed(&parsed_root) {
        main.push(entry);
    }

    for pos in sub_positions {
        tables.table_set[*pos].borrow_mut().clear();
    }
}

pub fn expand_sub_table(tables: &mut Tables, parent_name: &str, sub_name: &str) {
    let main_positions = match tables.header_to_pos.get(parent_name) {
        Some(p) if !p.is_empty() => p.clone(),
        _ => return,
    };
    if main_positions.len() != 1 {
        return;
    }

    let prefix_with_dot = format!("{sub_name}.");
    let mut entries: Vec<(String, SyntaxElement)> = Vec::new();
    let mut entries_to_remove: HashSet<usize> = HashSet::new();

    {
        let main = tables.table_set[*main_positions.first().unwrap()].borrow();

        for (entry_index, element) in main.iter().filter(|e| e.kind() == KEY_VALUE).enumerate() {
            let key_text = get_key_text(element);

            if key_text.starts_with(&prefix_with_dot) {
                let rest = &key_text[prefix_with_dot.len()..];
                entries.push((String::from(rest), element.clone()));
                entries_to_remove.insert(entry_index);
            }
        }
    }

    if entries.is_empty() {
        return;
    }

    filter_entries(
        &mut tables.table_set[*main_positions.first().unwrap()].borrow_mut(),
        &entries_to_remove,
    );

    let full_name = format!("{parent_name}.{sub_name}");
    let mut new_table = make_table_entry(&full_name);

    for (simple_key, original_entry) in entries {
        let value_text = get_value_text(&original_entry);

        let new_entry_text = format!("{simple_key} ={value_text}\n");
        let parsed_root = parse(&new_entry_text);
        for child in parsed_root.children_with_tokens() {
            if child.kind() == KEY_VALUE_GROUP {
                for kv in child.as_node().unwrap().children_with_tokens() {
                    new_table.push(kv);
                }
            } else if child.kind() == KEY_VALUE || child.kind() == LINE_BREAK {
                new_table.push(child);
            }
        }
    }

    let pos = tables.table_set.len();
    tables.table_set.push(RefCell::new(new_table));
    tables.header_to_pos.entry(full_name).or_default().push(pos);
}

fn unquoted_dot_positions(s: &str) -> impl Iterator<Item = usize> + '_ {
    let mut in_quotes = false;
    s.char_indices().filter_map(move |(i, c)| match c {
        '"' => {
            in_quotes = !in_quotes;
            None
        }
        '.' if !in_quotes => Some(i),
        _ => None,
    })
}

pub fn count_unquoted_dots(s: &str) -> usize {
    unquoted_dot_positions(s).count()
}

pub fn first_unquoted_dot(s: &str) -> usize {
    unquoted_dot_positions(s).next().expect("no unquoted dot found")
}

pub fn split_table_name(full_name: &str) -> (&str, &str) {
    let mut depth = 0;
    for (i, c) in full_name.char_indices().rev() {
        match c {
            '"' => depth = 1 - depth,
            '.' if depth == 0 => return (&full_name[..i], &full_name[i + 1..]),
            _ => {}
        }
    }
    unreachable!("split_table_name called with name without dots: {full_name}")
}

pub fn apply_table_formatting<F>(tables: &mut Tables, should_collapse: F, prefixes: &[&str], column_width: usize)
where
    F: Fn(&str) -> bool,
{
    let mut all_sub_tables: Vec<String> = Vec::new();
    for prefix in prefixes {
        collect_all_sub_tables(tables, prefix, &mut all_sub_tables);
    }
    all_sub_tables.sort_by(|a, b| {
        let depth_a = count_unquoted_dots(a);
        let depth_b = count_unquoted_dots(b);
        match depth_b.cmp(&depth_a) {
            std::cmp::Ordering::Equal => a.cmp(b),
            other => other,
        }
    });
    for full_name in all_sub_tables {
        let (parent, sub) = split_table_name(&full_name);
        if should_collapse(&full_name) {
            collapse_sub_table(tables, parent, sub, column_width);
        } else {
            expand_sub_table(tables, parent, sub);
        }
    }
}

pub fn collect_all_sub_tables(tables: &Tables, parent_name: &str, result: &mut Vec<String>) {
    let prefix = format!("{parent_name}.");
    let prefix_dots = count_unquoted_dots(parent_name);

    for key in tables.header_to_pos.keys() {
        if key.starts_with(&prefix) && key != parent_name && !result.contains(key) {
            result.push(key.clone());
            add_intermediate_parents(key, prefix_dots, result);
        }
    }

    let Some(pos) = tables.header_to_pos.get(parent_name).and_then(|p| p.first()) else {
        return;
    };
    let main = tables.table_set[*pos].borrow();
    for element in main.iter().filter(|e| e.kind() == KEY_VALUE) {
        let key_text = get_key_text(element);
        if let Some(dot_pos) = unquoted_dot_positions(&key_text).next() {
            let sub_name = &key_text[..dot_pos];
            let full_name = format!("{parent_name}.{sub_name}");
            if !result.contains(&full_name) {
                result.push(full_name);
            }
        }
    }
}

fn add_intermediate_parents(table_name: &str, prefix_dots: usize, result: &mut Vec<String>) {
    let mut current = table_name;
    loop {
        let (parent, _) = split_table_name(current);
        if count_unquoted_dots(parent) <= prefix_dots {
            break;
        }
        if !result.contains(&String::from(parent)) {
            result.push(String::from(parent));
        }
        current = parent;
    }
}

pub struct InlineTableSchema {
    pub discriminator: &'static str,
    pub key_order: &'static [&'static str],
}

fn inline_table_key_name(kv_node: &SyntaxNode) -> String {
    let raw = kv_node
        .children_with_tokens()
        .find(|c| c.kind() == KEYS)
        .and_then(|c| c.as_node().map(|n| n.text().to_string().trim().to_string()))
        .unwrap_or_default();
    raw.strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .or_else(|| raw.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')))
        .unwrap_or(&raw)
        .to_string()
}

fn detect_schema<'a>(keys: &[String], schemas: &'a [InlineTableSchema]) -> Option<&'a [&'static str]> {
    schemas
        .iter()
        .find(|s| keys.iter().any(|k| k == s.discriminator))
        .map(|s| s.key_order)
}

/// One key-value of an inline table together with the comments bound to it: `leading` holds
/// own-line comments that sit before the key, `trailing` the same-line comment after its comma.
/// Carrying both lets a key keep its comments when the keys get reordered.
struct InlineEntry {
    key: String,
    text: String,
    leading: Vec<String>,
    trailing: Option<String>,
}

/// Split a `KEY_VALUE` node into its `key = value` text and the own-line comments that precede
/// the key. Tombi stores those comments (and the multi-line layout) as leading trivia before the
/// `KEYS` child, so everything from `KEYS` onward is the value text and the rest is layout.
fn clean_key_value(kv: &SyntaxNode) -> (String, Vec<String>) {
    let (mut text, mut leading, mut started) = (String::new(), Vec::new(), false);
    for child in kv.children_with_tokens() {
        if started {
            text.push_str(&child.to_string());
        } else if child.kind() == KEYS {
            started = true;
            text.push_str(&child.to_string());
        } else if child.kind() == COMMENT {
            leading.push(child.to_string().trim().to_string());
        }
    }
    (text.trim().to_string(), leading)
}

/// Collect the inline table's entries with their comments, or `None` when the table holds a
/// comment that is not bound to a key. Own-line comments before a key live in that key's leading
/// trivia and trailing comments live in the preceding `COMMA`, so both move with their key. A
/// comment that is a direct child of the inline table (a dangling comment before the closing
/// brace) belongs to no key and would be lost by the rebuild, so the caller leaves the table
/// untouched instead.
fn collect_inline_entries(node: &SyntaxNode) -> Option<Vec<InlineEntry>> {
    // Comments before `BRACE_START` are the array entry's own leading trivia (the splice keeps
    // them); a comment inside the braces that is not part of a key or comma (a dangling comment
    // before the closing brace) would be lost by the rebuild, so leave the table untouched.
    if node
        .children_with_tokens()
        .skip_while(|c| c.kind() != BRACE_START)
        .any(|c| matches!(c.kind(), COMMENT | DANGLING_COMMENT_GROUP))
    {
        return None;
    }
    let mut entries: Vec<InlineEntry> = Vec::new();
    for group in node.children().filter(|n| n.kind() == KEY_VALUE_WITH_COMMA_GROUP) {
        for child in group.children_with_tokens() {
            match child.kind() {
                KEY_VALUE => {
                    let kv = child.as_node().unwrap();
                    let (text, leading) = clean_key_value(kv);
                    entries.push(InlineEntry {
                        key: inline_table_key_name(kv),
                        text,
                        leading,
                        trailing: None,
                    });
                }
                COMMA => {
                    if let Some(comment) = child
                        .as_node()
                        .and_then(|comma| comma.children_with_tokens().find(|c| c.kind() == COMMENT))
                    {
                        entries
                            .last_mut()
                            .expect("a comma always follows a key-value in an inline table")
                            .trailing = Some(comment.to_string().trim().to_string());
                    }
                }
                _ => {}
            }
        }
    }
    Some(entries)
}

/// Render the reordered entries back into inline-table source. With no comments the compact
/// single-line form is kept; any comment forces the multi-line form, the only shape that can
/// carry a comment inside an inline table.
fn build_inline_table_text(entries: &[&InlineEntry]) -> String {
    if entries.iter().all(|e| e.leading.is_empty() && e.trailing.is_none()) {
        let joined = entries.iter().map(|e| e.text.as_str()).collect::<Vec<_>>().join(", ");
        return format!("_x = {{ {joined} }}\n");
    }
    let mut out = String::from("_x = {\n");
    for (idx, entry) in entries.iter().enumerate() {
        for comment in &entry.leading {
            out.push_str("  ");
            out.push_str(comment);
            out.push('\n');
        }
        out.push_str("  ");
        out.push_str(&entry.text);
        if idx + 1 < entries.len() {
            out.push(',');
        }
        if let Some(comment) = &entry.trailing {
            out.push(' ');
            out.push_str(comment);
        }
        out.push('\n');
    }
    out.push_str("}\n");
    out
}

fn reorder_single_inline_table(node: &SyntaxNode, schemas: &[InlineTableSchema]) {
    let Some(entries) = collect_inline_entries(node) else {
        return;
    };
    if entries.len() < 2 {
        return;
    }

    let keys: Vec<String> = entries.iter().map(|e| e.key.clone()).collect();
    let Some(schema) = detect_schema(&keys, schemas) else {
        return;
    };

    let key_position = |k: &str| -> usize { schema.iter().position(|s| *s == k).unwrap_or(usize::MAX) };
    let mut order: Vec<usize> = (0..entries.len()).collect();
    order.sort_by_key(|&i| key_position(&entries[i].key));

    if order.iter().enumerate().all(|(new, &old)| new == old) {
        return;
    }

    let sorted: Vec<&InlineEntry> = order.iter().map(|&i| &entries[i]).collect();
    let rebuilt = build_inline_table_text(&sorted);
    let parsed = parse(&rebuilt);
    let new_children: Option<Vec<SyntaxElement>> = parsed
        .descendants()
        .find(|n| n.kind() == INLINE_TABLE)
        .map(|n| n.children_with_tokens().collect());

    if let Some(children) = new_children {
        let original: Vec<SyntaxElement> = node.children_with_tokens().collect();
        let brace_idx = original.iter().position(|c| c.kind() == BRACE_START).unwrap_or(0);
        let mut merged = original[..brace_idx].to_vec();
        merged.extend(children);
        node.splice_children(0..original.len(), merged);
    }
}

pub fn reorder_inline_table_keys(root_ast: &SyntaxNode, schemas: &[InlineTableSchema]) {
    let inline_tables: Vec<SyntaxNode> = root_ast.descendants().filter(|n| n.kind() == INLINE_TABLE).collect();
    for node in inline_tables {
        reorder_single_inline_table(&node, schemas);
    }
}
