use tombi_config::TomlVersion;
use tombi_syntax::SyntaxKind::{
    BARE_KEY, BASIC_STRING, INLINE_TABLE, KEY, LITERAL_STRING, MULTI_LINE_BASIC_STRING, MULTI_LINE_LITERAL_STRING,
};
use tombi_syntax::{SyntaxElement, SyntaxKind, SyntaxNode};

fn is_string_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        BASIC_STRING | LITERAL_STRING | MULTI_LINE_BASIC_STRING | MULTI_LINE_LITERAL_STRING
    )
}

use crate::create::{make_literal_string_node, make_multiline_string_node, make_string_node};

fn escape(text: &str) -> String {
    let escaped = tombi_toml_text::to_basic_string(text);
    escaped[1..escaped.len() - 1].to_string()
}

fn unescape(content: &str) -> Result<String, tombi_toml_text::ParseError> {
    let quoted = format!("\"{content}\"");
    tombi_toml_text::try_from_basic_string(&quoted, TomlVersion::default())
}

fn is_inside_inline_table(node: &SyntaxNode) -> bool {
    let mut current = node.parent();
    while let Some(parent) = current {
        if parent.kind() == INLINE_TABLE {
            return true;
        }
        current = parent.parent();
    }
    false
}

fn get_key_prefix_len(value_node: &SyntaxNode) -> usize {
    if let Some(entry_node) = value_node.parent() {
        for child in entry_node.children_with_tokens() {
            if child.kind() == KEY {
                return child.as_node().map_or(0, |n| n.text().to_string().len()) + 3;
            }
        }
    }
    0
}

fn can_use_literal_string(s: &str) -> bool {
    !s.contains('\'') && !s.chars().any(|c| c.is_control() && c != '\t')
}

fn wrap_string_with_continuations(text: &str, max_line_len: usize, indent: &str) -> String {
    let escaped = escape(text);
    let mut result = String::from("\"\"\"\\\n");
    let mut line_start = 0;
    let effective_width = max_line_len.saturating_sub(indent.len() + 1).max(10);

    while line_start < escaped.len() {
        let remaining = &escaped[line_start..];
        if remaining.len() + indent.len() <= max_line_len {
            result.push_str(indent);
            result.push_str(remaining);
            result.push_str("\\\n");
            break;
        }
        let chunk_end = effective_width.min(remaining.len());
        let split_at = find_best_wrap_point(remaining, chunk_end);
        result.push_str(indent);
        result.push_str(&remaining[..split_at]);
        result.push_str("\\\n");
        line_start += split_at;
    }
    result.push_str(indent);
    result.push_str("\"\"\"");
    result
}

fn find_best_wrap_point(text: &str, max_len: usize) -> usize {
    if max_len >= text.len() {
        return text.len();
    }
    let search_text = &text[..max_len];
    if let Some(pos) = search_text.rfind(" :: ") {
        return (pos + 4).min(text.len());
    }
    search_text.rfind(' ').map_or(max_len, |pos| pos + 1).max(1)
}

fn make_wrapped_string_node(text: &str, column_width: usize, indent: &str) -> SyntaxElement {
    let escaped = escape(text);
    if escaped.len() + 2 <= column_width {
        return make_string_node(text);
    }
    make_multiline_string_node(&wrap_string_with_continuations(text, column_width, indent))
}

pub fn strip_quotes(s: &str) -> String {
    s.trim_start_matches('"')
        .trim_end_matches('"')
        .trim_start_matches('\'')
        .trim_end_matches('\'')
        .to_string()
}

pub fn get_string_token(node: &SyntaxNode) -> Option<tombi_syntax::SyntaxToken> {
    let kind = node.kind();
    node.descendants_with_tokens()
        .filter_map(|elem| elem.into_token())
        .find(|token| token.kind() == kind)
}

pub fn load_text(value: &str, kind: SyntaxKind) -> String {
    let offset = if [BASIC_STRING, LITERAL_STRING].contains(&kind) {
        1
    } else if kind == BARE_KEY {
        0
    } else {
        3
    };

    let mut chars = value.chars();
    for _ in 0..offset {
        chars.next();
    }
    for _ in 0..offset {
        chars.next_back();
    }
    let content = chars.as_str();

    let content = if kind == MULTI_LINE_BASIC_STRING || kind == MULTI_LINE_LITERAL_STRING {
        content
            .strip_prefix("\r\n")
            .or_else(|| content.strip_prefix('\n'))
            .unwrap_or(content)
    } else {
        content
    };

    if kind == BASIC_STRING || kind == MULTI_LINE_BASIC_STRING {
        unescape(content).unwrap_or_else(|_| content.to_string())
    } else {
        content.to_string()
    }
}

