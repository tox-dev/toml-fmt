use indoc::indoc;
use rstest::rstest;
use taplo::formatter::{format_syntax, Options};
use taplo::parser::parse;

use crate::table::{
    apply_table_formatting, collapse_sub_table, collapse_sub_tables, collect_all_sub_tables, expand_sub_table,
    expand_sub_tables, find_key, for_entries, get_table_name, reorder_table_keys, Tables,
};

#[test]
fn test_tables_from_ast_empty() {
    let root_ast = parse("").into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
    let tables = Tables::from_ast(&root_ast);

    let result = tables.get("nonexistent");
    assert!(result.is_none());
}

#[rstest]
#[case::simple_reorder(
    indoc! {r#"
        [tool.black]
        line-length = 120

        [project]
        name = "foo"
    "#},
    &["project", "tool.black"],
    indoc! {r#"
        [project]
        name = "foo"

        [tool.black]
        line-length = 120
    "#}
)]
#[case::keep_sub_tables_together(
    indoc! {r#"
        [tool.pytest]
        testpaths = ["tests"]

        [project]
        name = "foo"

        [tool.black]
        line-length = 120
    "#},
    &["project", "tool"],
    indoc! {r#"
        [project]
        name = "foo"

        [tool.pytest]
        testpaths = ["tests"]

        [tool.black]
        line-length = 120
    "#}
)]
#[case::unknown_tools_file_order_subtables_alphabetical(
    // tool.flake8 appears first in file, so it stays first
    // tool.cff-from-621 appears second, with its subtable after
    // tool.coverage subtables are sorted alphabetically (report < run)
    // tool.hatch subtables are sorted alphabetically (linting < mypy < unit-tests)
    indoc! {r#"
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
    "#},
    &["project", "tool"],
    indoc! {r#"
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
    "#}
)]
#[case::same_tool_subtables_sorted_short_to_long(
    // Within same tool, subtables sorted alphabetically (short before long)
    indoc! {r#"
        [tool.hatch.envs.test.scripts]
        [tool.hatch.envs.test]
        [tool.hatch]
    "#},
    &["tool"],
    indoc! {r#"
        [tool.hatch]
        [tool.hatch.envs.test]
        [tool.hatch.envs.test.scripts]
    "#}
)]
#[case::different_tools_preserve_file_order(
    // tool.zebra appears first in file, tool.alpha appears second
    // file order is preserved between different tools
    indoc! {r#"
        [tool.zebra]

        [tool.alpha]
    "#},
    &["tool"],
    indoc! {r#"
        [tool.zebra]

        [tool.alpha]
    "#}
)]
fn test_tables_reorder(#[case] start: &str, #[case] order: &[&str], #[case] expected: &str) {
    let root_ast = parse(start).into_syntax().clone_for_update();
    let tables = Tables::from_ast(&root_ast);
    tables.reorder(&root_ast, order, &["tool"]); // tool.* uses two-part keys

    let res = format_syntax(root_ast, Options::default());
    assert_eq!(res, expected);
}

#[test]
fn test_get_table_name_table_header() {
    let toml = "[project]";
    let root_ast = parse(toml).into_syntax().clone_for_update();

    let child = root_ast.children_with_tokens().next().expect("No table header found");
    let name = get_table_name(&child);
    assert_eq!(name, "project");
}

#[test]
fn test_get_table_name_array_header() {
    let toml = "[[project.authors]]";
    let root_ast = parse(toml).into_syntax().clone_for_update();

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
    let root_ast = parse(toml).into_syntax().clone_for_update();

    let child = root_ast.children_with_tokens().next().expect("No entry found");
    let name = get_table_name(&child);
    assert_eq!(name, "");
}

