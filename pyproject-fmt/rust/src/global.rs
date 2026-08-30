use common::sections;
use toml_doc::Document;

use crate::{hatch, poetry, pyright};

/// The order a tool builds from the file itself, since the keys it ranks are the ones the file
/// happens to define. Ordering a table twice with a different order would undo the first.
fn dynamic_key_order(table: &str, entries: &[toml_doc::Entry<'_>]) -> Option<Vec<String>> {
    match table {
        "tool.poetry" => Some(poetry::build_root_key_order(entries)),
        "tool.hatch" => Some(hatch::build_key_order(entries)),
        "tool.pyright" | "tool.basedpyright" => Some(pyright::build_key_order(entries)),
        _ => None,
    }
}

fn static_key_order(table: &str) -> Option<Vec<String>> {
    crate::rules::every_order()
        .find(|(name, _)| *name == table)
        .map(|(_, order)| order.iter().map(|key| (*key).to_string()).collect())
}

/// Where a sub-table sits among its parent's keys. Ranking needs no file context, so the order a
/// tool builds from the file is taken with none.
fn key_order(table: &[String]) -> Option<Vec<String>> {
    let table = sections::dotted_name(table);
    dynamic_key_order(&table, &[]).or_else(|| static_key_order(&table))
}

/// Where each table sits in a formatted file.
const TABLE_ORDER: &[&str] = &[
    "build-system",
    "project",
    "dependency-groups",
    "tool.poetry",
    "tool.poetry-dynamic-versioning",
    "tool.pdm",
    "tool.setuptools",
    "tool.distutils",
    "tool.setuptools_scm",
    "tool.hatch",
    "tool.flit",
    "tool.scikit-build",
    "tool.meson-python",
    "tool.maturin",
    "tool.pixi",
    "tool.whey",
    "tool.py-build-cmake",
    "tool.sphinx-theme-builder",
    "tool.uv",
    "tool.cibuildwheel",
    "tool.nuitka",
    "tool.autopep8",
    "tool.black",
    "tool.yapf",
    "tool.djlint",
    "tool.ruff",
    "tool.isort",
    "tool.flake8",
    "tool.pycln",
    "tool.nbqa",
    "tool.pylint",
    "tool.repo-review",
    "tool.codespell",
    "tool.docformatter",
    "tool.pydoclint",
    "tool.interrogate",
    "tool.tomlsort",
    "tool.check-manifest",
    "tool.check-sdist",
    "tool.check-wheel-contents",
    "tool.deptry",
    "tool.vulture",
    "tool.pyproject-fmt",
    "tool.typos",
    "tool.bandit",
    "tool.mypy",
    "tool.pyrefly",
    "tool.pyright",
    "tool.ty",
    "tool.django-stubs",
    "tool.pytest",
    "tool.pytest_env",
    "tool.pytest-enabler",
    "tool.coverage",
    "tool.doit",
    "tool.spin",
    "tool.tox",
    "tool.bumpversion",
    "tool.commitizen",
    "tool.jupyter-releaser",
    "tool.semantic_release",
    "tool.tbump",
    "tool.towncrier",
    "tool.vendoring",
];

pub fn reorder_tables(document: &mut Document<'_>) {
    sections::reorder_within_keeping(document, TABLE_ORDER, &["tool"], &key_order, &keep_within);
    for section in &mut document.sections {
        let path = sections::dispatch_name(&section.header.key);
        // a tool that builds its order from the file gets that order here too, or this pass would
        // rank the keys it discovered as though the file had never named them
        let Some(order) = dynamic_key_order(&path, &section.entries).or_else(|| static_key_order(&path)) else {
            continue;
        };
        let order: Vec<&str> = order.iter().map(String::as_str).collect();
        let keep = keep_order(&path, &section.entries);
        let keep: Vec<&str> = keep.iter().map(String::as_str).collect();
        sections::reorder_keys_within(&mut section.entries, &order, &keep);
    }
}

/// The tables written under one of these names hold the place the file gave them, since a tool runs
/// its hooks and applies its overrides in the order they are written.
fn keep_within(table: &[String]) -> Vec<String> {
    match sections::dotted_name(table).as_str() {
        "tool.hatch" => ["hooks", "overrides", "matrix"].map(str::to_owned).to_vec(),
        "tool.coverage" => vec![String::from("paths")],
        "tool.semantic_release" => vec![String::from("branches")],
        _ => Vec::new(),
    }
}

/// The names whose keys hold the order the file gave them, since where each one sits among the
/// others is part of what the tool reading them does.
fn keep_order(table: &str, entries: &[toml_doc::Entry<'_>]) -> Vec<String> {
    match table {
        "tool.hatch" => hatch::keep_order(entries),
        "tool.coverage" => vec![String::from("paths")],
        "tool.semantic_release" => vec![String::from("branches")],
        _ => Vec::new(),
    }
}