pub fn update_content<F>(entry: &SyntaxNode, transform: F)
where
    F: Fn(&str) -> String,
{
    update_content_impl(entry, transform, None, "");
}

pub fn update_content_wrapped<F>(entry: &SyntaxNode, transform: F, column_width: usize, indent: &str)
where
    F: Fn(&str) -> String,
{
    update_content_impl(entry, transform, Some(column_width), indent);
}

fn update_content_impl<F>(entry: &SyntaxNode, transform: F, column_width: Option<usize>, indent: &str)
where
    F: Fn(&str) -> String,
{
    let (mut to_insert, mut count) = (Vec::<SyntaxElement>::new(), 0);
    let mut changed = false;
    for mut child in entry.children_with_tokens() {
        count += 1;
        let kind = child.kind();
        if [
            BASIC_STRING,
            LITERAL_STRING,
            MULTI_LINE_BASIC_STRING,
            MULTI_LINE_LITERAL_STRING,
        ]
        .contains(&kind)
        {
            let string_text = if let Some(token) = child.as_token() {
                token.text().to_string()
            } else if let Some(node) = child.as_node() {
                let Some(token) = node
                    .descendants_with_tokens()
                    .filter_map(|e| e.into_token())
                    .find(|t| t.kind() == kind)
                else {
                    to_insert.push(child);
                    continue;
                };
                token.text().to_string()
            } else {
                to_insert.push(child);
                continue;
            };
            let found_str_value = load_text(&string_text, kind);
            let output = transform(found_str_value.as_str());

            let is_multiline = kind == MULTI_LINE_BASIC_STRING || kind == MULTI_LINE_LITERAL_STRING;
            let is_literal = kind == LITERAL_STRING || kind == MULTI_LINE_LITERAL_STRING;
            let content_changed = output != found_str_value;
            let multiline_to_single = is_multiline && !output.contains('\n');

            let use_literal = output.contains('"') && can_use_literal_string(&output);
            let quote_style_change = is_literal != use_literal;

            let escaped_len = escape(&output).len() + 2;
            let key_prefix_len = get_key_prefix_len(entry);
            let total_line_len = key_prefix_len + escaped_len;
            let in_inline_table = is_inside_inline_table(entry);
            let needs_wrap = column_width.is_some_and(|cw| total_line_len > cw) && !in_inline_table;

            changed = content_changed || multiline_to_single || quote_style_change || needs_wrap;
            if changed {
                child = if use_literal {
                    make_literal_string_node(&output)
                } else if needs_wrap {
                    make_wrapped_string_node(&output, column_width.unwrap(), indent)
                } else {
                    make_string_node(&output)
                };
            }
        }
        to_insert.push(child);
    }
    if changed {
        entry.splice_children(0..count, to_insert);
    }
}

pub fn wrap_all_long_strings(root: &SyntaxNode, column_width: usize, indent: &str) {
    for descendant in root.descendants() {
        if is_string_kind(descendant.kind()) {
            wrap_string_node_if_needed(&descendant, column_width, indent);
        }
    }
}

fn wrap_string_node_if_needed(string_node: &SyntaxNode, column_width: usize, indent: &str) {
    let kind = string_node.kind();
    let Some(token) = string_node
        .descendants_with_tokens()
        .filter_map(|e| e.into_token())
        .find(|token| token.kind() == kind)
    else {
        return;
    };

    let text = load_text(token.text(), kind);

    let is_multiline = kind == MULTI_LINE_BASIC_STRING || kind == MULTI_LINE_LITERAL_STRING;
    let is_literal = kind == LITERAL_STRING || kind == MULTI_LINE_LITERAL_STRING;

    let use_literal = text.contains('"') && can_use_literal_string(&text);
    let quote_style_change = is_literal != use_literal;
    let multiline_to_single = is_multiline && !text.contains('\n');

    let escaped_len = escape(&text).len() + 2;
    let key_prefix_len = get_key_prefix_len(string_node);
    let total_line_len = key_prefix_len + escaped_len;
    let in_inline_table = is_inside_inline_table(string_node);
    let needs_wrap = total_line_len > column_width && !in_inline_table;

    let changed = quote_style_change || multiline_to_single || needs_wrap;
    if changed {
        let new_element = if needs_wrap {
            make_wrapped_string_node(&text, column_width, indent)
        } else if use_literal {
            make_literal_string_node(&text)
        } else {
            make_string_node(&text)
        };
        let mut new_children: Vec<SyntaxElement> = Vec::new();
        let count = string_node.children_with_tokens().count();
        for child in string_node.children_with_tokens() {
            if is_string_kind(child.kind()) {
                new_children.push(new_element.clone());
            } else {
                new_children.push(child);
            }
        }
        string_node.splice_children(0..count, new_children);
    }
}
