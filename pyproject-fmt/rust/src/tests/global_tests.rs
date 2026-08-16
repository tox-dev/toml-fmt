use common::array::ensure_all_arrays_multiline;
use common::table::Tables;
use indoc::indoc;

use super::{format_syntax, parse};
use crate::global::reorder_tables;

fn reorder_table_helper(start: &str) -> String {
    let root_ast = parse(start);
    let tables = Tables::from_ast(&root_ast);
    reorder_tables(&root_ast, &tables, "\n", "");
    ensure_all_arrays_multiline(&root_ast, 120);
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
    requires = [ "c", "d" ]

    [project]
    name = "alpha"
    dependencies = [ "e" ]

    [dependency-groups]
    docs = [ "s" ]
    test = [ "p", "q" ]

    [tool.uv]
    vu = "uv"

    [tool.ruff]
    mr = "vr"

    [tool.ruff.test]
    mrt = "vrt"

    [tool.mypy]
    mk = "mv"

    [tool.pytest]
    mk = "mv"

    [tool.coverage]
    aa = "bb"

    [tool.coverage.run]
    ef = "fg"

    [tool.coverage.paths]
    ab = "bc"

    [tool.coverage.report]
    cd = "de"

    [tool.undefined]
    mu = "mu"

    [extra]
    ek = "ev"

    [demo]
    ed = "ed"
    "#);
}

#[test]
fn test_reorder_sub_tables_follow_key_order() {
    let start = indoc! {r#"
    [tool.hatch.build]
    a = 1
    [tool.hatch.metadata]
    b = 2
    [tool.hatch.zzz]
    c = 3
    [tool.hatch.version]
    d = 4
    [tool.hatch.aaa]
    e = 5
    [tool.hatch]
    f = 6
    "#};
    let res = reorder_table_helper(start);
    insta::assert_snapshot!(res, @r#"
    [tool.hatch]
    f = 6

    [tool.hatch.version]
    d = 4

    [tool.hatch.metadata]
    b = 2

    [tool.hatch.build]
    a = 1

    [tool.hatch.aaa]
    e = 5

    [tool.hatch.zzz]
    c = 3
    "#);
}

#[test]
fn test_reorder_sub_tables_of_unknown_tool_stay_alphabetical() {
    let start = indoc! {r#"
    [tool.unknown.zzz]
    a = 1
    [tool.unknown.aaa]
    b = 2
    "#};
    let res = reorder_table_helper(start);
    insta::assert_snapshot!(res, @r#"
    [tool.unknown.aaa]
    b = 2

    [tool.unknown.zzz]
    a = 1
    "#);
}

#[test]
fn test_reorder_pixi_as_build_backend() {
    let start = indoc! {r#"
    [tool.ruff]
    mr="vr"
    [tool.pixi.project]
    pk="pv"
    [tool.pixi]
    pk="pv"
    [tool.mypy]
    mk="mv"
    "#};
    let res = reorder_table_helper(start);
    insta::assert_snapshot!(res, @r#"
    [tool.pixi]
    pk = "pv"

    [tool.pixi.project]
    pk = "pv"

    [tool.ruff]
    mr = "vr"

    [tool.mypy]
    mk = "mv"
    "#);
}

#[test]
fn test_reorder_bandit_as_linter() {
    let start = indoc! {r#"
    [tool.mypy]
    mk="mv"
    [tool.bandit]
    skips=["B101"]
    [tool.pytest]
    mk="mv"
    [tool.ruff]
    mr="vr"
    "#};
    let res = reorder_table_helper(start);
    insta::assert_snapshot!(res, @r#"
    [tool.ruff]
    mr = "vr"

    [tool.bandit]
    skips = [ "B101" ]

    [tool.mypy]
    mk = "mv"

    [tool.pytest]
    mk = "mv"
    "#);
}

#[test]
fn test_reorder_newly_categorized_tools() {
    let start = indoc! {r#"
    [tool.tbump]
    tk="tv"
    [tool.yapf]
    yk="yv"
    [tool.vulture]
    vk="vv"
    [tool.semantic_release]
    sk="sv"
    [tool.djlint]
    dk="dv"
    [tool.commitizen]
    ck="cv"
    [tool.interrogate]
    ik="iv"
    [tool.black]
    bk="bv"
    [tool.deptry]
    dk="dv"
    [tool.pydoclint]
    pk="pv"
    [tool.bumpversion]
    buk="buv"
    "#};
    let res = reorder_table_helper(start);
    insta::assert_snapshot!(res, @r#"
    [tool.black]
    bk = "bv"

    [tool.yapf]
    yk = "yv"

    [tool.djlint]
    dk = "dv"

    [tool.pydoclint]
    pk = "pv"

    [tool.interrogate]
    ik = "iv"

    [tool.deptry]
    dk = "dv"

    [tool.vulture]
    vk = "vv"

    [tool.bumpversion]
    buk = "buv"

    [tool.commitizen]
    ck = "cv"

    [tool.semantic_release]
    sk = "sv"

    [tool.tbump]
    tk = "tv"
    "#);
}
