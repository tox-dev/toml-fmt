use tombi_syntax::SyntaxNode;

pub use crate::test_util::format_toml_str;

pub mod array_tests;
pub mod create_tests;
pub mod pep508_tests;
pub mod string_tests;
pub mod table_tests;
pub mod util_tests;

pub fn format_toml(node: &SyntaxNode, column_width: usize) -> String {
    format_toml_str(&node.to_string(), column_width)
}

#[test]
#[should_panic(expected = "Invalid TOML output")]
fn test_assert_valid_toml_panics_on_invalid() {
    crate::test_util::assert_valid_toml("invalid = [");
}

#[test]
fn test_tombi_format_with_tables() {
    let input = indoc::indoc! {r#"
        requires = ["tox>=4.22"]
        env_list = ["3.13", "3.12"]
        skip_missing_interpreters = true

        [env_run_base]
        description = "run the tests with pytest under {env_name}"
        commands = [["pytest"]]

        [env.type]
        description = "run type check on code base"
        commands = [["mypy", "src{/}tox_toml_fmt"], ["mypy", "tests"]]
    "#};
    let result = format_toml_str(input, 80);
    assert!(
        result.contains("[env_run_base]"),
        "tables should be preserved, got:\n{result}"
    );
    crate::test_util::assert_valid_toml(&result);
}

#[test]
fn test_tombi_format_no_trailing_newline() {
    let input = "requires = [\"tox>=4.22\"]\nenv_list = [\"3.13\", \"3.12\"]\nskip_missing_interpreters = true\n[env_run_base]\ndescription = \"run the tests\"\ncommands = [[\"pytest\"]]\n";
    let result = format_toml_str(input, 80);
    eprintln!("INPUT:\n{input}");
    eprintln!("FORMATTED:\n{result}");
    assert!(
        result.contains("[env_run_base]"),
        "tables should be preserved, got:\n{result}"
    );
    crate::test_util::assert_valid_toml(&result);
}
