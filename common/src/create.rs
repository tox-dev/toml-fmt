//! Parsing each snippet costs more than hand-building nodes, but buys valid syntax and correct escape
//! handling for free, since the parser owns those rules.

use tombi_syntax::SyntaxKind::{
    ARRAY, BASIC_STRING, COMMA, COMMENT, KEY_VALUE_GROUP, KEYS, LINE_BREAK, LITERAL_STRING, MULTI_LINE_BASIC_STRING,
    MULTI_LINE_LITERAL_STRING, VALUE_WITH_COMMA_GROUP, WHITESPACE,
};
use tombi_syntax::{SyntaxElement, SyntaxNode};

fn escape(text: &str) -> String {
    let escaped = tombi_toml_text::to_basic_string(text);
    escaped[1..escaped.len() - 1].to_string()
}

fn parse(source: &str) -> SyntaxNode {
    tombi_parser::parse(source).syntax_node().clone_for_update()
}

fn first_key_value(root: &SyntaxNode) -> SyntaxNode {
    let first = root.first_child().expect("parsed TOML has children");
    if first.kind() == KEY_VALUE_GROUP {
        first.first_child().expect("KEY_VALUE_GROUP has KEY_VALUE")
    } else {
        first
    }
}

fn find_in_array(array_node: &SyntaxNode, target: tombi_syntax::SyntaxKind) -> Option<SyntaxElement> {
    for child in array_node.children_with_tokens() {
        if child.kind() == target {
            return Some(child);
        }
        if child.kind() == VALUE_WITH_COMMA_GROUP {
            for inner in child.as_node().unwrap().children_with_tokens() {
                if inner.kind() == target {
                    return Some(inner);
                }
            }
        }
    }
    None
}

pub fn make_multiline_string_node(wrapped: &str) -> SyntaxElement {
    let expr = format!("a = {wrapped}");
    let root = parse(&expr);
    first_key_value(&root)
        .children_with_tokens()
        .find(|n| n.kind() == MULTI_LINE_BASIC_STRING)
        .expect("KEY_VALUE contains MULTI_LINE_BASIC_STRING")
}

/// `text` is inserted verbatim between the `'''` delimiters; callers must ensure it does not contain `'''`.
pub fn make_multiline_literal_string_node(text: &str) -> SyntaxElement {
    let leading = if text.starts_with(['\n', '\r']) { "\n" } else { "" };
    let expr = format!("a = '''{leading}{text}'''");
    let root = parse(&expr);
    first_key_value(&root)
        .children_with_tokens()
        .find(|n| n.kind() == MULTI_LINE_LITERAL_STRING)
        .expect("KEY_VALUE contains MULTI_LINE_LITERAL_STRING")
}

pub fn make_string_node(text: &str) -> SyntaxElement {
    let escaped = escape(text);
    let expr = format!("a = \"{escaped}\"");
    let root = parse(&expr);
    first_key_value(&root)
        .children_with_tokens()
        .find(|n| n.kind() == BASIC_STRING)
        .expect("KEY_VALUE contains BASIC_STRING")
}

/// `text` must not contain a single quote, which literal strings cannot escape.
pub fn make_literal_string_node(text: &str) -> SyntaxElement {
    let expr = format!("a = '{text}'");
    let root = parse(&expr);
    first_key_value(&root)
        .children_with_tokens()
        .find(|n| n.kind() == LITERAL_STRING)
        .expect("KEY_VALUE contains LITERAL_STRING")
}

/// Returns two LINE_BREAK elements: tombi models each newline as its own token.
pub fn make_empty_newline() -> Vec<SyntaxElement> {
    parse("\n\n")
        .children_with_tokens()
        .filter(|n| n.kind() == LINE_BREAK)
        .collect()
}

pub fn make_newline() -> SyntaxElement {
    parse("\n")
        .children_with_tokens()
        .find(|n| n.kind() == LINE_BREAK)
        .expect("parsed newline contains LINE_BREAK")
}

pub fn make_comment(text: &str) -> SyntaxElement {
    let src = format!("{text}\na = 1\n");
    let root = parse(&src);
    for c in root.descendants_with_tokens() {
        if c.kind() == COMMENT {
            return c;
        }
    }
    unreachable!("parsed comment TOML contains COMMENT")
}

pub fn make_comma() -> SyntaxElement {
    let root = parse("a=[1,2]");
    let array_node = first_key_value(&root)
        .children_with_tokens()
        .find(|n| n.kind() == ARRAY)
        .expect("KEY_VALUE has ARRAY");
    find_in_array(array_node.as_node().unwrap(), COMMA).expect("ARRAY contains COMMA")
}

pub fn make_whitespace_n(count: usize) -> SyntaxElement {
    let spaces = " ".repeat(count.max(1));
    let sample = format!("a=[1,{}2]", spaces);
    let root = parse(&sample);
    let array_node = first_key_value(&root)
        .children_with_tokens()
        .find(|n| n.kind() == ARRAY)
        .expect("KEY_VALUE has ARRAY");
    find_in_array(array_node.as_node().unwrap(), WHITESPACE).expect("ARRAY contains WHITESPACE")
}

pub fn make_key(text: &str) -> SyntaxElement {
    let root = parse(format!("{text}=1").as_str());
    first_key_value(&root)
        .children_with_tokens()
        .find(|n| n.kind() == KEYS)
        .expect("KEY_VALUE has KEYS")
}

pub fn make_array(key: &str) -> SyntaxElement {
    let txt = format!("{key} = []");
    let root = parse(txt.as_str());
    SyntaxElement::Node(first_key_value(&root))
}

pub fn make_array_entry(key: &str) -> SyntaxElement {
    let txt = format!("a = [\"{key}\"]");
    let root = parse(txt.as_str());
    let array_node = first_key_value(&root)
        .children_with_tokens()
        .find(|n| n.kind() == ARRAY)
        .expect("KEY_VALUE has ARRAY");
    find_in_array(array_node.as_node().unwrap(), BASIC_STRING).expect("ARRAY contains BASIC_STRING")
}

pub fn make_entry_of_string(key: &String, value: &String) -> SyntaxElement {
    let txt = format!("{key} = \"{value}\"\n");
    let root = parse(txt.as_str());
    SyntaxElement::Node(first_key_value(&root))
}

pub fn make_empty_inline_table(key: &str) -> SyntaxElement {
    let txt = format!("{key} = {{}}\n");
    let root = parse(txt.as_str());
    SyntaxElement::Node(first_key_value(&root))
}

pub fn make_table_entry(key: &str) -> Vec<SyntaxElement> {
    let txt = format!("[{key}]\n");
    parse(txt.as_str()).children_with_tokens().collect()
}

pub fn make_entry_with_array_of_inline_tables(key: &str, inline_tables: &[String]) -> SyntaxElement {
    let tables_str = inline_tables.join(", ");
    let txt = format!("{key} = [{tables_str}]\n");
    let root = parse(txt.as_str());
    SyntaxElement::Node(first_key_value(&root))
}

pub fn make_table_array_entry(key: &str) -> Vec<SyntaxElement> {
    let txt = format!("[[{key}]]\n");
    parse(txt.as_str()).children_with_tokens().collect()
}

pub fn make_table_array_with_entries(key: &str, entries: &[(String, String)]) -> Vec<SyntaxElement> {
    let entries_str = entries
        .iter()
        .map(|(k, v)| format!("{k} = {v}"))
        .collect::<Vec<_>>()
        .join("\n");
    let txt = format!("[[{key}]]\n{entries_str}\n");
    parse(txt.as_str()).children_with_tokens().collect()
}
