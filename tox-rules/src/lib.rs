use std::collections::HashSet;

use common::arrays::{map_strings, sort, sort_names_in, sort_strings_in, string_of};
use common::pep508::Requirement;
use common::sections::{self, InlineSchema};
use lexical_sort::natural_lexical_cmp;
use toml_doc::{Array, Document, Entry, InlineTable, Key, Member, Piece, Value};

/// The table the root rules run against: a `tox.toml` writes its own keys before any header, while
/// a `pyproject.toml` writes them under the table `prefix` names.
fn root_of(prefix: &str) -> Vec<String> {
    if prefix.is_empty() {
        Vec::new()
    } else {
        sections::parse_name(prefix)
    }
}

/// Whether the table names one tox environment, under the `prefix` that lets the same rules serve
/// `tox.toml` and the `[tool.tox]` tables of a `pyproject.toml`.
///
/// The check counts segments rather than dots, since an environment may be named anything:
/// `[env.".pkg-cpython311"]` is one environment, not a table two levels down.
fn is_env_table(segments: &[String], prefix: &str) -> bool {
    env_table_of(segments, prefix).is_some_and(|named| named == segments)
}

/// The environment table the path names or sits under, where it names one at all.
fn env_table_of(segments: &[String], prefix: &str) -> Option<Vec<String>> {
    let wanted: Vec<&str> = prefix.split('.').filter(|part| !part.is_empty()).collect();
    if segments.len() < wanted.len() || !segments.iter().zip(&wanted).all(|(held, want)| held == want) {
        return None;
    }
    let held = match &segments[wanted.len()..] {
        [only, ..] if only == "env_run_base" || only == "env_pkg_base" => 1,
        [head, _name, ..] if head == "env" || head == "env_base" => 2,
        _ => return None,
    };
    Some(segments[..wanted.len() + held].to_vec())
}

const ROOT_ALIASES: &[(&str, &str)] = &[
    ("envlist", "env_list"),
    ("toxinidir", "tox_root"),
    ("toxworkdir", "work_dir"),
    ("skipsdist", "no_package"),
    ("isolated_build_env", "package_env"),
    ("setupdir", "package_root"),
    ("minversion", "min_version"),
    ("ignore_basepython_conflict", "ignore_base_python_conflict"),
];

const ENV_ALIASES: &[(&str, &str)] = &[
    ("setenv", "set_env"),
    ("passenv", "pass_env"),
    ("envdir", "env_dir"),
    ("envtmpdir", "env_tmp_dir"),
    ("envlogdir", "env_log_dir"),
    ("changedir", "change_dir"),
    ("basepython", "base_python"),
    ("usedevelop", "use_develop"),
    ("sitepackages", "system_site_packages"),
    ("alwayscopy", "always_copy"),
];

const ROOT_KEY_ORDER: &[&str] = &[
    "min_version",
    "requires",
    "provision_tox_env",
    "env_list",
    "labels",
    "base",
    "package_env",
    "package_root",
    "no_package",
    "skip_missing_interpreters",
    "ignore_base_python_conflict",
    "work_dir",
    "temp_dir",
    "tox_root",
];

/// tox reads a `set_env` table in the order it is written: a key after `file` overrides what that
/// file said, and one before it does not.
const KEEP_ORDER: &[&str] = &["set_env"];

const ENV_KEY_ORDER: &[&str] = &[
    "factors",
    "runner",
    "description",
    "base_python",
    "base_python_file",
    "default_base_python",
    "system_site_packages",
    "always_copy",
    "download",
    "virtualenv_spec",
    "package",
    "package_env",
    "wheel_build_env",
    "package_tox_env_type",
    "package_root",
    "skip_install",
    "use_develop",
    "meta_dir",
    "pkg_dir",
    "pip_pre",
    "install_command",
    "list_dependencies_command",
    "deps",
    "dependency_groups",
    "pylock",
    "constraints",
    "constrain_package_deps",
    "use_frozen_constraints",
    "extras",
    "recreate",
    "recreate_commands",
    "parallel_show_output",
    "skip_missing_interpreters",
    "fail_fast",
    "pass_env",
    "disallow_pass_env",
    "set_env",
    "change_dir",
    "platform",
    "args_are_paths",
    "ignore_errors",
    "commands_retry",
    "ignore_outcome",
    "extra_setup_commands",
    "commands_pre",
    "commands",
    "commands_post",
    "allowlist_externals",
    "labels",
    "suicide_timeout",
    "interrupt_timeout",
    "terminate_timeout",
    "depends",
    "env_dir",
    "env_tmp_dir",
    "env_log_dir",
];

