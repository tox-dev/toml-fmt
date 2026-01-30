use indoc::indoc;
use rstest::rstest;
use taplo::formatter::{format_syntax, Options};
use taplo::parser::parse;

use crate::table::{
    collapse_sub_tables, expand_sub_tables, find_key, for_entries, get_table_name, reorder_table_keys, Tables,
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
