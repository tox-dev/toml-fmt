use std::string::String;

use pyo3::prelude::{PyModule, PyModuleMethods};
use pyo3::{pyclass, pyfunction, pymethods, pymodule, wrap_pyfunction, Bound, PyResult};
use tombi_config::TomlVersion;

use crate::global::{normalize_strings, reorder_tables};
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

fn parse(source: &str) -> tombi_syntax::SyntaxNode {
    tombi_parser::parse(source, TomlVersion::default())
        .syntax_node()
        .clone_for_update()
}

async fn format_with_tombi(content: &str, column_width: usize, indent: usize) -> String {
    let options = common::format_options::create_format_options(column_width, indent);
    let schema_store = tombi_schema_store::SchemaStore::new();
    let formatter = tombi_formatter::Formatter::new(TomlVersion::default(), &options, None, &schema_store);
    formatter.format(content).await.unwrap_or_else(|_| content.to_string())
}

#[must_use]
#[pyfunction]
pub fn format_toml(content: &str, opt: &Settings) -> String {
    let root_ast = parse(content);
    common::string::normalize_key_quotes(&root_ast);
    let tables = Tables::from_ast(&root_ast);

    normalize_strings(&tables);
    reorder_tables(&root_ast, &tables);

    let modified_content = root_ast.to_string();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let formatted = rt.block_on(format_with_tombi(&modified_content, opt.column_width, opt.indent));

    let formatted_ast = parse(&formatted);
    common::array::align_array_comments(&formatted_ast);
    let aligned = formatted_ast.to_string();

    common::util::limit_blank_lines(&aligned, 2)
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
