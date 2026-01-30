use common::taplo::formatter::{format_syntax, Options};
use common::taplo::parser::parse;
use indoc::indoc;
use rstest::rstest;

use crate::global::reorder_tables;
use common::table::Tables;

#[rstest]
#[case::reorder(
        indoc ! {r#"
    # comment
    a= "b"
    [project]
    name="alpha"
    dependencies=["e"]
    [build-system]
    build-backend="backend"
    requires=["c", "d"]
    [dependency-groups]
    docs=["s"]
    test=["p", "q"]
    [tool.mypy]
    mk="mv"
    [tool.ruff.test]
    mrt="vrt"
    [extra]
    ek = "ev"
    [tool.undefined]
    mu="mu"
    [tool.ruff]
    mr="vr"
    [demo]
    ed = "ed"
    [tool.coverage.report]
    cd="de"
    [tool.coverage]
    aa = "bb"
    [tool.coverage.paths]
    ab="bc"
    [tool.coverage.run]
    ef="fg"
    [tool.pytest]
    mk="mv"
    [tool.uv]
    vu="uv"
    "#},
        indoc ! {r#"
    # comment
    a = "b"

    [build-system]
    build-backend = "backend"
    requires = [
      "c",
      "d",
    ]

    [project]
    name = "alpha"
    dependencies = [
      "e",
    ]

    [dependency-groups]
    docs = [
      "s",
    ]
    test = [
      "p",
      "q",
    ]

    [tool.uv]
    vu = "uv"

    [tool.ruff]
    mr = "vr"
    [tool.ruff.test]
    mrt = "vrt"

    [tool.pytest]
    mk = "mv"

    [tool.coverage]
    aa = "bb"
    [tool.coverage.paths]
    ab = "bc"
    [tool.coverage.report]
    cd = "de"
    [tool.coverage.run]
    ef = "fg"

    [tool.mypy]
    mk = "mv"

    [extra]
    ek = "ev"

    [tool.undefined]
    mu = "mu"

    [demo]
    ed = "ed"
    "#},
)]
fn test_reorder_table(#[case] start: &str, #[case] expected: &str) {
    let root_ast = parse(start).into_syntax().clone_for_update();
    let tables = Tables::from_ast(&root_ast);
    reorder_tables(&root_ast, &tables);
    let opt = Options {
        column_width: 1,
        ..Options::default()
    };
    let got = format_syntax(root_ast, opt);
    assert_eq!(got, expected);
}
