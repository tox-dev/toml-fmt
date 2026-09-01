use super::{evaluate_full as evaluate, evaluate_long};

#[test]
fn test_bandit_top_level_order() {
    let start = indoc::indoc! {r#"
    [tool.bandit]
    skips = ["B101"]
    tests = ["B201"]
    targets = ["src"]
    exclude_dirs = ["tests"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.bandit]
    exclude_dirs = [ "tests" ]
    targets = [ "src" ]
    tests = [ "B201" ]
    skips = [ "B101" ]
    "#);
}

#[test]
fn test_bandit_arrays_sorted() {
    let start = indoc::indoc! {r#"
    [tool.bandit]
    skips = ["B311", "B101", "B201"]
    tests = ["B999", "B101"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.bandit]
    tests = [ "B101", "B999" ]
    skips = [ "B101", "B201", "B311" ]
    "#);
}

#[test]
fn test_bandit_assert_used_inner() {
    let start = indoc::indoc! {r#"
    [tool.bandit.assert_used]
    skips = ["*_test.py", "test_*.py"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.bandit]
    assert_used.skips = [ "*_test.py", "test_*.py" ]
    "#);
}

#[test]
fn test_bandit_idempotent() {
    let start = indoc::indoc! {r#"
    [tool.bandit]
    exclude_dirs = [ "tests" ]
    skips = [ "B101", "B201" ]
    "#};
    let once = evaluate(start);
    let twice = evaluate(&once);
    assert_eq!(once, twice);
}

#[test]
fn test_bandit_no_table_noop() {
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
fn test_bandit_inner_tmp_dirs_sorted() {
    let start = indoc::indoc! {r#"
    [tool.bandit.hardcoded_tmp_directory]
    tmp_dirs = ["/var/tmp", "/tmp", "/dev/shm"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.bandit]
    hardcoded_tmp_directory.tmp_dirs = [ "/dev/shm", "/tmp", "/var/tmp" ]
    "#);
}

#[test]
fn test_bandit_inner_no_shell_sorted() {
    let start = indoc::indoc! {r#"
    [tool.bandit.any_other_function_with_shell_equals_true]
    no_shell = ["os.system", "os.popen"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.bandit]
    any_other_function_with_shell_equals_true.no_shell = [ "os.popen", "os.system" ]
    "#);
}

#[test]
fn test_bandit_inner_shell_sorted() {
    let start = indoc::indoc! {r#"
    [tool.bandit.shell_injection]
    shell = ["zsh", "bash", "sh"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.bandit]
    shell_injection.shell = [ "bash", "sh", "zsh" ]
    "#);
}

#[test]
fn test_bandit_inner_subprocess_sorted() {
    let start = indoc::indoc! {r#"
    [tool.bandit.shell_injection]
    subprocess = ["subprocess.run", "subprocess.Popen"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.bandit]
    shell_injection.subprocess = [ "subprocess.Popen", "subprocess.run" ]
    "#);
}

#[test]
fn test_bandit_inner_tests_sorted() {
    let start = indoc::indoc! {r#"
    [tool.bandit.some_plugin]
    tests = ["B999", "B101"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.bandit]
    some_plugin.tests = [ "B101", "B999" ]
    "#);
}

#[test]
fn test_bandit_inner_non_matching_preserved() {
    let start = indoc::indoc! {r#"
    [tool.bandit.assert_used]
    word_list = ["zeta", "alpha"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.bandit]
    assert_used.word_list = [ "zeta", "alpha" ]
    "#);
}

#[test]
fn test_bandit_long_format() {
    let start = indoc::indoc! {r#"
    [tool.bandit]
    skips = ["B311", "B101"]
    "#};
    let result = evaluate_long(start);
    assert!(result.contains("[tool.bandit]"));
    assert!(result.find("B101").unwrap() < result.find("B311").unwrap());
}
