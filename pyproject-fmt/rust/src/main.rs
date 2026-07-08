use std::collections::HashSet;
use std::string::String;

use pyo3::prelude::{PyModule, PyModuleMethods};
use pyo3::{pyclass, pyfunction, pymethods, pymodule, wrap_pyfunction, Bound, PyResult, Python};

use crate::global::reorder_tables;
use common::array::ensure_all_arrays_multiline;
use common::table::{apply_table_formatting, Tables};
use tombi_config::TomlVersion;

mod build_system;
mod dependency_groups;
mod project;

mod autopep8;
mod bandit;
mod black;
mod bumpversion;
mod check_manifest;
mod cibuildwheel;
mod codespell;
mod commitizen;
mod coverage;
mod deptry;
mod djlint;
mod docformatter;
mod global;
mod hatch;
mod interrogate;
mod isort;
mod maturin;
mod mypy;
mod pdm;
mod pixi;
mod poetry;
mod pylint;
mod pyrefly;
mod pyright;
mod pytest;
mod ruff;
mod scikit_build;
mod semantic_release;
mod setuptools;
#[cfg(test)]
mod tests;
mod towncrier;
mod tox;
mod ty;
mod uv;
mod vulture;
mod yapf;

#[pyclass(frozen, get_all)]
pub struct Settings {
    column_width: usize,
    indent: usize,
    keep_full_version: bool,
    max_supported_python: (u8, u8),
    min_supported_python: (u8, u8),
    generate_python_version_classifiers: bool,
    table_format: String,
    sub_table_spacing: String,
    separate_root_table: String,
    expand_tables: Vec<String>,
    collapse_tables: Vec<String>,
    skip_wrap_for_keys: Vec<String>,
}

#[pymethods]
impl Settings {
    #[new]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (*, column_width, indent, keep_full_version, max_supported_python, min_supported_python, generate_python_version_classifiers, table_format, sub_table_spacing, separate_root_table, expand_tables, collapse_tables, skip_wrap_for_keys))]
    fn new(
        column_width: usize,
        indent: usize,
        keep_full_version: bool,
        max_supported_python: (u8, u8),
        min_supported_python: (u8, u8),
        generate_python_version_classifiers: bool,
        table_format: String,
        sub_table_spacing: String,
        separate_root_table: String,
        expand_tables: Vec<String>,
        collapse_tables: Vec<String>,
        skip_wrap_for_keys: Vec<String>,
    ) -> Self {
        Self {
            column_width,
            indent,
            keep_full_version,
            max_supported_python,
            min_supported_python,
            generate_python_version_classifiers,
            table_format,
            sub_table_spacing,
            separate_root_table,
            expand_tables,
            collapse_tables,
            skip_wrap_for_keys,
        }
    }
}

pub struct TableFormatConfig {
    pub default_collapse: bool,
    pub expand_tables: HashSet<String>,
    pub collapse_tables: HashSet<String>,
}

impl TableFormatConfig {
    pub fn from_settings(settings: &Settings) -> Self {
        Self {
            default_collapse: settings.table_format == "short",
            expand_tables: settings.expand_tables.iter().cloned().collect(),
            collapse_tables: settings.collapse_tables.iter().cloned().collect(),
        }
    }

    pub fn should_collapse(&self, table_name: &str) -> bool {
        let mut current = table_name;
        loop {
            if self.collapse_tables.contains(current) {
                return true;
            }
            if self.expand_tables.contains(current) {
                return false;
            }
            match current.rfind('.') {
                Some(dot_pos) => current = &current[..dot_pos],
                None => break,
            }
        }
        self.default_collapse
    }
}

fn parse(source: &str) -> tombi_syntax::SyntaxNode {
    tombi_parser::parse(source).syntax_node().clone_for_update()
}

async fn format_with_tombi(content: &str, column_width: usize, indent: usize) -> String {
    let options = common::format_options::create_format_options(column_width, indent);
    let schema_store = tombi_schema_store::SchemaStore::new();
    let formatter = tombi_formatter::Formatter::new(TomlVersion::default(), &options, None, &schema_store);
    formatter.format(content).await.unwrap_or_else(|_| content.to_string())
}

#[pyfunction]
#[pyo3(name = "format_toml")]
fn format_toml_py(py: Python<'_>, content: &str, opt: &Settings) -> String {
    py.detach(|| format_toml(content, opt))
}

