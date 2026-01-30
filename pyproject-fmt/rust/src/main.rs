use std::collections::HashSet;
use std::string::String;

use common::taplo::formatter::{format_syntax, Options};
use common::taplo::parser::parse;
use pyo3::prelude::{PyModule, PyModuleMethods};
use pyo3::{pyclass, pyfunction, pymethods, pymodule, wrap_pyfunction, Bound, PyResult};

use crate::global::reorder_tables;
use common::table::Tables;

mod build_system;
mod dependency_groups;
mod project;

mod global;
mod ruff;
#[cfg(test)]
mod tests;

#[pyclass(frozen, get_all)]
pub struct Settings {
    column_width: usize,
    indent: usize,
    keep_full_version: bool,
    max_supported_python: (u8, u8),
    min_supported_python: (u8, u8),
    generate_python_version_classifiers: bool,
    table_format: String,
    expand_tables: Vec<String>,
    collapse_tables: Vec<String>,
}

#[pymethods]
impl Settings {
    #[new]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (*, column_width, indent, keep_full_version, max_supported_python, min_supported_python, generate_python_version_classifiers, table_format, expand_tables, collapse_tables))]
    fn new(
        column_width: usize,
        indent: usize,
        keep_full_version: bool,
        max_supported_python: (u8, u8),
        min_supported_python: (u8, u8),
        generate_python_version_classifiers: bool,
        table_format: String,
        expand_tables: Vec<String>,
        collapse_tables: Vec<String>,
    ) -> Self {
        Self {
            column_width,
            indent,
            keep_full_version,
            max_supported_python,
            min_supported_python,
            generate_python_version_classifiers,
            table_format,
            expand_tables,
            collapse_tables,
        }
    }
}

/// Configuration for table formatting behavior.
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

    /// Determine if a table should be collapsed based on configuration.
    /// Priority: collapse_tables > expand_tables > default_collapse
    pub fn should_collapse(&self, table_name: &str) -> bool {
        if self.collapse_tables.contains(table_name) {
            return true;
        }
        if self.expand_tables.contains(table_name) {
            return false;
        }
        self.default_collapse
    }
}

/// Format toml file
#[must_use]
#[pyfunction]
pub fn format_toml(content: &str, opt: &Settings) -> String {
    let root_ast = parse(content).into_syntax().clone_for_update();
    let mut tables = Tables::from_ast(&root_ast);
    let table_config = TableFormatConfig::from_settings(opt);

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
    ruff::fix(&mut tables, &table_config);
    reorder_tables(&root_ast, &tables);

    let options = Options {
        align_entries: false,         // do not align by =
        align_comments: true,         // align inline comments
        align_single_comments: true,  // align comments after entries
        array_trailing_comma: true,   // ensure arrays finish with trailing comma
        array_auto_expand: true,      // arrays go to multi line when too long
        array_auto_collapse: false,   // do not collapse for easier diffs
        compact_arrays: false,        // leave whitespace
        compact_inline_tables: false, // leave whitespace
        compact_entries: false,       // leave whitespace
        column_width: opt.column_width,
        indent_tables: false,
        indent_entries: false,
        inline_table_expand: true,
        trailing_newline: true,
        allowed_blank_lines: 1, // one blank line to separate
        indent_string: " ".repeat(opt.indent),
        reorder_keys: false,   // respect custom order
        reorder_arrays: false, // for natural sorting we need to this ourselves
        crlf: false,
        reorder_inline_tables: false,
    };
    format_syntax(root_ast, options)
}

/// # Errors
///
/// Will return `PyErr` if an error is raised during formatting.
#[pymodule]
#[pyo3(name = "_lib")]
pub fn _lib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(format_toml, m)?)?;
    m.add_class::<Settings>()?;
    Ok(())
}
