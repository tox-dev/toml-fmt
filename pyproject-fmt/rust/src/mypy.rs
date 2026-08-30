use common::arrays::sort_names_in;
use common::sections::{self, InlineSchema};
use toml_doc::{Document, Value};

// Grouped to match the section structure of the official mypy config reference.
pub const KEY_ORDER: &[&str] = &[
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
    "python_version",
    "platform",
    "always_true",
    "always_false",
    "disallow_any_unimported",
    "disallow_any_expr",
    "disallow_any_decorated",
    "disallow_any_explicit",
    "disallow_any_generics",
    "disallow_subclassing_any",
    "disallow_untyped_calls",
    "untyped_calls_exclude",
    "disallow_untyped_defs",
    "disallow_incomplete_defs",
    "check_untyped_defs",
    "disallow_untyped_decorators",
    "implicit_optional",
    "strict_optional",
    "warn_redundant_casts",
    "warn_unused_ignores",
    "warn_no_return",
    "warn_return_any",
    "warn_unreachable",
    "deprecated_calls_exclude",
    "report_deprecated_as_note",
    "ignore_errors",
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
    "show_error_context",
    "show_column_numbers",
    "show_error_end",
    "hide_error_codes",
    "show_error_code_links",
    "pretty",
    "color_output",
    "error_summary",
    "show_absolute_path",
    "incremental",
    "cache_dir",
    "sqlite_cache",
    "cache_fine_grained",
    "skip_version_check",
    "skip_cache_mtime_checks",
    "plugins",
    "pdb",
    "show_traceback",
    "raise_exceptions",
    "custom_typing_module",
    "custom_typeshed_dir",
    "warn_incomplete_stub",
    "native_parser",
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
    "junit_xml",
    "junit_format",
    "scripts_are_modules",
    "warn_unused_configs",
    "verbosity",
    "overrides",
];

// module is required, so it leads; the rest mirror the parent groupings, restricted to the per-module-overridable
// subset.
const OVERRIDES_KEY_ORDER: &[&str] = &[
    "module",
    "ignore_missing_imports",
    "follow_untyped_imports",
    "follow_imports",
    "follow_imports_for_stubs",
    "always_true",
    "always_false",
    "disallow_any_unimported",
    "disallow_any_expr",
    "disallow_any_decorated",
    "disallow_any_explicit",
    "disallow_any_generics",
    "disallow_subclassing_any",
    "disallow_untyped_calls",
    "disallow_untyped_defs",
    "disallow_incomplete_defs",
    "check_untyped_defs",
    "disallow_untyped_decorators",
    "implicit_optional",
    "strict_optional",
    "warn_unused_ignores",
    "warn_no_return",
    "warn_return_any",
    "warn_unreachable",
    "ignore_errors",
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

pub fn fix(document: &mut Document<'_>) {
    fix_expanded_overrides(document);
}

/// Whether what the name holds is a list of names, which sorts.
pub fn sorts(key: &str) -> bool {
    TOP_LEVEL_SORT_ARRAYS.contains(&key)
}

fn fix_expanded_overrides(document: &mut Document<'_>) {
    for section in sections::named(document, "tool.mypy.overrides") {
        sections::for_entries(section, |key, value| {
            if OVERRIDES_SORT_ARRAYS.contains(&key) {
                sort_names_in(value);
            }
        });
        sections::reorder_keys(&mut section.entries, OVERRIDES_KEY_ORDER);
    }
    // an override folded into its parent is written as a table inside an array, and it is one
    // override whichever way the file holds it
    sections::reorder_array_tables_at(
        document,
        &sections::parse_name("tool.mypy.overrides"),
        OVERRIDES_KEY_ORDER,
    );
}

// Discriminators avoid collisions: `disable_error_code` and `enable_error_code` are mypy-specific in pyproject.toml,
// while `module` alone could match unrelated inline tables. Several discriminators map to the same OVERRIDES_KEY_ORDER,
// so an entry with only `module` + `ignore_missing_imports` is still recognized.
pub const INLINE_TABLE_SCHEMAS: &[InlineSchema<'static>] = &[
    InlineSchema {
        discriminator: "disable_error_code",
        key_order: OVERRIDES_KEY_ORDER,
    },
    InlineSchema {
        discriminator: "enable_error_code",
        key_order: OVERRIDES_KEY_ORDER,
    },
    InlineSchema {
        discriminator: "ignore_missing_imports",
        key_order: OVERRIDES_KEY_ORDER,
    },
    InlineSchema {
        discriminator: "follow_untyped_imports",
        key_order: OVERRIDES_KEY_ORDER,
    },
    InlineSchema {
        discriminator: "ignore_errors",
        key_order: OVERRIDES_KEY_ORDER,
    },
    InlineSchema {
        discriminator: "warn_unused_ignores",
        key_order: OVERRIDES_KEY_ORDER,
    },
    InlineSchema {
        discriminator: "disallow_untyped_defs",
        key_order: OVERRIDES_KEY_ORDER,
    },
    InlineSchema {
        discriminator: "check_untyped_defs",
        key_order: OVERRIDES_KEY_ORDER,
    },
];

pub fn reorder_inline_tables(document: &mut Document<'_>) {
    let name = ["tool", "mypy"].map(str::to_owned);
    sections::reorder_inline_tables(document, &name, INLINE_TABLE_SCHEMAS);
    sort_arrays_inside_overrides(document);
}

/// A collapsed `[[tool.mypy.overrides]]` becomes `overrides = [ {...}, {...} ]`, which puts its
/// arrays inside a value rather than under a table, out of reach of the entry walk above.
fn sort_arrays_inside_overrides(document: &mut Document<'_>) {
    let path = ["tool", "mypy", "overrides"].map(str::to_owned);
    common::sections::for_value_at(document, &path, |value| {
        let Value::Array(array) = value else {
            return;
        };
        for member in &mut array.members {
            let Value::InlineTable(table) = &mut member.item else {
                continue;
            };
            for inner in &mut table.members {
                let name = common::sections::dispatch_name(&inner.item.key);
                if OVERRIDES_SORT_ARRAYS.contains(&name.as_str()) {
                    sort_names_in(&mut inner.item.value);
                }
            }
        }
    });
}
