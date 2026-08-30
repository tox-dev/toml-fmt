pub const KEY_ORDER: &[&str] = &[
    "name",
    "version_type",
    "schema",
    "schema_pattern",
    "allowed_prefixes",
    "version",
    "version_scheme",
    "version_provider",
    "version_files",
    "bump_message",
    "always_signoff",
    "retry_after_failure",
    "encoding",
    "major_version_zero",
    "tag_format",
    "annotated_tag",
    "annotated_tag_message",
    "gpg_sign",
    "use_shortcuts",
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
    "pre_bump_hooks",
    "post_bump_hooks",
    "customize",
    "discover_secret",
];

const SORT_ARRAYS: &[&str] = &["version_files", "allowed_prefixes", "extras", "extra_files"];

/// Whether what the name holds is a list of names, which sorts.
pub fn sorts(key: &str) -> bool {
    SORT_ARRAYS.contains(&key)
}
