use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::iter::zip;
use std::ops::Index;

use taplo::syntax::SyntaxKind::{COMMENT, ENTRY, IDENT, KEY, NEWLINE, TABLE_ARRAY_HEADER, TABLE_HEADER, VALUE};
use taplo::syntax::{SyntaxElement, SyntaxNode};
use taplo::HashSet;

use crate::create::{make_empty_newline, make_key, make_newline, make_table_entry};
use crate::string::load_text;

// Re-export taplo for use in expand_sub_tables
use taplo;

#[derive(Debug)]
pub struct Tables {
    pub header_to_pos: HashMap<String, Vec<usize>>,
    pub table_set: Vec<RefCell<Vec<SyntaxElement>>>,
}

impl Tables {
    pub fn get(&self, key: &str) -> Option<Vec<&RefCell<Vec<SyntaxElement>>>> {
        if self.header_to_pos.contains_key(key) {
            let mut res = Vec::<&RefCell<Vec<SyntaxElement>>>::new();
            for pos in &self.header_to_pos[key] {
                res.push(&self.table_set[*pos]);
            }
            Some(res)
        } else {
            None
        }
    }

    pub fn from_ast(root_ast: &SyntaxNode) -> Self {
        let mut header_to_pos = HashMap::<String, Vec<usize>>::new();
        let mut table_set = Vec::<RefCell<Vec<SyntaxElement>>>::new();
        let entry_set = RefCell::new(Vec::<SyntaxElement>::new());
        let mut table_kind = TABLE_HEADER;
        let mut add_to_table_set = |kind| {
            let mut entry_set_borrow = entry_set.borrow_mut();
            if !entry_set_borrow.is_empty() {
                // Find the table header in the entry set (may not be first element due to comments)
                let table_name = entry_set_borrow
                    .iter()
                    .find(|e| [TABLE_HEADER, TABLE_ARRAY_HEADER].contains(&e.kind()))
                    .map_or_else(String::new, get_table_name);
                let indexes = header_to_pos.entry(table_name).or_default();
                if kind == TABLE_ARRAY_HEADER || (kind == TABLE_HEADER && indexes.is_empty()) {
                    indexes.push(table_set.len());
                    table_set.push(RefCell::new(entry_set_borrow.clone()));
                } else if kind == TABLE_HEADER && !indexes.is_empty() {
                    // join tables
                    let pos = indexes.first().unwrap();
                    let mut res = table_set.index(*pos).borrow_mut();
                    let mut new = entry_set_borrow.clone();
                    if let Some(last_non_trailing_newline_index) = new.iter().rposition(|x| x.kind() != NEWLINE) {
                        new.truncate(last_non_trailing_newline_index + 1);
                    }
                    if res.last().unwrap().kind() != NEWLINE {
                        res.push(make_newline());
                    }
                    res.extend(
                        new.into_iter()
                            .skip_while(|x| [NEWLINE, TABLE_HEADER].contains(&x.kind())),
                    );
                }
                entry_set_borrow.clear();
            }
        };
        for c in root_ast.children_with_tokens() {
            if [TABLE_ARRAY_HEADER, TABLE_HEADER].contains(&c.kind()) {
                // Find comments that appear after the last ENTRY - these belong to the new table
                let mut borrow = entry_set.borrow_mut();

                // Find the first COMMENT after the last ENTRY - that's where we split
                let last_entry_pos = borrow.iter().rposition(|x| x.kind() == ENTRY);
                let first_comment_pos = borrow.iter().position(|x| x.kind() == COMMENT);

                let comments_start = match (last_entry_pos, first_comment_pos) {
                    (Some(entry_pos), Some(comment_pos)) if comment_pos > entry_pos => comment_pos,
                    (None, Some(comment_pos)) => comment_pos, // No entries, but has comments
                    _ => borrow.len(),                        // No comments to move
                };

                // Split: keep elements for previous table, extract comments for new table
                let comments_for_new_table: Vec<SyntaxElement> = borrow.drain(comments_start..).collect();
                drop(borrow);

                add_to_table_set(table_kind);
                table_kind = c.kind();

                // Add the comments first (they belong to this new table)
                entry_set.borrow_mut().extend(comments_for_new_table);
            }
            entry_set.borrow_mut().push(c);
        }
        add_to_table_set(table_kind);
        Self {
            header_to_pos,
            table_set,
        }
    }

