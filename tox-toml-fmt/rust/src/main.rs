use std::string::String;

use common::taplo::formatter::{format_syntax, Options};
use common::taplo::parser::parse;
use pyo3::prelude::{PyModule, PyModuleMethods};
use pyo3::{pyclass, pyfunction, pymethods, pymodule, wrap_pyfunction, Bound, PyResult};

use crate::global::reorder_tables;
use common::table::Tables;
mod global;
#[cfg(test)]
mod tests;

#[pyclass(frozen, get_all)]
pub struct Settings {
    column_width: usize,
    indent: usize,
}

#[pymethods]
impl Settings {
    #[new]
    #[pyo3(signature = (*, column_width, indent ))]
    const fn new(column_width: usize, indent: usize) -> Self {
        Self { column_width, indent }
    }
}

/// Format toml file
#[must_use]
#[pyfunction]
pub fn format_toml(content: &str, opt: &Settings) -> String {
    let root_ast = parse(content).into_syntax().clone_for_update();
    let tables = Tables::from_ast(&root_ast);

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