pub fn normalize_aliases(document: &mut Document<'_>) {
    normalize_aliases_with_prefix(document, "");
}

pub fn normalize_aliases_with_prefix(document: &mut Document<'_>, prefix: &str) {
    let mut renamed: Vec<(Vec<String>, String, String)> = Vec::new();
    let root = root_of(prefix);
    renamed.extend(
        sections::rename_under(document, &root, ROOT_ALIASES)
            .into_iter()
            .map(|(from, to)| (root.clone(), from, to)),
    );
    renamed.extend(sections::rename_tables_of(
        document,
        &|named| env_table_of(named, prefix),
        ENV_ALIASES,
    ));
    // a `{ replace = "ref" }` names a key rather than holding text that looks like one, so a key
    // that moved takes the references to it along
    for value in sections::every_value(document) {
        follow_renames(value, &renamed, &root);
    }
}

/// Rewrite the reference paths inside the value that name a key one of the renames moved.
fn follow_renames(value: &mut Value<'_>, renamed: &[(Vec<String>, String, String)], root: &[String]) {
    match value {
        Value::Scalar(_) => {}
        Value::Array(array) => {
            for member in &mut array.members {
                follow_renames(&mut member.item, renamed, root);
            }
        }
        Value::InlineTable(table) => {
            for member in &mut table.members {
                follow_renames(&mut member.item.value, renamed, root);
            }
            if !names_a_reference(table) {
                return;
            }
            for member in &mut table.members {
                if !member.item.key.is_path("of") {
                    continue;
                }
                let Value::Array(path) = &mut member.item.value else {
                    continue;
                };
                rename_in_path(path, renamed, root);
            }
        }
    }
}

/// Whether the inline table is one of tox's `replace = "ref"` substitutions.
fn names_a_reference(table: &InlineTable<'_>) -> bool {
    table.members.iter().any(|member| {
        member.item.key.is_path("replace")
            && common::strings::text_of(&member.item.value).is_some_and(|held| held == "ref")
    })
}

/// The last segment of a reference path is the key it names; the ones before it name the table.
fn rename_in_path(path: &mut Array<'_>, renamed: &[(Vec<String>, String, String)], root: &[String]) {
    let named: Vec<String> = path.members.iter().filter_map(string_of).collect();
    if named.len() != path.members.len() {
        return;
    }
    let Some((key, table)) = named.split_last() else {
        return;
    };
    let Some((_, _, to)) = renamed
        .iter()
        .find(|(held, from, _)| from == key && names_the_same_table(held, table, root))
    else {
        return;
    };
    path.members
        .last_mut()
        .expect("the path names at least one segment")
        .item = common::build::string(to);
}

/// Whether the reference names the table the rename happened in, spelled in full or with the
/// prefix the document is nested under left off. Any other tail is a different table: an
/// environment named `src` is not the root table of that name.
fn names_the_same_table(held: &[String], table: &[String], root: &[String]) -> bool {
    held == table || held.strip_prefix(root).is_some_and(|rest| rest == table)
}

pub fn fix_root(document: &mut Document<'_>) {
    fix_root_with_prefix(document, "");
}

