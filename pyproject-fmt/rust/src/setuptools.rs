use common::arrays::sort_names_in;
use common::sections::{self, InlineSchema};
use toml_doc::Document;

// Sub-tables collapse to dotted keys (packages.find.where, package-data."*", etc.); the "packages" prefix catches
// them all, with finer entries added for inner ordering.
pub const KEY_ORDER: &[&str] = &[
    "py-modules",
    "packages.find.where",
    "packages.find.include",
    "packages.find.exclude",
    "packages.find.namespaces",
    "packages.find-namespace.where",
    "packages.find-namespace.include",
    "packages.find-namespace.exclude",
    "packages.find-namespace.namespaces",
    "packages",
    "package-dir",
    "include-package-data",
    "package-data",
    "exclude-package-data",
    "dynamic",
    "ext-modules",
    "cmdclass",
    "platforms",
    "provides",
    "obsoletes",
    "license-files",
    "data-files",
    "script-files",
    "namespace-packages",
    "zip-safe",
    "eager-resources",
    "dependency-links",
];

// Safe-to-sort arrays only; packages, license-files, ext-module paths/argv, `script-files` and the
// `data-files` lists are left out because order affects build, link, PEP-639 concatenation, or which
// of two files sharing a name is the one installed.
const TOP_LEVEL_SORT_ARRAYS: &[&str] = &[
    "py-modules",
    "platforms",
    "provides",
    "obsoletes",
    "namespace-packages",
    "eager-resources",
    "packages.find.include",
    "packages.find.exclude",
    "packages.find-namespace.include",
    "packages.find-namespace.exclude",
];

pub const SCM_KEY_ORDER: &[&str] = &[
    "version_file",
    "version_file_template",
    "version_scheme",
    "local_scheme",
    "version_cls",
    "normalize",
    "root",
    "relative_to",
    "fallback_root",
    "parent",
    "search_parent_directories",
    "dist_name",
    "tag_regex",
    "parse",
    "parentdir_prefix_version",
    "fallback_version",
    "scm.git.pre_parse",
    "scm.git.describe_command",
    "scm",
    "git_describe_command",
    "write_to",
    "write_to_template",
    "version_class",
    "template",
];

pub fn fix(document: &mut Document<'_>) {
    fix_setuptools_scm(document);
    fix_expanded_packages_find(document);
    fix_expanded_dynamic_table(document);
    fix_expanded_data_tables(document, "tool.setuptools.package-data", Patterns::AreASet);
    fix_expanded_data_tables(document, "tool.setuptools.exclude-package-data", Patterns::AreASet);
    fix_expanded_data_tables(document, "tool.setuptools.data-files", Patterns::AreInOrder);
    fix_expanded_alpha_table(document, "tool.setuptools.cmdclass");
}

/// Whether what the name holds is a list of names, which sorts.
pub fn sorts(key: &str) -> bool {
    TOP_LEVEL_SORT_ARRAYS.contains(&key) || is_inner_package_data_array(key)
}

fn is_inner_package_data_array(key: &str) -> bool {
    for prefix in ["package-data.", "exclude-package-data."] {
        if key.strip_prefix(prefix).is_some_and(|rest| !rest.is_empty()) {
            return true;
        }
    }
    false
}

fn fix_setuptools_scm(document: &mut Document<'_>) {
    let Some(section) = sections::first(document, "tool.setuptools_scm") else {
        return;
    };
    sections::reorder_keys(&mut section.entries, SCM_KEY_ORDER);
}

fn fix_expanded_packages_find(document: &mut Document<'_>) {
    for key in [
        "tool.setuptools.packages.find",
        "tool.setuptools.packages.find-namespace",
    ] {
        let Some(section) = sections::first(document, key) else {
            continue;
        };
        sections::for_entries(section, |inner, value| {
            if matches!(inner, "include" | "exclude") {
                sort_names_in(value);
            }
        });
        sections::reorder_keys(&mut section.entries, &["where", "include", "exclude", "namespaces"]);
    }
}

fn fix_expanded_dynamic_table(document: &mut Document<'_>) {
    let Some(section) = sections::first(document, "tool.setuptools.dynamic") else {
        return;
    };
    // [""] sorts every key alphabetically.
    sections::reorder_keys(&mut section.entries, &[]);
}

/// Whether the patterns written under one destination name a set, or a list read in order.
#[derive(Clone, Copy, PartialEq)]
enum Patterns {
    AreASet,
    AreInOrder,
}

fn fix_expanded_data_tables(document: &mut Document<'_>, table_key: &str, patterns: Patterns) {
    let Some(section) = sections::first(document, table_key) else {
        return;
    };
    // `*` is not a name TOML reads bare, so the file writes it in quotes and a rule matching it
    // spells it the same way
    let catch_all = sections::quoted_segment("*");
    let mut others: Vec<String> = Vec::new();
    sections::for_entries(section, |key, value| {
        if patterns == Patterns::AreASet {
            sort_names_in(value);
        }
        // a table names each of its keys once, so the catch-all is the only one that is not one of
        // the names that sort
        if key != catch_all {
            others.push(key.to_owned());
        }
    });
    // `*` catch-all first, then alphabetical.
    let mut order: Vec<String> = vec![String::new(), catch_all];
    others.sort();
    order.extend(others);
    let refs: Vec<&str> = order.iter().map(String::as_str).collect();
    sections::reorder_keys(&mut section.entries, &refs);
}

fn fix_expanded_alpha_table(document: &mut Document<'_>, table_key: &str) {
    let Some(section) = sections::first(document, table_key) else {
        return;
    };
    sections::reorder_keys(&mut section.entries, &[]);
}

// Discriminators attr/content-type are unique to dynamic directives; file is too generic, so it is omitted from the
// discriminator set.
const DYNAMIC_DIRECTIVE_ORDER: &[&str] = &["attr", "file", "content-type"];

pub const INLINE_TABLE_SCHEMAS: &[InlineSchema<'static>] = &[
    InlineSchema {
        discriminator: "attr",
        key_order: DYNAMIC_DIRECTIVE_ORDER,
    },
    InlineSchema {
        discriminator: "content-type",
        key_order: DYNAMIC_DIRECTIVE_ORDER,
    },
];

pub fn reorder_inline_tables(document: &mut Document<'_>) {
    let name = ["tool", "setuptools"].map(str::to_owned);
    sections::reorder_inline_tables(document, &name, INLINE_TABLE_SCHEMAS);
}
