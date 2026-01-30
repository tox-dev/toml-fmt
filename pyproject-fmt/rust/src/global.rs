use common::taplo::rowan::SyntaxNode;
use common::taplo::syntax::Lang;

use common::table::Tables;

pub fn reorder_tables(root_ast: &SyntaxNode<Lang>, tables: &Tables) {
    tables.reorder(
        root_ast,
        &[
            "",
            "build-system",
            "project",
            "dependency-groups",
            // Build backends
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
            "tool.whey",
            "tool.py-build-cmake",
            "tool.sphinx-theme-builder",
            "tool.uv",
            // Builders
            "tool.cibuildwheel",
            "tool.nuitka",
            // Formatters and linters
            "tool.autopep8",
            "tool.black",
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
            "tool.tomlsort",
            "tool.check-manifest",
            "tool.check-sdist",
            "tool.check-wheel-contents",
            "tool.deptry",
            "tool.pyproject-fmt",
            "tool.typos",
            // Testing
            "tool.pytest",
            "tool.pytest_env",
            "tool.pytest-enabler",
            "tool.coverage",
            // Runners
            "tool.doit",
            "tool.spin",
            "tool.tox",
            // Releasers/bumpers
            "tool.bumpversion",
            "tool.jupyter-releaser",
            "tool.tbump",
            "tool.towncrier",
            "tool.vendoring",
            // Type checking
            "tool.mypy",
            "tool.pyrefly",
            "tool.pyright",
            "tool.ty",
            "tool.django-stubs",
        ],
        &["tool"], // Treat tool.* as distinct base keys (e.g., tool.black != tool.ruff)
    );
}
