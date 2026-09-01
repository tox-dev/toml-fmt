//! How a tox configuration is formatted, wherever the file writes it.

use indoc::indoc;
use insta::assert_snapshot;
use toml_doc::{Document, LineEnding};
use tox_rules::reorder_tables;

#[test]
fn test_reorder_table_reorder_no_env_list() {
    let start = indoc! {r#"
        # comment
        requires = ["tox>=4.22"]

        [demo]
        desc = "demo"

        [env.type]
        description = "type"

        [env.docs]
        description = "docs"

        [env_run_base]
        description = "base"

    "#};
    let got = reorder_table_helper(start);
    assert_snapshot!(got, @r#"
    # comment
    requires = [ "tox>=4.22" ]

    [env_run_base]
    description = "base"

    [env.docs]
    description = "docs"

    [env.type]
    description = "type"

    [demo]
    desc = "demo"
    "#);
}

#[test]
fn test_reorder_table_reorder_with_env_list() {
    let start = indoc! {r#"
        env_list = ["docs", "type", "lint"]

        [env.type]
        description = "type"

        [env.docs]
        description = "docs"

        [env.lint]
        description = "lint"

    "#};
    let got = reorder_table_helper(start);
    assert_snapshot!(got, @r#"
    env_list = [ "docs", "type", "lint" ]

    [env.docs]
    description = "docs"

    [env.type]
    description = "type"

    [env.lint]
    description = "lint"
    "#);
}

#[test]
fn test_reorder_table_reorder_env_list_partial() {
    let start = indoc! {r#"
        env_list = ["type"]

        [env.lint]
        description = "lint"

        [env.docs]
        description = "docs"

        [env.type]
        description = "type"

    "#};
    let got = reorder_table_helper(start);
    assert_snapshot!(got, @r#"
    env_list = [ "type" ]

    [env.type]
    description = "type"

    [env.docs]
    description = "docs"

    [env.lint]
    description = "lint"
    "#);
}

#[test]
fn test_reorder_no_root_table() {
    let start = indoc! {r#"
        [env.test]
        description = "test"
    "#};
    let got = reorder_table_helper(start);
    assert_snapshot!(got, @r#"
    [env.test]
    description = "test"
    "#);
}

#[test]
fn test_reorder_root_table_no_env_list_key() {
    let start = indoc! {r#"
        requires = ["tox>=4"]

        [env.test]
        description = "test"
    "#};
    let got = reorder_table_helper(start);
    assert_snapshot!(got, @r#"
    requires = [ "tox>=4" ]

    [env.test]
    description = "test"
    "#);
}

#[test]
fn test_reorder_env_list_not_array() {
    let start = indoc! {r#"
        env_list = "test"

        [env.type]
        description = "type"

        [env.docs]
        description = "docs"
    "#};
    let got = reorder_table_helper(start);
    assert_snapshot!(got, @r#"
    env_list = "test"

    [env.docs]
    description = "docs"

    [env.type]
    description = "type"
    "#);
}

#[test]
fn test_reorder_empty_env_list() {
    let start = indoc! {r#"
        env_list = []

        [env.type]
        description = "type"

        [env.docs]
        description = "docs"
    "#};
    let got = reorder_table_helper(start);
    assert_snapshot!(got, @r#"
    env_list = []

    [env.docs]
    description = "docs"

    [env.type]
    description = "type"
    "#);
}

#[test]
fn test_reorder_env_list_with_env_run_base() {
    let start = indoc! {r#"
        env_list = ["test"]

        [env.test]
        description = "test"

        [env_run_base]
        commands = ["pytest"]
    "#};
    let got = reorder_table_helper(start);
    assert_snapshot!(got, @r#"
    env_list = [ "test" ]

    [env_run_base]
    commands = [ "pytest" ]

    [env.test]
    description = "test"
    "#);
}

/// `pyproject.toml` writes the same tables under `tool.tox`, which the prefixed entry points read.
#[test]
fn test_fix_envs_under_a_prefix_leaves_other_tools_alone() {
    let start = indoc! {r#"
        [project]
        dependencies = ["b", "a"]

        [tool.tox.env.test]
        deps = ["b", "a"]
    "#};
    let got = evaluate(start, |document| {
        tox_rules::fix_envs_with_prefix(document, "tool.tox");
    });
    assert_snapshot!(got, @r#"
    [project]
    dependencies = [ "b", "a" ]

    [tool.tox.env.test]
    deps = [ "a", "b" ]
    "#);
}

/// The same rules serve a `pyproject.toml`, where the keys they read sit under a table rather than
/// before the first header.
#[test]
fn test_root_rules_read_the_table_a_prefix_names() {
    let start = indoc! {r#"
        [tool.tox]
        envlist = ["py313", "py312"]
        minversion = "4.0"
        "#};

    let result = evaluate(start, |document| {
        tox_rules::normalize_aliases_with_prefix(document, "tool.tox");
        tox_rules::fix_root_with_prefix(document, "tool.tox");
    });

    assert_snapshot!(result, @r#"
    [tool.tox]
    min_version = "4.0"
    env_list = [ "py313", "py312" ]
    "#);
}

/// A reference inside a `pyproject.toml` spells its path from the `tool.tox` table, so a rename in
/// a table that prefix holds is one it names; a table whose path merely ends the same way is not.
#[test]
fn test_a_reference_under_a_prefix_names_the_table_the_prefix_holds() {
    let start = indoc! {r#"
        [tool.tox.env.docs]
        setenv = { A = "1" }

        [tool.tox.env.test]
        held = { replace = "ref", of = ["env", "docs", "setenv"] }
        other = { replace = "ref", of = ["docs", "setenv"] }
        "#};

    let result = evaluate(start, |document| {
        tox_rules::normalize_aliases_with_prefix(document, "tool.tox");
    });

    assert_snapshot!(result, @r#"
    [tool.tox.env.docs]
    set_env = { A = "1" }

    [tool.tox.env.test]
    held = { replace = "ref", of = [ "env", "docs", "set_env" ] }
    other = { replace = "ref", of = [ "docs", "setenv" ] }
    "#);
}

/// A key that names a list of requirements holds nothing to normalize where the file wrote
/// something else, and the same goes for the one that names a list of variables to pass through.
#[test]
fn a_value_that_is_not_a_list_is_left_as_written() {
    let source = "[env.test]\ndeps = \"not-an-array\"\npass_env = \"not-an-array\"\n";

    assert_eq!(evaluate(source, tox_rules::fix_envs), source);
}

/// The older spellings tox still reads are written the way it documents them, and a `{ replace =
/// "ref" }` that names a key one of those moved takes the reference along.
#[test]
fn test_an_alias_is_written_the_way_tox_documents_it() {
    let start = indoc! {r#"
        envlist = ["a"]
        toxinidir = "."

        [env.a]
        setenv = { A = "1" }
        basepython = { replace = "ref", of = ["env", "a", "setenv"] }
        usedevelop = false
        "#};

    assert_snapshot!(formatted(start), @r#"
    env_list = [ "a" ]
    tox_root = "."
    [env.a]
    base_python = { replace = "ref", of = [ "env", "a", "set_env" ] }
    use_develop = false
    set_env = { A = "1" }
    "#);
}

/// `requires` names distributions, so each is normalized and the list sorts.
#[test]
fn test_the_root_requires_list_is_normalized_and_sorted() {
    assert_snapshot!(formatted("requires = [ \"Tox_Uv>=1.0.0\", \"tox>=4.22\" ]\n"), @r#"
    requires = [ "tox>=4.22", "tox-uv>=1" ]
    "#);
}

/// An environment written as dotted keys under the root reads the same as one written under a
/// header, so the root order holds a slot for every key of it.
#[test]
fn test_an_environment_written_as_dotted_keys_reads_like_a_table() {
    let start = indoc! {r#"
        env.test.commands = [["pytest"]]
        env.test.description = "test"
        env_list = ["test"]
        env_run_base.description = "base"
        "#};

    assert_snapshot!(formatted(start), @r#"
    env_list = [ "test" ]
    env.test.description = "test"
    env.test.commands = [ [ "pytest" ] ]
    env_run_base.description = "base"
    "#);
}

/// tox reads `use_develop` before `package` and installs an editable package whatever `package`
/// says, so a `package` key already there is given the mode the environment ran with, and the
/// comments the older key carried move with it.
#[test]
fn test_use_develop_gives_an_existing_package_key_the_mode_it_ran_with() {
    let start = indoc! {r#"
        [env.a]
        # why editable
        use_develop = true  # beside
        package = "sdist"

        [env.b]
        use_develop = true

        [env.c]
        use_develop = false

        [env.d]
        use_develop = "true"
        "#};

    assert_snapshot!(formatted(start), @r#"
    [env.a]
    # why editable
    package = "editable"  # beside

    [env.b]
    package = "editable"

    [env.c]
    use_develop = false

    [env.d]
    use_develop = "true"
    "#);
}

/// A comment the file wrote beside `package` already says something about it, so the one the older
/// key carried takes a line of its own above it.
#[test]
fn test_a_package_key_that_already_carries_a_comment_keeps_it() {
    let start = indoc! {r#"
        [env.a]
        use_develop = true  # was develop
        package = "sdist"  # was sdist
        "#};

    assert_snapshot!(formatted(start), @r#"
    [env.a]
    # was develop
    package = "editable"  # was sdist
    "#);
}

/// pip reads `deps` the way it reads a requirements file, so a list holding an option, a path, a
/// URL or an artifact keeps the order the file wrote it in while every requirement beside them is
/// still normalized.
#[test]
fn test_a_deps_list_holding_more_than_requirements_keeps_its_order() {
    let start = indoc! {r#"
        [env.a]
        deps = ["Zed_Pkg>=1.0.0", "-r req.txt", "Alpha>=2.0.0"]

        [env.b]
        deps = ["./local", "../up", "/abs", "{tox_root}/x", "https://example.com/x", "pkg-1.0.whl"]

        [env.c]
        deps = ["Zed_Pkg>=1.0.0", "Alpha>=2.0.0"]
        "#};

    assert_snapshot!(formatted(start), @r#"
    [env.a]
    deps = [ "zed-pkg>=1", "-r req.txt", "alpha>=2" ]

    [env.b]
    deps = [ "./local", "../up", "/abs", "{tox_root}/x", "https://example.com/x", "pkg-1.0.whl" ]

    [env.c]
    deps = [ "alpha>=2", "zed-pkg>=1" ]
    "#);
}

/// `pass_env` names variables to pass through, and an entry that names none of them by itself
/// leads the ones that do.
#[test]
fn test_pass_env_leads_with_the_entries_that_name_no_variable() {
    let start = indoc! {r#"
        [env.a]
        pass_env = ["ZED", { replace = "ref", of = ["env_run_base", "pass_env"] }, "alpha"]
        constraints = ["c.txt", "a.txt"]
        extras = ["zed", "alpha"]
        "#};

    assert_snapshot!(formatted(start), @r#"
    [env.a]
    constraints = [ "c.txt", "a.txt" ]
    extras = [ "alpha", "zed" ]
    pass_env = [ { replace = "ref", of = [ "env_run_base", "pass_env" ] }, "alpha", "ZED" ]
    "#);
}

/// The same rules read a `pyproject.toml`, where every one of them is written under `[tool.tox]`.
#[test]
fn test_the_rules_read_the_table_a_pyproject_writes_them_under() {
    let start = indoc! {r#"
        [tool.tox]
        envlist = ["b", "a"]

        [tool.tox.env.a]
        setenv = { A = "1" }
        deps = ["Zed>=1.0.0"]
        "#};

    assert_snapshot!(under_tool_tox(start), @r#"
    [tool.tox]
    env_list = [ "a", "b" ]

    [tool.tox.env.a]
    deps = [ "zed>=1" ]
    set_env = { A = "1" }
    "#);
}

/// A pin names the environments the list leads with, in the order the pin gives them, and the
/// tables follow the same order.
#[test]
fn test_a_pin_leads_the_list_and_the_tables() {
    let start = indoc! {r#"
        env_list = ["3.13", "lint", "fix"]

        [env.lint]
        description = "lint"

        [env.fix]
        description = "fix"
        "#};
    let pins = ["fix".to_owned(), "lint".to_owned()];

    assert_snapshot!(with_pins(start, &pins), @r#"
    env_list = [ "fix", "lint", "3.13" ]
    [env.fix]
    description = "fix"

    [env.lint]
    description = "lint"
    "#);
}

/// `env_list` is written newest interpreter first, CPython before PyPy, and a name the version
/// grammar does not read after them by name.
#[test]
fn test_the_env_list_is_written_newest_interpreter_first() {
    let written = |held: &str| formatted(&format!("env_list = {held}\n"));

    assert_eq!(
        written(r#"[ "py3.11", "pypy39", "py312", "docs", "pypy3.10", "3.13t" ]"#),
        "env_list = [ \"py312\", \"py3.11\", \"pypy39\", \"pypy3.10\", \"3.13t\", \"docs\" ]\n"
    );
    // a name the grammar reads only as far as a dot, and one it does not read at all
    assert_eq!(
        written(r#"[ "py3.", "py3.x", "pypy", "py" ]"#),
        "env_list = [ \"py3.\", \"py\", \"py3.x\", \"pypy\" ]\n"
    );
    // a value that is not a name says nothing to sort by
    assert_eq!(written("[ 2, 1 ]"), "env_list = [ 2, 1 ]\n");
    assert_eq!(written("\"not-a-list\""), "env_list = \"not-a-list\"\n");
}

/// A reference names a key by the path it sits at, and a path that names anything else is left as
/// the file wrote it.
#[test]
fn test_a_reference_path_that_names_no_moved_key_is_left_alone() {
    let start = indoc! {r#"
        [env.a]
        setenv = { A = "1" }
        base_python = { replace = "ref", of = ["env", "a", 1] }
        commands = [{ replace = "ref", of = [] }]
        allowlist_externals = [{ replace = "ref", env = "x" }]
        "#};

    assert_snapshot!(formatted(start), @r#"
    [env.a]
    base_python = { replace = "ref", of = [ "env", "a", 1 ] }
    set_env = { A = "1" }
    commands = [ { replace = "ref", of = [] } ]
    allowlist_externals = [ { replace = "ref", env = "x" } ]
    "#);
}

/// A reusable base written as dotted keys names one table however deep the keys go, and a key
/// under something else names no environment at all.
#[test]
fn test_the_root_order_holds_a_slot_for_every_folded_environment() {
    let start = indoc! {r#"
        other.thing = 1
        env_base.shared.description = "shared"
        env_pkg_base.package = "wheel"
        env_run_base.description = "base"
        env_list = ["a"]
        "#};

    assert_snapshot!(formatted(start), @r#"
    env_list = [ "a" ]
    env_base.shared.description = "shared"
    env_pkg_base.package = "wheel"
    env_run_base.description = "base"
    other.thing = 1
    "#);
}

/// A reference names a key with the path it sits at, so `of` holding something other than a path
/// says nothing to follow.
#[test]
fn test_a_reference_that_names_no_path_is_left_alone() {
    let start = indoc! {r#"
        [env.a]
        setenv = { A = "1" }
        base_python = { replace = "ref", of = "env.a.setenv" }
        "#};

    assert_snapshot!(formatted(start), @r#"
    [env.a]
    base_python = { replace = "ref", of = "env.a.setenv" }
    set_env = { A = "1" }
    "#);
}

/// A key written under something that names no environment reaches no environment rule, wherever
/// the file wrote it.
#[test]
fn test_a_key_outside_every_environment_reaches_no_environment_rule() {
    let start = indoc! {r#"
        [other]
        use_develop = true
        deps = ["Zed>=1.0.0", "Alpha>=2.0.0"]
        "#};

    assert_snapshot!(formatted(start), @r#"
    [other]
    use_develop = true
    deps = [ "Zed>=1.0.0", "Alpha>=2.0.0" ]
    "#);
}

/// A requirement this parser cannot read is left as the file wrote it, and the list it sits in
/// keeps the order it names them in.
#[test]
fn test_a_dependency_that_does_not_read_is_left_as_written() {
    assert_snapshot!(formatted("[env.a]\ndeps = [ \"zed\", \"not a requirement!\" ]\n"), @r#"
    [env.a]
    deps = [ "zed", "not a requirement!" ]
    "#);
}

/// An environment written as one value names the table itself, which holds no key of its own to
/// read, and a table written under an environment is deeper than the environment it belongs to.
#[test]
fn test_an_environment_is_read_however_deep_the_file_wrote_its_keys() {
    let start = indoc! {r#"
        env.a = { deps = ["Zed>=1.0.0", "Alpha>=2.0.0"] }

        [env.b.set_env]
        Z = "1"
        A = "2"
        "#};

    assert_snapshot!(formatted(start), @r#"
    env.a = { deps = [ "alpha>=2", "zed>=1" ] }
    [env.b.set_env]
    Z = "1"
    A = "2"
    "#);
}

/// The older key carries no comment of its own here, so the key that replaces it says only what
/// the file already said beside it.
#[test]
fn test_a_use_develop_without_a_comment_leaves_the_package_comment_alone() {
    let start = indoc! {r#"
        [env.a]
        use_develop = true
        package = "sdist"  # was sdist
        "#};

    assert_snapshot!(formatted(start), @r#"
    [env.a]
    package = "editable"  # was sdist
    "#);
}

fn reorder_table_helper(start: &str) -> String {
    evaluate(start, |document| {
        reorder_tables(document);
        common::spacing::Spacing {
            between_groups: 1,
            within_group: None,
            nested_prefixes: &["env_base", "env"],
            ending: toml_doc::LineEnding::Lf,
        }
        .apply(document);
    })
}

fn formatted(start: &str) -> String {
    with_pins(start, &[])
}

fn with_pins(start: &str, pins: &[String]) -> String {
    evaluate(start, |document| {
        tox_rules::normalize_aliases(document);
        tox_rules::fix_root(document);
        tox_rules::fix_envs(document);
        tox_rules::sort_env_list(document, pins);
        tox_rules::reorder_inline_tables(document);
        tox_rules::reorder_tables_with_pins(document, pins);
    })
}

fn under_tool_tox(start: &str) -> String {
    evaluate(start, |document| {
        tox_rules::normalize_aliases_with_prefix(document, "tool.tox");
        tox_rules::fix_root_with_prefix(document, "tool.tox");
        tox_rules::fix_envs_with_prefix(document, "tool.tox");
        tox_rules::sort_env_list_with_prefix(document, &[], "tool.tox");
        tox_rules::reorder_inline_tables_with_prefix(document, "tool.tox");
    })
}

fn evaluate(start: &str, apply: impl FnOnce(&mut Document<'_>)) -> String {
    let mut document = toml_doc::parse(start).expect("the test input parses");
    apply(&mut document);
    common::layout::Layout {
        column_width: 120,
        indent: 2,
        ending: LineEnding::Lf,
    }
    .apply(&mut document);
    let written = document.to_string();
    assert!(
        toml_doc::parse(&written).is_ok(),
        "the rules wrote something that does not parse:\n{written}"
    );
    written
}