#[rstest]
#[case::simple_reorder(
    indoc! {r#"
        [project]
        version = "1.0"
        name = "foo"
    "#},
    &["name", "version"],
    vec!["name", "version"]
)]
#[case::with_comments(
    indoc! {r#"
        [project]
        # version comment
        version = "1.0"
        # name comment
        name = "foo"
    "#},
    &["name", "version"],
    vec!["name", "version"]
)]
#[case::nested_keys(
    indoc! {r#"
        [project]
        z = "last"
        a.b = "nested"
        a.c = "another"
    "#},
    &["a", "z"],
    vec!["a.b", "a.c", "z"]
)]
fn test_reorder_table_keys(#[case] start: &str, #[case] order: &[&str], #[case] expected_order: Vec<&str>) {
    let root_ast = parse(start).into_syntax().clone_for_update();
    let tables = Tables::from_ast(&root_ast);

    if let Some(table_refs) = tables.get("project") {
        for table_ref in table_refs {
            let mut table = table_ref.borrow_mut();
            reorder_table_keys(&mut table, order);
        }
    }

    if let Some(table_refs) = tables.get("project") {
        for table_ref in table_refs {
            let table = table_ref.borrow();
            let mut keys = Vec::new();
            for_entries(&table, &mut |key, _node| {
                keys.push(key);
            });
            assert_eq!(keys, expected_order);
        }
    }
}

