use toml_doc::Document;

use crate::TableFormatConfig;

// Delegates to the shared tox rules; the `"tool.tox"` prefix resolves tables under that namespace instead of the
// root, so `[tool.tox]` in pyproject.toml formats identically to a standalone tox.toml.
const TOOL_TOX: &str = "tool.tox";

pub fn fix(document: &mut Document<'_>, table_config: &TableFormatConfig, width: common::nesting::Width) {
    // a file is free to write only `[tool.tox.env.test]`, or to spell the whole path in one dotted
    // key, and the rules below are as much for those as for the table they were written under
    if !holds_tox(document) {
        return;
    }
    // every rule below finds an environment by its own header, and the short format has already folded each one into
    // `[tool.tox]`. Writing them back out is what lets one set of rules serve both formats, and lets `[tool.tox]` say
    // what a standalone `tox.toml` says.
    let folded = table_config.should_collapse(&tox_name());
    if folded {
        common::nesting::expand_of(document, &tox_name());
        for name in ["env", "env_base"] {
            common::nesting::expand_of(document, &under_tox(name));
        }
    }
    tox_rules::normalize_aliases_with_prefix(document, TOOL_TOX);
    tox_rules::fix_envs_with_prefix(document, TOOL_TOX);
    // a folded environment is a run of keys of `[tool.tox]`, so it takes its place in that table's
    // order only once it is folded back
    if folded {
        common::nesting::collapse_of(document, &tox_name(), &|sub| table_config.should_collapse(sub), width);
    }
    tox_rules::fix_root_with_prefix(document, TOOL_TOX);
    // `pin_envs` is a tox-toml-fmt setting with no flag here, so the list sorts the way an unpinned
    // `tox.toml` sorts one
    tox_rules::sort_env_list_with_prefix(document, &[], TOOL_TOX);
}

fn tox_name() -> Vec<String> {
    ["tool", "tox"].map(str::to_owned).to_vec()
}

fn under_tox(name: &str) -> Vec<String> {
    ["tool", "tox", name].map(str::to_owned).to_vec()
}

/// Whether the file says anything under `tool.tox`, wherever it split the path.
fn holds_tox(document: &mut Document<'_>) -> bool {
    let named = tox_name();
    if document
        .sections
        .iter()
        .any(|section| section.header.key.segments().starts_with(&named))
    {
        return true;
    }
    let mut held = false;
    common::sections::for_table_at(document, &named, |_| held = true);
    common::sections::for_names_under(document, &named, |_, _| held = true);
    held
}

pub fn reorder_inline_tables(document: &mut Document<'_>) {
    tox_rules::reorder_inline_tables_with_prefix(document, TOOL_TOX);
}
