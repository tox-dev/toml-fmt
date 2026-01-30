use taplo::syntax::SyntaxKind::{IDENT, MULTI_LINE_STRING, MULTI_LINE_STRING_LITERAL, STRING, STRING_LITERAL};
use taplo::syntax::{SyntaxElement, SyntaxKind, SyntaxNode};
use taplo::util::{check_escape, unescape};

use crate::create::make_string_node;

/// Check if a string contains backslash sequences that would be invalid in a TOML basic string.
/// Valid escapes are: \b, \t, \n, \f, \r, \", \\, \uXXXX, \UXXXXXXXX, and line continuations.
///
/// Uses taplo's escape checking for TOML spec compliance.
fn has_invalid_escapes(s: &str) -> bool {
    check_escape(s).is_err()
}

/// Load the text content from a TOML string value, handling all escape sequences.
///
/// This function:
/// 1. Strips the appropriate delimiters based on string kind (", ', """, ''')
/// 2. Unescapes all TOML escape sequences using taplo's unescape function
/// 3. Returns the final string content
///
/// Uses taplo's unescape for TOML spec compliance, which handles:
/// - Basic escapes: \b, \t, \n, \f, \r, \", \\
/// - Unicode escapes: \uXXXX, \UXXXXXXXX
/// - Line continuations: \<newline>
pub fn load_text(value: &str, kind: SyntaxKind) -> String {
    // Determine delimiter offset based on string type
    let offset = if [STRING, STRING_LITERAL].contains(&kind) {
        1 // Single quote or double quote
    } else if kind == IDENT {
        0 // No delimiters
    } else {
        3 // Triple quotes (""" or ''')
    };

    // Strip delimiters
    let mut chars = value.chars();
    for _ in 0..offset {
        chars.next();
    }
    for _ in 0..offset {
        chars.next_back();
    }
    let content = chars.as_str();

    // For multiline strings, strip leading newline if present (per TOML spec)
    let content = if kind == MULTI_LINE_STRING || kind == MULTI_LINE_STRING_LITERAL {
        content
            .strip_prefix("\r\n")
            .or_else(|| content.strip_prefix('\n'))
            .unwrap_or(content)
    } else {
        content
    };

    // Unescape for basic and multiline basic strings (not literals)
    if kind == STRING || kind == MULTI_LINE_STRING {
        unescape(content).unwrap_or_else(|_| content.to_string())
    } else {
        // Literal strings don't have escape sequences
        content.to_string()
    }
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