#[test]
fn test_for_entries() {
    let toml = indoc! {r#"
        [project]
        name = "foo"
        version = "1.0"
    "#};
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();

    let result = find_key(&root_ast, "name");
    assert!(result.is_some());
}

#[test]
fn test_find_key_non_existing() {
    let toml = indoc! {r#"
        name = "foo"
        version = "1.0"
    "#};
    let root_ast = parse(toml).into_syntax().clone_for_update();

    let result = find_key(&root_ast, "nonexistent");
    assert!(result.is_none());
}

#[rstest]
#[case::collapse_simple(
    indoc! {r#"
        [project]
        name = "foo"

        [project.optional-dependencies]
        dev = ["pytest"]
    "#},
    "project",
    true
)]
#[case::no_sub_tables(
    indoc! {r#"
        [project]
        name = "foo"

        [tool.black]
        line-length = 120
    "#},
    "project",
    false
)]
fn test_collapse_sub_tables(#[case] start: &str, #[case] table_name: &str, #[case] has_sub_tables: bool) {
    let root_ast = parse(start).into_syntax().clone_for_update();
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

#[test]
fn test_collapse_sub_tables_creates_main_if_missing() {
    let toml = indoc! {r#"
        [project.scripts]
        cli = "pkg:main"
    "#};
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
    let tables = Tables::from_ast(&root_ast);

    assert!(tables.header_to_pos.contains_key(""));

    tables.reorder(&root_ast, &["", "project"], &[]);
    let res = format_syntax(root_ast, Options::default());
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
    let tables = Tables::from_ast(&root_ast);
    tables.reorder(&root_ast, &["project", "tool"], &["tool"]);

    let res = format_syntax(root_ast, Options::default());
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
    let tables = Tables::from_ast(&root_ast);

    if let Some(table_refs) = tables.get("project") {
        for table_ref in table_refs {
            let mut table = table_ref.borrow_mut();
            reorder_table_keys(&mut table, &["name"]);
        }
    }

    if let Some(table_refs) = tables.get("project") {
        for table_ref in table_refs {
            let table = table_ref.borrow();
            let mut keys = Vec::new();
            for_entries(&table, &mut |key, _node| {
                keys.push(key);
            });
            assert_eq!(keys[0], "name");
        }
    }
}

#[test]
fn test_tables_duplicate_no_newline_between() {
    let toml = "[project]\nname = \"foo\"\n[tool]\nx = 1\n[project]\nversion = \"1.0\"";
    let root_ast = parse(toml).into_syntax().clone_for_update();
    let tables = Tables::from_ast(&root_ast);

    let refs = tables.get("project").unwrap();
    assert_eq!(refs.len(), 1);
}

#[test]
fn test_tables_duplicate_immediate() {
    let toml = "[project]\nname = \"foo\"\n[tool]\nx = 1\n[project]\nversion = \"1.0\"";
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
    let tables = Tables::from_ast(&root_ast);

    tables.reorder(&root_ast, &["", "project"], &[]);
    let res = format_syntax(root_ast, Options::default());
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
    let tables = Tables::from_ast(&root_ast);

    if let Some(table_refs) = tables.get("project.nested") {
        for table_ref in table_refs {
            let mut table = table_ref.borrow_mut();
            reorder_table_keys(&mut table, &["a"]);
        }
    }
}

#[test]
fn test_reorder_table_no_trailing_newline() {
    let toml = "[project]\nname = \"foo\"";
    let root_ast = parse(toml).into_syntax().clone_for_update();
    let tables = Tables::from_ast(&root_ast);

    if let Some(table_refs) = tables.get("project") {
        for table_ref in table_refs {
            let mut table = table_ref.borrow_mut();
            reorder_table_keys(&mut table, &["name"]);
        }
    }
}

#[test]
fn test_reorder_table_keys_consecutive_entries() {
    let toml = indoc! {r#"
        [project]
        a = 1
        b = 2
        c = 3
    "#};
    let root_ast = parse(toml).into_syntax().clone_for_update();
    let tables = Tables::from_ast(&root_ast);

    if let Some(table_refs) = tables.get("project") {
        for table_ref in table_refs {
            let mut table = table_ref.borrow_mut();
            reorder_table_keys(&mut table, &["c", "b", "a"]);
        }
    }

    if let Some(table_refs) = tables.get("project") {
        for table_ref in table_refs {
            let table = table_ref.borrow();
            let mut keys = Vec::new();
            for_entries(&table, &mut |key, _node| {
                keys.push(key);
            });
            assert_eq!(keys, vec!["c", "b", "a"]);
        }
    }
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
    let mut tables = Tables::from_ast(&root_ast);

    collapse_sub_tables(&mut tables, "project");
}

#[test]
fn test_reorder_keys_consecutive_no_newline() {
    let toml = "[project]\na = 1\nb = 2";
    let root_ast = parse(toml).into_syntax().clone_for_update();
    let tables = Tables::from_ast(&root_ast);

    if let Some(table_refs) = tables.get("project") {
        for table_ref in table_refs {
            let mut table = table_ref.borrow_mut();
            reorder_table_keys(&mut table, &["b", "a"]);
        }
    }
}

#[test]
fn test_reorder_same_tool_group() {
    let toml = indoc! {r#"
        [tool.black]
        line-length = 120

        [tool.ruff]
        select = ["E"]
    "#};
    let root_ast = parse(toml).into_syntax().clone_for_update();
    let tables = Tables::from_ast(&root_ast);
    tables.reorder(&root_ast, &["tool"], &["tool"]);

    let res = format_syntax(root_ast, Options::default());
    assert!(res.contains("[tool.black]"));
    assert!(res.contains("[tool.ruff]"));
}

#[test]
fn test_reorder_different_groups_no_trailing_newline() {
    let toml = "[tool.black]\nline-length = 120\n[project]\nname = \"foo\"";
    let root_ast = parse(toml).into_syntax().clone_for_update();
    let tables = Tables::from_ast(&root_ast);
    tables.reorder(&root_ast, &["project", "tool"], &["tool"]);

    let res = format_syntax(root_ast, Options::default());
    assert!(res.contains("[project]"));
    assert!(res.contains("[tool.black]"));
}

#[test]
fn test_load_keys_entries_without_newline() {
    let toml = "[project]\na = 1\nb = 2\nc = 3";
    let root_ast = parse(toml).into_syntax().clone_for_update();
    let tables = Tables::from_ast(&root_ast);

    if let Some(table_refs) = tables.get("project") {
        for table_ref in table_refs {
            let mut table = table_ref.borrow_mut();
            reorder_table_keys(&mut table, &["c", "b", "a"]);
        }
    }

    if let Some(table_refs) = tables.get("project") {
        for table_ref in table_refs {
            let table = table_ref.borrow();
            let mut keys = Vec::new();
            for_entries(&table, &mut |key, _node| {
                keys.push(key);
            });
            assert_eq!(keys, vec!["c", "b", "a"]);
        }
    }
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
    let tables = Tables::from_ast(&root_ast);
    tables.reorder(&root_ast, &["build-system", "project"], &[]);

    let res = format_syntax(root_ast, Options::default());
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
    let tables = Tables::from_ast(&root_ast);
    tables.reorder(&root_ast, &["build-system", "project"], &[]);

    let res = format_syntax(root_ast, Options::default());
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
    let tables = Tables::from_ast(&root_ast);
    tables.reorder(&root_ast, &["build-system", "project"], &[]);

    let res = format_syntax(root_ast, Options::default());
    // Comment should stay with [build-system] even with blank line before it
    assert!(res.starts_with("# comment for build-system\n[build-system]"));
}

#[rstest]
#[case::issue_124_comment_inside_table_and_before_next_table(
    indoc! {r#"
        [project]
        name = "test"
        # comment inside project table
        version = "1.0"

        scripts.main = "app:main"

        # comment for dependency-groups
        [dependency-groups]
        test = ["pytest"]
    "#},
    &["dependency-groups", "project"],
    indoc! {r#"
        # comment for dependency-groups
        [dependency-groups]
        test = ["pytest"]

        [project]
        name = "test"
        # comment inside project table
        version = "1.0"

        scripts.main = "app:main"
    "#}
)]
fn test_issue_124(#[case] start: &str, #[case] order: &[&str], #[case] expected: &str) {
    let root_ast = parse(start).into_syntax().clone_for_update();
    let tables = Tables::from_ast(&root_ast);
    tables.reorder(&root_ast, order, &[]);
    let result = format_syntax(root_ast, Options::default());
    assert_eq!(result, expected);
}

#[test]
fn test_expand_sub_tables_creates_sub_table() {
    let toml = indoc! {r#"
        [project]
        name = "foo"
        urls.homepage = "https://example.com"
        urls.repository = "https://github.com/example"
    "#};
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
    let mut tables = Tables::from_ast(&root_ast);

    collapse_sub_table(&mut tables, "project", "authors", 80);

    let authors = tables.get("project.authors").unwrap();
    let authors_table = authors[0].borrow();
    assert!(!authors_table.is_empty(), "wide array tables should not be collapsed");
}

#[test]
fn test_collapse_sub_table_non_existent() {
    let toml = indoc! {r#"
        [project]
        name = "foo"
    "#};
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
    let mut tables = Tables::from_ast(&root_ast);

    collapse_sub_table(&mut tables, "project", "urls", 120);
}

#[test]
fn test_collect_all_sub_tables_non_existent_parent() {
    let toml = indoc! {r#"
        [project]
        name = "foo"
    "#};
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
    let mut tables = Tables::from_ast(&root_ast);

    collapse_sub_table(&mut tables, "project", "urls", 120);
}

#[test]
fn test_expand_sub_table_entry_without_key() {
    let toml = indoc! {r#"
        [project]
        name = "foo"
        urls.homepage = "https://example.com"
    "#};
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
    let mut tables = Tables::from_ast(&root_ast);

    apply_table_formatting(&mut tables, |_| false, &["tool.ruff"], 120);

    assert!(tables
        .header_to_pos
        .contains_key("tool.ruff.lint.flake8-tidy-imports.\"banned-api\""));
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
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
    let root_ast = parse(toml).into_syntax().clone_for_update();
    let tables = Tables::from_ast(&root_ast);

    let mut result = Vec::new();
    collect_all_sub_tables(&tables, "tool.ruff", &mut result);

    assert!(result.contains(&String::from("tool.ruff.lint")));
    assert!(result.contains(&String::from("tool.ruff.lint.flake8-tidy-imports")));
    assert!(result.contains(&String::from("tool.ruff.lint.flake8-tidy-imports.banned-api")));
}
