use common::array::sort_strings;
use common::table::{for_entries, reorder_inline_table_keys, reorder_table_keys, InlineTableSchema, Tables};
use lexical_sort::natural_lexical_cmp;
use tombi_syntax::SyntaxKind::{ARRAY, INLINE_TABLE, KEYS, KEY_VALUE};
use tombi_syntax::SyntaxNode;

// Grouped to match the section structure of the official mypy config reference.
const KEY_ORDER: &[&str] = &[
    "",
    // 1. Import discovery
    "mypy_path",
    "files",
    "modules",
    "packages",
    "exclude",
    "exclude_gitignore",
    "namespace_packages",
    "explicit_package_bases",
    "ignore_missing_imports",
    "follow_untyped_imports",
    "follow_imports",
    "follow_imports_for_stubs",
    "python_executable",
    "no_site_packages",
    "no_silence_site_packages",
    // 2. Platform configuration
    "python_version",
    "platform",
    "always_true",
    "always_false",
    // 3. Disallow dynamic typing
    "disallow_any_unimported",
    "disallow_any_expr",
    "disallow_any_decorated",
    "disallow_any_explicit",
    "disallow_any_generics",
    "disallow_subclassing_any",
    // 4. Untyped definitions and calls
    "disallow_untyped_calls",
    "untyped_calls_exclude",
    "disallow_untyped_defs",
    "disallow_incomplete_defs",
    "check_untyped_defs",
    "disallow_untyped_decorators",
    // 5. None and Optional handling
    "implicit_optional",
    "strict_optional",
    // 6. Configuring warnings
    "warn_redundant_casts",
    "warn_unused_ignores",
    "warn_no_return",
    "warn_return_any",
    "warn_unreachable",
    "deprecated_calls_exclude",
    "report_deprecated_as_note",
    // 7. Suppressing errors
    "ignore_errors",
    // 8. Miscellaneous strictness flags (strict meta-flag last)
    "allow_untyped_globals",
    "allow_redefinition",
    "allow_redefinition_new",
    "allow_redefinition_old",
    "local_partial_types",
    "disable_error_code",
    "enable_error_code",
    "extra_checks",
    "implicit_reexport",
    "strict_concatenate",
    "strict_equality",
    "strict_equality_for_none",
    "strict_bytes",
    "strict",
    // 9. Configuring error messages
    "show_error_context",
    "show_column_numbers",
    "show_error_end",
    "hide_error_codes",
    "show_error_code_links",
    "pretty",
    "color_output",
    "error_summary",
    "show_absolute_path",
    // 10. Incremental mode
    "incremental",
    "cache_dir",
    "sqlite_cache",
    "cache_fine_grained",
    "skip_version_check",
    "skip_cache_mtime_checks",
    // 11. Advanced options
    "plugins",
    "pdb",
    "show_traceback",
    "raise_exceptions",
    "custom_typing_module",
    "custom_typeshed_dir",
    "warn_incomplete_stub",
    "native_parser",
    // 12. Report generation
    "any_exprs_report",
    "cobertura_xml_report",
    "html_report",
    "linecount_report",
    "linecoverage_report",
    "lineprecision_report",
    "txt_report",
    "xml_report",
    "xslt_html_report",
    "xslt_txt_report",
    // 13. Miscellaneous
    "junit_xml",
    "junit_format",
    "scripts_are_modules",
    "warn_unused_configs",
    "verbosity",
    // 14. Overrides (AoT or inline-array of inline tables) — always last
    "overrides",
];

// module is required, so it leads; the rest mirror the parent groupings, restricted to
// the per-module-overridable subset.
const OVERRIDES_KEY_ORDER: &[&str] = &[
    "",
    "module",
    // Import behavior
    "ignore_missing_imports",
    "follow_untyped_imports",
    "follow_imports",
    "follow_imports_for_stubs",
    // Platform truthy/falsy markers
    "always_true",
    "always_false",
    // Disallow dynamic typing
    "disallow_any_unimported",
    "disallow_any_expr",
    "disallow_any_decorated",
    "disallow_any_explicit",
    "disallow_any_generics",
    "disallow_subclassing_any",
    // Untyped defs and calls
    "disallow_untyped_calls",
    "disallow_untyped_defs",
    "disallow_incomplete_defs",
    "check_untyped_defs",
    "disallow_untyped_decorators",
    // None and Optional
    "implicit_optional",
    "strict_optional",
    // Warnings
    "warn_unused_ignores",
    "warn_no_return",
    "warn_return_any",
    "warn_unreachable",
    // Suppression
    "ignore_errors",
    // Misc strictness
    "allow_untyped_globals",
    "allow_redefinition",
    "allow_redefinition_old",
    "local_partial_types",
    "disable_error_code",
    "enable_error_code",
    "extra_checks",
    "implicit_reexport",
    "strict_concatenate",
    "strict_equality",
    "strict_equality_for_none",
    "strict",
];

// Set-semantics arrays only; plugins and mypy_path are excluded as order-sensitive.
const TOP_LEVEL_SORT_ARRAYS: &[&str] = &[
    "files",
    "modules",
    "packages",
    "exclude",
    "always_true",
    "always_false",
    "untyped_calls_exclude",
    "deprecated_calls_exclude",
    "disable_error_code",
    "enable_error_code",
];

