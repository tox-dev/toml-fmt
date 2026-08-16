use common::table::Tables;
use tombi_syntax::SyntaxNode;

use crate::{
    autopep8, bandit, black, bumpversion, check_manifest, cibuildwheel, codespell, commitizen, coverage, deptry,
    djlint, docformatter, hatch, interrogate, isort, maturin, mypy, pdm, pixi, poetry, project, pylint, pyproject_fmt,
    pyrefly, pyright, pytest, ruff, scikit_build, semantic_release, setuptools, towncrier, ty, uv, vulture, yapf,
};

/// The key order each tool applies to its table, so expanded sub-tables line up with the dotted-key form.
const TABLE_KEY_ORDERS: &[(&str, &[&str])] = &[
    ("project", project::KEY_ORDER),
    ("tool.autopep8", autopep8::KEY_ORDER),
    ("tool.bandit", bandit::KEY_ORDER),
    ("tool.black", black::KEY_ORDER),
    ("tool.bumpversion", bumpversion::KEY_ORDER),
    ("tool.check-manifest", check_manifest::KEY_ORDER),
    ("tool.cibuildwheel", cibuildwheel::KEY_ORDER),
    ("tool.codespell", codespell::KEY_ORDER),
    ("tool.commitizen", commitizen::KEY_ORDER),
    ("tool.coverage", coverage::KEY_ORDER),
    ("tool.deptry", deptry::KEY_ORDER),
    ("tool.djlint", djlint::KEY_ORDER),
    ("tool.docformatter", docformatter::KEY_ORDER),
    ("tool.hatch", hatch::KEY_ORDER),
    ("tool.interrogate", interrogate::KEY_ORDER),
    ("tool.isort", isort::KEY_ORDER),
    ("tool.maturin", maturin::KEY_ORDER),
    ("tool.mypy", mypy::KEY_ORDER),
    ("tool.pdm", pdm::KEY_ORDER),
    ("tool.pixi", pixi::KEY_ORDER),
    ("tool.pylint", pylint::KEY_ORDER),
    ("tool.pyproject-fmt", pyproject_fmt::KEY_ORDER),
    ("tool.pyrefly", pyrefly::KEY_ORDER),
    ("tool.pyright", pyright::KEY_ORDER_PRE_REPORTS),
    ("tool.basedpyright", pyright::KEY_ORDER_PRE_REPORTS),
    ("tool.pytest", pytest::KEY_ORDER),
    ("tool.ruff", ruff::KEY_ORDER),
    ("tool.scikit-build", scikit_build::KEY_ORDER),
    ("tool.semantic_release", semantic_release::KEY_ORDER),
    ("tool.setuptools", setuptools::KEY_ORDER),
    ("tool.setuptools_scm", setuptools::SCM_KEY_ORDER),
    ("tool.towncrier", towncrier::KEY_ORDER),
    ("tool.ty", ty::KEY_ORDER),
    ("tool.uv", uv::KEY_ORDER),
    ("tool.vulture", vulture::KEY_ORDER),
    ("tool.yapf", yapf::KEY_ORDER),
];

fn key_order(table: &str) -> Option<Vec<String>> {
    // The poetry order is built per file, since each dependency group contributes its own keys.
    if table == "tool.poetry" {
        return Some(poetry::root_key_order(&[]));
    }
    TABLE_KEY_ORDERS
        .iter()
        .find(|(name, _)| *name == table)
        .map(|(_, order)| order.iter().map(|key| (*key).to_string()).collect())
}

pub fn reorder_tables(root_ast: &SyntaxNode, tables: &Tables, root_table_spacing: &str, sub_table_spacing: &str) {
    tables.reorder_with_key_order(
        root_ast,
        &[
            "",
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
        ],
        &["tool"], // Treat tool.* as distinct base keys (e.g., tool.black != tool.ruff)
        root_table_spacing,
        sub_table_spacing,
        &key_order,
    );
}
