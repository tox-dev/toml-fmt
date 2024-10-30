use taplo::syntax::{SyntaxElement, SyntaxKind, SyntaxNode};

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
            } else {
                find_first(entry.as_node().unwrap(), &paths[1..], extract);
            }
        }
    }
    None
}
