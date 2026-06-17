use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use tombi_syntax::SyntaxKind::{
    ARRAY, BASIC_STRING, BRACKET_END, BRACKET_START, COMMA, COMMENT, DOUBLE_BRACKET_END, DOUBLE_BRACKET_START,
    LINE_BREAK, LITERAL_STRING, VALUE_WITH_COMMA_GROUP, WHITESPACE,
};
use tombi_syntax::{SyntaxElement, SyntaxKind, SyntaxNode};

fn flat_array_children(array: &SyntaxNode) -> Vec<SyntaxElement> {
    let mut result = Vec::new();
    for child in array.children_with_tokens() {
        if child.kind() == VALUE_WITH_COMMA_GROUP {
            for inner in child.as_node().unwrap().children_with_tokens() {
                unwrap_leading_trivia(&mut result, inner);
            }
        } else {
            result.push(child);
        }
    }
    result
}

fn unwrap_leading_trivia(result: &mut Vec<SyntaxElement>, elem: SyntaxElement) {
    if !is_array_value(elem.kind()) || elem.as_node().is_none() {
        result.push(elem);
        return;
    }
    let node = elem.as_node().unwrap();
    let children: Vec<_> = node.children_with_tokens().collect();
    let leading_count = children
        .iter()
        .take_while(|c| {
            matches!(c.kind(), LINE_BREAK | WHITESPACE) || (c.kind() == COMMENT && is_group_marker(&c.to_string()))
        })
        .count();
    if leading_count == 0 {
        result.push(elem);
        return;
    }
    for child in &children[..leading_count] {
        result.push(child.clone());
    }
    let remaining: Vec<_> = children[leading_count..].to_vec();
    let trailing_comment_start = remaining
        .iter()
        .position(|c| matches!(c.kind(), WHITESPACE | COMMENT) && !is_array_value(c.kind()))
        .filter(|&pos| {
            remaining[pos..]
                .iter()
                .all(|c| matches!(c.kind(), WHITESPACE | COMMENT))
        });
    if let Some(tcs) = trailing_comment_start {
        node.splice_children(0..children.len(), remaining[..tcs].to_vec());
        result.push(elem);
        for child in &remaining[tcs..] {
            result.push(child.clone());
        }
    } else {
        node.splice_children(0..children.len(), remaining);
        result.push(elem);
    }
}

fn flatten_array_in_place(array: &SyntaxNode) {
    let count = array.children_with_tokens().count();
    let flat = flat_array_children(array);
    array.splice_children(0..count, flat);
}

use crate::create::{make_comma, make_newline, make_whitespace_n};
use crate::string::{load_text, update_content};
use crate::util::is_group_marker;

fn is_array_value(kind: SyntaxKind) -> bool {
    !matches!(
        kind,
        BRACKET_START
            | BRACKET_END
            | DOUBLE_BRACKET_START
            | DOUBLE_BRACKET_END
            | COMMA
            | LINE_BREAK
            | WHITESPACE
            | COMMENT
    )
}

fn has_trailing_comma(array: &SyntaxNode) -> bool {
    flat_array_children(array)
        .iter()
        .rfind(|x| x.kind() == COMMA || is_array_value(x.kind()))
        .is_some_and(|x| x.kind() == COMMA)
}

fn is_multiline(array: &SyntaxNode) -> bool {
    flat_array_children(array).iter().any(|x| x.kind() == LINE_BREAK)
}

fn add_trailing_comma_if_missing(array: &SyntaxNode) {
    flatten_array_in_place(array);
    if let Some((i, _)) = array
        .children_with_tokens()
        .enumerate()
        .filter(|(_, x)| is_array_value(x.kind()))
        .last()
    {
        array.splice_children(i + 1..i + 1, vec![make_comma()]);
    }
}

fn insert_newlines_around_content(array: &SyntaxNode) {
    flatten_array_in_place(array);
    if let Some((start_idx, _)) = array
        .children_with_tokens()
        .enumerate()
        .find(|(_, x)| x.kind() == BRACKET_START)
    {
        array.splice_children(start_idx + 1..start_idx + 1, vec![make_newline()]);
    }
    if let Some((end_idx, _)) = array
        .children_with_tokens()
        .enumerate()
        .find(|(_, x)| x.kind() == BRACKET_END)
    {
        array.splice_children(end_idx..end_idx, vec![make_newline()]);
    }
}

