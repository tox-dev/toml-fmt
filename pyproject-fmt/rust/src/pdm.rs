use common::array::sort_strings;
use common::table::{for_entries, reorder_table_keys, Tables};
use lexical_sort::natural_lexical_cmp;

// Sub-tables collapse to dotted keys (version.source, build.includes, etc.).
const KEY_ORDER: &[&str] = &[
    "",
    "distribution",
    "package-type",
    "plugins",
    "resolution.respect-source-order",
    "resolution.allow-prereleases",
    "resolution.excludes",
    "resolution.overrides",
    "resolution",
    "version.source",
    "version.path",
    "version.getter",
    "version.write_to",
    "version.write_template",
    "version.tag_regex",
    "version.tag_filter",
    "version.fallback_version",
    "version.version_format",
    "version",
    "build.includes",
    "build.excludes",
    "build.source-includes",
    "build.package-dir",
    "build.is-purelib",
    "build.run-setuptools",
    "build.custom-hook",
    "build.editable-backend",
    "build",
    "scripts",
    "source",
    "dev-dependencies",
    "publish.repository",
    "publish.username",
    "publish.password",
    "publish.ca_certs",
    "publish.verify_ssl",
    "publish",
    "options.install",
    "options.lock",
    "options.update",
    "options.add",
    "options.remove",
    "options.list",
    "options.sync",
    "options.run",
    "options",
];

const SORT_ARRAYS_EXACT: &[&str] = &[
    "plugins",
    "build.includes",
    "build.excludes",
    "build.source-includes",
    "resolution.excludes",
];

pub fn fix(tables: &mut Tables) {
    fix_root(tables);
    fix_expanded_scripts(tables);
    fix_expanded_dev_dependencies(tables);
    fix_source_aot(tables);
}

fn fix_root(tables: &mut Tables) {
    let Some(elements) = tables.get("tool.pdm") else {
        return;
    };
    let table = &mut elements.first().unwrap().borrow_mut();
    for_entries(table, &mut |key, entry| {
        let k = key.as_str();
        if SORT_ARRAYS_EXACT.contains(&k) || is_dev_deps_value(k) {
            sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
        }
    });
    reorder_table_keys(table, KEY_ORDER);
}

fn is_dev_deps_value(key: &str) -> bool {
    if let Some(rest) = key.strip_prefix("dev-dependencies.") {
        return !rest.is_empty();
    }
    false
}

fn fix_expanded_scripts(tables: &mut Tables) {
    if let Some(elements) = tables.get("tool.pdm.scripts") {
        let table = &mut elements.first().unwrap().borrow_mut();
        reorder_table_keys(table, &[""]);
    }
}

fn fix_expanded_dev_dependencies(tables: &mut Tables) {
    let Some(elements) = tables.get("tool.pdm.dev-dependencies") else {
        return;
    };
    let table = &mut elements.first().unwrap().borrow_mut();
    for_entries(table, &mut |_key, entry| {
        sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
    });
    reorder_table_keys(table, &[""]);
}

fn fix_source_aot(tables: &mut Tables) {
    let Some(entries) = tables.get("tool.pdm.source") else {
        return;
    };
    for entry_ref in entries {
        let table = &mut entry_ref.borrow_mut();
        for_entries(table, &mut |key, entry| match key.as_str() {
            "include_packages" | "exclude_packages" => {
                sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
            }
            _ => {}
        });
        reorder_table_keys(
            table,
            &[
                "",
                "name",
                "url",
                "type",
                "verify_ssl",
                "include_packages",
                "exclude_packages",
            ],
        );
    }
}
