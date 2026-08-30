use common::arrays::sort_names_in;
use common::sections;
use toml_doc::Document;

// Config lives in sub-tables, so KEY_ORDER targets the collapsed parent's dotted keys (version.source,
// build.exclude, envs.default.dependencies); group order follows the hatch reference (https://hatch.pypa.io).
pub const KEY_ORDER: &[&str] = &[
    "version.source",
    "version.path",
    "version.pattern",
    "version.expression",
    "version.scheme",
    "version.validate-bump",
    "version.fallback-version",
    "version.raw-options",
    "version",
    "metadata.allow-direct-references",
    "metadata.allow-ambiguous-features",
    "metadata.hooks",
    "metadata",
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
    "publish.index.disable",
    "publish.index.repos",
    "publish.index",
    "publish",
    "workspace.members",
    "workspace.exclude",
    "workspace",
    // `envs` intentionally NOT here: build_key_order inserts per-env entries dynamically with each environment's
    // full inner key list, then appends a bare `envs` catch-all last so any envs.* outside the canonical inner-key
    // list still lands in the envs block.
];

// hatch reads `include`, `exclude` and `artifacts` the way a gitignore is read, where a `!pattern`
// after a broader one takes back what it matched, so each of those keeps the order it was written
// in. What is left here selects files by name alone.
const SORT_ARRAYS_EXACT: &[&str] = &[
    "build.packages",
    "build.sources",
    "build.dev-mode-dirs",
    "build.targets.wheel.packages",
    "workspace.members",
];

pub fn fix(document: &mut Document<'_>) {
    fix_root(document);
    fix_env_tables(document);
}

fn fix_root(document: &mut Document<'_>) {
    let path = sections::parse_name("tool.hatch");
    let mut names: Vec<Vec<String>> = Vec::new();
    sections::for_names_under(document, &path, |tail, _| names.push(tail.to_vec()));
    let order = key_order_of(&names);
    // the name a rule reads here is the key's own segments, so an environment the file quoted
    // because it holds a dot is the one name it wrote
    for segments in &names {
        if SORT_ARRAYS_EXACT.contains(&segments.join(".").as_str()) || is_dynamic_sort_array(segments) {
            let named: Vec<String> = path.iter().chain(segments).cloned().collect();
            sections::for_value_at(document, &named, sort_names_in);
        }
    }
    let refs: Vec<&str> = order.iter().map(String::as_str).collect();
    let keep = keep_order_of(&names);
    let keep_refs: Vec<&str> = keep.iter().map(String::as_str).collect();
    sections::reorder_under_keeping(document, &path, &refs, &keep_refs);
}

/// hatch runs its hooks and applies its overrides in the order they are written, and reads a matrix
/// element in that order to build the names it generates, so the keys under each of those names
/// keep the order the file gave them.
pub fn keep_order(entries: &[toml_doc::Entry<'_>]) -> Vec<String> {
    keep_order_of(&segments_of(entries))
}

/// The same, read from the names the table holds however the file wrote them.
fn keep_order_of(names: &[Vec<String>]) -> Vec<String> {
    let mut held = vec![String::from("build.hooks"), String::from("metadata.hooks")];
    for target in below(names, &["build", "targets"]) {
        held.push(format!("build.targets.{}.hooks", sections::quoted_segment(&target)));
    }
    for env in below(names, &["envs"]) {
        let env = sections::quoted_segment(&env);
        held.push(format!("envs.{env}.overrides"));
        held.push(format!("envs.{env}.matrix"));
    }
    held
}

/// The names a table holds, each spelled the way the file wrote its segments.
fn segments_of(entries: &[toml_doc::Entry<'_>]) -> Vec<Vec<String>> {
    entries.iter().map(|entry| entry.key_value.key.segments()).collect()
}

/// The one segment each name writes below `prefix`, in the order they read.
fn below(names: &[Vec<String>], prefix: &[&str]) -> Vec<String> {
    let mut held: Vec<String> = names
        .iter()
        .filter(|name| name.len() > prefix.len() && name.iter().zip(prefix).all(|(held, want)| held == want))
        .map(|name| name[prefix.len()].clone())
        .collect();
    held.sort();
    held.dedup();
    held
}

pub fn build_key_order(entries: &[toml_doc::Entry<'_>]) -> Vec<String> {
    key_order_of(&segments_of(entries))
}

/// The same, read from the names the table holds however the file wrote them.
fn key_order_of(names: &[Vec<String>]) -> Vec<String> {
    let mut order: Vec<String> = KEY_ORDER.iter().map(|name| (*name).to_string()).collect();
    for env in below(names, &["envs"]) {
        // the name is spelled the way a dispatch name spells it, so both sides match
        let prefix = format!("envs.{}", sections::quoted_segment(&env));
        for name in [
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
            order.push(format!("{prefix}.{name}"));
        }
        order.push(prefix);
    }
    order.push(String::from("envs"));
    order
}

/// These per-env arrays carry set semantics in hatch, so they sort. The environment's name is the
/// one segment after `envs`, whatever it holds.
fn is_dynamic_sort_array(key: &[String]) -> bool {
    matches!(
        key,
        [head, _, tail]
            if head == "envs"
                && matches!(
                    tail.as_str(),
                    "dependencies"
                        | "extra-dependencies"
                        | "features"
                        | "platforms"
                        | "env-include"
                        | "env-exclude"
                )
    )
}

fn fix_env_tables(document: &mut Document<'_>) {
    let env_names = collect_header_segments(document, "tool.hatch.envs.");
    for env in env_names {
        let key = ["tool", "hatch", "envs", &env].map(str::to_owned);
        if let Some(section) = sections::first_of(document, &key) {
            sections::for_entries(section, |key, value| {
                if matches!(
                    key,
                    "dependencies" | "extra-dependencies" | "features" | "platforms" | "env-include" | "env-exclude"
                ) {
                    sort_names_in(value);
                }
            });
            sections::reorder_keys(
                &mut section.entries,
                &[
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
        for sub in ["scripts", "env-vars"] {
            let key = ["tool", "hatch", "envs", &env, sub].map(str::to_owned);
            if let Some(section) = sections::first_of(document, &key) {
                sections::reorder_keys(&mut section.entries, &[]);
            }
        }
    }
}

fn collect_header_segments(document: &Document<'_>, prefix: &str) -> Vec<String> {
    sections::headers_below(
        document,
        &prefix.trim_end_matches('.').split('.').collect::<Vec<&str>>(),
    )
}
