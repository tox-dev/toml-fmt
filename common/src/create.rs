use taplo::parser::parse;
use taplo::syntax::SyntaxElement;
use taplo::syntax::SyntaxKind::{
    ARRAY, COMMA, ENTRY, KEY, MULTI_LINE_STRING, MULTI_LINE_STRING_LITERAL, NEWLINE, STRING, STRING_LITERAL, VALUE,
};

use crate::string::StringType;

pub fn make_string_node(text: &str, target_type: StringType) -> SyntaxElement {
    let expr = match target_type {
        StringType::String => format!("a = \"{}\"", text.replace('"', "\\\"")),
        StringType::Multiline => format!("a = \"\"\"{}\"\"\"", text.replace('"', "\\\"")),
        StringType::Literal => format!("a = '{}'", text),
        StringType::MultilineLiteral => format!("a = '''{}'''", text),
    };
    for root in parse(&expr)
        .into_syntax()
        .clone_for_update()
        .first_child()
        .unwrap()
        .children_with_tokens()
    {
        if root.kind() == VALUE {
            for entries in root.as_node().unwrap().children_with_tokens() {
                if (target_type == StringType::String && entries.kind() == STRING)
                    || (target_type == StringType::Multiline && entries.kind() == MULTI_LINE_STRING)
                    || (target_type == StringType::Literal && entries.kind() == STRING_LITERAL)
                    || (target_type == StringType::MultilineLiteral && entries.kind() == MULTI_LINE_STRING_LITERAL)
                {
                    return entries;
                }
            }
        }
    }
    panic!("Could not create string element for {text:?}")
}

pub fn make_empty_newline() -> SyntaxElement {
    for root in parse("\n\n").into_syntax().clone_for_update().children_with_tokens() {
        if root.kind() == NEWLINE {
            return root;
        }
    }
    panic!("Could not create empty newline");
}

pub fn make_newline() -> SyntaxElement {
    for root in parse("\n").into_syntax().clone_for_update().children_with_tokens() {
        if root.kind() == NEWLINE {
            return root;
        }
    }
    panic!("Could not create newline");
}

pub fn make_comma() -> SyntaxElement {
    for root in parse("a=[1,2]").into_syntax().clone_for_update().children_with_tokens() {
        if root.kind() == ENTRY {
            for value in root.as_node().unwrap().children_with_tokens() {
                if value.kind() == VALUE {
                    for array in value.as_node().unwrap().children_with_tokens() {
                        if array.kind() == ARRAY {
                            for e in array.as_node().unwrap().children_with_tokens() {
                                if e.kind() == COMMA {
                                    return e;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    panic!("Could not create comma");
}

pub fn make_key(text: &str) -> SyntaxElement {
    for root in parse(format!("{text}=1").as_str())
        .into_syntax()
        .clone_for_update()
        .children_with_tokens()
    {
        if root.kind() == ENTRY {
            for value in root.as_node().unwrap().children_with_tokens() {
                if value.kind() == KEY {
                    return value;
                }
            }
        }
    }
    panic!("Could not create key {text}");
}

pub fn make_array(key: &str) -> SyntaxElement {
    let txt = format!("{key} = []");
    for root in parse(txt.as_str())
        .into_syntax()
        .clone_for_update()
        .children_with_tokens()
    {
        if root.kind() == ENTRY {
            return root;
        }
    }
    panic!("Could not create array");
}

pub fn make_array_entry(key: &str) -> SyntaxElement {
    let txt = format!("a = [\"{key}\"]");
    for root in parse(txt.as_str())
        .into_syntax()
        .clone_for_update()
        .children_with_tokens()
    {
        if root.kind() == ENTRY {
            for value in root.as_node().unwrap().children_with_tokens() {
                if value.kind() == VALUE {
                    for array in value.as_node().unwrap().children_with_tokens() {
                        if array.kind() == ARRAY {
                            for e in array.as_node().unwrap().children_with_tokens() {
                                if e.kind() == VALUE {
                                    return e;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    panic!("Could not create array");
}

pub fn make_entry_of_string(key: &String, value: &String) -> SyntaxElement {
    let txt = format!("{key} = \"{value}\"\n");
    for root in parse(txt.as_str())
        .into_syntax()
        .clone_for_update()
        .children_with_tokens()
    {
        if root.kind() == ENTRY {
            return root;
        }
    }
    panic!("Could not create entry of string");
}

pub fn make_table_entry(key: &str) -> Vec<SyntaxElement> {
    let txt = format!("[{key}]\n");
    let mut res = Vec::<SyntaxElement>::new();
    for root in parse(txt.as_str())
        .into_syntax()
        .clone_for_update()
        .children_with_tokens()
    {
        res.push(root);
    }
    res
}
