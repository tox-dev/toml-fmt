use common::array::{dedupe_strings, sort_strings};
use common::table::{for_entries, reorder_inline_table_keys, reorder_table_keys, InlineTableSchema, Tables};
use lexical_sort::natural_lexical_cmp;
use tombi_syntax::SyntaxNode;

// Sub-table prefixes are appended dynamically because some (group.<name>.*) need per-instance entries to control
// inner key order.
const TOP_LEVEL_ORDER: &[&str] = &[
    "",
    "name",
    "version",
    "description",
    "package-mode",
    "license",
    "authors",
    "maintainers",
    "readme",
    "homepage",
    "repository",
    "documentation",
    "keywords",
    "classifiers",
    "packages",
    "include",
    "exclude",
];

// Deprecated source keys (default, secondary) sort last so reordering never promotes them above current keys.
const SOURCE_KEY_ORDER: &[&str] = &[
    "",
    "name",
    "url",
    "priority",
    "links",
    "indexed",
    "default",
    "secondary",
];

const BUILD_KEY_ORDER: &[&str] = &["", "script", "generate-setup-file"];

const GROUP_KEY_ORDER: &[&str] = &["", "optional", "include-groups", "dependencies"];

// Within [tool.poetry.dependencies] (and the per-group equivalents), `python` is the interpreter constraint and
// conventionally leads; everything else sorts.
const DEPENDENCIES_KEY_ORDER: &[&str] = &["", "python"];

pub fn fix(tables: &mut Tables) {
    fix_root(tables);
    fix_expanded_sub_tables(tables);
    fix_source(tables);
}

// Inline-table key order for specs collapsed to inline form. Discriminators are Poetry-specific
// (priority/links/indexed/secondary appear only on sources; git/path/file only on dependencies) to avoid colliding
// with inline tables in other `[tool.*]` sections that share generic keys like `name` or `url`.
const SOURCE_INLINE_KEYS: &[&str] = &["name", "url", "priority", "links", "indexed", "default", "secondary"];

const GIT_DEP_INLINE_KEYS: &[&str] = &[
    "git",
    "branch",
    "tag",
    "rev",
    "subdirectory",
    "python",
    "platform",
    "markers",
    "allow-prereleases",
    "allows-prereleases",
    "optional",
    "extras",
    "develop",
];

const PATH_DEP_INLINE_KEYS: &[&str] = &[
    "path",
    "develop",
    "subdirectory",
    "python",
    "platform",
    "markers",
    "optional",
    "extras",
];

const FILE_DEP_INLINE_KEYS: &[&str] = &[
    "file",
    "subdirectory",
    "python",
    "platform",
    "markers",
    "optional",
    "extras",
];

pub const INLINE_TABLE_SCHEMAS: &[InlineTableSchema] = &[
    InlineTableSchema {
        discriminator: "priority",
        key_order: SOURCE_INLINE_KEYS,
    },
    InlineTableSchema {
        discriminator: "links",
        key_order: SOURCE_INLINE_KEYS,
    },
    InlineTableSchema {
        discriminator: "indexed",
        key_order: SOURCE_INLINE_KEYS,
    },
    InlineTableSchema {
        discriminator: "secondary",
        key_order: SOURCE_INLINE_KEYS,
    },
    InlineTableSchema {
        discriminator: "git",
        key_order: GIT_DEP_INLINE_KEYS,
    },
    InlineTableSchema {
        discriminator: "path",
        key_order: PATH_DEP_INLINE_KEYS,
    },
    InlineTableSchema {
        discriminator: "file",
        key_order: FILE_DEP_INLINE_KEYS,
    },
];

pub fn reorder_inline_tables(root_ast: &SyntaxNode) {
    reorder_inline_table_keys(root_ast, INLINE_TABLE_SCHEMAS);
}

