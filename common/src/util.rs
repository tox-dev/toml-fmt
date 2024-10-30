use taplo::syntax::{SyntaxKind, SyntaxNode};

pub fn iter<F>(node: &SyntaxNode, paths: &[SyntaxKind], transform: &F)
where
    F: Fn(&SyntaxNode),
{
    for entry in node.children_with_tokens() {
        if entry.kind() == paths[0] {
            let found = entry.as_node().unwrap();
            if paths.len() == 1 {
                transform(found);
            } else {
                iter(found, &paths[1..], transform);
            }
        }
    }
}
