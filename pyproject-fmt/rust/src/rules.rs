//! What each tool table asks of the formatter, in one place: where its keys sit, and which of them
//! hold a list of names.
//!
//! A table's name, its key order and how its values are read all say something about the same
//! table, so they are written together rather than in three lists that have to agree.

use common::arrays::{dedupe_strings_in, sort_names_in};
use common::sections;
use toml_doc::Document;

use crate::{
    autopep8, bandit, black, bumpversion, check_manifest, cibuildwheel, codespell, commitizen, coverage, deptry,
    djlint, docformatter, hatch, interrogate, isort, maturin, mypy, pdm, pixi, project, pylint, pyproject_fmt, pyrefly,
    pyright, pytest, ruff, scikit_build, semantic_release, setuptools, towncrier, ty, uv, vulture, yapf,
};

/// A table the formatter reads by its key order alone.
pub struct TableRules {
    /// The table these rules format, spelled as the file spells its path.
    pub table: &'static str,
    /// Where each key sits among the others.
    pub order: &'static [&'static str],
    /// Whether what the name holds is a list of names, which sorts.
    pub sorts: fn(&str) -> bool,
    /// Whether that list also says each name once.
    pub dedupes: fn(&str) -> bool,
    /// The keys the file's own sequence orders, since where each one sits says something.
    pub keeps_order: &'static [&'static str],
}

/// Whether a name holds nothing this reads, which is what a table without such values says of them.
fn nothing(_key: &str) -> bool {
    false
}

/// A table read by its key order alone, whose arrays `sorts` names.
macro_rules! rules {
    ($table:expr, $order:expr, $sorts:expr $(,)?) => {
        TableRules {
            table: $table,
            order: $order,
            sorts: $sorts,
            dedupes: nothing,
            keeps_order: &[],
        }
    };
}

/// The tables whose formatting is their key order and which of their arrays name a set.
const PLAIN: &[TableRules] = &[
    rules!("tool.autopep8", autopep8::KEY_ORDER, autopep8::sorts),
    rules!("tool.bandit", bandit::KEY_ORDER, bandit::sorts),
    rules!("tool.black", black::KEY_ORDER, black::sorts),
    rules!("tool.bumpversion", bumpversion::KEY_ORDER, bumpversion::sorts),
    rules!("tool.check-manifest", check_manifest::KEY_ORDER, check_manifest::sorts),
    TableRules {
        table: "tool.coverage",
        order: coverage::KEY_ORDER,
        sorts: coverage::sorts,
        dedupes: nothing,
        // coverage maps a file with the first `[paths]` group that matches
        keeps_order: &["paths"],
    },
    rules!("tool.codespell", codespell::KEY_ORDER, codespell::sorts),
    rules!("tool.commitizen", commitizen::KEY_ORDER, commitizen::sorts),
    rules!("tool.deptry", deptry::KEY_ORDER, deptry::sorts),
    rules!("tool.djlint", djlint::KEY_ORDER, djlint::sorts),
    rules!("tool.docformatter", docformatter::KEY_ORDER, docformatter::sorts),
    rules!("tool.interrogate", interrogate::KEY_ORDER, interrogate::sorts),
    rules!("tool.isort", isort::KEY_ORDER, isort::sorts),
    rules!("tool.maturin", maturin::KEY_ORDER, maturin::sorts),
    rules!("tool.pylint", pylint::KEY_ORDER, pylint::sorts),
    TableRules {
        table: "tool.pyproject-fmt",
        order: pyproject_fmt::KEY_ORDER,
        sorts: pyproject_fmt::sorts,
        // every one of them is read as a set, so a name written twice says no more than once
        dedupes: pyproject_fmt::sorts,
        keeps_order: &[],
    },
    rules!("tool.pyrefly", pyrefly::KEY_ORDER, pyrefly::sorts),
    rules!("tool.pixi", pixi::KEY_ORDER, pixi::sorts),
    rules!(
        "tool.pixi.workspace",
        pixi::WORKSPACE_KEY_ORDER,
        pixi::sorts_in_workspace,
    ),
    rules!("tool.pytest", pytest::KEY_ORDER, pytest::sorts),
    rules!("tool.ruff", ruff::KEY_ORDER, ruff::sorts),
    rules!("tool.scikit-build", scikit_build::KEY_ORDER, scikit_build::sorts),
    TableRules {
        table: "tool.semantic_release",
        order: semantic_release::KEY_ORDER,
        sorts: semantic_release::sorts,
        dedupes: nothing,
        // the branch rules are read in order and the first match decides the release policy
        keeps_order: &["branches"],
    },
    rules!("tool.ty", ty::KEY_ORDER, ty::sorts),
    rules!("tool.ty.src", ty::SRC_KEY_ORDER, ty::sorts_in_src),
    rules!("tool.mypy", mypy::KEY_ORDER, mypy::sorts),
    rules!("tool.pdm", pdm::KEY_ORDER, pdm::sorts),
    rules!("tool.setuptools", setuptools::KEY_ORDER, setuptools::sorts),
    rules!("tool.towncrier", towncrier::KEY_ORDER, towncrier::sorts),
    rules!("tool.uv", uv::KEY_ORDER, uv::sorts),
    rules!("tool.uv.pip", uv::PIP_KEY_ORDER, uv::sorts_in_pip),
    // a source names one dependency, and nothing ranks one above another
    rules!("tool.uv.sources", &[], nothing),
    rules!("tool.vulture", vulture::KEY_ORDER, vulture::sorts),
    rules!("tool.yapf", yapf::KEY_ORDER, yapf::sorts),
];

/// The key order of the tables their own module formats, so a sub-table written out lines up with
/// the dotted-key form.
const ORDERS: &[(&str, &[&str])] = &[
    ("project", project::KEY_ORDER),
    ("tool.cibuildwheel", cibuildwheel::KEY_ORDER),
    ("tool.hatch", hatch::KEY_ORDER),
    ("tool.pyright", pyright::KEY_ORDER_PRE_REPORTS),
    ("tool.basedpyright", pyright::KEY_ORDER_PRE_REPORTS),
    ("tool.setuptools_scm", setuptools::SCM_KEY_ORDER),
];

/// Where the keys of each table sit, whichever formats them.
pub fn every_order() -> impl Iterator<Item = (&'static str, &'static [&'static str])> {
    PLAIN
        .iter()
        .filter(|rules| !rules.order.is_empty())
        .map(|rules| (rules.table, rules.order))
        .chain(ORDERS.iter().copied())
}

/// Format every table these rules speak for.
pub fn fix(document: &mut Document<'_>) {
    for rules in PLAIN {
        let path = sections::parse_name(rules.table);
        sections::for_keys_under(document, &path, |key, value| {
            if (rules.dedupes)(key) {
                dedupe_strings_in(value, &str::to_string);
            }
            if (rules.sorts)(key) {
                sort_names_in(value);
            }
        });
        sections::reorder_under_keeping(document, &path, rules.order, rules.keeps_order);
    }
}
