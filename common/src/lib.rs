pub mod array;
pub mod create;
pub mod format_options;
pub mod pep508;
pub mod string;
pub mod table;
pub mod util;

pub use tombi_config as config;
pub use tombi_formatter as formatter;
pub use tombi_parser as parser;
pub use tombi_schema_store as schema_store;
pub use tombi_syntax as syntax;
pub use tombi_toml_text as toml_text;

#[cfg(test)]
mod tests;

#[cfg(any(test, feature = "test-util"))]
pub mod test_util {
    use std::sync::OnceLock;

    use tombi_config::TomlVersion;
    use tombi_formatter::Formatter;
    use tombi_schema_store::SchemaStore;
    use tombi_syntax::SyntaxNode;

    use crate::format_options::create_format_options;

    static TOKIO_RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

    pub fn get_runtime() -> &'static tokio::runtime::Runtime {
        TOKIO_RT.get_or_init(|| tokio::runtime::Runtime::new().expect("Failed to create tokio runtime"))
    }

    pub fn parse(source: &str) -> SyntaxNode {
        tombi_parser::parse(source, TomlVersion::default())
            .syntax_node()
            .clone_for_update()
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

    pub fn format_syntax(node: SyntaxNode, column_width: usize) -> String {
        format_toml_str(&node.to_string(), column_width)
    }

    pub fn assert_valid_toml(s: &str) {
        if let Err(e) = s.parse::<toml::Table>() {
            panic!("Invalid TOML output:\n{s}\n\nError: {e}");
        }
    }
}
