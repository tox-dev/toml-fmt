use std::collections::HashSet;
use std::string::String;

use pyo3::prelude::{PyModule, PyModuleMethods};
use pyo3::{pyclass, pyfunction, pymethods, pymodule, wrap_pyfunction, Bound, PyResult};
use tombi_config::TomlVersion;

use crate::global::{normalize_strings, reorder_tables};
use common::array::ensure_all_arrays_multiline;
use common::table::{apply_table_formatting, Tables};

mod global;
#[cfg(test)]
mod tests;

#[pyclass(frozen, get_all)]
pub struct Settings {
    column_width: usize,
    indent: usize,
    table_format: String,
    expand_tables: Vec<String>,
    collapse_tables: Vec<String>,
    skip_wrap_for_keys: Vec<String>,
}

#[pymethods]
impl Settings {
    #[new]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (*, column_width, indent, table_format, expand_tables, collapse_tables, skip_wrap_for_keys))]
    fn new(
        column_width: usize,
        indent: usize,
        table_format: String,
        expand_tables: Vec<String>,
        collapse_tables: Vec<String>,
        skip_wrap_for_keys: Vec<String>,
    ) -> Self {
        Self {
            column_width,
            indent,
            table_format,
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
    let mut tables = Tables::from_ast(&root_ast);
    let table_config = TableFormatConfig::from_settings(opt);

    let mut prefixes: Vec<String> = vec![String::from("env_run_base")];
    for key in tables.header_to_pos.keys() {
        if let Some(env_name) = key.strip_prefix("env.") {
            let env_prefix = format!("env.{}", env_name.split('.').next().unwrap_or(env_name));
            if !prefixes.contains(&env_prefix) {
                prefixes.push(env_prefix);
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

    normalize_strings(&tables);
    reorder_tables(&root_ast, &tables);
    ensure_all_arrays_multiline(&root_ast, opt.column_width);

    let indent_string = " ".repeat(opt.indent);
    common::string::wrap_all_long_strings(&root_ast, opt.column_width, &indent_string, &opt.skip_wrap_for_keys);

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