pub fn fix_root_with_prefix(document: &mut Document<'_>, prefix: &str) {
    let root = root_of(prefix);
    sections::for_keys_under(document, &root, |key, value| {
        if key == "requires" {
            normalize_and_sort_requirements(value);
        }
    });
    let mut names: Vec<Vec<String>> = Vec::new();
    sections::for_names_under(document, &root, |tail, _| names.push(tail.to_vec()));
    let order = root_key_order_of(&names);
    let refs: Vec<&str> = order.iter().map(String::as_str).collect();
    // the root table is the run of keys the file wrote for it; an environment below it is a table of
    // its own, which its own rules order
    sections::for_entry_runs(document, &root, |entries, under| {
        if under == root {
            sections::reorder_keys(entries, &refs);
        }
    });
}

/// The root key order, with a slot for every key of an environment folded into the table, so an
/// environment written as dotted keys reads the way its own table does.
fn root_key_order_of(names: &[Vec<String>]) -> Vec<String> {
    let mut order: Vec<String> = ROOT_KEY_ORDER.iter().map(|key| (*key).to_owned()).collect();
    for group in folded_env_names_of(names) {
        order.extend(
            ENV_KEY_ORDER
                .iter()
                .filter(|key| !key.is_empty())
                .map(|key| format!("{group}.{key}")),
        );
        order.push(group);
    }
    order
}

/// The environments a table holds as dotted keys, each named the way its keys name it, so a name
/// holding a dot stays the one segment the file wrote.
fn folded_env_names_of(held: &[Vec<String>]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for segments in held {
        let name = match segments.as_slice() {
            [head, _, ..] if head == "env_run_base" || head == "env_pkg_base" => head.clone(),
            [head, name, _, ..] if head == "env" || head == "env_base" => {
                format!("{head}.{}", sections::quoted_segment(name))
            }
            _ => continue,
        };
        if !names.contains(&name) {
            names.push(name);
        }
    }
    names.sort();
    names
}

pub fn fix_envs(document: &mut Document<'_>) {
    fix_envs_with_prefix(document, "");
}

pub fn fix_envs_with_prefix(document: &mut Document<'_>, prefix: &str) {
    upgrade_use_develop_in_envs(document, prefix);
    // one walk of the file serves every environment in it, whichever one each key belongs to
    let root = root_of(prefix);
    sections::for_key_paths_under(document, &root, |named, key_value| {
        if let Some(env) = env_table_of(named, prefix) {
            let tail = &named[env.len()..];
            if !tail.is_empty() {
                fix_env_entry(&sections::dotted_name(tail), &mut key_value.value);
            }
        }
    });
    sections::reorder_tables_of(
        document,
        &|named| env_table_of(named, prefix),
        ENV_KEY_ORDER,
        KEEP_ORDER,
    );
}

fn upgrade_use_develop_in_envs(document: &mut Document<'_>, prefix: &str) {
    sections::for_entry_runs(document, &[], |entries, under| {
        // the environments this run writes keys of, each named the way its keys spell it here
        let mut held: Vec<Vec<String>> = Vec::new();
        for entry in entries.iter() {
            let named: Vec<String> = under.iter().chain(&entry.key_value.key.segments()).cloned().collect();
            let Some(env) = env_table_of(&named, prefix) else {
                continue;
            };
            let Some(head) = env.strip_prefix(under) else {
                continue;
            };
            if !held.contains(&head.to_vec()) {
                held.push(head.to_vec());
            }
        }
        for head in held {
            upgrade_use_develop(entries, &head);
        }
    });
}

