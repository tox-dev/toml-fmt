use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use tombi_syntax::SyntaxKind::{
    ARRAY, BASIC_STRING, BRACKET_END, BRACKET_START, COMMA, COMMENT, DOUBLE_BRACKET_END, DOUBLE_BRACKET_START,
    LINE_BREAK, LITERAL_STRING, WHITESPACE,
};
use tombi_syntax::{SyntaxElement, SyntaxKind, SyntaxNode};

use crate::create::{make_comma, make_newline, make_whitespace_n};
use crate::string::{load_text, update_content};

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
    array
        .children_with_tokens()
        .filter(|x| x.kind() == COMMA || is_array_value(x.kind()))
        .last()
        .is_some_and(|x| x.kind() == COMMA)
}

fn is_multiline(array: &SyntaxNode) -> bool {
    array.children_with_tokens().any(|x| x.kind() == LINE_BREAK)
}

fn add_trailing_comma_if_missing(array: &SyntaxNode) {
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
    for descendant in root.descendants() {
        if descendant.kind() == ARRAY {
            ensure_array_multiline(&descendant, column_width);
        }
    }
}

fn ensure_array_multiline(array: &SyntaxNode, column_width: usize) {
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
    let children: Vec<_> = array.children_with_tokens().collect();
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
    for entry in array.children_with_tokens() {
        if (entry.kind() == BASIC_STRING || entry.kind() == LITERAL_STRING)
            && let Some(string_node) = entry.as_node()
        {
            update_content(string_node, transform);
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
    let has_trailing_comma = array
        .children_with_tokens()
        .map(|x| x.kind())
        .filter(|x| *x == COMMA || is_array_value(*x))
        .last()
        == Some(COMMA);
    let multiline = array.children_with_tokens().any(|e| e.kind() == LINE_BREAK);

    let mut entries = Vec::<SyntaxElement>::new();
    let mut order_sets = Vec::<Vec<SyntaxElement>>::new();
    let mut key_to_order_set = HashMap::<T, usize>::new();
    let current_set = RefCell::new(Vec::<SyntaxElement>::new());
    let mut current_set_value: Option<T> = None;
    let mut after_bracket_start = false;

    let mut add_to_order_sets = |entry: T| {
        let mut entry_set_borrow = current_set.borrow_mut();
        if !entry_set_borrow.is_empty() {
            if let Some(&existing_idx) = key_to_order_set.get(&entry) {
                order_sets[existing_idx].extend(entry_set_borrow.clone());
            } else {
                key_to_order_set.insert(entry, order_sets.len());
                order_sets.push(entry_set_borrow.clone());
            }
            entry_set_borrow.clear();
        }
    };

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
        match &entry.kind() {
            BRACKET_START => {
                entries.push(entry);
                after_bracket_start = true;
            }
            BRACKET_END => {
                match current_set_value.take() {
                    None => {
                        entries.extend(current_set.borrow_mut().drain(..));
                    }
                    Some(val) => {
                        add_to_order_sets(val);
                    }
                }
                entries.push(entry);
            }
            LINE_BREAK => {
                current_set.borrow_mut().push(entry);
                if current_set_value.is_some() {
                    add_to_order_sets(current_set_value.take().unwrap());
                }
            }
            COMMA => {
                let has_comment = entry
                    .as_node()
                    .is_some_and(|n| n.children_with_tokens().any(|c| c.kind() == COMMENT));
                if has_comment {
                    current_set.borrow_mut().push(entry);
                } else {
                    current_set.borrow_mut().push(make_comma());
                }
            }
            kind if is_array_value(*kind) => {
                match current_set_value.take() {
                    None => {}
                    Some(val) => {
                        add_to_order_sets(val);
                    }
                }
                if let Some(value_node) = entry.as_node() {
                    current_set_value = to_key(value_node);
                }

                current_set.borrow_mut().push(entry);
            }
            _ => {
                current_set.borrow_mut().push(entry);
            }
        }
    }

    let remaining: Vec<SyntaxElement> = current_set.borrow_mut().drain(..).collect();

    let trailing_content = entries.split_off(if multiline { 2 } else { 1 });
    let mut order: Vec<T> = key_to_order_set.keys().cloned().collect();
    order.sort_by(&cmp);

    for set in &mut order_sets {
        let has_comma = set.iter().any(|e| e.kind() == COMMA);
        if !has_comma && let Some(pos) = set.iter().position(|e| is_array_value(e.kind())) {
            set.insert(pos + 1, make_comma());
        }
    }

    for key in order {
        entries.extend(order_sets[key_to_order_set[&key]].clone());
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
    let mut to_insert: Vec<SyntaxElement> = Vec::new();
    let mut skip_until_next_value = false;
    let count = array.children_with_tokens().count();

    for entry in array.children_with_tokens() {
        let kind = entry.kind();
        if is_array_value(kind) {
            let key = if kind == BASIC_STRING || kind == LITERAL_STRING {
                entry
                    .as_node()
                    .and_then(|n| {
                        n.descendants_with_tokens()
                            .filter_map(|e| e.into_token())
                            .find(|token| token.kind() == kind)
                    })
                    .map(|token| to_key(&load_text(token.text(), kind)))
            } else {
                None
            };
            if let Some(k) = key {
                if seen.contains(&k) {
                    skip_until_next_value = true;
                    continue;
                }
                seen.insert(k);
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
    for descendant in root.descendants() {
        if descendant.kind() == ARRAY {
            align_comments_in_array(&descendant);
        }
    }
}

fn align_comments_in_array(array: &SyntaxNode) {
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
