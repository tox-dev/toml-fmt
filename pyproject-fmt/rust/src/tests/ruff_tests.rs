use std::collections::HashSet;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use common::taplo::formatter::{format_syntax, Options};
use common::taplo::parser::parse;
use common::taplo::syntax::SyntaxElement;
use rstest::{fixture, rstest};

use crate::ruff::fix;
use crate::TableFormatConfig;
use common::table::Tables;

fn evaluate(start: &str) -> String {
    let root_ast = parse(start).into_syntax().clone_for_update();
    let count = root_ast.children_with_tokens().count();
    let mut tables = Tables::from_ast(&root_ast);
    let table_config = TableFormatConfig {
        default_collapse: true,
        expand_tables: HashSet::new(),
        collapse_tables: HashSet::new(),
    };
    fix(&mut tables, &table_config);
    let entries = tables
        .table_set
        .iter()
        .flat_map(|e| e.borrow().clone())
        .collect::<Vec<SyntaxElement>>();
    root_ast.splice_children(0..count, entries);
    let opt = Options {
        column_width: 1,
        ..Options::default()
    };
    format_syntax(root_ast, opt)
}
#[fixture]
fn data() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("rust")
        .join("src")
        .join("data")
}

#[rstest]
fn test_order_ruff(data: PathBuf) {
    let start = read_to_string(data.join("ruff-order.start.toml")).unwrap();
    let got = evaluate(start.as_str());
    let expected = read_to_string(data.join("ruff-order.expected.toml")).unwrap();
    assert_eq!(got, expected);
}

#[rstest]
fn test_ruff_comment_21(data: PathBuf) {
    let start = read_to_string(data.join("ruff-21.start.toml")).unwrap();
    let got = evaluate(start.as_str());
    let expected = read_to_string(data.join("ruff-21.expected.toml")).unwrap();
    assert_eq!(got, expected);
}