pub fn ensure_trailing_comma(array: &SyntaxNode) {
    flatten_array_in_place(array);
    if !array.children_with_tokens().any(|x| is_array_value(x.kind())) {
        return;
    }

    let has_trailing = has_trailing_comma(array);
    let multiline = is_multiline(array);

    if has_trailing && multiline {
        return;
    }

    if !has_trailing {
        add_trailing_comma_if_missing(array);
    }

    if !multiline {
        insert_newlines_around_content(array);
    }
}

pub fn ensure_all_arrays_multiline(root: &SyntaxNode, column_width: usize) {
    let arrays: Vec<_> = root.descendants().filter(|d| d.kind() == ARRAY).collect();
    for array in arrays.iter().rev() {
        ensure_array_multiline(array, column_width);
    }
}

fn ensure_array_multiline(array: &SyntaxNode, column_width: usize) {
    flatten_array_in_place(array);
    if !array.children_with_tokens().any(|x| is_array_value(x.kind())) {
        return;
    }

    let has_trailing = has_trailing_comma(array);
    if is_multiline(array) {
        return;
    }

    let has_comment_inside = has_comment_before_bracket_end(array);
    let exceeds_width = array.text().to_string().len() > column_width;
    if !has_trailing && !exceeds_width && !has_comment_inside {
        return;
    }

    if !has_trailing {
        add_trailing_comma_if_missing(array);
    }

    insert_newlines_around_content(array);
}

fn has_comment_before_bracket_end(array: &SyntaxNode) -> bool {
    let children: Vec<_> = flat_array_children(array);
    let mut seen_bracket_end = false;
    for child in children.iter().rev() {
        if child.kind() == BRACKET_END {
            seen_bracket_end = true;
        } else if seen_bracket_end && child.kind() == COMMENT {
            return true;
        } else if seen_bracket_end
            && is_array_value(child.kind())
            && let Some(node) = child.as_node()
            && node.descendants_with_tokens().any(|x| x.kind() == COMMENT)
        {
            return true;
        }
    }
    false
}

pub fn transform<F>(array: &SyntaxNode, transform: &F)
where
    F: Fn(&str) -> String,
{
    if array.kind() != ARRAY {
        return;
    }
    flatten_array_in_place(array);
    for entry in array.children_with_tokens() {
        if (entry.kind() == BASIC_STRING || entry.kind() == LITERAL_STRING)
            && let Some(string_node) = entry.as_node()
        {
            update_content(string_node, transform);
        }
    }
}

/// A run of array values bounded by `# Group:` markers. Values are reordered only within a group;
/// the marker `header` stays anchored at the group's start and groups keep their original order.
struct ValueGroup<T> {
    header: Vec<SyntaxElement>,
    order_sets: Vec<Vec<SyntaxElement>>,
    key_to_order_set: HashMap<T, usize>,
}

impl<T: Clone + Eq + Hash> ValueGroup<T> {
    fn new(header: Vec<SyntaxElement>) -> Self {
        Self {
            header,
            order_sets: Vec::new(),
            key_to_order_set: HashMap::new(),
        }
    }

    fn add(&mut self, key: T, set: &mut Vec<SyntaxElement>) {
        if let Some(&existing_idx) = self.key_to_order_set.get(&key) {
            self.order_sets[existing_idx].append(set);
        } else {
            self.key_to_order_set.insert(key, self.order_sets.len());
            self.order_sets.push(std::mem::take(set));
        }
    }

    fn emit<C: Fn(&T, &T) -> Ordering>(mut self, entries: &mut Vec<SyntaxElement>, cmp: &C) {
        entries.extend(self.header);
        for set in &mut self.order_sets {
            let has_comma = set.iter().any(|e| e.kind() == COMMA);
            if !has_comma && let Some(pos) = set.iter().position(|e| is_array_value(e.kind())) {
                set.insert(pos + 1, make_comma());
            }
        }
        let mut order: Vec<T> = self.key_to_order_set.keys().cloned().collect();
        order.sort_by(cmp);
        for key in order {
            entries.extend(self.order_sets[self.key_to_order_set[&key]].clone());
        }
    }
}

