use taplo::syntax::SyntaxKind::{IDENT, MULTI_LINE_STRING, MULTI_LINE_STRING_LITERAL, STRING, STRING_LITERAL};
use taplo::syntax::{SyntaxElement, SyntaxKind, SyntaxNode};

use crate::create::make_string_node;

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
    res
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StringUpdateMode {
    /// Preserve the string type.
    PreserveType,

    /// Convert to a simple string (non-literal, non-multiline).
    ConvertToString,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StringType {
    String,
    Literal,
    Multiline,
    MultilineLiteral,
}

pub fn update_content<F>(entry: &SyntaxNode, transform: F, mode: StringUpdateMode)
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

            changed = output != found_str_value || (mode == StringUpdateMode::ConvertToString && kind != STRING);
            if changed {
                let literal = [STRING_LITERAL, MULTI_LINE_STRING_LITERAL].contains(&kind);
                let multiline = [MULTI_LINE_STRING, MULTI_LINE_STRING_LITERAL].contains(&kind);

                let target_type = match mode {
                    StringUpdateMode::ConvertToString => StringType::String,
                    StringUpdateMode::PreserveType => match (literal, multiline) {
                        (false, false) => StringType::String,
                        (true, false) => StringType::Literal,
                        (false, true) => StringType::Multiline,
                        (true, true) => StringType::MultilineLiteral,
                    },
                };

                child = make_string_node(output.as_str(), target_type);
            }
        }
        to_insert.push(child);
    }
    if changed {
        entry.splice_children(0..count, to_insert);
    }
}
