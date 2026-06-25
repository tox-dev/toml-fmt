use common::array::sort_strings;
use common::table::{for_entries, reorder_table_keys, Tables};
use lexical_sort::natural_lexical_cmp;

const KEY_ORDER: &[&str] = &[
    "",
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

pub fn fix(tables: &mut Tables) {
    fix_root(tables);
    fix_type_aot(tables);
    fix_section_aot(tables);
}

fn fix_root(tables: &mut Tables) {
    let Some(elements) = tables.get("tool.towncrier") else {
        return;
    };
    let table = &mut elements.first().unwrap().borrow_mut();
    for_entries(table, &mut |key, entry| {
        if key.as_str() == "ignore" {
            sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
        }
    });
    reorder_table_keys(table, KEY_ORDER);
}

fn fix_type_aot(tables: &mut Tables) {
    let Some(entries) = tables.get("tool.towncrier.type") else {
        return;
    };
    for entry_ref in entries {
        let table = &mut entry_ref.borrow_mut();
        reorder_table_keys(table, &["", "directory", "name", "showcontent"]);
    }
}

fn fix_section_aot(tables: &mut Tables) {
    let Some(entries) = tables.get("tool.towncrier.section") else {
        return;
    };
    for entry_ref in entries {
        let table = &mut entry_ref.borrow_mut();
        reorder_table_keys(table, &["", "path", "name", "showcontent"]);
    }
}
