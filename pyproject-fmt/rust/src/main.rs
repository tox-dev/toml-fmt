use std::collections::HashSet;
use std::string::String;

use pyo3::prelude::{PyModule, PyModuleMethods};
use pyo3::{pyclass, pyfunction, pymethods, pymodule, wrap_pyfunction, Bound, PyResult};
use tombi_config::TomlVersion;

use crate::global::reorder_tables;
use common::array::ensure_all_arrays_multiline;
use common::table::{apply_table_formatting, Tables};

mod build_system;
mod dependency_groups;
mod project;

mod global;
mod ruff;
#[cfg(test)]
mod tests;
mod uv;

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

/// Format toml file
#[must_use]
#[pyfunction]
pub fn format_toml(content: &str, opt: &Settings) -> String {
    let root_ast = parse(content);
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
    reorder_tables(&root_ast, &tables);
    ensure_all_arrays_multiline(&root_ast, opt.column_width);
    common::string::wrap_all_long_strings(&root_ast, opt.column_width, &indent_string);

    let modified_content = root_ast.to_string();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let formatted = rt.block_on(format_with_tombi(&modified_content, opt.column_width, opt.indent));

    let formatted_ast = parse(&formatted);
    common::array::align_array_comments(&formatted_ast);
    let formatted = formatted_ast.to_string();

    // Remove blank lines that tombi adds between tables in the same group
    // but only when table_format is "long" (compact formatting)
    let result = if opt.table_format == "long" {
        remove_blank_lines_between_same_group_tables(&formatted, &prefix_refs)
    } else {
        formatted
    };
    common::util::limit_blank_lines(&result, 2)
}

fn remove_blank_lines_between_same_group_tables(content: &str, multi_level_prefixes: &[&str]) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();

    for i in 0..lines.len() {
        // Check if this is a blank line between two table headers in the same group
        if lines[i].is_empty() && i > 0 && i + 1 < lines.len() {
            let trimmed_next = lines[i + 1].trim();
            let next_is_table = trimmed_next.starts_with('[');

            if next_is_table {
                // Look backwards to find the previous table header
                let mut prev_table_name = None;
                for j in (0..i).rev() {
                    if let Some(name) = extract_any_table_name(lines[j]) {
                        prev_table_name = Some(name);
                        break;
                    }
                }

                let next_table_name = extract_any_table_name(lines[i + 1]);

                if let (Some(prev), Some(next)) = (prev_table_name, next_table_name) {
                    let prev_key = get_table_key(&prev, multi_level_prefixes);
                    let next_key = get_table_key(&next, multi_level_prefixes);

                    if prev_key == next_key {
                        // Same group - skip this blank line
                        continue;
                    }
                }
            }
        }

        result.push(lines[i]);
    }

    let mut output = result.join("\n");
    // Preserve trailing newline if original content had one
    if content.ends_with('\n') && !output.ends_with('\n') {
        output.push('\n');
    }
    output
}

fn extract_any_table_name(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.starts_with("[[") {
        let end = trimmed.find("]]")?;
        Some(trimmed[2..end].to_string())
    } else if trimmed.starts_with('[') {
        let end = trimmed.find(']')?;
        Some(trimmed[1..end].to_string())
    } else {
        None
    }
}

fn get_table_key(name: &str, multi_level_prefixes: &[&str]) -> String {
    for prefix in multi_level_prefixes {
        if name == *prefix || name.starts_with(&format!("{}.", prefix)) {
            return prefix.to_string();
        }
    }
    name.split('.')
        .next()
        .expect("split returns at least one element")
        .to_string()
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