fn fix_root(tables: &mut Tables) {
    let Some(elements) = tables.get("tool.poetry") else {
        return;
    };
    let table = &mut elements.first().unwrap().borrow_mut();

    for_entries(table, &mut |key, entry| {
        let k = key.as_str();
        match k {
            "keywords" | "classifiers" => {
                dedupe_strings(entry, |s| s.to_lowercase());
                sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
            }
            "exclude" => {
                sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
            }
            _ => {
                if is_sort_value_array(k) {
                    sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| {
                        natural_lexical_cmp(lhs, rhs)
                    });
                }
            }
        }
    });

    let order = build_root_key_order(table);
    let order_refs: Vec<&str> = order.iter().map(String::as_str).collect();
    reorder_table_keys(table, &order_refs);
}

/// Extras lists, include-groups, and per-dependency extras are name sets, so they sort.
fn is_sort_value_array(key: &str) -> bool {
    if let Some(rest) = key.strip_prefix("extras.") {
        return !rest.contains('.');
    }
    if let Some(rest) = key.strip_prefix("group.") {
        if let Some((_group, tail)) = split_first_segment(rest) {
            if tail == "include-groups" {
                return true;
            }
            if let Some(inner) = tail.strip_prefix("dependencies.") {
                return is_dep_extras(inner);
            }
        }
        return false;
    }
    for dep_table in [
        "dependencies",
        "dev-dependencies",
        "requires-plugins",
        "build-constraints",
    ] {
        if let Some(rest) = key.strip_prefix(dep_table) {
            if let Some(inner) = rest.strip_prefix('.') {
                return is_dep_extras(inner);
            }
        }
    }
    false
}

fn is_dep_extras(s: &str) -> bool {
    let Some((_pkg, tail)) = split_first_segment(s) else {
        return false;
    };
    tail == "extras"
}

fn split_first_segment(s: &str) -> Option<(&str, &str)> {
    s.find('.').map(|i| (&s[..i], &s[i + 1..]))
}

fn build_root_key_order(table: &[tombi_syntax::SyntaxElement]) -> Vec<String> {
    let mut order: Vec<String> = TOP_LEVEL_ORDER.iter().map(|s| (*s).to_string()).collect();

    // `build` may appear as a scalar (build = "build.py"), an inline-table key, or via dotted sub-keys
    // (build.script, build.generate-setup-file); the `build` prefix entry catches every form.
    order.push(String::from("build.script"));
    order.push(String::from("build.generate-setup-file"));
    order.push(String::from("build"));

    order.push(String::from("dependencies.python"));
    order.push(String::from("dependencies"));
    order.push(String::from("dev-dependencies"));

    let group_names = collect_dotted_segment(table, "group");
    for group in &group_names {
        order.push(format!("group.{group}.optional"));
        order.push(format!("group.{group}.include-groups"));
        order.push(format!("group.{group}.dependencies.python"));
        order.push(format!("group.{group}.dependencies"));
    }
    order.push(String::from("group"));

    order.push(String::from("extras"));
    order.push(String::from("scripts"));
    order.push(String::from("plugins"));
    order.push(String::from("urls"));
    order.push(String::from("source"));
    order.push(String::from("requires-poetry"));
    order.push(String::from("requires-plugins"));
    order.push(String::from("build-constraints"));

    order
}

fn collect_dotted_segment(table: &[tombi_syntax::SyntaxElement], prefix: &str) -> Vec<String> {
    use tombi_syntax::SyntaxKind::{KEYS, KEY_VALUE};
    let prefix_dot = format!("{prefix}.");
    let mut names: Vec<String> = Vec::new();
    for element in table.iter().filter(|e| e.kind() == KEY_VALUE) {
        let Some(kv) = element.as_node() else { continue };
        let Some(keys_node) = kv.children().find(|c| c.kind() == KEYS) else {
            continue;
        };
        let raw = keys_node.text().to_string().trim().to_string();
        if let Some(rest) = raw.strip_prefix(&prefix_dot) {
            let next = rest.split('.').next().unwrap_or(rest);
            let name = unquote(next).to_string();
            if !names.contains(&name) {
                names.push(name);
            }
        }
    }
    names
}

