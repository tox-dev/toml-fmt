use common::arrays::{dedupe_strings_in, sort_names_in};
use common::sections::{self, InlineSchema};
use toml_doc::Document;

// Sub-table prefixes are appended dynamically because some (group.<name>.*) need per-instance entries to control
// inner key order.
const TOP_LEVEL_ORDER: &[&str] = &[
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
const SOURCE_KEY_ORDER: &[&str] = &["name", "url", "priority", "links", "indexed", "default", "secondary"];

const BUILD_KEY_ORDER: &[&str] = &["script", "generate-setup-file"];

const GROUP_KEY_ORDER: &[&str] = &["optional", "include-groups", "dependencies"];

// Within [tool.poetry.dependencies] (and the per-group equivalents), `python` is the interpreter constraint and
// conventionally leads; everything else sorts.
const DEPENDENCIES_KEY_ORDER: &[&str] = &["python"];

pub fn fix(document: &mut Document<'_>) {
    fix_root(document);
    fix_expanded_sub_tables(document);
    fix_source(document);
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

pub const INLINE_TABLE_SCHEMAS: &[InlineSchema<'static>] = &[
    InlineSchema {
        discriminator: "priority",
        key_order: SOURCE_INLINE_KEYS,
    },
    InlineSchema {
        discriminator: "links",
        key_order: SOURCE_INLINE_KEYS,
    },
    InlineSchema {
        discriminator: "indexed",
        key_order: SOURCE_INLINE_KEYS,
    },
    InlineSchema {
        discriminator: "secondary",
        key_order: SOURCE_INLINE_KEYS,
    },
    InlineSchema {
        discriminator: "git",
        key_order: GIT_DEP_INLINE_KEYS,
    },
    InlineSchema {
        discriminator: "path",
        key_order: PATH_DEP_INLINE_KEYS,
    },
    InlineSchema {
        discriminator: "file",
        key_order: FILE_DEP_INLINE_KEYS,
    },
];

pub fn reorder_inline_tables(document: &mut Document<'_>) {
    let name = ["tool", "poetry"].map(str::to_owned);
    sections::reorder_inline_tables(document, &name, INLINE_TABLE_SCHEMAS);
}

fn fix_root(document: &mut Document<'_>) {
    let path = sections::parse_name("tool.poetry");
    let mut held: Vec<Vec<String>> = Vec::new();
    sections::for_keys_under(document, &path, |key, value| match key {
        // a keyword is free text, while a classifier is one of a fixed set of strings: two that
        // differ in case are an invalid spelling beside a valid one rather than one claim twice
        "keywords" => {
            dedupe_strings_in(value, &|text| text.to_lowercase());
            sort_names_in(value);
        }
        "classifiers" => {
            dedupe_strings_in(value, &ToOwned::to_owned);
            sort_names_in(value);
        }
        "exclude" => {
            sort_names_in(value);
        }
        _ => {}
    });
    // the name a rule reads here is the key's own segments, so a group the file quoted because it
    // holds a dot is the one name it wrote rather than the two a dotted path would read
    sections::for_names_under(document, &path, |tail, _| {
        held.push(tail.to_vec());
    });
    for tail in &held {
        if is_sort_value_array(tail) {
            sections::for_value_at(document, &[path.clone(), tail.clone()].concat(), |value| {
                sort_names_in(value);
            });
        }
    }

    let order = build_root_key_order_under(document, &path);
    let order_refs: Vec<&str> = order.iter().map(String::as_str).collect();
    sections::reorder_under(document, &path, &order_refs);
}

/// Extras lists, include-groups, and per-dependency extras are name sets, so they sort.
fn is_sort_value_array(key: &[String]) -> bool {
    match key {
        [head, _] if head == "extras" => true,
        // the group's name is the one segment after `group`, whatever it holds
        [head, _, tail @ ..] if head == "group" => match tail {
            [one] if one == "include-groups" => true,
            [one, rest @ ..] if one == "dependencies" => is_dep_extras(rest),
            _ => false,
        },
        [head, rest @ ..]
            if matches!(
                head.as_str(),
                "dependencies" | "dev-dependencies" | "requires-plugins" | "build-constraints"
            ) =>
        {
            is_dep_extras(rest)
        }
        _ => false,
    }
}

/// `<package>.extras`, whatever the package is called.
fn is_dep_extras(key: &[String]) -> bool {
    matches!(key, [_, tail] if tail == "extras")
}

pub fn build_root_key_order(entries: &[toml_doc::Entry<'_>]) -> Vec<String> {
    root_key_order(&collect_dotted_segment(entries, "group"))
}

/// The same, read from the whole table however the file split its path.
fn build_root_key_order_under(document: &mut Document<'_>, path: &[String]) -> Vec<String> {
    let mut groups: Vec<String> = Vec::new();
    sections::for_names_under(document, path, |tail, _| {
        if let [head, name, ..] = tail {
            if head == "group" && !groups.contains(name) {
                groups.push(name.clone());
            }
        }
    });
    root_key_order(&groups)
}

/// The `[tool.poetry]` key order; `group_names` are the dependency groups whose keys get their own slots.
pub fn root_key_order(group_names: &[String]) -> Vec<String> {
    let mut order: Vec<String> = TOP_LEVEL_ORDER.iter().map(|name| (*name).to_string()).collect();

    // `build` may appear as a scalar (build = "build.py"), an inline-table key, or via dotted sub-keys
    // (build.script, build.generate-setup-file); the `build` prefix entry catches every form.
    order.push(String::from("build.script"));
    order.push(String::from("build.generate-setup-file"));
    order.push(String::from("build"));

    order.push(String::from("dependencies.python"));
    order.push(String::from("dependencies"));
    order.push(String::from("dev-dependencies"));

    for group in group_names {
        // the name is spelled the way a dispatch name spells it, so both sides match
        let group = sections::quoted_segment(group);
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

/// The dependency groups a file happens to define, whose keys each get their own slot in the order.
fn collect_dotted_segment(entries: &[toml_doc::Entry<'_>], prefix: &str) -> Vec<String> {
    sections::keys_below(entries, &prefix.split('.').collect::<Vec<&str>>())
}

fn fix_expanded_sub_tables(document: &mut Document<'_>) {
    // In `table_format = "long"` mode sub-tables stay as their own headers, so normalize each one here.
    fix_expanded_dependencies(document, &["tool", "poetry", "dependencies"].map(str::to_owned));
    fix_expanded_dependencies(document, &["tool", "poetry", "dev-dependencies"].map(str::to_owned));
    fix_expanded_dependencies(document, &["tool", "poetry", "requires-plugins"].map(str::to_owned));
    fix_expanded_dependencies(document, &["tool", "poetry", "build-constraints"].map(str::to_owned));
    sections::sort_names_under(document, "tool.poetry.extras");
    fix_expanded_alpha(document, "tool.poetry.scripts");
    fix_expanded_alpha(document, "tool.poetry.urls");
    fix_expanded_plugins(document);
    fix_expanded_groups(document);
    fix_expanded_build(document);
}

fn fix_expanded_dependencies(document: &mut Document<'_>, table_key: &[String]) {
    let Some(section) = sections::first_of(document, table_key) else {
        return;
    };
    sections::reorder_keys(&mut section.entries, DEPENDENCIES_KEY_ORDER);
}

fn fix_expanded_alpha(document: &mut Document<'_>, table_key: &str) {
    let Some(section) = sections::first(document, table_key) else {
        return;
    };
    sections::reorder_keys(&mut section.entries, &[]);
}

fn fix_expanded_plugins(document: &mut Document<'_>) {
    if let Some(section) = sections::first(document, "tool.poetry.plugins") {
        sections::reorder_keys(&mut section.entries, &[]);
    }
    for section in &mut document.sections {
        if is_below(&section.header.key.segments(), &["tool", "poetry", "plugins"]) {
            sections::reorder_keys(&mut section.entries, &[]);
        }
    }
}

fn fix_expanded_groups(document: &mut Document<'_>) {
    for group in collect_header_segments(document, "tool.poetry.group.") {
        let key = ["tool", "poetry", "group", &group].map(str::to_owned);
        if let Some(section) = sections::first_of(document, &key) {
            sections::for_entries(section, |key, value| {
                if key == "include-groups" {
                    sort_names_in(value);
                }
            });
            sections::reorder_keys(&mut section.entries, GROUP_KEY_ORDER);
        }
        fix_expanded_dependencies(
            document,
            &["tool", "poetry", "group", &group, "dependencies"].map(str::to_owned),
        );
    }
}

fn fix_expanded_build(document: &mut Document<'_>) {
    let Some(section) = sections::first(document, "tool.poetry.build") else {
        return;
    };
    sections::reorder_keys(&mut section.entries, BUILD_KEY_ORDER);
}

fn fix_source(document: &mut Document<'_>) {
    for section in sections::named(document, "tool.poetry.source") {
        sections::reorder_keys(&mut section.entries, SOURCE_KEY_ORDER);
    }
    // a source folded into its parent is written as a table inside an array, and it is one source
    // whichever way the file holds it
    sections::reorder_array_tables_at(document, &sections::parse_name("tool.poetry.source"), SOURCE_KEY_ORDER);
}

fn collect_header_segments(document: &Document<'_>, prefix: &str) -> Vec<String> {
    sections::headers_below(
        document,
        &prefix.trim_end_matches('.').split('.').collect::<Vec<&str>>(),
    )
}

/// Whether the header names a table under `wanted`, compared segment by segment so a quoted name
/// holding a dot counts as the one segment it is.
fn is_below(segments: &[String], wanted: &[&str]) -> bool {
    segments.len() > wanted.len() && segments.iter().zip(wanted).all(|(held, want)| held == want)
}