#[allow(clippy::range_plus_one, clippy::too_many_lines)]
pub fn sort<T, K, C>(array: &SyntaxNode, to_key: K, cmp: &C)
where
    K: Fn(&SyntaxNode) -> Option<T>,
    C: Fn(&T, &T) -> Ordering,
    T: Clone + Eq + Hash,
{
    if array.kind() != ARRAY {
        return;
    }
    flatten_array_in_place(array);
    let has_trailing_comma = array
        .children_with_tokens()
        .map(|x| x.kind())
        .filter(|x| *x == COMMA || is_array_value(*x))
        .last()
        == Some(COMMA);

    let mut entries = Vec::<SyntaxElement>::new();
    let mut groups = vec![ValueGroup::<T>::new(Vec::new())];
    let mut current_set = Vec::<SyntaxElement>::new();
    let mut current_set_value: Option<T> = None;
    let mut after_bracket_start = false;
    let mut pending_marker = false;

    let mut count = 0;

    for entry in array.children_with_tokens() {
        count += 1;
        if after_bracket_start {
            if entry.kind() == LINE_BREAK {
                entries.push(entry);
                continue;
            }
            after_bracket_start = false;
        }
        match entry.kind() {
            BRACKET_START => {
                entries.push(entry);
                after_bracket_start = true;
            }
            BRACKET_END => {
                match current_set_value.take() {
                    None => entries.append(&mut current_set),
                    Some(val) => groups.last_mut().unwrap().add(val, &mut current_set),
                }
                entries.push(entry);
            }
            LINE_BREAK => {
                current_set.push(entry);
                if pending_marker {
                    groups.push(ValueGroup::new(std::mem::take(&mut current_set)));
                    pending_marker = false;
                } else if let Some(val) = current_set_value.take() {
                    groups.last_mut().unwrap().add(val, &mut current_set);
                }
            }
            COMMA => {
                let has_comment = entry
                    .as_node()
                    .is_some_and(|n| n.children_with_tokens().any(|c| c.kind() == COMMENT));
                if has_comment {
                    current_set.push(entry);
                } else {
                    current_set.push(make_comma());
                }
            }
            COMMENT if is_group_marker(&entry.to_string()) => {
                current_set.push(entry);
                pending_marker = true;
            }
            kind if is_array_value(kind) => {
                if let Some(val) = current_set_value.take() {
                    groups.last_mut().unwrap().add(val, &mut current_set);
                }
                if let Some(value_node) = entry.as_node() {
                    current_set_value = to_key(value_node);
                }
                current_set.push(entry);
            }
            _ => current_set.push(entry),
        }
    }

    let remaining: Vec<SyntaxElement> = std::mem::take(&mut current_set);

    let leading_count = entries
        .iter()
        .take_while(|e| matches!(e.kind(), BRACKET_START | LINE_BREAK))
        .count();
    let trailing_content = entries.split_off(leading_count);

    for group in groups {
        group.emit(&mut entries, cmp);
    }
    entries.extend(trailing_content);
    entries.extend(remaining);
    array.splice_children(0..count, entries);

    if !has_trailing_comma {
        let now_has_trailing = array
            .children_with_tokens()
            .filter(|x| x.kind() == COMMA || is_array_value(x.kind()))
            .last()
            .is_some_and(|x| x.kind() == COMMA);
        if now_has_trailing
            && let Some((i, _)) = array
                .children_with_tokens()
                .enumerate()
                .filter(|(_, x)| x.kind() == COMMA)
                .last()
        {
            array.splice_children(i..i + 1, vec![]);
        }
    }
}

#[allow(clippy::range_plus_one, clippy::too_many_lines)]
pub fn sort_strings<T, K, C>(node: &SyntaxNode, to_key: K, cmp: &C)
where
    K: Fn(String) -> String,
    C: Fn(&String, &String) -> Ordering,
    T: Clone + Eq + Hash,
{
    sort(
        node,
        |e| {
            let kind = e.kind();
            (kind == BASIC_STRING || kind == LITERAL_STRING).then(|| {
                e.descendants_with_tokens()
                    .filter_map(|elem| elem.into_token())
                    .find(|token| token.kind() == kind)
                    .map(|token| to_key(load_text(token.text(), kind)))
            })?
        },
        cmp,
    );
}

