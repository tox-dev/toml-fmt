use common::array::sort_strings;
use common::table::{for_entries, reorder_table_keys, Tables};
use lexical_sort::natural_lexical_cmp;

const KEY_ORDER: &[&str] = &[
    "",
    // Rule selection
    "name",
    "version_type",
    "schema",
    "schema_pattern",
    "allowed_prefixes",
    // Version source
    "version",
    "version_scheme",
    "version_provider",
    "version_files",
    // Bump behavior
    "bump_message",
    "always_signoff",
    "retry_after_failure",
    "encoding",
    "major_version_zero",
    // Tag / sign
    "tag_format",
    "annotated_tag",
    "annotated_tag_message",
    "gpg_sign",
    "use_shortcuts",
    // Changelog
    "changelog_file",
    "changelog_format",
    "changelog_incremental",
    "changelog_start_rev",
    "changelog_merge_prerelease",
    "update_changelog_on_bump",
    "changelog_pattern",
    "extras",
    "extra_files",
    "template",
    // Hooks
    "pre_bump_hooks",
    "post_bump_hooks",
    // Customization
    "customize",
    // Commit
    "discover_secret",
];

const SORT_ARRAYS: &[&str] = &["version_files", "allowed_prefixes", "extras", "extra_files"];

pub fn fix(tables: &mut Tables) {
    let Some(elements) = tables.get("tool.commitizen") else {
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
