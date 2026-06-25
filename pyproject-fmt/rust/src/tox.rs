use common::table::Tables;
use tombi_syntax::SyntaxNode;

// Delegates to the shared tox-toml-fmt rules; the `"tool.tox"` prefix resolves tables under that namespace instead of
// the root, so `[tool.tox]` in pyproject.toml formats identically to a standalone tox.toml.
const TOOL_TOX: &str = "tool.tox";

pub fn fix(tables: &mut Tables) {
    if tables.get(TOOL_TOX).is_none() {
        return;
    }
    _tox_toml_fmt::global::normalize_aliases_with_prefix(tables, TOOL_TOX);
    _tox_toml_fmt::global::fix_root_with_prefix(tables, TOOL_TOX);
    _tox_toml_fmt::global::fix_envs_with_prefix(tables, TOOL_TOX);
    // pin_envs is a tox-toml-fmt Setting with no pyproject-fmt CLI surface, so pass none, matching what standalone
    // tox.toml users get without a pin_envs override.
    _tox_toml_fmt::global::sort_env_list_with_prefix(tables, &[], TOOL_TOX);
}

pub fn reorder_inline_tables(root_ast: &SyntaxNode) {
    _tox_toml_fmt::global::reorder_inline_tables(root_ast);
}
