use common::array::sort_strings;
use common::table::{for_entries, reorder_table_keys, Tables};
use lexical_sort::natural_lexical_cmp;

const KEY_ORDER: &[&str] = &[
    "",
    // Selection
    "build",
    "skip",
    "test-skip",
    "archs",
    "enable",
    "free-threaded-support",
    // Build configuration
    "build-frontend",
    "build-verbosity",
    "config-settings",
    "dependency-versions",
    "environment",
    "environment-pass",
    // Build phases
    "before-all",
    "before-build",
    "repair-wheel-command",
    // Test phases
    "before-test",
    "test-command",
    "test-requires",
    "test-extras",
    "test-groups",
    "test-sources",
    // Platform images
    "manylinux-x86_64-image",
    "manylinux-i686-image",
    "manylinux-aarch64-image",
    "manylinux-ppc64le-image",
    "manylinux-s390x-image",
    "manylinux-armv7l-image",
    "manylinux-pypy_x86_64-image",
    "manylinux-pypy_i686-image",
    "manylinux-pypy_aarch64-image",
    "musllinux-x86_64-image",
    "musllinux-i686-image",
    "musllinux-aarch64-image",
    "musllinux-ppc64le-image",
    "musllinux-s390x-image",
    "musllinux-armv7l-image",
    // Engine
    "container-engine",
    // Per-platform sub-tables (collapsed)
    "linux",
    "macos",
    "windows",
    "android",
    "ios",
    "pyodide",
    // Overrides last
    "overrides",
];

// Most arrays are CLI argv (order matters); only these are set semantics.
const SORT_ARRAYS: &[&str] = &["enable", "test-extras", "test-groups"];

pub fn fix(tables: &mut Tables) {
    fix_one(tables, "tool.cibuildwheel");
    // Per-platform tables follow the same order when not collapsed into the parent.
    for plat in ["linux", "macos", "windows", "android", "ios", "pyodide"] {
        fix_one(tables, &format!("tool.cibuildwheel.{plat}"));
    }
    fix_overrides_aot(tables);
}

fn fix_one(tables: &mut Tables, table_name: &str) {
    let Some(elements) = tables.get(table_name) else {
        return;
    };
    let table = &mut elements.first().unwrap().borrow_mut();
    for_entries(table, &mut |key, entry| {
        if SORT_ARRAYS.contains(&key.as_str()) {
            sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
        }
    });
    reorder_table_keys(table, KEY_ORDER);
}

fn fix_overrides_aot(tables: &mut Tables) {
    let Some(entries) = tables.get("tool.cibuildwheel.overrides") else {
        return;
    };
    for entry_ref in entries {
        let table = &mut entry_ref.borrow_mut();
        for_entries(table, &mut |key, entry| {
            if SORT_ARRAYS.contains(&key.as_str()) {
                sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
            }
        });
        // `select` always first (required), then the regular cibuildwheel keys.
        let mut order: Vec<&str> = vec!["", "select"];
        order.extend(KEY_ORDER.iter().filter(|k| !k.is_empty() && **k != "overrides"));
        reorder_table_keys(table, &order);
    }
}
