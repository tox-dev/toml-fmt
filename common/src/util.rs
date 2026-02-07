use tombi_syntax::{SyntaxElement, SyntaxKind, SyntaxNode};

pub fn iter<F>(node: &SyntaxNode, paths: &[SyntaxKind], handle: &F)
where
    F: Fn(&SyntaxNode),
{
    for entry in node.children_with_tokens() {
        if entry.kind() == paths[0] {
            let found = entry.as_node().unwrap();
            if paths.len() == 1 {
                handle(found);
            } else {
                iter(found, &paths[1..], handle);
            }
        }
    }
}

pub fn find_first<F, T>(node: &SyntaxNode, paths: &[SyntaxKind], extract: &F) -> Option<T>
where
    F: Fn(SyntaxElement) -> T,
{
    for entry in node.children_with_tokens() {
        if entry.kind() == paths[0] {
            if paths.len() == 1 {
                return Some(extract(entry));
            } else if let Some(result) = find_first(entry.as_node().unwrap(), &paths[1..], extract) {
                return Some(result);
            }
        }
    }
    None
}

pub fn limit_blank_lines(content: &str, max_blank_lines: usize) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut consecutive_blanks = 0;

    for line in lines {
        let trimmed_line = line.trim_end();
        if trimmed_line.is_empty() {
            consecutive_blanks += 1;
            if consecutive_blanks <= max_blank_lines {
                result.push(trimmed_line);
            }
        } else {
            consecutive_blanks = 0;
            result.push(trimmed_line);
        }
    }

    let output = result.join("\n");
    if content.ends_with('\n') {
        format!("{output}\n")
    } else {
        output
    }
}
