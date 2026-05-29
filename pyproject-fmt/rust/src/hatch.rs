use common::array::sort_strings;
use common::table::{for_entries, reorder_table_keys, Tables};
use lexical_sort::natural_lexical_cmp;
use tombi_syntax::SyntaxElement;

// Config lives in sub-tables, so KEY_ORDER works on the collapsed parent's dotted keys
// (version.source, build.exclude, envs.default.dependencies). Order within each group
// follows the hatch reference (https://hatch.pypa.io).
const KEY_ORDER: &[&str] = &[
    "",
    // === Version ===
    "version.source",
    "version.path",
    "version.pattern",
    "version.expression",
    "version.scheme",
    "version.validate-bump",
    "version.fallback-version",
    "version.raw-options",
    "version",
    // === Metadata ===
    "metadata.allow-direct-references",
    "metadata.allow-ambiguous-features",
    "metadata.hooks",
    "metadata",
    // === Build ===
    "build.dev-mode-dirs",
    "build.dev-mode-exact",
    "build.directory",
    "build.sources",
    "build.packages",
    "build.include",
    "build.exclude",
    "build.force-include",
    "build.artifacts",
    "build.ignore-vcs",
    "build.skip-excluded-dirs",
    "build.reproducible",
    "build.hooks",
    "build.targets.wheel.packages",
    "build.targets.wheel.include",
    "build.targets.wheel.exclude",
    "build.targets.wheel.force-include",
    "build.targets.wheel.artifacts",
    "build.targets.wheel.hooks",
    "build.targets.wheel.shared-data",
    "build.targets.wheel.extra-metadata",
    "build.targets.wheel.bypass-selection",
    "build.targets.wheel.zip-safe",
    "build.targets.wheel.core-metadata-version",
    "build.targets.sdist.include",
    "build.targets.sdist.exclude",
    "build.targets.sdist.force-include",
    "build.targets.sdist.support-legacy",
    "build.targets.sdist.strict-naming",
    "build.targets.sdist.core-metadata-version",
    "build.targets.app",
    "build.targets.custom",
    "build.targets",
    "build",
    // === Publish ===
    "publish.index.disable",
    "publish.index.repos",
    "publish.index",
    "publish",
    // === Workspace ===
    "workspace.members",
    "workspace.exclude",
    "workspace",
    // `envs` intentionally NOT here — per-env entries are inserted dynamically by
    // build_key_order with each environment's full inner key list, then a bare `envs`
    // catch-all is appended last so any envs.* not in the canonical inner-key list still
    // ends up in the envs block.
];

const SORT_ARRAYS_EXACT: &[&str] = &[
    "build.include",
    "build.exclude",
    "build.force-include",
    "build.artifacts",
    "build.packages",
    "build.sources",
    "build.dev-mode-dirs",
    "build.targets.wheel.include",
    "build.targets.wheel.exclude",
    "build.targets.wheel.force-include",
    "build.targets.wheel.artifacts",
    "build.targets.wheel.packages",
    "build.targets.sdist.include",
    "build.targets.sdist.exclude",
    "build.targets.sdist.force-include",
    "workspace.members",
    "workspace.exclude",
];

pub fn fix(tables: &mut Tables) {
    fix_root(tables);
    fix_env_tables(tables);
    fix_overrides_aot(tables);
}

fn fix_root(tables: &mut Tables) {
    let Some(elements) = tables.get("tool.hatch") else {
        return;
    };
    let order = {
        let table_elements: &Vec<SyntaxElement> = &elements.first().unwrap().borrow();
        build_key_order(table_elements)
    };
    let table = &mut elements.first().unwrap().borrow_mut();
    for_entries(table, &mut |key, entry| {
        let k = key.as_str();
        if SORT_ARRAYS_EXACT.contains(&k) || is_dynamic_sort_array(k) {
            sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
        }
    });
    let refs: Vec<&str> = order.iter().map(String::as_str).collect();
    reorder_table_keys(table, &refs);
}

/// Build a per-input KEY_ORDER that interpolates per-env entries so each environment
/// keeps a consistent inner order (`type` → `python` → `dependencies` → `scripts` → …).
fn build_key_order(table: &[SyntaxElement]) -> Vec<String> {
    let mut order: Vec<String> = KEY_ORDER.iter().map(|s| (*s).to_string()).collect();
    for env in collect_dynamic_segments(table, "envs") {
        let p = format!("envs.{env}");
        for k in [
            "type",
            "template",
            "detached",
            "description",
            "platforms",
            "python",
            "path",
            "installer",
            "skip-install",
            "system-packages",
            "dev-mode",
            "features",
            "dependencies",
            "extra-dependencies",
            "extra-args",
            "pre-install-commands",
            "post-install-commands",
            "env-include",
            "env-exclude",
            "env-vars",
            "scripts",
            "matrix",
            "matrix-name-format",
            "overrides",
        ] {
            order.push(format!("{p}.{k}"));
        }
        order.push(p);
    }
    // Bare catch-all for any envs.* keys not in the canonical inner-key list.
    order.push(String::from("envs"));
    order
}

