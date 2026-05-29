use tombi_syntax::SyntaxElement;

pub use common::test_util::{assert_valid_toml, format_syntax, format_toml_str, parse};

mod autopep8_tests;
mod bandit_tests;
mod black_tests;
mod build_systems_tests;
mod bumpversion_tests;
mod check_manifest_tests;
mod cibuildwheel_tests;
mod codespell_tests;
mod commitizen_tests;
mod coverage_tests;
mod dependency_groups_tests;
mod deptry_tests;
mod djlint_tests;
mod docformatter_tests;
mod global_tests;
mod hatch_tests;
mod interrogate_tests;
mod isort_tests;
mod main_tests;
mod maturin_tests;
mod mypy_tests;
mod pdm_tests;
mod pixi_tests;
mod poetry_tests;
mod project_tests;
mod pylint_tests;
mod pyrefly_tests;
mod pyright_tests;
mod pytest_tests;
mod ruff_tests;
mod scikit_build_tests;
mod semantic_release_tests;
mod setuptools_tests;
mod towncrier_tests;
mod tox_tests;
mod ty_tests;
mod uv_tests;
mod vulture_tests;
mod yapf_tests;

pub fn collect_entries(tables: &common::table::Tables) -> Vec<SyntaxElement> {
    tables.table_set.iter().flat_map(|e| e.borrow().clone()).collect()
}