// module globs are sorted too, matching the existing project convention.
const OVERRIDES_SORT_ARRAYS: &[&str] = &[
    "module",
    "always_true",
    "always_false",
    "disable_error_code",
    "enable_error_code",
];

pub fn fix(tables: &mut Tables) {
    fix_root(tables);
    fix_expanded_overrides(tables);
}

fn fix_root(tables: &mut Tables) {
    let Some(elements) = tables.get("tool.mypy") else {
        return;
    };
    let table = &mut elements.first().unwrap().borrow_mut();

    for_entries(table, &mut |key, entry| {
        let k = key.as_str();
        if TOP_LEVEL_SORT_ARRAYS.contains(&k) {
            sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
        }
    });
    reorder_table_keys(table, KEY_ORDER);
}

fn fix_expanded_overrides(tables: &mut Tables) {
    let Some(entries) = tables.get("tool.mypy.overrides") else {
        return;
    };
    for entry_ref in entries {
        let table = &mut entry_ref.borrow_mut();
        for_entries(table, &mut |key, entry| {
            if OVERRIDES_SORT_ARRAYS.contains(&key.as_str()) {
                sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
            }
        });
        reorder_table_keys(table, OVERRIDES_KEY_ORDER);
    }
}

// Discriminator chosen to avoid collisions: `disable_error_code` and `enable_error_code`
// are mypy-specific in pyproject.toml; `module` alone would risk matching unrelated
// inline tables. Several discriminators map to the same OVERRIDES_KEY_ORDER so an entry
// with only `module` + `ignore_missing_imports` (for example) is still recognized.
pub const INLINE_TABLE_SCHEMAS: &[InlineTableSchema] = &[
    InlineTableSchema {
        discriminator: "disable_error_code",
        key_order: OVERRIDES_KEY_ORDER,
    },
    InlineTableSchema {
        discriminator: "enable_error_code",
        key_order: OVERRIDES_KEY_ORDER,
    },
    InlineTableSchema {
        discriminator: "ignore_missing_imports",
        key_order: OVERRIDES_KEY_ORDER,
    },
    InlineTableSchema {
        discriminator: "follow_untyped_imports",
        key_order: OVERRIDES_KEY_ORDER,
    },
    InlineTableSchema {
        discriminator: "ignore_errors",
        key_order: OVERRIDES_KEY_ORDER,
    },
    InlineTableSchema {
        discriminator: "warn_unused_ignores",
        key_order: OVERRIDES_KEY_ORDER,
    },
    InlineTableSchema {
        discriminator: "disallow_untyped_defs",
        key_order: OVERRIDES_KEY_ORDER,
    },
    InlineTableSchema {
        discriminator: "check_untyped_defs",
        key_order: OVERRIDES_KEY_ORDER,
    },
];

pub fn reorder_inline_tables(root_ast: &SyntaxNode) {
    reorder_inline_table_keys(root_ast, INLINE_TABLE_SCHEMAS);
    sort_arrays_inside_overrides(root_ast);
}

/// When `[[tool.mypy.overrides]]` is collapsed into `overrides = [ {...}, {...} ]`, the
/// arrays inside each inline entry (e.g. `disable_error_code = ["x", "y"]`) live inside
/// values, not table entries — `for_entries` on the parent table won't see them. Walk the
/// AST under the `overrides` key and sort the known array-of-string fields in-place.
fn sort_arrays_inside_overrides(root_ast: &SyntaxNode) {
    for kv in root_ast.descendants().filter(|n| n.kind() == KEY_VALUE) {
        let Some(keys) = kv.children().find(|c| c.kind() == KEYS) else {
            continue;
        };
        let key_text = keys.text().to_string();
        let key_trim = key_text.trim();
        // Match either `overrides = [...]` (within `[tool.mypy]`) or
        // `mypy.overrides = [...]` (when collapsed under tool).
        if !(key_trim == "overrides" || key_trim.ends_with(".overrides")) {
            continue;
        }
        let Some(array) = kv.children().find(|c| c.kind() == ARRAY) else {
            continue;
        };
        for inline in array.descendants().filter(|n| n.kind() == INLINE_TABLE) {
            // INLINE_TABLE children are wrapped in KEY_VALUE_WITH_COMMA_GROUP — use
            // descendants() to reach KEY_VALUE nodes regardless of nesting depth.
            for inner_kv in inline.descendants().filter(|n| n.kind() == KEY_VALUE) {
                let Some(inner_keys) = inner_kv.children().find(|c| c.kind() == KEYS) else {
                    continue;
                };
                let inner_key = inner_keys.text().to_string().trim().to_string();
                if !OVERRIDES_SORT_ARRAYS.contains(&inner_key.as_str()) {
                    continue;
                }
                if let Some(inner_array) = inner_kv.children().find(|c| c.kind() == ARRAY) {
                    sort_string_array_in_place(&inner_array);
                }
            }
        }
    }
}

fn sort_string_array_in_place(array: &SyntaxNode) {
    sort_strings::<String, _, _>(array, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
}