fn collect_dynamic_segments(table: &[SyntaxElement], prefix: &str) -> Vec<String> {
    use tombi_syntax::SyntaxKind::{KEYS, KEY_VALUE};
    let prefix_dot = format!("{prefix}.");
    let mut names: Vec<String> = Vec::new();
    let raw_keys = table
        .iter()
        .filter(|e| e.kind() == KEY_VALUE)
        .filter_map(|element| element.as_node())
        .filter_map(|kv| kv.children().find(|c| c.kind() == KEYS))
        .map(|keys| keys.text().to_string().trim().to_string());
    for raw in raw_keys {
        if let Some(rest) = raw.strip_prefix(&prefix_dot) {
            let seg = rest.split('.').next().unwrap_or(rest);
            let name = seg.trim_matches('"').trim_matches('\'').to_string();
            if !names.contains(&name) && !name.is_empty() {
                names.push(name);
            }
        }
    }
    names
}

fn is_dynamic_sort_array(key: &str) -> bool {
    // envs.<n>.dependencies, .extra-dependencies, .features, .pre-install-commands,
    // .post-install-commands, .platforms — sortable (set semantics in hatch).
    let Some(rest) = key.strip_prefix("envs.") else {
        return false;
    };
    let Some((_env, tail)) = rest.split_once('.') else {
        return false;
    };
    matches!(
        tail,
        "dependencies"
            | "extra-dependencies"
            | "features"
            | "platforms"
            | "env-include"
            | "env-exclude"
            | "pre-install-commands"
            | "post-install-commands"
    )
}

fn fix_env_tables(tables: &mut Tables) {
    // Expanded form: [tool.hatch.envs.<name>] as own table — reorder keys, sort arrays.
    let env_names = collect_header_segments(tables, "tool.hatch.envs.");
    for env in env_names {
        let key = format!("tool.hatch.envs.{env}");
        if let Some(elements) = tables.get(&key) {
            let table = &mut elements.first().unwrap().borrow_mut();
            for_entries(table, &mut |k, entry| {
                if matches!(
                    k.as_str(),
                    "dependencies"
                        | "extra-dependencies"
                        | "features"
                        | "platforms"
                        | "env-include"
                        | "env-exclude"
                        | "pre-install-commands"
                        | "post-install-commands"
                ) {
                    sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| {
                        natural_lexical_cmp(lhs, rhs)
                    });
                }
            });
            reorder_table_keys(
                table,
                &[
                    "",
                    "type",
                    "template",
                    "detached",
                    "description",
                    "platforms",
                    "python",
                    "path",
                    "installer",
                    "skip-install",
                    "system-packages",
                    "dev-mode",
                    "features",
                    "dependencies",
                    "extra-dependencies",
                    "extra-args",
                    "pre-install-commands",
                    "post-install-commands",
                    "env-include",
                    "env-exclude",
                    "env-vars",
                    "scripts",
                    "matrix",
                    "matrix-name-format",
                    "overrides",
                ],
            );
        }
        // scripts and env-vars sub-tables: alphabetize keys
        for sub in ["scripts", "env-vars"] {
            let k = format!("tool.hatch.envs.{env}.{sub}");
            if let Some(elements) = tables.get(&k) {
                let table = &mut elements.first().unwrap().borrow_mut();
                reorder_table_keys(table, &[""]);
            }
        }
    }
}

fn fix_overrides_aot(tables: &mut Tables) {
    // [[tool.hatch.envs.<n>.matrix]] and overrides.* are AoT — preserve order, but reorder
    // inner keys.
    for env in collect_header_segments(tables, "tool.hatch.envs.") {
        let matrix_key = format!("tool.hatch.envs.{env}.matrix");
        if let Some(entries) = tables.get(&matrix_key) {
            for entry_ref in entries {
                let table = &mut entry_ref.borrow_mut();
                reorder_table_keys(table, &[""]);
            }
        }
    }
}

fn collect_header_segments(tables: &Tables, prefix: &str) -> Vec<String> {
    let mut names: Vec<String> = tables
        .header_to_pos
        .keys()
        .filter_map(|h| h.strip_prefix(prefix))
        .map(|rest| rest.split('.').next().unwrap_or(rest).trim_matches('"').to_string())
        .collect();
    names.sort();
    names.dedup();
    names
}
