use common::table::Tables;
use tombi_syntax::SyntaxNode;

// Thin wrapper around the shared tox-toml-fmt rules — anything the standalone tox.toml
// formatter knows (key aliases, root key order, per-env key order, requirement
// normalization, env list sorting, inline-table reordering) applies identically to
// `[tool.tox]` in pyproject.toml. We just pass the `"tool.tox"` prefix so the shared
// functions look up tables under that namespace instead of at the root.
const TOOL_TOX: &str = "tool.tox";

pub fn fix(tables: &mut Tables) {
    if tables.get(TOOL_TOX).is_none() {
        // Skip the work (and the env-table scans) when the project doesn't use tox.
        return;
    }
    _tox_toml_fmt::global::normalize_aliases_with_prefix(tables, TOOL_TOX);
    _tox_toml_fmt::global::fix_root_with_prefix(tables, TOOL_TOX);
    _tox_toml_fmt::global::fix_envs_with_prefix(tables, TOOL_TOX);
    // `pin_envs` is a tox-toml-fmt Setting that doesn't surface in pyproject-fmt's CLI
    // (matching what `[tool.tox]` users get from the standalone formatter without a
    // pin_envs override).
    _tox_toml_fmt::global::sort_env_list_with_prefix(tables, &[], TOOL_TOX);
}

pub fn reorder_inline_tables(root_ast: &SyntaxNode) {
    _tox_toml_fmt::global::reorder_inline_tables(root_ast);
}
