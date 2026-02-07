use std::sync::OnceLock;

use tombi_config::TomlVersion;
use tombi_formatter::Formatter;
use tombi_schema_store::SchemaStore;
use tombi_syntax::SyntaxNode;

use crate::format_options::create_format_options;

pub mod array_tests;
pub mod create_tests;
pub mod pep508_tests;
pub mod string_tests;
pub mod table_tests;
pub mod util_tests;

static TOKIO_RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

fn get_runtime() -> &'static tokio::runtime::Runtime {
    TOKIO_RT.get_or_init(|| tokio::runtime::Runtime::new().expect("Failed to create tokio runtime"))
}

pub fn format_toml(node: &SyntaxNode, column_width: usize) -> String {
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
    crate::util::limit_blank_lines(&formatted, 2)
}
