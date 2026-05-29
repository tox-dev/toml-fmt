use common::array::sort_strings;
use common::table::{for_entries, reorder_table_keys, Tables};
use lexical_sort::natural_lexical_cmp;

const KEY_ORDER: &[&str] = &[
    "",
    // Module identity
    "module-name",
    "bindings",
    "python-source",
    "python-packages",
    "python-bin-path",
    // Source layout
    "src",
    "manifest-path",
    "include",
    "exclude",
    "sdist-include",
    "sdist-generator",
    "data",
    // Cargo
    "features",
    "no-default-features",
    "all-features",
    "cargo-extra-args",
    "rustc-extra-args",
    "config",
    "profile",
    "target",
    "target-dir",
    // Compatibility / strip
    "compatibility",
    "auditwheel",
    "skip-auditwheel",
    "strip",
    "frozen",
    "locked",
    "offline",
    "zig",
    // Behavior
    "use-cross",
];

const SORT_ARRAYS: &[&str] = &["python-packages", "include", "exclude", "sdist-include", "features"];

pub fn fix(tables: &mut Tables) {
    let Some(elements) = tables.get("tool.maturin") else {
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
