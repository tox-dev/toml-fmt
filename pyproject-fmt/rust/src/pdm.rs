use common::arrays::sort_names_in;
use common::sections;
use toml_doc::Document;

// Sub-tables collapse to dotted keys (version.source, build.includes, etc.).
pub const KEY_ORDER: &[&str] = &[
    "distribution",
    "package-type",
    "plugins",
    "resolution.respect-source-order",
    "resolution.allow-prereleases",
    "resolution.excludes",
    "resolution.overrides",
    "resolution",
    "version.source",
    "version.path",
    "version.getter",
    "version.write_to",
    "version.write_template",
    "version.tag_regex",
    "version.tag_filter",
    "version.fallback_version",
    "version.version_format",
    "version",
    "build.includes",
    "build.excludes",
    "build.source-includes",
    "build.package-dir",
    "build.is-purelib",
    "build.run-setuptools",
    "build.custom-hook",
    "build.editable-backend",
    "build",
    "scripts",
    "source",
    "dev-dependencies",
    "publish.repository",
    "publish.username",
    "publish.password",
    "publish.ca_certs",
    "publish.verify_ssl",
    "publish",
    "options.install",
    "options.lock",
    "options.update",
    "options.add",
    "options.remove",
    "options.list",
    "options.sync",
    "options.run",
    "options",
];

const SORT_ARRAYS_EXACT: &[&str] = &[
    "plugins",
    "build.includes",
    "build.excludes",
    "build.source-includes",
    "resolution.excludes",
];

pub fn fix(document: &mut Document<'_>) {
    fix_expanded_scripts(document);
    sections::sort_names_under(document, "tool.pdm.dev-dependencies");
    fix_source_aot(document);
}

/// Whether what the name holds is a list of names, which sorts.
pub fn sorts(key: &str) -> bool {
    SORT_ARRAYS_EXACT.contains(&key) || is_dev_deps_value(key)
}

fn is_dev_deps_value(key: &str) -> bool {
    if let Some(rest) = key.strip_prefix("dev-dependencies.") {
        return !rest.is_empty();
    }
    false
}

fn fix_expanded_scripts(document: &mut Document<'_>) {
    if let Some(section) = sections::first(document, "tool.pdm.scripts") {
        sections::reorder_keys(&mut section.entries, &[]);
    }
}

const SOURCE_KEY_ORDER: &[&str] = &[
    "name",
    "url",
    "type",
    "verify_ssl",
    "include_packages",
    "exclude_packages",
];

fn fix_source_aot(document: &mut Document<'_>) {
    sections::for_array_elements(document, &source_name(), SOURCE_KEY_ORDER, &mut |key, value| {
        if matches!(key, "include_packages" | "exclude_packages") {
            sort_names_in(value);
        }
    });
}

fn source_name() -> Vec<String> {
    ["tool", "pdm", "source"].map(str::to_owned).to_vec()
}
