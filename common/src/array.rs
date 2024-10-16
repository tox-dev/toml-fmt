use std::cell::RefCell;
use std::collections::HashMap;

use lexical_sort::{natural_lexical_cmp, StringSort};
use taplo::syntax::SyntaxKind::{ARRAY, COMMA, NEWLINE, STRING, VALUE, WHITESPACE};
use taplo::syntax::{SyntaxElement, SyntaxKind, SyntaxNode};

use crate::create::{make_comma, make_newline};
use crate::string::{load_text, update_content};

pub fn transform<F>(node: &SyntaxNode, transform: &F)
where
    F: Fn(&str) -> String,
{
    for array in node.children_with_tokens() {
        if array.kind() == ARRAY {
            for array_entry in array.as_node().unwrap().children_with_tokens() {
                if array_entry.kind() == VALUE {
                    update_content(array_entry.as_node().unwrap(), transform);
                }
            }
        }
    }
}

#[allow(clippy::range_plus_one, clippy::too_many_lines)]
pub fn sort<F>(node: &SyntaxNode, transform: F)
where
    F: Fn(&str) -> String,
{
    for array in node.children_with_tokens() {
        if array.kind() == ARRAY {
            let array_node = array.as_node().unwrap();
            let has_trailing_comma = array_node
                .children_with_tokens()
                .map(|x| x.kind())
                .filter(|x| *x == COMMA || *x == VALUE)
                .last()
                == Some(COMMA);
            let multiline = array_node.children_with_tokens().any(|e| e.kind() == NEWLINE);
            let mut value_set = Vec::<Vec<SyntaxElement>>::new();
            let entry_set = RefCell::new(Vec::<SyntaxElement>::new());
            let mut key_to_pos = HashMap::<String, usize>::new();

            let mut add_to_value_set = |entry: String| {
                let mut entry_set_borrow = entry_set.borrow_mut();
                if !entry_set_borrow.is_empty() {
                    key_to_pos.insert(entry, value_set.len());
                    value_set.push(entry_set_borrow.clone());
                    entry_set_borrow.clear();
                }
            };
            let mut entries = Vec::<SyntaxElement>::new();
            let mut has_value = false;
            let mut previous_is_bracket_open = false;
            let mut entry_value = String::new();
            let mut count = 0;

            for entry in array_node.children_with_tokens() {
                count += 1;
                if previous_is_bracket_open {
                    // make sure ends with trailing comma
                    if entry.kind() == NEWLINE || entry.kind() == WHITESPACE {
                        continue;
                    }
                    previous_is_bracket_open = false;
                }
                match &entry.kind() {
                    SyntaxKind::BRACKET_START => {
                        entries.push(entry);
                        if multiline {
                            entries.push(make_newline());
                        }
                        previous_is_bracket_open = true;
                    }
                    SyntaxKind::BRACKET_END => {
                        if has_value {
                            add_to_value_set(entry_value.clone());
                        } else {
                            entries.extend(entry_set.borrow_mut().clone());
                        }
                        entries.push(entry);
                    }
                    VALUE => {
                        if has_value {
                            if multiline {
                                entry_set.borrow_mut().push(make_newline());
                            }
                            add_to_value_set(entry_value.clone());
                        }
                        has_value = true;
                        let value_node = entry.as_node().unwrap();
                        let mut found_string = false;
                        for child in value_node.children_with_tokens() {
                            let kind = child.kind();
                            if kind == STRING {
                                entry_value = transform(load_text(child.as_token().unwrap().text(), STRING).as_str());
                                found_string = true;
                                break;
                            }
                        }
                        if !found_string {
                            // abort if not correct types
                            return;
                        }
                        entry_set.borrow_mut().push(entry);
                        entry_set.borrow_mut().push(make_comma());
                    }
                    NEWLINE => {
                        entry_set.borrow_mut().push(entry);
                        if has_value {
                            add_to_value_set(entry_value.clone());
                            has_value = false;
                        }
                    }
                    COMMA => {}
                    _ => {
                        entry_set.borrow_mut().push(entry);
                    }
                }
            }

            let mut order: Vec<String> = key_to_pos.clone().into_keys().collect();
            order.string_sort_unstable(natural_lexical_cmp);
            let end = entries.split_off(if multiline { 2 } else { 1 });
            for key in order {
                entries.extend(value_set[key_to_pos[&key]].clone());
            }
            entries.extend(end);
            array_node.splice_children(0..count, entries);
            if !has_trailing_comma {
                if let Some((i, _)) = array_node
                    .children_with_tokens()
                    .enumerate()
                    .filter(|(_, x)| x.kind() == COMMA)
                    .last()
                {
                    array_node.splice_children(i..i + 1, vec![]);
                }
            }
        }
    }
}