    /// Reorder tables according to the given order.
    /// `multi_level_prefixes` specifies which prefixes should use two-part keys (e.g., `["tool", "env"]` means
    /// `tool.black` and `env.docs` are treated as distinct base keys instead of grouping under `tool` or `env`).
    pub fn reorder(&self, root_ast: &SyntaxNode, order: &[&str], multi_level_prefixes: &[&str]) {
        let mut to_insert = Vec::<SyntaxElement>::new();
        let order = calculate_order(&self.header_to_pos, &self.table_set, order, multi_level_prefixes);
        let mut next = order.clone();
        if !next.is_empty() {
            next.remove(0);
        }
        next.push(String::new());
        for (name, next_name) in zip(order.iter(), next.iter()) {
            for entries in self.get(name).unwrap() {
                let got = entries.borrow_mut();
                if !got.is_empty() {
                    let last = got.last().unwrap();
                    if name.is_empty() && last.kind() == NEWLINE && got.len() == 1 {
                        continue;
                    }
                    let mut add = got.clone();
                    if get_key(name, multi_level_prefixes) != get_key(next_name, multi_level_prefixes) {
                        if last.kind() == NEWLINE {
                            // replace existing newline to ensure single newline
                            add.pop();
                        }
                        add.push(make_empty_newline());
                    }
                    to_insert.extend(add);
                }
            }
        }
        root_ast.splice_children(0..root_ast.children_with_tokens().count(), to_insert);
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

    let mut header_pos: Vec<(String, usize)> = header_to_pos
        .clone()
        .into_iter()
        .filter(|(_k, v)| v.iter().any(|p| !table_set.get(*p).unwrap().borrow().is_empty()))
        .map(|(k, v)| (k, *v.iter().min().unwrap()))
        .collect();

    // Calculate the first file position for each base key (e.g., tool.hatch)
    let mut base_key_first_pos: HashMap<String, usize> = HashMap::new();
    for (k, file_pos) in &header_pos {
        let base = get_key(k, multi_level_prefixes);
        base_key_first_pos
            .entry(base)
            .and_modify(|p| *p = (*p).min(*file_pos))
            .or_insert(*file_pos);
    }

    header_pos.sort_by(|(k1, _), (k2, _)| {
        let key1 = get_key(k1, multi_level_prefixes);
        let key2 = get_key(k2, multi_level_prefixes);
        let pos1 = key_to_pos.get(&key1.as_str());
        let pos2 = key_to_pos.get(&key2.as_str());

        match (pos1, pos2) {
            // Both in ordering list: sort by position, then alphabetically within same tool
            (Some(&p1), Some(&p2)) => {
                let offset1 = usize::from(key1 != *k1);
                let offset2 = usize::from(key2 != *k2);
                (p1 + offset1)
                    .cmp(&(p2 + offset2))
                    .then_with(|| k1.to_lowercase().cmp(&k2.to_lowercase()))
            }
            // Only first in ordering list: first comes before
            (Some(_), None) => std::cmp::Ordering::Less,
            // Only second in ordering list: second comes before
            (None, Some(_)) => std::cmp::Ordering::Greater,
            // Neither in ordering list: sort by first file position of the base key, then alphabetically
            (None, None) => {
                let base_pos1 = base_key_first_pos.get(&key1).unwrap_or(&usize::MAX);
                let base_pos2 = base_key_first_pos.get(&key2).unwrap_or(&usize::MAX);
                base_pos1
                    .cmp(base_pos2)
                    .then_with(|| k1.to_lowercase().cmp(&k2.to_lowercase()))
            }
        }
    });
    header_pos.into_iter().map(|(k, _)| k).collect()
}

fn get_key(k: &str, multi_level_prefixes: &[&str]) -> String {
    let parts: Vec<&str> = k.splitn(3, '.').collect();
    if !parts.is_empty() {
        // Check if the first part is in the multi-level prefixes list
        let is_multi_level = multi_level_prefixes.iter().any(|prefix| *prefix == parts[0]);
        return if is_multi_level && parts.len() >= 2 {
            parts[0..2].join(".")
        } else {
            String::from(parts[0])
        };
    }
    String::from(k)
}

pub fn reorder_table_keys(table: &mut RefMut<Vec<SyntaxElement>>, order: &[&str]) {
    let (size, mut to_insert) = (table.len(), Vec::<SyntaxElement>::new());
    let (key_to_position, key_set) = load_keys(table);
    let mut handled_positions = HashSet::<usize>::new();
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
        matching_keys.sort_by_key(|key| key.to_lowercase().replace('"', ""));
        for key in matching_keys {
            let position = key_to_position[key];
            to_insert.extend(key_set[position].clone());
            handled_positions.insert(position);
        }
    }
    for (position, entries) in key_set.into_iter().enumerate() {
        if !handled_positions.contains(&position) {
            to_insert.extend(entries);
        }
    }
    table.splice(0..size, to_insert);
}