#[must_use]
pub fn format_toml(content: &str, opt: &Settings) -> String {
    common::disabled::with_disabled_keys(content, |content| format_core(content, opt))
}

fn format_core(content: &str, opt: &Settings) -> String {
    let root_ast = parse(content);
    common::string::normalize_key_quotes(&root_ast);
    let mut tables = Tables::from_ast(&root_ast);
    let table_config = TableFormatConfig::from_settings(opt);

    let mut prefixes: Vec<String> = vec![String::from("build-system"), String::from("project")];
    for key in tables.header_to_pos.keys() {
        if let Some(tool_name) = key.strip_prefix("tool.") {
            let tool_prefix = format!("tool.{}", tool_name.split('.').next().unwrap_or(tool_name));
            if !prefixes.contains(&tool_prefix) {
                prefixes.push(tool_prefix);
            }
        }
    }
    let prefix_refs: Vec<&str> = prefixes.iter().map(|s| s.as_str()).collect();
    apply_table_formatting(
        &mut tables,
        |name| table_config.should_collapse(name),
        &prefix_refs,
        opt.column_width,
    );

    let indent_string = " ".repeat(opt.indent);
    build_system::fix(&tables, opt.keep_full_version);
    project::fix(
        &mut tables,
        opt.keep_full_version,
        opt.max_supported_python,
        opt.min_supported_python,
        opt.generate_python_version_classifiers,
        &table_config,
    );
    dependency_groups::fix(&mut tables, opt.keep_full_version);
    ruff::fix(&mut tables);
    uv::fix(&mut tables);
    pixi::fix(&mut tables);
    commitizen::fix(&mut tables);
    poetry::fix(&mut tables);
    mypy::fix(&mut tables);
    setuptools::fix(&mut tables);
    pytest::fix(&mut tables);
    black::fix(&mut tables);
    hatch::fix(&mut tables);
    isort::fix(&mut tables);
    pyright::fix(&mut tables);
    pdm::fix(&mut tables);
    cibuildwheel::fix(&mut tables);
    tox::fix(&mut tables);
    bandit::fix(&mut tables);
    maturin::fix(&mut tables);
    codespell::fix(&mut tables);
    towncrier::fix(&mut tables);
    pylint::fix(&mut tables);
    djlint::fix(&mut tables);
    yapf::fix(&mut tables);
    check_manifest::fix(&mut tables);
    pyrefly::fix(&mut tables);
    semantic_release::fix(&mut tables);
    scikit_build::fix(&mut tables);
    bumpversion::fix(&mut tables);
    interrogate::fix(&mut tables);
    docformatter::fix(&mut tables);
    vulture::fix(&mut tables);
    autopep8::fix(&mut tables);
    deptry::fix(&mut tables);
    ty::fix(&mut tables);
    coverage::fix(&mut tables);
    reorder_tables(&root_ast, &tables, &opt.separate_root_table, &opt.sub_table_spacing);
    // Must follow reorder_tables: only then have AoT entries collapsed to inline arrays of inline tables
    // (e.g. [[tool.poetry.source]] → source = [{...}]) and become INLINE_TABLE descendants of root_ast.
    poetry::reorder_inline_tables(&root_ast);
    mypy::reorder_inline_tables(&root_ast);
    setuptools::reorder_inline_tables(&root_ast);
    tox::reorder_inline_tables(&root_ast);
    ensure_all_arrays_multiline(&root_ast, opt.column_width);
    common::string::wrap_all_long_strings(&root_ast, opt.column_width, &indent_string, &opt.skip_wrap_for_keys);

    let modified_content = root_ast.to_string();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let formatted = rt.block_on(format_with_tombi(&modified_content, opt.column_width, opt.indent));

    let formatted_ast = parse(&formatted);
    common::array::align_array_comments(&formatted_ast);
    let formatted = formatted_ast.to_string();

    let sub_spacing = (opt.table_format == "long").then_some(opt.sub_table_spacing.as_str());
    let result = common::table::normalize_table_spacing(&formatted, &["tool"], &opt.separate_root_table, sub_spacing);
    common::util::limit_blank_lines(&result, 2)
}

/// # Errors
///
/// Will return `PyErr` if an error is raised during formatting.
#[pymodule(gil_used = false)]
#[pyo3(name = "_lib")]
pub fn _lib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(format_toml_py, m)?)?;
    m.add_class::<Settings>()?;
    Ok(())
}
