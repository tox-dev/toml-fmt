use std::string::String;
use std::vec;

use common::taplo::dom::node::DomNode;
use common::taplo::formatter::{format_syntax, Options};
use common::taplo::parser::parse;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::{PyModule, PyModuleMethods};
use pyo3::types::PyTuple;
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
    do_not_collapse: Vec<Vec<String>>,
}

#[pymethods]
impl Settings {
    #[new]
    #[pyo3(signature = (*, column_width, indent, keep_full_version, max_supported_python, min_supported_python, generate_python_version_classifiers, do_not_collapse))]
    const fn new(
        column_width: usize,
        indent: usize,
        keep_full_version: bool,
        max_supported_python: (u8, u8),
        min_supported_python: (u8, u8),
        generate_python_version_classifiers: bool,
        do_not_collapse: Vec<Vec<String>>,
    ) -> Self {
        Self {
            column_width,
            indent,
            keep_full_version,
            max_supported_python,
            min_supported_python,
            generate_python_version_classifiers,
            do_not_collapse,
        }
    }
}

/// Format toml file
#[must_use]
#[pyfunction]
pub fn format_toml(content: &str, opt: &Settings) -> String {
    let root_ast = parse(content).into_syntax().clone_for_update();
    let mut tables = Tables::from_ast(&root_ast);

    build_system::fix(&tables, opt.keep_full_version);
    project::fix(
        &mut tables,
        opt.keep_full_version,
        opt.max_supported_python,
        opt.min_supported_python,
        opt.generate_python_version_classifiers,
        opt.do_not_collapse.as_slice(),
    );
    dependency_groups::fix(&mut tables, opt.keep_full_version, opt.do_not_collapse.as_slice());
    ruff::fix(&mut tables, opt.do_not_collapse.as_slice());
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

#[pyfunction]
pub fn parse_ident<'py>(py: pyo3::Python<'py>, ident: String) -> PyResult<Bound<'py, PyTuple>> {
    let parsed = parse(&format!("{ident} = 1"));
    if let Some(e) = parsed.errors.first() {
        return Err(PyValueError::new_err(format!("syntax error: {e}")));
    }

    let root = parsed.into_dom();
    let errors = root.errors();
    if let Some(e) = errors.get().first() {
        return Err(PyValueError::new_err(format!("semantic error: {e}")));
    }

    dbg!(&root.errors());

    // We cannot use `.into_syntax()` since only the DOM transformation
    // allows accessing ident `.value()`s without quotes.
    let mut node = root;
    let mut parts = vec![];
    while let Ok(table) = node.try_into_table() {
        let entries = table.entries().get();
        if entries.len() != 1 {
            return Err(PyValueError::new_err("expected exactly one entry"));
        }
        let mut it = entries.iter();
        let (key, next_node) = it.next().unwrap(); // checked if len == 1 above

        parts.push(key.value().to_string());
        node = next_node.clone();
    }
    PyTuple::new(py, parts)
}

/// # Errors
///
/// Will return `PyErr` if an error is raised during formatting.
#[pymodule]
#[pyo3(name = "_lib")]
pub fn _lib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(format_toml, m)?)?;
    m.add_function(wrap_pyfunction!(parse_ident, m)?)?;
    m.add_class::<Settings>()?;
    Ok(())
}
