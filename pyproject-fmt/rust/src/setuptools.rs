use common::array::sort_strings;
use common::table::{for_entries, reorder_inline_table_keys, reorder_table_keys, InlineTableSchema, Tables};
use lexical_sort::natural_lexical_cmp;
use tombi_syntax::SyntaxNode;

// Sub-tables collapse to dotted keys (packages.find.where, package-data."*", etc.); the
// "packages" prefix catches them all, with finer entries added for inner ordering.
const KEY_ORDER: &[&str] = &[
    "",
    // --- Packaging discovery ---
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
    // --- Package data ---
    "include-package-data",
    "package-data",
    "exclude-package-data",
    // --- Dynamic metadata ---
    "dynamic",
    // --- Extensions / build customization ---
    "ext-modules",
    "cmdclass",
    // --- Distribution metadata ---
    "platforms",
    "provides",
    "obsoletes",
    "license-files",
    // --- Discouraged / legacy data files ---
    "data-files",
    // --- Deprecated / obsolete (pushed last) ---
    "script-files",
    "namespace-packages",
    "zip-safe",
    "eager-resources",
    "dependency-links",
];

// Safe-to-sort arrays only; packages, license-files, and ext-module paths/argv are left
// out — order affects build, link, or PEP-639 concatenation.
const TOP_LEVEL_SORT_ARRAYS: &[&str] = &[
    "py-modules",
    "platforms",
    "provides",
    "obsoletes",
    "script-files",
    "namespace-packages",
    "eager-resources",
    "packages.find.include",
    "packages.find.exclude",
    "packages.find-namespace.include",
    "packages.find-namespace.exclude",
];

const SCM_KEY_ORDER: &[&str] = &[
    "",
    // --- Version output ---
    "version_file",
    "version_file_template",
    // --- Version computation / scheme ---
    "version_scheme",
    "local_scheme",
    "version_cls",
    "normalize",
    // --- Root / discovery ---
    "root",
    "relative_to",
    "fallback_root",
    "parent",
    "search_parent_directories",
    "dist_name",
    // --- Tag / parse ---
    "tag_regex",
    "parse",
    "parentdir_prefix_version",
    "fallback_version",
    // --- Nested SCM-specific tables (collapse to dotted keys) ---
    "scm.git.pre_parse",
    "scm.git.describe_command",
    "scm",
    // --- Deprecated (push last) ---
    "git_describe_command",
    "write_to",
    "write_to_template",
    "version_class",
    "template",
];

pub fn fix(tables: &mut Tables) {
    fix_setuptools(tables);
    fix_setuptools_scm(tables);
    fix_expanded_packages_find(tables);
    fix_expanded_dynamic_table(tables);
    fix_expanded_data_tables(tables, "tool.setuptools.package-data");
    fix_expanded_data_tables(tables, "tool.setuptools.exclude-package-data");
    fix_expanded_data_tables(tables, "tool.setuptools.data-files");
    fix_expanded_alpha_table(tables, "tool.setuptools.cmdclass");
}

fn fix_setuptools(tables: &mut Tables) {
    let Some(elements) = tables.get("tool.setuptools") else {
        return;
    };
    let table = &mut elements.first().unwrap().borrow_mut();
    for_entries(table, &mut |key, entry| {
        let k = key.as_str();
        if TOP_LEVEL_SORT_ARRAYS.contains(&k) {
            sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
        } else if is_inner_package_data_array(k) {
            sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
        }
    });
    reorder_table_keys(table, KEY_ORDER);
}

fn is_inner_package_data_array(key: &str) -> bool {
    for prefix in ["package-data.", "exclude-package-data.", "data-files."] {
        if let Some(rest) = key.strip_prefix(prefix) {
            if !rest.is_empty() {
                return true;
            }
        }
    }
    false
}

fn fix_setuptools_scm(tables: &mut Tables) {
    let Some(elements) = tables.get("tool.setuptools_scm") else {
        return;
    };
    let table = &mut elements.first().unwrap().borrow_mut();
    reorder_table_keys(table, SCM_KEY_ORDER);
}

fn fix_expanded_packages_find(tables: &mut Tables) {
    for key in [
        "tool.setuptools.packages.find",
        "tool.setuptools.packages.find-namespace",
    ] {
        let Some(elements) = tables.get(key) else {
            continue;
        };
        let table = &mut elements.first().unwrap().borrow_mut();
        for_entries(table, &mut |inner, entry| {
            if matches!(inner.as_str(), "include" | "exclude") {
                sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
            }
        });
        reorder_table_keys(table, &["", "where", "include", "exclude", "namespaces"]);
    }
}

fn fix_expanded_dynamic_table(tables: &mut Tables) {
    let Some(elements) = tables.get("tool.setuptools.dynamic") else {
        return;
    };
    let table = &mut elements.first().unwrap().borrow_mut();
    // [""] sorts every key alphabetically.
    reorder_table_keys(table, &[""]);
}

fn fix_expanded_data_tables(tables: &mut Tables, table_key: &str) {
    let Some(elements) = tables.get(table_key) else {
        return;
    };
    let table = &mut elements.first().unwrap().borrow_mut();
    for_entries(table, &mut |_key, entry| {
        sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
    });
    // `*` catch-all first, then alphabetical.
    let mut order: Vec<String> = vec![String::new(), String::from("*")];
    let mut others: Vec<String> = Vec::new();
    for_entries(table, &mut |key, _| {
        let k = key.as_str();
        if k != "*" && !others.contains(&k.to_string()) {
            others.push(k.to_string());
        }
    });
    others.sort();
    order.extend(others);
    let refs: Vec<&str> = order.iter().map(String::as_str).collect();
    reorder_table_keys(table, &refs);
}

fn fix_expanded_alpha_table(tables: &mut Tables, table_key: &str) {
    let Some(elements) = tables.get(table_key) else {
        return;
    };
    let table = &mut elements.first().unwrap().borrow_mut();
    reorder_table_keys(table, &[""]);
}

// Discriminators attr/content-type are unique to dynamic directives; file is too generic
// to discriminate on, so it is omitted from the discriminator set.
const DYNAMIC_DIRECTIVE_ORDER: &[&str] = &["attr", "file", "content-type"];

pub const INLINE_TABLE_SCHEMAS: &[InlineTableSchema] = &[
    InlineTableSchema {
        discriminator: "attr",
        key_order: DYNAMIC_DIRECTIVE_ORDER,
    },
    InlineTableSchema {
        discriminator: "content-type",
        key_order: DYNAMIC_DIRECTIVE_ORDER,
    },
];

pub fn reorder_inline_tables(root_ast: &SyntaxNode) {
    reorder_inline_table_keys(root_ast, INLINE_TABLE_SCHEMAS);
}
