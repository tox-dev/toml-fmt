use std::path::{Path, PathBuf};

use _pyproject_fmt::{format_toml, Settings};

/// The files a case reads its input from, which are too long to write inline.
pub fn data_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("rust")
        .join("tests")
        .join("data")
}

/// The settings a case runs under where it says nothing else about them.
pub fn default_settings() -> Settings {
    Settings {
        column_width: 120,
        indent: 2,
        keep_full_version: false,
        max_supported_python: (3, 13),
        min_supported_python: (3, 9),
        generate_python_version_classifiers: false,
        table_format: String::from("short"),
        sub_table_spacing: String::new(),
        separate_root_table: String::from("\n"),
        expand_tables: vec![],
        collapse_tables: vec![],
        skip_wrap_for_keys: vec![],
    }
}

/// Run the whole formatter over the source, the way a user does.
pub fn evaluate_full(start: &str) -> String {
    evaluate_settings(start, &default_settings())
}

/// [`evaluate_full`] in the format that writes every sub-table under a header of its own.
pub fn evaluate_long(start: &str) -> String {
    evaluate_settings(
        start,
        &Settings {
            table_format: String::from("long"),
            ..default_settings()
        },
    )
}

/// [`evaluate_full`] under settings a case spells out.
pub fn evaluate_settings(start: &str, settings: &Settings) -> String {
    let written = format_toml(start, settings).expect("the formatter accepts it");
    assert_valid_toml(&written);
    written
}

/// The output has to be TOML a parser accepts, whatever the formatting rules did to it.
///
/// An independent parser reads it back as well as our own, so a document that is well formed but
/// says something no TOML file can say, like a table defined twice, is caught here.
pub fn assert_valid_toml(written: &str) {
    assert!(
        toml_doc::parse(written).is_ok(),
        "the formatter wrote something that does not parse:\n{written}"
    );
    assert!(
        written.parse::<toml::Table>().is_ok(),
        "the formatter wrote something no TOML document can say:\n{written}"
    );
}

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
mod disabled_tests;
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
mod pyproject_fmt_tests;
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
