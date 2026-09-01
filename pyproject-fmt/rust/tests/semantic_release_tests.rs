use super::evaluate_full as evaluate;

#[test]
fn test_semantic_release_order() {
    let start = indoc::indoc! {r#"
    [tool.semantic_release]
    version_toml = ["pyproject.toml:project.version"]
    assets = ["zebra.txt", "alpha.txt"]
    tag_format = "v{version}"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.semantic_release]
    tag_format = "v{version}"
    version_toml = [ "pyproject.toml:project.version" ]
    assets = [ "zebra.txt", "alpha.txt" ]
    "#);
}

/// Later declarations override earlier ones, so their order carries behavior.
#[test]
fn test_semantic_release_declarations_keep_their_order() {
    let start = indoc::indoc! {r#"
    [tool.semantic_release]
    version_variables = ["version.txt:*:tf", "version.txt:*:nf"]
    assets = ["z/dist.whl", "a/dist.whl"]
    "#};
    let result = evaluate(start);

    insta::assert_snapshot!(result, @r#"
    [tool.semantic_release]
    version_variables = [ "version.txt:*:tf", "version.txt:*:nf" ]
    assets = [ "z/dist.whl", "a/dist.whl" ]
    "#);
}

/// The patterns are matched with `any`, so they are a set and sort.
#[test]
fn test_semantic_release_exclude_commit_patterns_sort() {
    let start = indoc::indoc! {r#"
    [tool.semantic_release]
    exclude_commit_patterns = ["^zeta", "^alpha"]
    "#};
    let result = evaluate(start);

    insta::assert_snapshot!(result, @r#"
    [tool.semantic_release]
    exclude_commit_patterns = [ "^alpha", "^zeta" ]
    "#);
}

/// semantic release reads the branch rules in the order they are written, and the first match
/// decides the release policy.
#[test]
fn test_semantic_release_branches_keep_their_order() {
    let start = indoc::indoc! {r#"
    [tool.semantic_release.branches.z-specific]
    match = "release/.*"

    [tool.semantic_release.branches.a-fallback]
    match = ".*"
    "#};
    let result = evaluate(start);

    insta::assert_snapshot!(result, @r#"
    [tool.semantic_release]
    branches.z-specific.match = "release/.*"
    branches.a-fallback.match = ".*"
    "#);
}
