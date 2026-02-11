use indoc::indoc;
use tombi_config::TomlVersion;

use super::format_toml;
use crate::table::{
    Tables, apply_table_formatting, collapse_sub_table, collapse_sub_tables, collect_all_sub_tables, expand_sub_table,
    expand_sub_tables, find_key, for_entries, get_table_name, reorder_table_keys,
};

fn parse(source: &str) -> tombi_syntax::SyntaxNode {
    tombi_parser::parse(source, TomlVersion::default())
        .syntax_node()
        .clone_for_update()
}

fn tables_reorder_helper(start: &str, order: &[&str]) -> String {
    let root_ast = parse(start);
    let tables = Tables::from_ast(&root_ast);
    tables.reorder(&root_ast, order, &["tool"]);
    format_toml(&root_ast, 120)
}

fn reorder_and_get_keys(tables: &Tables, table_name: &str, order: &[&str]) -> Vec<String> {
    let mut keys = Vec::new();
    if let Some(table_refs) = tables.get(table_name) {
        for table_ref in table_refs {
            reorder_table_keys(&mut table_ref.borrow_mut(), order);

            let table = table_ref.borrow();
            for_entries(&table, &mut |key, _node| {
                keys.push(key);
            });
        }
    }
    keys
}

fn reorder_table_keys_helper(start: &str, order: &[&str], expected_order: Vec<&str>) {
    let root_ast = parse(start);
    let tables = Tables::from_ast(&root_ast);
    let keys = reorder_and_get_keys(&tables, "project", order);
    assert_eq!(keys, expected_order);
}

fn collapse_sub_tables_helper(start: &str, table_name: &str, has_sub_tables: bool) {
    let root_ast = parse(start);
    let mut tables = Tables::from_ast(&root_ast);

    let initial_count = tables.header_to_pos.len();
    collapse_sub_tables(&mut tables, table_name);

    if has_sub_tables {
        if let Some(refs) = tables.get(&format!("{table_name}.optional-dependencies")) {
            for r in refs {
                assert!(r.borrow().is_empty());
            }
        }
    } else {
        assert_eq!(tables.header_to_pos.len(), initial_count);
    }
}

fn issue_124_helper(start: &str, order: &[&str]) -> String {
    let root_ast = parse(start);
    let tables = Tables::from_ast(&root_ast);
    tables.reorder(&root_ast, order, &[]);
    format_toml(&root_ast, 120)
}

#[test]
fn test_tables_from_ast_empty() {
    let root_ast = parse("");
    let tables = Tables::from_ast(&root_ast);
    assert!(tables.header_to_pos.is_empty());
    assert!(tables.table_set.is_empty());
}

#[test]
fn test_tables_from_ast_single_table() {
    let toml = indoc! {r#"
        [project]
        name = "foo"
    "#};
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);

    assert!(tables.header_to_pos.contains_key("project"));
    assert_eq!(tables.header_to_pos["project"].len(), 1);
}

#[test]
fn test_tables_from_ast_multiple_tables() {
    let toml = indoc! {r#"
        [project]
        name = "foo"

        [tool.black]
        line-length = 120
    "#};
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);

    assert!(tables.header_to_pos.contains_key("project"));
    assert!(tables.header_to_pos.contains_key("tool.black"));
}

#[test]
fn test_tables_from_ast_array_of_tables() {
    let toml = indoc! {r#"
        [[project.authors]]
        name = "Alice"

        [[project.authors]]
        name = "Bob"
    "#};
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);

    assert!(tables.header_to_pos.contains_key("project.authors"));
    assert_eq!(tables.header_to_pos["project.authors"].len(), 2);
}

#[test]
fn test_tables_get_existing() {
    let toml = indoc! {r#"
        [project]
        name = "foo"
    "#};
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);

    let result = tables.get("project");
    assert!(result.is_some());
    assert_eq!(result.unwrap().len(), 1);
}

#[test]
fn test_tables_get_non_existing() {
    let toml = indoc! {r#"
        [project]
        name = "foo"
    "#};
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);

    let result = tables.get("nonexistent");
    assert!(result.is_none());
}

