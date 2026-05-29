use tombi_syntax::SyntaxElement;

pub use common::test_util::{assert_valid_toml, format_syntax, format_toml_str, parse};

mod black_tests;
mod bandit_tests;
mod build_systems_tests;
mod commitizen_tests;
mod cibuildwheel_tests;
mod coverage_tests;
mod dependency_groups_tests;
mod global_tests;
mod hatch_tests;
mod isort_tests;
mod main_tests;
mod mypy_tests;
mod pdm_tests;
mod maturin_tests;
mod pixi_tests;
mod poetry_tests;
mod project_tests;
mod pytest_tests;
mod pyright_tests;
mod ruff_tests;
mod setuptools_tests;
mod tox_tests;
mod uv_tests;

pub fn collect_entries(tables: &common::table::Tables) -> Vec<SyntaxElement> {
    tables.table_set.iter().flat_map(|e| e.borrow().clone()).collect()
}