fn load_keys(table: &[SyntaxElement]) -> (HashMap<String, usize>, Vec<Vec<SyntaxElement>>) {
    let table_clone = if table.last().unwrap().kind() == NEWLINE {
        // drop the final element if it is a new line, multiple new lines are handled together and add unwanted
        // empty lines within the table when reordered
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
        if kind == ENTRY {
            if cutoff {
                add_to_key_set(key.clone());
                cutoff = false;
            }
            for e in element.as_node().unwrap().children_with_tokens() {
                if e.kind() == KEY {
                    key = e.as_node().unwrap().text().to_string().trim().to_string();
                    break;
                }
            }
        }
        if [ENTRY, TABLE_HEADER, TABLE_ARRAY_HEADER].contains(&kind) {
            cutoff = true;
        }
        entry_set.borrow_mut().push(element.clone());
        if cutoff && kind == NEWLINE {
            add_to_key_set(key.clone());
            cutoff = false;
        }
    }
    add_to_key_set(key);
    (key_to_pos, key_set)
}

pub fn get_table_name(entry: &SyntaxElement) -> String {
    if [TABLE_HEADER, TABLE_ARRAY_HEADER].contains(&entry.kind()) {
        for child in entry.as_node().unwrap().children_with_tokens() {
            if child.kind() == KEY {
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
        if table_entry.kind() == ENTRY {
            for entry in table_entry.as_node().unwrap().children_with_tokens() {
                if entry.kind() == KEY {
                    key = entry.as_node().unwrap().text().to_string().trim().to_string();
                } else if entry.kind() == VALUE {
                    f(key.clone(), entry.as_node().unwrap());
                }
            }
        }
    }
}

pub fn find_key(table: &SyntaxNode, key: &str) -> Option<SyntaxNode> {
    let mut current_key = String::new();
    for table_entry in table.children_with_tokens() {
        if table_entry.kind() == ENTRY {
            for entry in table_entry.as_node().unwrap().children_with_tokens() {
                if entry.kind() == KEY {
                    current_key = entry.as_node().unwrap().text().to_string().trim().to_string();
                } else if entry.kind() == VALUE && current_key == key {
                    return Some(entry.as_node().unwrap().clone());
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
    if !tables.header_to_pos.contains_key(name) {
        tables
            .header_to_pos
            .insert(String::from(name), vec![tables.table_set.len()]);
        tables.table_set.push(RefCell::new(make_table_entry(name)));
    }
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

        // Skip array tables (those with TABLE_ARRAY_HEADER instead of TABLE_HEADER)
        let is_array_table = sub.iter().any(|child| child.kind() == TABLE_ARRAY_HEADER);
        if is_array_table {
            continue;
        }

        let sub_name = key.strip_prefix(sub_name_prefix.as_str()).unwrap();
        let mut header = false;
        for child in sub.iter() {
            let kind = child.kind();
            if kind == TABLE_HEADER {
                header = true;
                continue;
            }
            if header && kind == NEWLINE {
                continue;
            }
            if kind == ENTRY {
                let mut to_insert = Vec::<SyntaxElement>::new();
                let child_node = child.as_node().unwrap();
                for mut entry in child_node.children_with_tokens() {
                    if entry.kind() == KEY {
                        let mut key_parts = vec![String::from(sub_name)];
                        for array_entry_value in entry.as_node().unwrap().children_with_tokens() {
                            if array_entry_value.kind() == IDENT {
                                let txt = load_text(array_entry_value.as_token().unwrap().text(), IDENT);
                                key_parts.push(txt);
                            }
                        }
                        entry = make_key(&key_parts.join("."));
                    }
                    to_insert.push(entry);
                }
                child_node.splice_children(0..to_insert.len(), to_insert);
            }
            if main.last().unwrap().kind() != NEWLINE {
                main.push(make_newline());
            }
            main.push(child.clone());
        }
        sub.clear();
    }
}

/// Expand dotted keys in a table into separate sub-tables.
/// This is the reverse of `collapse_sub_tables`.
/// For example, `urls.homepage = "..."` becomes a `[project.urls]` table with `homepage = "..."`.
pub fn expand_sub_tables(tables: &mut Tables, name: &str) {
    let main_positions = match tables.header_to_pos.get(name) {
        Some(p) if !p.is_empty() => p.clone(),
        _ => return,
    };
    if main_positions.len() != 1 {
        return;
    }

    // Collect all dotted keys and group by first segment
    let mut groups: HashMap<String, Vec<(String, SyntaxElement)>> = HashMap::new();
    let mut entries_to_remove: HashSet<usize> = HashSet::new();

    {
        let main = tables.table_set[*main_positions.first().unwrap()].borrow();
        let mut entry_index = 0;

        for element in main.iter() {
            if element.kind() == ENTRY {
                let mut key_text = String::new();
                for child in element.as_node().unwrap().children_with_tokens() {
                    if child.kind() == KEY {
                        key_text = child.as_node().unwrap().text().to_string().trim().to_string();
                        break;
                    }
                }

                // Check if this is a dotted key (contains a dot)
                if let Some(dot_pos) = key_text.find('.') {
                    let prefix = &key_text[..dot_pos];
                    let rest = &key_text[dot_pos + 1..];

                    groups
                        .entry(String::from(prefix))
                        .or_default()
                        .push((String::from(rest), element.clone()));
                    entries_to_remove.insert(entry_index);
                }
                entry_index += 1;
            }
        }
    }

    if groups.is_empty() {
        return;
    }

    // Remove the dotted key entries from the main table
    {
        let mut main = tables.table_set[*main_positions.first().unwrap()].borrow_mut();
        let mut new_elements = Vec::new();
        let mut entry_index = 0;

        for element in main.iter() {
            if element.kind() == ENTRY {
                if !entries_to_remove.contains(&entry_index) {
                    new_elements.push(element.clone());
                }
                entry_index += 1;
            } else {
                new_elements.push(element.clone());
            }
        }

        // Remove trailing newlines
        while new_elements.last().is_some_and(|e| e.kind() == NEWLINE) {
            new_elements.pop();
        }
        // Add one newline at the end
        new_elements.push(make_newline());

        let main_len = main.len();
        main.splice(0..main_len, new_elements);
    }

    // Create new sub-tables for each group
    for (sub_name, entries) in groups {
        let full_name = format!("{name}.{sub_name}");

        // Create the new table
        let mut new_table = make_table_entry(&full_name);

        // Add entries with simplified keys
        for (simple_key, original_entry) in entries {
            // Rebuild the entry with the simplified key
            let entry_node = original_entry.as_node().unwrap();
            let mut value_text = String::new();

            for child in entry_node.children_with_tokens() {
                if child.kind() == VALUE {
                    value_text = child.as_node().unwrap().text().to_string();
                    break;
                }
            }

            // Create a new entry with simplified key
            let new_entry_text = format!("{simple_key} ={value_text}\n");
            let parsed = taplo::parser::parse(&new_entry_text);
            let parsed_root = parsed.into_syntax().clone_for_update();
            for child in parsed_root.children_with_tokens() {
                if child.kind() == ENTRY {
                    new_table.push(child);
                    break;
                }
            }
        }

        // Register the new table
        let pos = tables.table_set.len();
        tables.table_set.push(RefCell::new(new_table));
        tables.header_to_pos.entry(full_name).or_default().push(pos);
    }
}
