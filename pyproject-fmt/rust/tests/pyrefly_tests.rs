use super::default_settings;
use super::evaluate_full;
use _pyproject_fmt::{format_toml, Settings};

fn evaluate(start: &str) -> String {
    evaluate_full(start)
}

fn long_settings() -> Settings {
    Settings {
        table_format: String::from("long"),
        ..default_settings()
    }
}

fn evaluate_long(start: &str) -> String {
    let result = format_toml(start, &long_settings()).unwrap();
    super::assert_valid_toml(&result);
    result
}

#[test]
fn test_pyrefly_order() {
    let start = indoc::indoc! {r#"
    [tool.pyrefly]
    project_includes = ["src"]
    python_version = "3.12"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pyrefly]
    python_version = "3.12"
    project_includes = [ "src" ]
    "#);
}

#[test]
fn test_pyrefly_no_table_noop() {
    let start = indoc::indoc! {r#"
    [project]
    name = "x"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "x"
    "#);
}

#[test]
fn test_pyrefly_arrays_sorted_but_the_search_paths() {
    let start = indoc::indoc! {r#"
    [tool.pyrefly]
    project_includes = ["zeta/**", "alpha/**"]
    project_excludes = ["zbuild", "abuild"]
    search_path = ["z_path", "a_path"]
    site_package_path = ["z_site", "a_site"]
    replace_imports_with_any = ["z_pkg", "a_pkg"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pyrefly]
    project_includes = [ "alpha/**", "zeta/**" ]
    project_excludes = [ "abuild", "zbuild" ]
    search_path = [ "z_path", "a_path" ]
    site_package_path = [ "z_site", "a_site" ]
    replace_imports_with_any = [ "z_pkg", "a_pkg" ]
    "#);
}

/// pyrefly takes the first replacement rule that matches, so a `!` rule exempts an import only while
/// it stands before a broader rule that would also match it, and moving it changes what is replaced.
#[test]
fn test_pyrefly_replacement_rules_keep_their_order() {
    let rules = |written: &str| format!("[tool.pyrefly]\nreplace-imports-with-any = {written}\n");
    let broad_first = evaluate(&rules(r#"["example.path.*", "!example.path.specific.*"]"#));
    let narrow_first = evaluate(&rules(r#"["!example.path.specific.*", "example.path.*"]"#));

    assert_eq!(
        broad_first,
        "[tool.pyrefly]\nreplace-imports-with-any = [ \"example.path.*\", \"!example.path.specific.*\" ]\n"
    );
    assert_eq!(
        narrow_first,
        "[tool.pyrefly]\nreplace-imports-with-any = [ \"!example.path.specific.*\", \"example.path.*\" ]\n"
    );
}

#[test]
fn test_pyrefly_non_sortable_preserved() {
    let start = indoc::indoc! {r#"
    [tool.pyrefly]
    python_interpreter = "/usr/bin/python3"
    use_untyped_imports = true
    python_version = "3.12"
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pyrefly]
    python_version = "3.12"
    python_interpreter = "/usr/bin/python3"
    use_untyped_imports = true
    "#);
}

#[test]
fn test_pyrefly_long_format() {
    let start = indoc::indoc! {r#"
    [tool.pyrefly]
    project_includes = ["zeta/**", "alpha/**"]
    python_version = "3.12"
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[tool.pyrefly]"));
    assert!(result.find("alpha").unwrap() < result.find("zeta").unwrap());
    assert!(result.find("python_version").unwrap() < result.find("project_includes").unwrap());
}

/// pyrefly spells its options with hyphens, and searches the paths in the order they are listed.
#[test]
fn test_pyrefly_reads_the_names_it_spells_today() {
    let start = indoc::indoc! {r#"
    [tool.pyrefly]
    search-path = ["z_path", "a_path"]
    project-includes = ["zeta/**", "alpha/**"]
    python-version = "3.13"
    "#};
    let result = evaluate(start);

    insta::assert_snapshot!(result, @r#"
    [tool.pyrefly]
    python-version = "3.13"
    project-includes = [ "alpha/**", "zeta/**" ]
    search-path = [ "z_path", "a_path" ]
    "#);
}