/// Remove duplicate string entries from an array (case-insensitive comparison)
pub fn dedupe_strings<K>(array: &SyntaxNode, to_key: K)
where
    K: Fn(&str) -> String,
{
    let mut seen: HashSet<String> = HashSet::new();
    remove_strings(array, |s| !seen.insert(to_key(s)));
}

/// Remove string entries from an array for which `predicate` returns true
pub fn remove_strings<P>(array: &SyntaxNode, mut predicate: P)
where
    P: FnMut(&str) -> bool,
{
    if array.kind() != ARRAY {
        return;
    }
    flatten_array_in_place(array);
    let mut to_insert: Vec<SyntaxElement> = Vec::new();
    let mut skip_until_next_value = false;
    let count = array.children_with_tokens().count();

    for entry in array.children_with_tokens() {
        let kind = entry.kind();
        if is_array_value(kind) {
            let text = if kind == BASIC_STRING || kind == LITERAL_STRING {
                entry
                    .as_node()
                    .and_then(|n| {
                        n.descendants_with_tokens()
                            .filter_map(|e| e.into_token())
                            .find(|token| token.kind() == kind)
                    })
                    .map(|token| load_text(token.text(), kind))
            } else {
                None
            };
            if let Some(value) = text
                && predicate(&value)
            {
                skip_until_next_value = true;
                continue;
            }
            skip_until_next_value = false;
            to_insert.push(entry);
        } else {
            match kind {
                COMMA | LINE_BREAK | WHITESPACE if skip_until_next_value => {
                    continue;
                }
                BRACKET_END if skip_until_next_value => {
                    while let Some(last) = to_insert.last() {
                        if last.kind() == COMMA || last.kind() == LINE_BREAK || last.kind() == WHITESPACE {
                            to_insert.pop();
                        } else {
                            break;
                        }
                    }
                    to_insert.push(entry);
                }
                _ => {
                    to_insert.push(entry);
                }
            }
        }
    }
    array.splice_children(0..count, to_insert);
}

pub fn align_array_comments(root: &SyntaxNode) {
    let arrays: Vec<_> = root.descendants().filter(|d| d.kind() == ARRAY).collect();
    for array in arrays.iter().rev() {
        align_comments_in_array(array);
    }
}

fn align_comments_in_array(array: &SyntaxNode) {
    flatten_array_in_place(array);
    let mut max_value_len = 0;
    let elements: Vec<_> = array.children_with_tokens().collect();
    let mut value_indices = Vec::new();

    for (i, child) in elements.iter().enumerate() {
        if (child.kind() == BASIC_STRING || child.kind() == LITERAL_STRING)
            && let Some(node) = child.as_node()
            && let Some(token) = node
                .descendants_with_tokens()
                .filter_map(|e| e.into_token())
                .find(|token| token.kind() == child.kind())
        {
            let text = load_text(token.text(), child.kind());
            let quoted_len = text.len() + 2 + 1;
            max_value_len = max_value_len.max(quoted_len);

            let mut j = i + 1;
            let mut comma_idx = None;

            while j < elements.len() {
                match elements[j].kind() {
                    COMMA => {
                        if let Some(comma_node) = elements[j].as_node() {
                            let has_comment = comma_node.children_with_tokens().any(|child| child.kind() == COMMENT);
                            if has_comment {
                                comma_idx = Some(j);
                                break;
                            }
                        }
                        j += 1;
                    }
                    WHITESPACE => j += 1,
                    _ => break,
                }
            }

            if let Some(idx) = comma_idx {
                value_indices.push((i, quoted_len, idx));
            }
        }
    }

    if value_indices.is_empty() {
        return;
    }

    for &(_value_idx, value_len, comma_idx) in &value_indices {
        if let Some(comma_elem) = elements.get(comma_idx)
            && let Some(comma_node) = comma_elem.as_node()
        {
            let comma_children: Vec<_> = comma_node.children_with_tokens().collect();
            let spaces_needed = max_value_len - value_len + 1;

            let mut new_comma_children = Vec::new();
            for child in comma_children {
                if child.kind() == WHITESPACE {
                    new_comma_children.push(make_whitespace_n(spaces_needed));
                } else {
                    new_comma_children.push(child);
                }
            }

            let count = comma_node.children_with_tokens().count();
            comma_node.splice_children(0..count, new_comma_children);
        }
    }
}
