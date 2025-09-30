use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::iter::zip;
use std::ops::Index;

use taplo::syntax::SyntaxKind::{ENTRY, IDENT, KEY, NEWLINE, TABLE_ARRAY_HEADER, TABLE_HEADER, VALUE};
use taplo::syntax::{SyntaxElement, SyntaxNode};
use taplo::HashSet;

use crate::create::{make_empty_newline, make_key, make_newline, make_table_entry};
use crate::string::load_text;

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
                let table_name = get_table_name(&entry_set_borrow[0]);
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
                add_to_table_set(table_kind);
                table_kind = c.kind();
            }
            entry_set.borrow_mut().push(c);
        }
        add_to_table_set(table_kind);
        Self {
            header_to_pos,
            table_set,
        }
    }

    pub fn reorder(&self, root_ast: &SyntaxNode, order: &[&str]) {
        let mut to_insert = Vec::<SyntaxElement>::new();
        let order = calculate_order(&self.header_to_pos, &self.table_set, order);
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
                    if get_key(name) != get_key(next_name) {
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
) -> Vec<String> {
    let max_ordering = ordering.len() * 2;
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

    header_pos.sort_by_cached_key(|(k, file_pos)| -> (usize, usize) {
        let key = get_key(k);
        let pos = key_to_pos.get(&key.as_str());

        (
            if let Some(&pos) = pos {
                let offset = usize::from(key != *k);
                pos + offset
            } else {
                max_ordering
            },
            *file_pos,
        )
    });
    header_pos.into_iter().map(|(k, _)| k).collect()
}

fn get_key(k: &str) -> String {
    let parts: Vec<&str> = k.splitn(3, '.').collect();
    if !parts.is_empty() {
        return if parts[0] == "tool" && parts.len() >= 2 {
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
