use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use common::taplo::formatter::{format_syntax, Options};
use common::taplo::parser::parse;
use common::taplo::syntax::SyntaxElement;
use indoc::indoc;
use rstest::{fixture, rstest};

use crate::ruff::fix;
use common::table::Tables;

fn evaluate(start: &str) -> String {
    let root_ast = parse(start).into_syntax().clone_for_update();
    let count = root_ast.children_with_tokens().count();
    let mut tables = Tables::from_ast(&root_ast);
    fix(&mut tables);
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

    similar_asserts::assert_eq!(expected: expected, actual: got);
}

#[rstest]
fn test_ruff_comment_21(data: PathBuf) {
    let start = read_to_string(data.join("ruff-21.start.toml")).unwrap();
    let got = evaluate(start.as_str());
    let expected = read_to_string(data.join("ruff-21.expected.toml")).unwrap();
    similar_asserts::assert_eq!(expected: expected, actual: got);
}

#[rstest]
#[case::string(
    indoc! {r#"
        [tool.ruff]
        lint.flake8-copyright.notice-rgx = "Copyright author year"
    "#},
)]
#[case::string_literal(
    // https://github.com/tox-dev/toml-fmt/issues/22
    indoc! {r#"
        [tool.ruff]
        lint.flake8-copyright.notice-rgx = 'SPDX-License-Identifier: MPL-2\.0'
    "#},
)]
#[case::multi_line_string(
    indoc! {r#"
        [tool.ruff]
        lint.flake8-copyright.notice-rgx = """
          Copyright author year
          Some more terms
        """
    "#},
)]
#[case::multi_line_string_literal(
    indoc! {r#"
        [tool.ruff]
        lint.flake8-copyright.notice-rgx = '''
          Copyright author year\.
          Some more terms\.
        '''
    "#},
)]
fn test_flake8_copyright_notice_preserve_string_type(#[case] start: &str) {
    let got = evaluate(start);
    similar_asserts::assert_eq!(expected: start, actual: got);
}
