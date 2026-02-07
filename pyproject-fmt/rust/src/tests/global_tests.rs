use common::array::ensure_all_arrays_multiline;
use common::table::Tables;
use indoc::indoc;

use super::{format_syntax, parse};
use crate::global::reorder_tables;

fn reorder_table_helper(start: &str) -> String {
    let root_ast = parse(start);
    let tables = Tables::from_ast(&root_ast);
    reorder_tables(&root_ast, &tables);
    ensure_all_arrays_multiline(&root_ast);
    format_syntax(root_ast, 120)
}

#[test]
fn test_reorder_table_reorder() {
    let start = indoc! {r#"
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
    "#};
    let res = reorder_table_helper(start);
    insta::assert_snapshot!(res, @r#"
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

    [tool.undefined]
    mu = "mu"

    [extra]
    ek = "ev"

    [demo]
    ed = "ed"
    "#);
}