#[test]
fn test_tables_reorder_case_simple_reorder() {
    let start = indoc! {r#"
        [tool.black]
        line-length = 120

        [project]
        name = "foo"
    "#};
    let res = tables_reorder_helper(start, &["project", "tool.black"]);
    insta::assert_snapshot!(res, @r#"
    [project]
    name = "foo"

    [tool.black]
    line-length = 120
    "#);
}

#[test]
fn test_tables_reorder_case_keep_sub_tables_together() {
    let start = indoc! {r#"
        [tool.pytest]
        testpaths = ["tests"]

        [project]
        name = "foo"

        [tool.black]
        line-length = 120
    "#};
    let res = tables_reorder_helper(start, &["project", "tool"]);
    insta::assert_snapshot!(res, @r#"
    [project]
    name = "foo"

    [tool.pytest]
    testpaths = [ "tests" ]

    [tool.black]
    line-length = 120
    "#);
}

#[test]
fn test_tables_reorder_case_unknown_tools_file_order_subtables_alphabetical() {
    let start = indoc! {r#"
        [tool.flake8]

        [tool.cff-from-621]
        [tool.cff-from-621.static]

        [tool.coverage.run]
        [tool.coverage.report]

        [tool.hatch.envs.linting]
        [tool.hatch.envs.linting.scripts]

        [tool.hatch.envs.unit-tests]
        [tool.hatch.envs.unit-tests.scripts]

        [tool.hatch.envs.mypy]
        [tool.hatch.envs.mypy.scripts]
    "#};
    let res = tables_reorder_helper(start, &["project", "tool"]);
    insta::assert_snapshot!(res, @"
    [tool.flake8]

    [tool.cff-from-621]
    [tool.cff-from-621.static]

    [tool.coverage.report]

    [tool.coverage.run]

    [tool.hatch.envs.linting]
    [tool.hatch.envs.linting.scripts]

    [tool.hatch.envs.mypy]
    [tool.hatch.envs.mypy.scripts]

    [tool.hatch.envs.unit-tests]
    [tool.hatch.envs.unit-tests.scripts]
    ");
}

#[test]
fn test_tables_reorder_case_same_tool_subtables_sorted_short_to_long() {
    let start = indoc! {r#"
        [tool.hatch.envs.test.scripts]
        [tool.hatch.envs.test]
        [tool.hatch]
    "#};
    let res = tables_reorder_helper(start, &["tool"]);
    insta::assert_snapshot!(res, @"
    [tool.hatch]
    [tool.hatch.envs.test]
    [tool.hatch.envs.test.scripts]
    ");
}

#[test]
fn test_tables_reorder_case_different_tools_preserve_file_order() {
    let start = indoc! {r#"
        [tool.zebra]

        [tool.alpha]
    "#};
    let res = tables_reorder_helper(start, &["tool"]);
    insta::assert_snapshot!(res, @"
    [tool.zebra]

    [tool.alpha]
    ");
}

#[test]
fn test_get_table_name_table_header() {
    let toml = "[project]";
    let root_ast = parse(toml);

    let child = root_ast.children_with_tokens().next().expect("No table header found");
    let name = get_table_name(&child);
    assert_eq!(name, "project");
}

#[test]
fn test_get_table_name_array_header() {
    let toml = "[[project.authors]]";
    let root_ast = parse(toml);

    let child = root_ast
        .children_with_tokens()
        .next()
        .expect("No table array header found");
    let name = get_table_name(&child);
    assert_eq!(name, "project.authors");
}

#[test]
fn test_get_table_name_non_header() {
    let toml = "name = \"foo\"";
    let root_ast = parse(toml);

    let child = root_ast.children_with_tokens().next().expect("No entry found");
    let name = get_table_name(&child);
    assert_eq!(name, "");
}

#[test]
fn test_reorder_table_keys_case_simple_reorder() {
    let start = indoc! {r#"
        [project]
        version = "1.0"
        name = "foo"
    "#};
    reorder_table_keys_helper(start, &["name", "version"], vec!["name", "version"]);
}

#[test]
fn test_reorder_table_keys_case_with_comments() {
    let start = indoc! {r#"
        [project]
        # version comment
        version = "1.0"
        # name comment
        name = "foo"
    "#};
    reorder_table_keys_helper(start, &["name", "version"], vec!["name", "version"]);
}

#[test]
fn test_reorder_table_keys_case_nested_keys() {
    let start = indoc! {r#"
        [project]
        z = "last"
        a.b = "nested"
        a.c = "another"
    "#};
    reorder_table_keys_helper(start, &["a", "z"], vec!["a.b", "a.c", "z"]);
}

#[test]
fn test_for_entries() {
    let toml = indoc! {r#"
        [project]
        name = "foo"
        version = "1.0"
    "#};
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);

    let mut entries_found = Vec::new();
    if let Some(table_refs) = tables.get("project") {
        for table_ref in table_refs {
            let table = table_ref.borrow();
            for_entries(&table, &mut |key, _node| {
                entries_found.push(key);
            });
        }
    }

    assert!(entries_found.contains(&String::from("name")));
    assert!(entries_found.contains(&String::from("version")));
}

#[test]
fn test_find_key_existing() {
    let toml = indoc! {r#"
        name = "foo"
        version = "1.0"
    "#};
    let root_ast = parse(toml);

    let result = find_key(&root_ast, "name");
    assert!(result.is_some());
}

#[test]
fn test_find_key_non_existing() {
    let toml = indoc! {r#"
        name = "foo"
        version = "1.0"
    "#};
    let root_ast = parse(toml);

    let result = find_key(&root_ast, "nonexistent");
    assert!(result.is_none());
}

#[test]
fn test_collapse_sub_tables_case_collapse_simple() {
    let start = indoc! {r#"
        [project]
        name = "foo"

        [project.optional-dependencies]
        dev = ["pytest"]
    "#};
    collapse_sub_tables_helper(start, "project", true);
}

#[test]
fn test_collapse_sub_tables_case_no_sub_tables() {
    let start = indoc! {r#"
        [project]
        name = "foo"

        [tool.black]
        line-length = 120
    "#};
    collapse_sub_tables_helper(start, "project", false);
}

#[test]
fn test_collapse_sub_tables_creates_main_if_missing() {
    let toml = indoc! {r#"
        [project.scripts]
        cli = "pkg:main"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    assert!(!tables.header_to_pos.contains_key("project"));

    collapse_sub_tables(&mut tables, "project");
    assert!(tables.header_to_pos.contains_key("project"));
}

#[test]
fn test_tables_from_ast_with_duplicate_table_headers() {
    let toml = indoc! {r#"
        [project]
        name = "foo"

        [tool.black]
        line-length = 120

        [project]
        version = "1.0"
    "#};
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);

    assert!(tables.header_to_pos.contains_key("project"));
    let refs = tables.get("project").unwrap();
    assert_eq!(refs.len(), 1);

    let table = refs[0].borrow();
    let txt = table.iter().map(|e| e.to_string()).collect::<String>();
    assert!(txt.contains("name"));
    assert!(txt.contains("version"));
}

#[test]
fn test_reorder_with_root_entries() {
    let toml = indoc! {r#"
        root_key = "value"

        [project]
        name = "foo"
    "#};
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);

    assert!(tables.header_to_pos.contains_key(""));

    tables.reorder(&root_ast, &["", "project"], &[]);
    let res = format_toml(&root_ast, 120);
    assert!(res.contains("root_key"));
    assert!(res.contains("[project]"));
}

#[test]
fn test_reorder_preserves_empty_lines_between_groups() {
    let toml = indoc! {r#"
        [tool.black]
        line-length = 120

        [project]
        name = "foo"

        [tool.ruff]
        select = ["E"]
    "#};
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);
    tables.reorder(&root_ast, &["project", "tool"], &["tool"]);

    let res = format_toml(&root_ast, 120);
    assert!(res.contains("\n\n"));
}

#[test]
fn test_collapse_sub_tables_multiple_sub_tables() {
    let toml = indoc! {r#"
        [project]
        name = "foo"

        [project.scripts]
        cli = "pkg:main"

        [project.gui-scripts]
        gui = "pkg:gui"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    collapse_sub_tables(&mut tables, "project");

    let main = tables.get("project").unwrap();
    let table = main[0].borrow();
    let txt = table.iter().map(|e| e.to_string()).collect::<String>();
    assert!(txt.contains("scripts.cli"));
    assert!(txt.contains("gui-scripts.gui"));
}

#[test]
fn test_reorder_table_keys_unordered_keys_at_end() {
    let toml = indoc! {r#"
        [project]
        zebra = "last"
        name = "foo"
        unordered = "value"
    "#};
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);
    let keys = reorder_and_get_keys(&tables, "project", &["name"]);
    assert_eq!(keys[0], "name");
}

#[test]
fn test_tables_duplicate_immediate() {
    let toml = "[project]\nname = \"foo\"\n[tool]\nx = 1\n[project]\nversion = \"1.0\"";
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);

    let refs = tables.get("project").unwrap();
    assert_eq!(refs.len(), 1);
    let table = refs[0].borrow();
    let txt = table.iter().map(|e| e.to_string()).collect::<String>();
    assert!(txt.contains("name"));
    assert!(txt.contains("version"));
}

#[test]
fn test_reorder_only_newline_table() {
    let toml = "\n\n[project]\nname = \"foo\"";
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);

    tables.reorder(&root_ast, &["", "project"], &[]);
    let res = format_toml(&root_ast, 120);
    assert!(res.contains("[project]"));
}

#[test]
fn test_collapse_with_array_table() {
    let toml = indoc! {r#"
        [[project.authors]]
        name = "Alice"

        [[project.authors]]
        name = "Bob"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    collapse_sub_tables(&mut tables, "project");
    assert!(tables.header_to_pos.contains_key("project"));
}

#[test]
fn test_reorder_keys_with_table_header_entry() {
    let toml = indoc! {r#"
        [project]
        [project.nested]
        a = 1
    "#};
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);
    reorder_and_get_keys(&tables, "project.nested", &["a"]);
}

#[test]
fn test_reorder_table_no_trailing_newline() {
    let toml = "[project]\nname = \"foo\"";
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);
    reorder_and_get_keys(&tables, "project", &["name"]);
}

#[test]
fn test_reorder_table_keys_consecutive_entries() {
    let toml = indoc! {r#"
        [project]
        a = 1
        b = 2
        c = 3
    "#};
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);
    let keys = reorder_and_get_keys(&tables, "project", &["c", "b", "a"]);
    assert_eq!(keys, vec!["c", "b", "a"]);
}

#[test]
fn test_reorder_table_keys_unhandled_sorted_alphabetically() {
    let toml = indoc! {r#"
        [dependency-groups]
        zebra = ["z"]
        alpha = ["a"]
        dev = ["dev-dep"]
        beta = ["b"]
        test = ["test-dep"]
    "#};
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);
    let keys = reorder_and_get_keys(&tables, "dependency-groups", &["", "dev", "test", "type", "docs"]);
    insta::assert_snapshot!(keys.join(", "), @"dev, test, alpha, beta, zebra");
}

#[test]
fn test_collapse_multiple_main_tables() {
    let toml = indoc! {r#"
        [[project]]
        name = "a"

        [[project]]
        name = "b"

        [project.sub]
        x = 1
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    collapse_sub_tables(&mut tables, "project");
}

#[test]
fn test_reorder_keys_consecutive_no_newline() {
    let toml = "[project]\na = 1\nb = 2";
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);
    reorder_and_get_keys(&tables, "project", &["b", "a"]);
}

#[test]
fn test_reorder_same_tool_group() {
    let toml = indoc! {r#"
        [tool.black]
        line-length = 120

        [tool.ruff]
        select = ["E"]
    "#};
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);
    tables.reorder(&root_ast, &["tool"], &["tool"]);

    let res = format_toml(&root_ast, 120);
    assert!(res.contains("[tool.black]"));
    assert!(res.contains("[tool.ruff]"));
}

#[test]
fn test_reorder_different_groups_no_trailing_newline() {
    let toml = "[tool.black]\nline-length = 120\n[project]\nname = \"foo\"";
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);
    tables.reorder(&root_ast, &["project", "tool"], &["tool"]);

    let res = format_toml(&root_ast, 120);
    assert!(res.contains("[project]"));
    assert!(res.contains("[tool.black]"));
}

#[test]
fn test_load_keys_entries_without_newline() {
    let toml = "[project]\na = 1\nb = 2\nc = 3";
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);
    let keys = reorder_and_get_keys(&tables, "project", &["c", "b", "a"]);
    assert_eq!(keys, vec!["c", "b", "a"]);
}

#[test]
fn test_comments_before_table_header_stay_with_that_table() {
    let toml = indoc! {r#"
        [project]
        name = "test"

        # comment for build-system
        [build-system]
        requires = ["hatchling"]
    "#};
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);
    tables.reorder(&root_ast, &["build-system", "project"], &[]);

    let res = format_toml(&root_ast, 120);
    // The comment should stay with [build-system], not [project]
    assert!(res.starts_with("# comment for build-system\n[build-system]"));
}

#[test]
fn test_multiple_comments_before_table_header() {
    let toml = indoc! {r#"
        [project]
        name = "test"

        # first comment
        # second comment
        [build-system]
        requires = ["hatchling"]
    "#};
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);
    tables.reorder(&root_ast, &["build-system", "project"], &[]);

    let res = format_toml(&root_ast, 120);
    // Both comments should stay with [build-system]
    assert!(res.contains("# first comment\n# second comment\n[build-system]"));
}

#[test]
fn test_comment_with_blank_line_before_table_header() {
    let toml = indoc! {r#"
        [project]
        name = "test"

        # comment for build-system
        [build-system]
        requires = ["hatchling"]
    "#};
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);
    tables.reorder(&root_ast, &["build-system", "project"], &[]);

    let res = format_toml(&root_ast, 120);
    // Comment should stay with [build-system] even with blank line before it
    assert!(res.starts_with("# comment for build-system\n[build-system]"));
}

#[test]
fn test_issue_124_case_comment_inside_table_and_before_next_table() {
    let start = indoc! {r#"
        [project]
        name = "test"
        # comment inside project table
        version = "1.0"

        scripts.main = "app:main"

        # comment for dependency-groups
        [dependency-groups]
        test = ["pytest"]
    "#};
    let result = issue_124_helper(start, &["dependency-groups", "project"]);
    insta::assert_snapshot!(result, @r#"
    # comment for dependency-groups
    [dependency-groups]
    test = [ "pytest" ]

    [project]
    name = "test"
    # comment inside project table
    version = "1.0"
    scripts.main = "app:main"
    "#);
}

#[test]
fn test_expand_sub_tables_creates_sub_table() {
    let toml = indoc! {r#"
        [project]
        name = "foo"
        urls.homepage = "https://example.com"
        urls.repository = "https://github.com/example"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    expand_sub_tables(&mut tables, "project");

    // Verify the sub-table was created
    assert!(tables.header_to_pos.contains_key("project.urls"));
}

#[test]
fn test_expand_sub_tables_removes_dotted_keys_from_parent() {
    let toml = indoc! {r#"
        [project]
        name = "foo"
        urls.homepage = "https://example.com"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    expand_sub_tables(&mut tables, "project");

    // Verify the dotted key is removed from parent
    let main = tables.get("project").unwrap();
    let table = main[0].borrow();
    let txt = table.iter().map(|e| e.to_string()).collect::<String>();
    assert!(!txt.contains("urls.homepage"));
    assert!(txt.contains("name"));
}

#[test]
fn test_expand_sub_tables_multiple_groups() {
    let toml = indoc! {r#"
        [project]
        name = "foo"
        urls.homepage = "https://example.com"
        scripts.main = "pkg:main"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    expand_sub_tables(&mut tables, "project");

    // Verify both sub-tables were created
    assert!(tables.header_to_pos.contains_key("project.urls"));
    assert!(tables.header_to_pos.contains_key("project.scripts"));
}

#[test]
fn test_expand_sub_tables_no_dotted_keys() {
    let toml = indoc! {r#"
        [project]
        name = "foo"
        version = "1.0"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    let initial_count = tables.header_to_pos.len();
    expand_sub_tables(&mut tables, "project");

    // No new tables should be created
    assert_eq!(tables.header_to_pos.len(), initial_count);
}

#[test]
fn test_expand_sub_tables_non_existent_table() {
    let toml = indoc! {r#"
        [project]
        name = "foo"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    // Should not panic when expanding non-existent table
    expand_sub_tables(&mut tables, "nonexistent");
}

#[test]
fn test_expand_and_collapse_are_inverses() {
    let toml = indoc! {r#"
        [project]
        name = "foo"

        [project.urls]
        homepage = "https://example.com"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    // Collapse should work
    collapse_sub_tables(&mut tables, "project");
    let main = tables.get("project").unwrap();
    let table = main[0].borrow();
    let txt = table.iter().map(|e| e.to_string()).collect::<String>();
    assert!(txt.contains("urls.homepage"));
}

#[test]
fn test_collapse_sub_table_single() {
    let toml = indoc! {r#"
        [project]
        name = "foo"

        [project.urls]
        homepage = "https://example.com"

        [project.scripts]
        cli = "pkg:main"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    collapse_sub_table(&mut tables, "project", "urls", 120);

    let main = tables.get("project").unwrap();
    let table = main[0].borrow();
    let txt = table.iter().map(|e| e.to_string()).collect::<String>();
    assert!(txt.contains("urls.homepage"), "urls should be collapsed");

    let scripts = tables.get("project.scripts").unwrap();
    let scripts_table = scripts[0].borrow();
    assert!(!scripts_table.is_empty(), "scripts should NOT be collapsed");
}

#[test]
fn test_collapse_sub_table_creates_parent() {
    let toml = indoc! {r#"
        [project.urls]
        homepage = "https://example.com"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    assert!(!tables.header_to_pos.contains_key("project"));
    collapse_sub_table(&mut tables, "project", "urls", 120);
    assert!(tables.header_to_pos.contains_key("project"));
}

#[test]
fn test_collapse_sub_table_converts_array_tables_to_inline() {
    let toml = indoc! {r#"
        [project]
        name = "foo"

        [[project.authors]]
        name = "Alice"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    collapse_sub_table(&mut tables, "project", "authors", 120);

    let authors = tables.get("project.authors").unwrap();
    let authors_table = authors[0].borrow();
    assert!(
        authors_table.is_empty(),
        "array tables should be collapsed to inline array"
    );

    let project = tables.get("project").unwrap();
    let project_table = project[0].borrow();
    let txt = project_table.iter().map(|e| e.to_string()).collect::<String>();
    assert!(
        txt.contains("authors = [{ name = \"Alice\" }]"),
        "should have inline array"
    );
}

#[test]
fn test_collapse_sub_table_keeps_wide_array_tables() {
    let toml = indoc! {r#"
        [project]
        name = "foo"

        [[project.authors]]
        name = "This is a very long author name that will definitely exceed the column width limit"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    collapse_sub_table(&mut tables, "project", "authors", 80);

    let authors = tables.get("project.authors").unwrap();
    let authors_table = authors[0].borrow();
    assert!(!authors_table.is_empty(), "wide array tables should not be collapsed");
}

#[test]
fn test_collapse_array_of_tables_preserves_comments() {
    let toml = indoc! {r#"
        [tool.cibuildwheel]
        name = "foo"

        [[tool.cibuildwheel.overrides]]
        # iOS environment comment
        select = "*_iphoneos"

        [[tool.cibuildwheel.overrides]]
        # iOS simulator comment
        select = "*_iphonesimulator"

        [[tool.cibuildwheel.overrides]]
        select = "*-win32"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    collapse_sub_table(&mut tables, "tool.cibuildwheel", "overrides", 120);

    let parent = tables.get("tool.cibuildwheel").unwrap();
    let parent_table = parent[0].borrow();
    let result = parent_table.iter().map(|e| e.to_string()).collect::<String>();
    insta::assert_snapshot!(result, @r#"
    [tool.cibuildwheel]
    name = "foo"
    overrides = [
      # iOS environment comment
      { select = "*_iphoneos" },
      # iOS simulator comment
      { select = "*_iphonesimulator" },
      { select = "*-win32" },
    ]
    "#);
}

#[test]
fn test_collapse_array_of_tables_preserves_multiple_comments_per_entry() {
    let toml = indoc! {r#"
        [tool.cibuildwheel]
        name = "foo"

        [[tool.cibuildwheel.overrides]]
        # iOS environment comment
        # yeah
        select = "*_iphoneos"
        # s
        pure = "ss"
        # oh yeah
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    collapse_sub_table(&mut tables, "tool.cibuildwheel", "overrides", 120);

    let parent = tables.get("tool.cibuildwheel").unwrap();
    let parent_table = parent[0].borrow();
    let result = parent_table.iter().map(|e| e.to_string()).collect::<String>();
    insta::assert_snapshot!(result, @r#"
    [tool.cibuildwheel]
    name = "foo"
    overrides = [
      # iOS environment comment
      # yeah
      { select = "*_iphoneos" },
      # s
      # oh yeah
      { pure = "ss" },
    ]
    "#);
}

#[test]
fn test_collapse_array_of_tables_no_comments() {
    let toml = indoc! {r#"
        [project]
        name = "foo"

        [[project.authors]]
        name = "Alice"

        [[project.authors]]
        name = "Bob"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    collapse_sub_table(&mut tables, "project", "authors", 120);

    let parent = tables.get("project").unwrap();
    let parent_table = parent[0].borrow();
    let result = parent_table.iter().map(|e| e.to_string()).collect::<String>();
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "foo"
    authors = [{ name = "Alice" }, { name = "Bob" }]
    "#);
}

#[test]
fn test_collapse_array_of_tables_wide_with_comments_between_keys() {
    let toml = indoc! {r#"
        [tool.test]
        name = "foo"

        [[tool.test.items]]
        # comment before first key
        first = "value"
        # comment before second key
        second = "another"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    collapse_sub_table(&mut tables, "tool.test", "items", 20);

    let items = tables.get("tool.test.items").unwrap();
    let items_table = items[0].borrow();
    assert!(
        !items_table.is_empty(),
        "wide entries with comments between keys should not be collapsed"
    );
}

#[test]
fn test_collapse_array_of_tables_wide_with_leading_comments() {
    let toml = indoc! {r#"
        [tool.test]
        name = "foo"

        [[tool.test.items]]
        # leading comment
        key = "this-is-a-very-long-value-that-exceeds-column-width"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    collapse_sub_table(&mut tables, "tool.test", "items", 30);

    let items = tables.get("tool.test.items").unwrap();
    let items_table = items[0].borrow();
    assert!(
        !items_table.is_empty(),
        "wide entries with leading comments should not be collapsed"
    );
}

#[test]
fn test_collapse_sub_table_non_existent() {
    let toml = indoc! {r#"
        [project]
        name = "foo"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    collapse_sub_table(&mut tables, "project", "nonexistent", 120);
}

#[test]
fn test_expand_sub_table_single() {
    let toml = indoc! {r#"
        [project]
        name = "foo"
        urls.homepage = "https://example.com"
        scripts.cli = "pkg:main"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    expand_sub_table(&mut tables, "project", "urls");

    assert!(tables.header_to_pos.contains_key("project.urls"));
    assert!(
        !tables.header_to_pos.contains_key("project.scripts"),
        "scripts should NOT be expanded"
    );

    let main = tables.get("project").unwrap();
    let table = main[0].borrow();
    let txt = table.iter().map(|e| e.to_string()).collect::<String>();
    assert!(txt.contains("scripts.cli"), "scripts should remain as dotted key");
    assert!(!txt.contains("urls.homepage"), "urls should be removed from parent");
}

#[test]
fn test_expand_sub_table_non_existent_parent() {
    let toml = indoc! {r#"
        [project]
        name = "foo"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    expand_sub_table(&mut tables, "nonexistent", "urls");
}

#[test]
fn test_expand_sub_table_no_matching_keys() {
    let toml = indoc! {r#"
        [project]
        name = "foo"
        version = "1.0"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    let initial_count = tables.header_to_pos.len();
    expand_sub_table(&mut tables, "project", "urls");
    assert_eq!(tables.header_to_pos.len(), initial_count);
}

#[test]
fn test_collect_all_sub_tables_simple() {
    let toml = indoc! {r#"
        [project]
        name = "foo"

        [project.urls]
        homepage = "https://example.com"

        [project.scripts]
        cli = "pkg:main"
    "#};
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);

    let mut result = Vec::new();
    collect_all_sub_tables(&tables, "project", &mut result);

    assert!(result.contains(&String::from("project.urls")));
    assert!(result.contains(&String::from("project.scripts")));
}

#[test]
fn test_collect_all_sub_tables_nested() {
    let toml = indoc! {r#"
        [project]
        name = "foo"

        [project.entry-points]
        [project.entry-points.tox]
        tox = "tox.plugin"
    "#};
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);

    let mut result = Vec::new();
    collect_all_sub_tables(&tables, "project", &mut result);

    assert!(result.contains(&String::from("project.entry-points")));
    assert!(result.contains(&String::from("project.entry-points.tox")));
}

#[test]
fn test_collect_all_sub_tables_from_dotted_keys() {
    let toml = indoc! {r#"
        [project]
        name = "foo"
        urls.homepage = "https://example.com"
    "#};
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);

    let mut result = Vec::new();
    collect_all_sub_tables(&tables, "project", &mut result);

    assert!(result.contains(&String::from("project.urls")));
}

#[test]
fn test_collect_all_sub_tables_empty() {
    let toml = indoc! {r#"
        [project]
        name = "foo"
    "#};
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);

    let mut result = Vec::new();
    collect_all_sub_tables(&tables, "project", &mut result);

    assert!(result.is_empty());
}

#[test]
fn test_collapse_sub_table_multiple_main_positions() {
    let toml = indoc! {r#"
        [[project]]
        name = "a"

        [[project]]
        name = "b"

        [project.urls]
        homepage = "https://example.com"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    collapse_sub_table(&mut tables, "project", "urls", 120);

    let urls = tables.get("project.urls").unwrap();
    assert!(
        !urls[0].borrow().is_empty(),
        "should not collapse when multiple main positions"
    );
}

#[test]
fn test_expand_sub_table_multiple_main_positions() {
    let toml = indoc! {r#"
        [[project]]
        name = "a"
        urls.homepage = "https://example.com"

        [[project]]
        name = "b"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    let initial_count = tables.header_to_pos.len();
    expand_sub_table(&mut tables, "project", "urls");

    assert_eq!(
        tables.header_to_pos.len(),
        initial_count,
        "should not expand when multiple main positions"
    );
}

#[test]
fn test_collapse_sub_table_multiple_sub_positions() {
    let toml = indoc! {r#"
        [project]
        name = "foo"

        [project.urls]
        homepage = "https://example.com"

        [project.urls]
        repository = "https://github.com"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    collapse_sub_table(&mut tables, "project", "urls", 120);
}

#[test]
fn test_collect_all_sub_tables_non_existent_parent() {
    let toml = indoc! {r#"
        [project]
        name = "foo"
    "#};
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);

    let mut result = Vec::new();
    collect_all_sub_tables(&tables, "nonexistent", &mut result);

    assert!(result.is_empty());
}

#[test]
fn test_collect_all_sub_tables_deduplication() {
    let toml = indoc! {r#"
        [project]
        urls.homepage = "https://example.com"

        [project.urls]
        repository = "https://github.com"
    "#};
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);

    let mut result = Vec::new();
    collect_all_sub_tables(&tables, "project", &mut result);

    let urls_count = result.iter().filter(|s| *s == "project.urls").count();
    assert_eq!(urls_count, 1, "should deduplicate sub-tables");
}

#[test]
fn test_collapse_sub_table_empty_sub_table() {
    let toml = indoc! {r#"
        [project]
        name = "foo"

        [project.urls]
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    collapse_sub_table(&mut tables, "project", "urls", 120);

    let main = tables.get("project").unwrap();
    let table = main[0].borrow();
    let txt = table.iter().map(|e| e.to_string()).collect::<String>();
    insta::assert_snapshot!(txt, @r#"
    [project]
    name = "foo"
    urls = {}
    "#);
}

#[test]
fn test_collapse_sub_tables_deeply_nested_empty_table() {
    let toml = indoc! {r#"
        [tool.hatch]

        [tool.hatch.metadata.hooks.docstring-description]
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    collapse_sub_tables(&mut tables, "tool.hatch");

    let main = tables.get("tool.hatch").unwrap();
    let table = main[0].borrow();
    let txt = table.iter().map(|e| e.to_string()).collect::<String>();
    insta::assert_snapshot!(txt, @r#"
    [tool.hatch]
    metadata.hooks.docstring-description = {}
    "#);
}

#[test]
fn test_expand_sub_table_entry_without_key() {
    let toml = indoc! {r#"
        [project]
        name = "foo"
        urls.homepage = "https://example.com"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    expand_sub_table(&mut tables, "project", "urls");

    assert!(tables.header_to_pos.contains_key("project.urls"));
}

#[test]
fn test_collect_all_sub_tables_parent_without_dotted_keys() {
    let toml = indoc! {r#"
        [project]
        name = "foo"
        version = "1.0"
    "#};
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);

    let mut result = Vec::new();
    collect_all_sub_tables(&tables, "project", &mut result);

    assert!(result.is_empty());
}

#[test]
fn test_collapse_sub_table_with_comments() {
    let toml = indoc! {r#"
        [project]
        name = "foo"

        [project.urls]
        # This is a comment
        homepage = "https://example.com"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    collapse_sub_table(&mut tables, "project", "urls", 120);

    let main = tables.get("project").unwrap();
    let table = main[0].borrow();
    let txt = table.iter().map(|e| e.to_string()).collect::<String>();
    assert!(txt.contains("urls.homepage"));
}

#[test]
fn test_expand_sub_table_with_multiple_dotted_keys() {
    let toml = indoc! {r#"
        [project]
        name = "foo"
        urls.homepage = "https://example.com"
        urls.repository = "https://github.com"
        urls.documentation = "https://docs.example.com"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    expand_sub_table(&mut tables, "project", "urls");

    assert!(tables.header_to_pos.contains_key("project.urls"));
    let urls = tables.get("project.urls").unwrap();
    let table = urls[0].borrow();
    let txt = table.iter().map(|e| e.to_string()).collect::<String>();
    assert!(txt.contains("homepage"));
    assert!(txt.contains("repository"));
    assert!(txt.contains("documentation"));
}

#[test]
fn test_tables_duplicate_merge_adds_newline() {
    let toml = concat!(
        "[project]\n",
        "name = \"foo\"", // no trailing newline here to trigger branch
        "[tool]\n",
        "x = 1\n",
        "[project]\n",
        "version = \"1.0\"",
    );
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);

    assert_eq!(tables.header_to_pos["project"].len(), 1);
    let project = tables.get("project").unwrap();
    let table = project[0].borrow();
    let txt = table.iter().map(|e| e.to_string()).collect::<String>();
    assert!(txt.contains("name"));
    assert!(txt.contains("version"));
}

#[test]
fn test_tables_from_ast_with_root_comments_only() {
    let toml = indoc! {r#"
        # This is a root comment
        # Another comment
        [project]
        name = "foo"
    "#};
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);

    assert!(tables.header_to_pos.contains_key("project"));
}

#[test]
fn test_apply_table_formatting_collapse() {
    let toml = indoc! {r#"
        [project]
        name = "foo"

        [project.urls]
        homepage = "https://example.com"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    apply_table_formatting(&mut tables, |_| true, &["project"], 120);

    let main = tables.get("project").unwrap();
    let table = main[0].borrow();
    let txt = table.iter().map(|e| e.to_string()).collect::<String>();
    assert!(txt.contains("urls.homepage"));
}

#[test]
fn test_apply_table_formatting_expand() {
    let toml = indoc! {r#"
        [project]
        name = "foo"
        urls.homepage = "https://example.com"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    apply_table_formatting(&mut tables, |_| false, &["project"], 120);

    assert!(tables.header_to_pos.contains_key("project.urls"));
}

#[test]
fn test_apply_table_formatting_deeply_nested() {
    let toml = indoc! {r#"
        [tool.ruff]
        line-length = 120

        [tool.ruff.lint.flake8-tidy-imports.banned-api]
        "collections.namedtuple".msg = "Use typing.NamedTuple"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    apply_table_formatting(
        &mut tables,
        |name| name != "tool.ruff.lint.flake8-tidy-imports.banned-api",
        &["tool.ruff"],
        120,
    );

    assert!(
        tables
            .header_to_pos
            .contains_key("tool.ruff.lint.flake8-tidy-imports.banned-api"),
        "deeply nested table should stay expanded"
    );
}

#[test]
fn test_apply_table_formatting_quoted_keys() {
    let toml = indoc! {r#"
        [tool.ruff.lint.flake8-tidy-imports."banned-api"]
        "typing.Dict".msg = "use dict"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    apply_table_formatting(&mut tables, |_| false, &["tool.ruff"], 120);

    assert!(
        tables
            .header_to_pos
            .contains_key("tool.ruff.lint.flake8-tidy-imports.\"banned-api\"")
    );
}

#[test]
fn test_apply_table_formatting_multiple_prefixes() {
    let toml = indoc! {r#"
        [project]
        name = "foo"
        urls.homepage = "https://example.com"

        [build-system]
        requires.build = "setuptools"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    apply_table_formatting(&mut tables, |_| false, &["project", "build-system"], 120);

    assert!(tables.header_to_pos.contains_key("project.urls"));
    assert!(tables.header_to_pos.contains_key("build-system.requires"));
}

#[test]
fn test_collect_all_sub_tables_includes_intermediate_parents() {
    let toml = indoc! {r#"
        [tool.ruff.lint.flake8-tidy-imports.banned-api]
        "typing.Dict".msg = "use dict"
    "#};
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);

    let mut result = Vec::new();
    collect_all_sub_tables(&tables, "tool.ruff", &mut result);

    assert!(result.contains(&String::from("tool.ruff.lint")));
    assert!(result.contains(&String::from("tool.ruff.lint.flake8-tidy-imports")));
    assert!(result.contains(&String::from("tool.ruff.lint.flake8-tidy-imports.banned-api")));
}

#[test]
fn test_tables_reorder_with_empty_key() {
    let toml = indoc! {r#"
        [tool]
        key = "value"
    "#};
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);
    tables.reorder(&root_ast, &["tool"], &[]);
    let result = format_toml(&root_ast, 120);
    insta::assert_snapshot!(result, @r#"
    [tool]
    key = "value"
    "#);
}

#[test]
fn test_expand_sub_table_multiple_main_positions_no_expand() {
    let toml = indoc! {r#"
        [project]
        name = "pkg1"

        [other]
        key = "value"

        [project]
        version = "1.0"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);
    expand_sub_table(&mut tables, "project", "scripts");
    tables.reorder(&root_ast, &["project", "other"], &[]);
    let result = format_toml(&root_ast, 120);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "pkg1"

    [project]
    version = "1.0"

    [other]
    key = "value"
    "#);
}

#[test]
fn test_collapse_sub_table_multiple_sub_positions_unchanged() {
    let toml = indoc! {r#"
        [project]
        name = "pkg"

        [project.scripts]
        script1 = "cmd1"

        [project.scripts]
        script2 = "cmd2"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);
    collapse_sub_table(&mut tables, "project", "scripts", 120);
    tables.reorder(&root_ast, &["project", "project.scripts"], &[]);
    let result = format_toml(&root_ast, 120);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "pkg"
    scripts.script1 = "cmd1"
    scripts.script2 = "cmd2"
    "#);
}

#[test]
fn test_apply_table_formatting_expand_mode() {
    let toml = indoc! {r#"
        [project]
        scripts.run = "python main.py"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);
    apply_table_formatting(&mut tables, |_name| false, &["project"], 120);
    tables.reorder(&root_ast, &["project", "project.scripts"], &[]);
    let result = format_toml(&root_ast, 120);
    insta::assert_snapshot!(result, @r#"
    [project]
    [project.scripts]
    run = "python main.py"
    "#);
}

#[test]
fn test_expand_dotted_to_sub_tables_array_of_tables_unchanged() {
    let toml = indoc! {r#"
        [[tool]]
        a.b = 1

        [[tool]]
        c.d = 2
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);
    expand_sub_tables(&mut tables, "tool");
    tables.reorder(&root_ast, &["tool"], &[]);
    let result = format_toml(&root_ast, 120);
    insta::assert_snapshot!(result, @r#"
    [[tool]]
    a.b = 1

    [[tool]]
    c.d = 2
    "#);
}

#[test]
fn test_tables_from_ast_with_table_children() {
    let toml = indoc! {r#"
        [project]
        name = "test"
        version = "1.0"
    "#};
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);
    tables.reorder(&root_ast, &["project"], &[]);
    let result = format_toml(&root_ast, 120);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    version = "1.0"
    "#);
}

#[test]
fn test_split_quoted_key_no_dot() {
    let toml = indoc! {r#"
        [project]
        name = "test"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);
    apply_table_formatting(&mut tables, |_| true, &["project"], 120);
    tables.reorder(&root_ast, &["project"], &[]);
    let result = format_toml(&root_ast, 120);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    "#);
}

#[test]
fn test_collapse_sub_table_with_quoted_key() {
    let toml = indoc! {r#"
        [project]
        name = "test"

        [project.scripts]
        "quoted-key" = "value"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);
    collapse_sub_table(&mut tables, "project", "scripts", 120);
    tables.reorder(&root_ast, &["project", "project.scripts"], &[]);
    let result = format_toml(&root_ast, 120);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    scripts."quoted-key" = "value"
    "#);
}

#[test]
fn test_collapse_sub_table_with_literal_quoted_key() {
    let toml = indoc! {r#"
        [project]
        name = "test"

        [project.scripts]
        'literal-key' = "value"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);
    collapse_sub_table(&mut tables, "project", "scripts", 120);
    tables.reorder(&root_ast, &["project", "project.scripts"], &[]);
    let result = format_toml(&root_ast, 120);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"
    scripts.'literal-key' = "value"
    "#);
}

#[test]
fn test_collapse_sub_tables_skips_array_of_tables() {
    let toml = indoc! {r#"
        [project]
        name = "test"

        [[project.authors]]
        name = "Alice"
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);
    collapse_sub_tables(&mut tables, "project");
    tables.reorder(&root_ast, &["project", "project.authors"], &[]);
    let result = format_toml(&root_ast, 120);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"

    [[project.authors]]
    name = "Alice"
    "#);
}

#[test]
fn test_reorder_array_of_tables_multiple_entries() {
    let toml = indoc! {r#"
        [[project.authors]]
        name = "Alice"

        [[project.authors]]
        name = "Bob"

        [project]
        name = "test"
    "#};
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);
    tables.reorder(&root_ast, &["project", "project.authors"], &[]);
    let result = format_toml(&root_ast, 120);
    insta::assert_snapshot!(result, @r#"
    [project]
    name = "test"

    [[project.authors]]
    name = "Alice"

    [[project.authors]]
    name = "Bob"
    "#);
}

#[test]
fn test_apply_table_formatting_with_nested_subtables_and_direct_entries() {
    let toml = indoc! {r#"
        [tool.ruff]
        line-length = 120

        [tool.ruff.lint]
        select = ["ALL"]

        [tool.ruff.lint.extra]
        ok = 1
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    let mut all_sub_tables: Vec<String> = Vec::new();
    collect_all_sub_tables(&tables, "tool.ruff", &mut all_sub_tables);
    eprintln!("All sub-tables: {:?}", all_sub_tables);

    apply_table_formatting(&mut tables, |_| true, &["tool.ruff"], 120);

    let main = tables.get("tool.ruff").unwrap();
    let table = main[0].borrow();
    let txt = table.iter().map(|e| e.to_string()).collect::<String>();
    insta::assert_snapshot!(txt, @r#"
    [tool.ruff]
    line-length = 120
    lint.select = ["ALL"]

    lint.extra.ok = 1
    "#);
}

#[test]
fn test_apply_table_formatting_with_empty_intermediate_table() {
    let toml = indoc! {r#"
        [tool.ruff]
        line-length = 120

        [tool.ruff.lint]

        [tool.ruff.lint.extra]
        ok = 1
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    apply_table_formatting(&mut tables, |_| true, &["tool.ruff"], 120);

    let main = tables.get("tool.ruff").unwrap();
    let table = main[0].borrow();
    let txt = table.iter().map(|e| e.to_string()).collect::<String>();
    insta::assert_snapshot!(txt, @r#"
    [tool.ruff]
    line-length = 120
    lint.extra.ok = 1
    "#);
}

#[test]
fn test_collapse_sub_table_trailing_comment_on_single_line_array() {
    let toml = indoc! {r#"
        [tool.coverage.run]
        core = "sysmon" # default for 3.14+, available for 3.12+
        disable_warnings = [ "no-sysmon" ] # 3.11 and earlier
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);
    collapse_sub_table(&mut tables, "tool.coverage", "run", 120);
    tables.reorder(&root_ast, &["tool.coverage", "tool.coverage.run"], &[]);
    let result = format_toml(&root_ast, 120);
    insta::assert_snapshot!(result, @r#"
    [tool.coverage]
    run.core = "sysmon"  # default for 3.14+, available for 3.12+
    run.disable_warnings = [ "no-sysmon" ]  # 3.11 and earlier
    "#);
}

#[test]
fn test_collapse_sub_table_empty_parent_with_subtable() {
    let toml = indoc! {r#"
        [parent]
        name = "test"

        [parent.child]

        [parent.child.nested]
        value = 1
    "#};
    let root_ast = parse(toml);
    let mut tables = Tables::from_ast(&root_ast);

    collapse_sub_table(&mut tables, "parent.child", "nested", 120);

    let txt = {
        let main = tables.get("parent.child").unwrap();
        let table = main[0].borrow();
        table.iter().map(|e| e.to_string()).collect::<String>()
    };
    insta::assert_snapshot!(txt, @r#"
    [parent.child]
    nested.value = 1
    "#);

    collapse_sub_table(&mut tables, "parent", "child", 120);

    let txt2 = {
        let main2 = tables.get("parent").unwrap();
        let table2 = main2[0].borrow();
        table2.iter().map(|e| e.to_string()).collect::<String>()
    };
    insta::assert_snapshot!(txt2, @r#"
    [parent]
    name = "test"
    child.nested.value = 1
    "#);
}

#[test]
fn test_reorder_table_keys_mixed_quote_styles() {
    let toml = indoc! {r#"
        [tool.ruff.lint.per-file-ignores]
        'tests/*' = [ "T20" ]
        "flexget/*" = [ "PTH" ]
    "#};
    let root_ast = parse(toml);
    let tables = Tables::from_ast(&root_ast);
    let keys = reorder_and_get_keys(&tables, "tool.ruff.lint.per-file-ignores", &[""]);
    insta::assert_snapshot!(keys.join(", "), @r#""flexget/*", 'tests/*'"#);
}
