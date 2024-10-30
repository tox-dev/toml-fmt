use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::Hash;
use taplo::syntax::SyntaxKind::{ARRAY, COMMA, NEWLINE, STRING, VALUE, WHITESPACE};
use taplo::syntax::{SyntaxElement, SyntaxKind, SyntaxNode};

use crate::create::{make_comma, make_newline};
use crate::string::{load_text, update_content};
use crate::util::{find_first, iter};

pub fn transform<F>(node: &SyntaxNode, transform: &F)
where
    F: Fn(&str) -> String,
{
    iter(node, [ARRAY, VALUE].as_ref(), &|array_entry| {
        update_content(array_entry, transform);
    });
}

#[allow(clippy::range_plus_one, clippy::too_many_lines)]
pub fn sort<T, K, C>(node: &SyntaxNode, to_key: K, cmp: &C)
where
    K: Fn(&SyntaxNode) -> Option<T>,
    C: Fn(&T, &T) -> Ordering,
    T: Clone + Eq + Hash,
{
    iter(node, [ARRAY].as_ref(), &|array| {
        let has_trailing_comma = array
            .children_with_tokens()
            .map(|x| x.kind())
            .filter(|x| *x == COMMA || *x == VALUE)
            .last()
            == Some(COMMA);
        let multiline = array.children_with_tokens().any(|e| e.kind() == NEWLINE);

        let mut entries = Vec::<SyntaxElement>::new();
        let mut order_sets = Vec::<Vec<SyntaxElement>>::new();
        let mut key_to_order_set = HashMap::<T, usize>::new();
        let current_set = RefCell::new(Vec::<SyntaxElement>::new());
        let mut current_set_value: Option<T> = None;
        let mut previous_is_bracket_open = false;

        let mut add_to_order_sets = |entry: T| {
            let mut entry_set_borrow = current_set.borrow_mut();
            if !entry_set_borrow.is_empty() {
                key_to_order_set.insert(entry, order_sets.len());
                order_sets.push(entry_set_borrow.clone());
                entry_set_borrow.clear();
            }
        };

        let mut count = 0;

        // collect elements to order into to_order_sets, the rest goes into entries
        for entry in array.children_with_tokens() {
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
                    match current_set_value.take() {
                        None => {
                            entries.extend(current_set.borrow_mut().clone());
                        }
                        Some(val) => {
                            add_to_order_sets(val);
                        }
                    }
                    entries.push(entry);
                }
                VALUE => {
                    match current_set_value.take() {
                        None => {}
                        Some(val) => {
                            if multiline {
                                current_set.borrow_mut().push(make_newline());
                            }
                            add_to_order_sets(val);
                        }
                    }
                    let value_node = entry.as_node().unwrap();
                    current_set_value = to_key(value_node);

                    current_set.borrow_mut().push(entry);
                    current_set.borrow_mut().push(make_comma());
                }
                NEWLINE => {
                    current_set.borrow_mut().push(entry);
                    if current_set_value.is_some() {
                        add_to_order_sets(current_set_value.unwrap());
                        current_set_value = None;
                    }
                }
                COMMA => {}
                _ => {
                    current_set.borrow_mut().push(entry);
                }
            }
        }

        let trailing_content = entries.split_off(if multiline { 2 } else { 1 });
        let mut order: Vec<T> = key_to_order_set.keys().cloned().collect();
        order.sort_by(&cmp);
        for key in order {
            entries.extend(order_sets[key_to_order_set[&key]].clone());
        }
        entries.extend(trailing_content);
        array.splice_children(0..count, entries);

        if !has_trailing_comma {
            if let Some((i, _)) = array
                .children_with_tokens()
                .enumerate()
                .filter(|(_, x)| x.kind() == COMMA)
                .last()
            {
                array.splice_children(i..i + 1, vec![]);
            }
        }
    });
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
        |e| -> Option<String> {
            find_first(e, &[STRING], &|s| -> String {
                to_key(load_text(s.as_token().unwrap().text(), STRING))
            })
        },
        cmp,
    );
}
