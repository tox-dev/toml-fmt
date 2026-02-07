use std::sync::OnceLock;

use common::format_options::create_format_options;
use tombi_config::TomlVersion;
use tombi_formatter::Formatter;
use tombi_schema_store::SchemaStore;
use tombi_syntax::{SyntaxElement, SyntaxNode};

mod build_systems_tests;
mod dependency_groups_tests;
mod global_tests;
mod main_tests;
mod project_tests;
mod ruff_tests;

static TOKIO_RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

fn get_runtime() -> &'static tokio::runtime::Runtime {
    TOKIO_RT.get_or_init(|| tokio::runtime::Runtime::new().expect("Failed to create tokio runtime"))
}

pub fn parse(source: &str) -> SyntaxNode {
    tombi_parser::parse(source, TomlVersion::default())
        .syntax_node()
        .clone_for_update()
}

pub fn format_syntax(node: SyntaxNode, column_width: usize) -> String {
    let source = node.to_string();
    format_toml_str(&source, column_width)
}

pub fn format_toml_str(source: &str, column_width: usize) -> String {
    let rt = get_runtime();
    let formatted = rt.block_on(async {
        let schema_store = SchemaStore::new();
        let options = create_format_options(column_width, 2);
        let formatter = Formatter::new(TomlVersion::default(), &options, None, &schema_store);
        formatter.format(source).await.unwrap_or_else(|_| source.to_string())
    });
    common::util::limit_blank_lines(&formatted, 2)
}

pub fn collect_entries(tables: &common::table::Tables) -> Vec<SyntaxElement> {
    tables.table_set.iter().flat_map(|e| e.borrow().clone()).collect()
}
