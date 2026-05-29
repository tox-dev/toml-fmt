use common::array::sort_strings;
use common::table::{for_entries, reorder_table_keys, Tables};
use lexical_sort::natural_lexical_cmp;

// Group: meta → length → targets → include/exclude → behavior flags → output.
const KEY_ORDER: &[&str] = &[
    "",
    "required-version",
    "target-version",
    "line-length",
    "include",
    "extend-exclude",
    "force-exclude",
    "exclude",
    "skip-string-normalization",
    "skip-magic-trailing-comma",
    "preview",
    "unstable",
    "enable-unstable-feature",
    "fast",
    "workers",
    "color",
    "verbose",
    "quiet",
];

pub fn fix(tables: &mut Tables) {
    let Some(elements) = tables.get("tool.black") else {
        return;
    };
    let table = &mut elements.first().unwrap().borrow_mut();
    for_entries(table, &mut |key, entry| match key.as_str() {
        "target-version" | "enable-unstable-feature" => {
            sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
        }
        _ => {}
    });
    reorder_table_keys(table, KEY_ORDER);
}
