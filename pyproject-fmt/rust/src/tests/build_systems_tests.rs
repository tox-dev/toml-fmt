use common::taplo::formatter::{format_syntax, Options};
use common::taplo::parser::parse;
use common::taplo::syntax::SyntaxElement;
use indoc::indoc;
use rstest::rstest;

use crate::build_system::fix;
use common::table::Tables;

fn evaluate(start: &str, keep_full_version: bool) -> String {
    let root_ast = parse(start).into_syntax().clone_for_update();
    let count = root_ast.children_with_tokens().count();
    let tables = Tables::from_ast(&root_ast);
    fix(&tables, keep_full_version);
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

#[rstest]
#[case::no_build_system(
        indoc ! {r""},
        "\n",
        false
)]
#[case::build_system_requires_no_keep(
        indoc ! {r#"
    [build-system]
    requires=["a>=1.0.0", "b.c>=1.5.0"]
    "#},
        indoc ! {r#"
    [build-system]
    requires = [
      "a>=1",
      "b-c>=1.5",
    ]
    "#},
        false
)]
#[case::build_system_requires_keep(
        indoc ! {r#"
    [build-system]
    requires=["a>=1.0.0", "b.c>=1.5.0"]
    "#},
        indoc ! {r#"
    [build-system]
    requires = [
      "a>=1.0.0",
      "b-c>=1.5.0",
    ]
    "#},
        true
)]
#[case::join(
        indoc ! {r#"
    [build-system]
    requires=["a"]
    [build-system]
    build-backend = "hatchling.build"
    [[build-system.a]]
    name = "Hammer"
    [[build-system.a]]  # empty table within the array
    [[build-system.a]]
    name = "Nail"
    "#},
        indoc ! {r#"
    [build-system]
    build-backend = "hatchling.build"
    requires = [
      "a",
    ]
    [[build-system.a]]
    name = "Hammer"
    [[build-system.a]] # empty table within the array
    [[build-system.a]]
    name = "Nail"
    "#},
        false
)]
fn test_format_build_systems(#[case] start: &str, #[case] expected: &str, #[case] keep_full_version: bool) {
    assert_eq!(evaluate(start, keep_full_version), expected);
}