/// `use_develop = true` says the same thing as `package = "editable"`, which is the spelling tox
/// documents. tox reads the older key first and installs an editable package whatever `package`
/// says, so the newer key is written with the value the environment runs with.
fn upgrade_use_develop(entries: &mut Vec<Entry<'_>>, head: &[String]) {
    let names = |entry: &Entry<'_>, name: &str| {
        let segments = entry.key_value.key.segments();
        segments.len() == head.len() + 1 && segments.starts_with(head) && segments[head.len()] == name
    };
    // a disabled key is one the comment beside it speaks for: migrating it would leave that comment
    // on a key the file wrote, and taking one onto a disabled key would say it twice
    let Some(at) = entries.iter().position(|entry| {
        names(entry, "use_develop") && is_true(&entry.key_value.value) && !common::disabled::is_enabled_here(entry)
    }) else {
        return;
    };
    // a key the file wrote as a comment reserves no name: what the environment runs with is what
    // the keys it wrote say
    let has_package = sections::active(entries).any(|entry| names(entry, "package"));
    let removed = entries.remove(at);
    if has_package {
        // the comments around the older key are about the environment, so they move to the key that
        // says the same thing; two comments cannot share one line, so the older one takes a line
        // above it
        let package = sections::active(entries)
            .find(|entry| names(entry, "package"))
            .expect("the package key was just found");
        package.key_value.value = common::build::string("editable");
        package.lead.pieces_mut().splice(0..0, removed.lead.pieces().to_vec());
        if let Some(text) = removed.trail.comment {
            match package.trail.comment {
                None => package.trail.comment = Some(text),
                Some(_) => package.lead.pieces_mut().push(Piece::Comment {
                    indent: "".into(),
                    text,
                    ending: removed.trail.ending,
                }),
            }
        }
        return;
    }
    // the key keeps whatever path the file wrote it under, so it stays the same environment's
    let named = head
        .iter()
        .map(String::as_str)
        .chain(["package"])
        .collect::<Vec<&str>>();
    let mut replacement = common::build::string_entry(&named.join("."), "editable");
    replacement.key_value.key = Key::new(named);
    replacement.lead = removed.lead;
    replacement.trail.comment = removed.trail.comment;
    entries.insert(at, replacement);
}

/// Whether the value is the boolean `true`, rather than text or a container that happens to be
/// written with those letters in it.
fn is_true(value: &Value<'_>) -> bool {
    matches!(value, Value::Scalar(repr) if repr.quoting().is_none() && repr.text() == "true")
}

/// Requirement files, paths and templated values are left as written; normalizing them would
/// rewrite something that is not a requirement.
fn should_skip_normalization(text: &str) -> bool {
    text.starts_with('-')
        || text.starts_with("./")
        || text.starts_with("../")
        || text.starts_with('/')
        || text.contains('{')
        || text.contains("://")
        || names_an_artifact(text)
}

/// Whether the text names a file pip installs rather than a distribution it looks up. A name a
/// requirement may hold ends in none of these, so the suffix is what tells them apart.
fn names_an_artifact(text: &str) -> bool {
    let lowered = text.to_lowercase();
    [".whl", ".zip", ".tar.gz", ".tar.bz2", ".tar.xz", ".tar.z", ".tgz"]
        .iter()
        .any(|suffix| lowered.ends_with(suffix))
}

fn normalize_and_sort_requirements(value: &mut Value<'_>) {
    let Value::Array(array) = value else { return };
    // a requirement this parser cannot read is left as the file wrote it
    map_strings(array, |text| {
        if should_skip_normalization(text) {
            return text.to_owned();
        }
        Requirement::new(text).map_or_else(|_| text.to_owned(), |found| found.normalize(false).to_string())
    });
    // pip reads this list the way it reads a requirements file, where a later `--index-url` replaces
    // the one before it, so a list holding anything but plain requirements keeps the order it names
    // them in
    if !array.members.iter().all(reads_as_a_requirement) {
        return;
    }
    sort_strings_in(
        value,
        &|text| Requirement::new(text).map_or(text.to_lowercase(), |found| found.canonical_name()),
        &|left, right| natural_lexical_cmp(left, right),
    );
}

/// Whether the member is an ordinary requirement rather than a line pip reads as something else: an
/// option, a file it pulls in, a path, a URL, or a name tox fills in.
fn reads_as_a_requirement(member: &Member<'_, Value<'_>>) -> bool {
    string_of(member).is_some_and(|text| !should_skip_normalization(&text) && Requirement::new(&text).is_ok())
}

fn fix_env_entry(key: &str, value: &mut Value<'_>) {
    match key {
        "deps" => normalize_and_sort_requirements(value),
        // each constraint is the path or URL of a file tox hands to pip, not a requirement
        "constraints" => {}
        "dependency_groups" | "allowlist_externals" | "extras" | "labels" | "depends" => {
            sort_names_in(value);
        }
        "pass_env" => sort_pass_env(value),
        _ => {}
    }
}

