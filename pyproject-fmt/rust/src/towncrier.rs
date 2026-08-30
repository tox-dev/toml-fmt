use common::sections;
use toml_doc::Document;

pub const KEY_ORDER: &[&str] = &[
    "name",
    "version",
    "package",
    "package_dir",
    "directory",
    "filename",
    "start_string",
    "template",
    "title_format",
    "issue_format",
    "underlines",
    "wrap",
    "all_bullets",
    "single_file",
    "orphan_prefix",
    "create_eof_newline",
    "create_add_extension",
    "ignore",
    "type",
    "section",
];

pub fn fix(document: &mut Document<'_>) {
    fix_type_aot(document);
    fix_section_aot(document);
}

/// Whether what the name holds is a list of names, which sorts.
pub fn sorts(key: &str) -> bool {
    key == "ignore"
}

fn fix_type_aot(document: &mut Document<'_>) {
    let name = ["tool", "towncrier", "type"].map(str::to_owned);
    sections::for_array_elements(document, &name, &["directory", "name", "showcontent"], &mut |_, _| {});
}

fn fix_section_aot(document: &mut Document<'_>) {
    let name = ["tool", "towncrier", "section"].map(str::to_owned);
    sections::for_array_elements(document, &name, &["path", "name", "showcontent"], &mut |_, _| {});
}
