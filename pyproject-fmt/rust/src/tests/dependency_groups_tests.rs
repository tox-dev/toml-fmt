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
#[case::multiple_groups_and_extra_line(
  indoc ! {r#"
    [dependency-groups]
    example = [ "c<1" ]
    coverage = ["b<2"]
    type = [ "a>1" ]

    "#},
  indoc ! {r#"
    [dependency-groups]
    type = [
      "a>1",
    ]
    example = [
      "c<1",
    ]
    coverage = [
      "b<2",
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
#[case::duplicate_package_names(
        indoc ! {r#"
    [dependency-groups]
    test=["pkg>=2.0","pkg>=1.0","other"]
    "#},
        indoc ! {r#"
    [dependency-groups]
    test = [
      "other",
      "pkg>=1",
      "pkg>=2",
    ]
    "#},
        false,
)]
#[case::inline_table_without_include_group(
        indoc ! {r#"
    [dependency-groups]
    test=["pkg>=1.0",{some-key="value"}]
    "#},
        indoc ! {r#"
    [dependency-groups]
    test = [
      "pkg>=1",
      { some-key = "value" },
    ]
    "#},
        false,
)]
#[case::multiple_include_groups_sorted(
        indoc ! {r#"
    [dependency-groups]
    test = ['a>=1']
    docs = ['b>=1']
    dev = ['c>=1']
    all = [
      {include-group='test'},
      {include-group='docs'},
      {include-group='dev'},
    ]
    "#},
        indoc ! {r#"
    [dependency-groups]
    dev = [
      "c>=1",
    ]
    test = [
      "a>=1",
    ]
    docs = [
      "b>=1",
    ]
    all = [
      { include-group = "dev" },
      { include-group = "docs" },
      { include-group = "test" },
    ]
    "#},
        false,
)]
#[case::array_with_comments(
        indoc ! {r#"
    [dependency-groups]
    test = [
      "pytest>=7.0",
      # This is a comment
      "black>=23.0",
      { include-group = "docs" },
      # Another comment
    ]
    docs = ["sphinx>=5.0"]
    "#},
        indoc ! {r#"
    [dependency-groups]
    test = [
      # This is a comment
      "black>=23",
      "pytest>=7",
      { include-group = "docs" },
      # Another comment
    ]
    docs = [
      "sphinx>=5",
    ]
    "#},
        false,
)]
#[case::mixed_packages_and_include_groups(
        indoc ! {r#"
    [dependency-groups]
    base = ['requests>=2']
    extended = [
      {include-group='base'},
      'pytest>=7',
      {include-group='base'},
      'black>=23',
    ]
    "#},
        indoc ! {r#"
    [dependency-groups]
    base = [
      "requests>=2",
    ]
    extended = [
      "black>=23",
      "pytest>=7",
      { include-group = "base" },
      { include-group = "base" },
    ]
    "#},
        false,
)]
fn test_format_dependency_groups(#[case] start: &str, #[case] expected: &str, #[case] keep_full_version: bool) {
    assert_eq!(evaluate(start, keep_full_version), expected);
}

#[test]
fn test_dependency_groups_with_integer() {
    let start = indoc! {r#"
        [dependency-groups]
        test = [
          "pkg>=1.0",
          42,
        ]
        "#};
    let result = evaluate(start, false);
    assert!(result.contains("[dependency-groups]"));
}