fn unquote(s: &str) -> &str {
    s.strip_prefix('"')
        .and_then(|x| x.strip_suffix('"'))
        .or_else(|| s.strip_prefix('\'').and_then(|x| x.strip_suffix('\'')))
        .unwrap_or(s)
}

fn fix_expanded_sub_tables(tables: &mut Tables) {
    // In `table_format = "long"` mode sub-tables stay as their own headers, so normalize each one here.
    fix_expanded_dependencies(tables, "tool.poetry.dependencies");
    fix_expanded_dependencies(tables, "tool.poetry.dev-dependencies");
    fix_expanded_dependencies(tables, "tool.poetry.requires-plugins");
    fix_expanded_dependencies(tables, "tool.poetry.build-constraints");
    fix_expanded_extras(tables);
    fix_expanded_alpha(tables, "tool.poetry.scripts");
    fix_expanded_alpha(tables, "tool.poetry.urls");
    fix_expanded_plugins(tables);
    fix_expanded_groups(tables);
    fix_expanded_build(tables);
}

fn fix_expanded_dependencies(tables: &mut Tables, table_key: &str) {
    let Some(elements) = tables.get(table_key) else {
        return;
    };
    let table = &mut elements.first().unwrap().borrow_mut();
    reorder_table_keys(table, DEPENDENCIES_KEY_ORDER);
}

fn fix_expanded_extras(tables: &mut Tables) {
    let Some(elements) = tables.get("tool.poetry.extras") else {
        return;
    };
    let table = &mut elements.first().unwrap().borrow_mut();
    for_entries(table, &mut |_key, entry| {
        sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
    });
    reorder_table_keys(table, &[""]);
}

fn fix_expanded_alpha(tables: &mut Tables, table_key: &str) {
    let Some(elements) = tables.get(table_key) else {
        return;
    };
    let table = &mut elements.first().unwrap().borrow_mut();
    reorder_table_keys(table, &[""]);
}

fn fix_expanded_plugins(tables: &mut Tables) {
    if let Some(elements) = tables.get("tool.poetry.plugins") {
        let table = &mut elements.first().unwrap().borrow_mut();
        reorder_table_keys(table, &[""]);
    }
    for group in collect_header_segments(tables, "tool.poetry.plugins.") {
        let key = format!("tool.poetry.plugins.{group}");
        if let Some(elements) = tables.get(&key) {
            let table = &mut elements.first().unwrap().borrow_mut();
            reorder_table_keys(table, &[""]);
        }
    }
}

fn fix_expanded_groups(tables: &mut Tables) {
    for group in collect_header_segments(tables, "tool.poetry.group.") {
        let key = format!("tool.poetry.group.{group}");
        if let Some(elements) = tables.get(&key) {
            let table = &mut elements.first().unwrap().borrow_mut();
            for_entries(table, &mut |key, entry| {
                if key.as_str() == "include-groups" {
                    sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| {
                        natural_lexical_cmp(lhs, rhs)
                    });
                }
            });
            reorder_table_keys(table, GROUP_KEY_ORDER);
        }
        fix_expanded_dependencies(tables, &format!("tool.poetry.group.{group}.dependencies"));
    }
}

fn fix_expanded_build(tables: &mut Tables) {
    let Some(elements) = tables.get("tool.poetry.build") else {
        return;
    };
    let table = &mut elements.first().unwrap().borrow_mut();
    reorder_table_keys(table, BUILD_KEY_ORDER);
}

fn fix_source(tables: &mut Tables) {
    let Some(source_entries) = tables.get("tool.poetry.source") else {
        return;
    };
    for entry_ref in source_entries {
        let table = &mut entry_ref.borrow_mut();
        reorder_table_keys(table, SOURCE_KEY_ORDER);
    }
}

fn collect_header_segments(tables: &Tables, prefix: &str) -> Vec<String> {
    let mut names: Vec<String> = tables
        .header_to_pos
        .keys()
        .filter_map(|h| h.strip_prefix(prefix))
        .map(|rest| rest.split('.').next().unwrap_or(rest).to_string())
        .collect();
    names.sort();
    names.dedup();
    names
}