/// Inline tables lead, then the plain names alphabetically.
fn sort_pass_env(value: &mut Value<'_>) {
    let Value::Array(array) = value else { return };
    sort(
        array,
        &|member| match &member.item {
            Value::InlineTable(_) => Some((0, String::new())),
            _ => string_of(member).map(|text| (1, text.to_lowercase())),
        },
        &|left: &(u8, String), right: &(u8, String)| {
            left.0
                .cmp(&right.0)
                .then_with(|| natural_lexical_cmp(&left.1, &right.1))
        },
    );
}

/// Put `env_list` in the order the formatter writes one.
///
/// The environments a pin names come first, in the order the pin gives them; then CPython versions,
/// newest first; then PyPy versions, newest first; then everything else by name. A compound name is
/// placed by the first part of it that reads as one of those.
///
/// An entry that generates environments rather than naming one, such as `{ product = ... }`, holds
/// the place the file gave it, since what it generates is read where it sits.
pub fn sort_env_list(document: &mut Document<'_>, pin_envs: &[String]) {
    sort_env_list_with_prefix(document, pin_envs, "");
}

/// [`sort_env_list`], under the `prefix` that lets the same rules serve `tox.toml` and the
/// `[tool.tox]` tables of a `pyproject.toml`.
pub fn sort_env_list_with_prefix(document: &mut Document<'_>, pin_envs: &[String], prefix: &str) {
    let mut path = root_of(prefix);
    path.push(String::from("env_list"));
    sections::for_value_at(document, &path, |value| {
        let Value::Array(array) = value else {
            return;
        };
        common::arrays::sort_placed(array, &|member| ranked_env(member, pin_envs), &|left, right| {
            left.0
                .cmp(&right.0)
                .then_with(|| left.1.cmp(&right.1))
                .then_with(|| left.2.cmp(&right.2))
                .then_with(|| natural_lexical_cmp(&left.3, &right.3))
        });
    });
}

/// Where a name sits in the list, or `None` where the entry names no one environment.
fn ranked_env(member: &Member<'_, Value<'_>>, pin_envs: &[String]) -> Option<(i32, i32, i32, String)> {
    let named = string_of(member)?.to_lowercase();
    for part in named.split('-') {
        if let Some(at) = pin_envs.iter().position(|pin| pin.to_lowercase() == part) {
            return Some((0, i32::try_from(at).unwrap_or(i32::MAX), 0, named.clone()));
        }
        if let Some((kind, major, minor)) = interpreter(part) {
            return Some((kind, major, minor, named.clone()));
        }
    }
    Some((3, 0, 0, named))
}

/// The interpreter a name part spells, with its version negated so the newest sorts first.
fn interpreter(part: &str) -> Option<(i32, i32, i32)> {
    let (kind, version) = match part.strip_prefix("pypy") {
        Some(version) => (2, version),
        None => (1, part.strip_prefix("py").unwrap_or(part)),
    };
    // `py312` and `py3.12` both name a version, and a name holding anything else names none
    let (major, minor) = version.split_once('.').unwrap_or((version, ""));
    if major.is_empty() || !major.bytes().chain(minor.bytes()).all(|held| held.is_ascii_digit()) {
        return None;
    }
    Some((
        kind,
        -major.parse::<i32>().unwrap_or(0),
        -minor.parse::<i32>().unwrap_or(0),
    ))
}

/// Put the environment tables in the order `env_list` names them, with the pinned ones first.
///
/// tox runs the environments in the order `env_list` gives them, so that list is left exactly as
/// the file wrote it; what a pin moves is where the table holding an environment is written.
pub fn table_order(document: &Document<'_>, pin_envs: &[String]) -> Vec<String> {
    let pinned: Vec<String> = pin_envs
        .iter()
        .map(|pin| format!("env.{}", sections::quoted_segment(pin)))
        .collect();
    let mut order = pinned.clone();
    for name in env_list_order(document) {
        if !order.contains(&name) {
            order.push(name);
        }
    }
    order
}

