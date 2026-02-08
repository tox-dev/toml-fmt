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
