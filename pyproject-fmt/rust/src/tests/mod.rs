use tombi_syntax::SyntaxElement;

pub use common::test_util::{assert_valid_toml, format_syntax, format_toml_str, parse};

mod build_systems_tests;
mod dependency_groups_tests;
mod global_tests;
mod main_tests;
mod project_tests;
mod ruff_tests;
mod uv_tests;

pub fn collect_entries(tables: &common::table::Tables) -> Vec<SyntaxElement> {
    tables.table_set.iter().flat_map(|e| e.borrow().clone()).collect()
}