/// The environments `env_list` names, in the order it names them.
fn env_list_order(document: &Document<'_>) -> Vec<String> {
    document
        .root
        .iter()
        // a list the file wrote as a comment names no environment tox runs, so it says nothing
        // about where the tables that hold them go
        .filter(|entry| !common::disabled::is_enabled_here(entry))
        .filter(|entry| entry.key_value.key.is_path("env_list"))
        .filter_map(|entry| match &entry.key_value.value {
            Value::Array(array) => Some(array),
            Value::Scalar(_) | Value::InlineTable(_) => None,
        })
        .flat_map(|array| array.members.iter().filter_map(string_of))
        // the name is spelled the way a header spells it, so both sides of the order match
        .map(|name| format!("env.{}", sections::quoted_segment(&name)))
        .collect()
}

pub fn reorder_tables(document: &mut Document<'_>) {
    reorder_tables_with_pins(document, &[]);
}

/// [`reorder_tables`], with the environments a pin names written first.
pub fn reorder_tables_with_pins(document: &mut Document<'_>, pin_envs: &[String]) {
    let listed = table_order(document, pin_envs);
    let mut order: Vec<String> = ["", "env_run_base", "env_pkg_base", "env_base"]
        .iter()
        .map(|name| (*name).to_owned())
        .collect();
    let listed_set: HashSet<&str> = listed.iter().map(String::as_str).collect();
    order.extend(listed.iter().cloned());

    // an environment is named by its own two segments, so one the file quoted because it holds a
    // dot is the one environment it is rather than a table two levels down. It sorts by the name it
    // was given and takes its place in the order by the way a header spells that name
    let mut rest: Vec<(String, String)> = document
        .sections
        .iter()
        .map(|section| section.header.key.segments())
        .filter(|segments| segments.len() == 2 && segments[0] == "env")
        .map(|segments| (segments[1].to_lowercase(), sections::dotted_name(&segments)))
        .filter(|(_, spelled)| !listed_set.contains(spelled.as_str()))
        .collect();
    rest.sort();
    rest.dedup();
    order.extend(rest.into_iter().map(|(_, spelled)| spelled));
    order.push(String::from("env"));

    let refs: Vec<&str> = order.iter().map(String::as_str).collect();
    sections::reorder_within(document, &refs, &["env_base", "env"], &|name| env_key_order(name));

    for section in &mut document.sections {
        if is_env_table(&section.header.key.segments(), "") {
            sections::reorder_keys_within(&mut section.entries, ENV_KEY_ORDER, KEEP_ORDER);
        }
    }
}

/// Every env table shares one key order, so `[env.py313.set_env]` sits where the dotted `set_env.A`
/// would.
fn env_key_order(table: &[String]) -> Option<Vec<String>> {
    is_env_table(table, "").then(|| ENV_KEY_ORDER.iter().map(|key| (*key).to_owned()).collect())
}

const TOX_INLINE_TABLE_SCHEMAS: &[InlineSchema<'static>] = &[
    InlineSchema {
        discriminator: "replace",
        key_order: &[
            "replace",
            "condition",
            "of",
            "env",
            "key",
            "name",
            "pattern",
            "then",
            "else",
            "default",
            "extend",
            "marker",
        ],
    },
    InlineSchema {
        discriminator: "prefix",
        key_order: &["prefix", "start", "stop"],
    },
    InlineSchema {
        discriminator: "product",
        key_order: &["product", "exclude"],
    },
    InlineSchema {
        discriminator: "value",
        key_order: &["value", "marker"],
    },
];

pub fn reorder_inline_tables(document: &mut Document<'_>) {
    reorder_inline_tables_with_prefix(document, "");
}

pub fn reorder_inline_tables_with_prefix(document: &mut Document<'_>, prefix: &str) {
    let name: Vec<String> = prefix
        .split('.')
        .filter(|part| !part.is_empty())
        .map(str::to_owned)
        .collect();
    sections::reorder_inline_tables(document, &name, TOX_INLINE_TABLE_SCHEMAS);
}
