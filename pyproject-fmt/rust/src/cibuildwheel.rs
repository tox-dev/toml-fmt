use std::sync::LazyLock;

use common::arrays::sort_names_in;
use common::sections;
use toml_doc::{Document, Value};

pub const KEY_ORDER: &[&str] = &[
    "build",
    "skip",
    "test-skip",
    "archs",
    "enable",
    "free-threaded-support",
    "build-frontend",
    "build-verbosity",
    "config-settings",
    "dependency-versions",
    "environment",
    "environment-pass",
    "before-all",
    "before-build",
    "repair-wheel-command",
    "before-test",
    "test-command",
    "test-requires",
    "test-extras",
    "test-groups",
    "test-sources",
    "manylinux-x86_64-image",
    "manylinux-i686-image",
    "manylinux-aarch64-image",
    "manylinux-ppc64le-image",
    "manylinux-s390x-image",
    "manylinux-armv7l-image",
    "manylinux-pypy_x86_64-image",
    "manylinux-pypy_i686-image",
    "manylinux-pypy_aarch64-image",
    "musllinux-x86_64-image",
    "musllinux-i686-image",
    "musllinux-aarch64-image",
    "musllinux-ppc64le-image",
    "musllinux-s390x-image",
    "musllinux-armv7l-image",
    "container-engine",
    "linux",
    "macos",
    "windows",
    "android",
    "ios",
    "pyodide",
    "overrides",
];

// Most arrays are CLI argv (order matters); only these are set semantics.
const SORT_ARRAYS: &[&str] = &["enable", "test-extras", "test-groups"];

// `select` leads because cibuildwheel requires it on every override entry.
static OVERRIDES_KEY_ORDER: LazyLock<Vec<&str>> = LazyLock::new(|| {
    let mut order = vec!["", "select"];
    order.extend(
        KEY_ORDER
            .iter()
            .filter(|name| !name.is_empty() && **name != "overrides"),
    );
    order
});

pub fn fix(document: &mut Document<'_>) {
    fix_one(document, "tool.cibuildwheel");
    // Per-platform tables reuse KEY_ORDER for when they stay expanded instead of collapsing into the parent.
    for plat in ["linux", "macos", "windows", "android", "ios", "pyodide"] {
        fix_one(document, &format!("tool.cibuildwheel.{plat}"));
    }
    fix_overrides_aot(document);
}

fn fix_one(document: &mut Document<'_>, table_name: &str) {
    let path = sections::parse_name(table_name);
    sections::for_keys_under(document, &path, |key, value| {
        if SORT_ARRAYS.contains(&key) {
            sort_array(value);
        } else if key == "overrides" {
            fix_overrides_inline(value);
        }
    });
    sections::reorder_under(document, &path, KEY_ORDER);
}

fn sort_array(value: &mut Value<'_>) {
    sort_names_in(value);
}

/// `[[tool.cibuildwheel.overrides]]` collapses to `overrides = [{ ... }]` before `fix` runs in the short table format,
/// so the inline entries need the same treatment as the array-of-tables form.
fn fix_overrides_inline(value: &mut Value<'_>) {
    let Value::Array(array) = value else { return };
    for member in &mut array.members {
        let Value::InlineTable(table) = &mut member.item else {
            continue;
        };
        // the discriminator is what says this inline table is an override rather than something else
        if !table.members.iter().any(|entry| entry.item.key.is_path("select")) {
            continue;
        }
        for entry in &mut table.members {
            if SORT_ARRAYS.contains(&common::sections::dispatch_name(&entry.item.key).as_str()) {
                sort_array(&mut entry.item.value);
            }
        }
        common::sections::sort_members(&mut table.members, |item| {
            let key = common::sections::dispatch_name(&item.key);
            (
                OVERRIDES_KEY_ORDER
                    .iter()
                    .position(|name| *name == key)
                    .unwrap_or(OVERRIDES_KEY_ORDER.len()),
                key,
            )
        });
    }
}

fn fix_overrides_aot(document: &mut Document<'_>) {
    for section in sections::named(document, "tool.cibuildwheel.overrides") {
        sections::for_entries(section, |key, value| {
            if SORT_ARRAYS.contains(&key) {
                sort_array(value);
            }
        });
        sections::reorder_keys(&mut section.entries, &OVERRIDES_KEY_ORDER);
    }
}
