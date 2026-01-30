use taplo::syntax::SyntaxKind::{IDENT, MULTI_LINE_STRING, MULTI_LINE_STRING_LITERAL, STRING, STRING_LITERAL};
use taplo::syntax::{SyntaxElement, SyntaxKind, SyntaxNode};

use crate::create::make_string_node;

/// Check if a string contains backslash sequences that would be invalid in a TOML basic string.
/// Valid escapes are: \b, \t, \n, \f, \r, \", \\, \uXXXX, \UXXXXXXXX
fn has_invalid_escapes(s: &str) -> bool {
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.peek() {
                Some('b' | 't' | 'n' | 'f' | 'r' | '"' | '\\') => {
                    chars.next();
                }
                Some('u') => {
                    chars.next();
                    for _ in 0..4 {
                        if !chars.next().is_some_and(|c| c.is_ascii_hexdigit()) {
                            return true;
                        }
                    }
                }
                Some('U') => {
                    chars.next();
                    for _ in 0..8 {
                        if !chars.next().is_some_and(|c| c.is_ascii_hexdigit()) {
                            return true;
                        }
                    }
                }
                _ => return true,
            }
        }
    }
    false
}

pub fn load_text(value: &str, kind: SyntaxKind) -> String {
    let mut chars = value.chars();
    let offset = if [STRING, STRING_LITERAL].contains(&kind) {
        1
    } else if kind == IDENT {
        0
    } else {
        3
    };
    for _ in 0..offset {
        chars.next();
    }
    for _ in 0..offset {
        chars.next_back();
    }
    let mut res = chars.as_str().to_string();
    if kind == STRING {
        res = res.replace("\\\"", "\"");
    }
    if kind == MULTI_LINE_STRING {
        // Handle line continuation: backslash followed by newline and any whitespace
        res = process_line_continuations(&res);
    }
    res
}

fn process_line_continuations(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if chars.peek() == Some(&'\n') || chars.peek() == Some(&'\r') {
                // Skip the backslash and consume newlines/whitespace
                while let Some(&next) = chars.peek() {
                    if next == '\n' || next == '\r' || next == ' ' || next == '\t' {
                        chars.next();
                    } else {
                        break;
                    }
                }
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }
    result
}

pub fn update_content<F>(entry: &SyntaxNode, transform: F)
where
    F: Fn(&str) -> String,
{
    let (mut to_insert, mut count) = (Vec::<SyntaxElement>::new(), 0);
    let mut changed = false;
    for mut child in entry.children_with_tokens() {
        count += 1;
        let kind = child.kind();
        if [STRING, STRING_LITERAL, MULTI_LINE_STRING, MULTI_LINE_STRING_LITERAL].contains(&kind) {
            let found_str_value = load_text(child.as_token().unwrap().text(), kind);
            let output = transform(found_str_value.as_str());

            let is_multiline = kind == MULTI_LINE_STRING || kind == MULTI_LINE_STRING_LITERAL;
            let is_literal = kind == STRING_LITERAL || kind == MULTI_LINE_STRING_LITERAL;
            let content_changed = output != found_str_value;
            let multiline_to_single = is_multiline && !output.contains('\n');
            // prefer "" over ''
            let normalize_literal = is_literal && !has_invalid_escapes(&output);

            changed = content_changed || multiline_to_single || normalize_literal;
            if changed {
                child = make_string_node(output.as_str());
            }
        }
        to_insert.push(child);
    }
    if changed {
        entry.splice_children(0..count, to_insert);
    }
}
