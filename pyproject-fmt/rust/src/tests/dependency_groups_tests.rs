use common::taplo::formatter::{format_syntax, Options};
use common::taplo::parser::parse;
use common::taplo::syntax::SyntaxElement;
use indoc::indoc;
use rstest::rstest;

use crate::dependency_groups::fix;
use common::table::Tables;

fn evaluate(start: &str, keep_full_version: bool) -> String {
    let root_ast = parse(start).into_syntax().clone_for_update();
    let count = root_ast.children_with_tokens().count();
    let mut tables = Tables::from_ast(&root_ast);
    fix(&mut tables, keep_full_version);
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
#[case::no_groups(
        indoc ! {r""},
        "\n",
        false,
)]
#[case::single_group_single_dep(
        indoc ! {r#"
    [dependency-groups]
    test=["a>1.0.0"]
    "#},
        indoc ! {r#"
    [dependency-groups]
    test = [
      "a>1",
    ]
    "#},
        false,
)]
#[case::single_group_single_dep_full_version(
        indoc ! {r#"
    [dependency-groups]
    test=["a>1.0.0"]
    "#},
        indoc ! {r#"
    [dependency-groups]
    test = [
      "a>1.0.0",
    ]
    "#},
        true,
)]
#[case::single_group_multiple_deps(
        indoc ! {r#"
    [dependency-groups]
    test=["b==2.0.*", "a>1"]
    "#},
        indoc ! {r#"
    [dependency-groups]
    test = [
      "a>1",
      "b==2.0.*",
    ]
    "#},
        false,
)]
#[case::multiple_groups(
        indoc ! {r#"
    [dependency-groups]
    example=["c<1"]
    docs=["b==1"]
    test=["a>1"]
    dev=["d>=2"]
    "#},
        indoc ! {r#"
    [dependency-groups]
    dev = [
      "d>=2",
    ]
    test = [
      "a>1",
    ]
    docs = [
      "b==1",
    ]
    example = [
      "c<1",
    ]
    "#},
        false,
)]
#[case::include_single_group(
        indoc ! {r#"
    [dependency-groups]
    docs=["b==1"]
    test=["a>1",{include-group="docs"}]
    "#},
        indoc ! {r#"
    [dependency-groups]
    test = [
      "a>1",
      { include-group = "docs" },
    ]
    docs = [
      "b==1",
    ]
    "#},
        false,
)]
#[case::include_many_groups(
        indoc ! {r#"
    [dependency-groups]
    all=['c<1', {include-group='test'}, {include-group='docs'}, 'd>1']
    docs = ['b==1']
    test = ['a>1']
    "#},
        indoc ! {r#"
    [dependency-groups]
    test = [
      "a>1",
    ]
    docs = [
      "b==1",
    ]
    all = [
      "c<1",
      "d>1",
      { include-group = "docs" },
      { include-group = "test" },
    ]
    "#},
        false,
)]
fn test_format_dependency_groups(#[case] start: &str, #[case] expected: &str, #[case] keep_full_version: bool) {
    assert_eq!(evaluate(start, keep_full_version), expected);
}
