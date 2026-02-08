use common::array::sort_strings;
use common::table::{for_entries, reorder_table_keys, Tables};
use lexical_sort::natural_lexical_cmp;
use tombi_syntax::SyntaxKind::KEY_VALUE;

const KEY_ORDER: &[&str] = &[
    "",
    "required-version",
    "python-preference",
    "python-downloads",
    "dev-dependencies",
    "default-groups",
    "dependency-groups",
    "constraint-dependencies",
    "override-dependencies",
    "exclude-dependencies",
    "dependency-metadata",
    "sources",
    "index",
    "index-url",
    "extra-index-url",
    "find-links",
    "no-index",
    "index-strategy",
    "keyring-provider",
    "no-binary",
    "no-binary-package",
    "no-build",
    "no-build-package",
    "no-build-isolation",
    "no-build-isolation-package",
    "no-sources",
    "no-sources-package",
    "reinstall",
    "reinstall-package",
    "upgrade",
    "upgrade-package",
    "resolution",
    "prerelease",
    "fork-strategy",
    "environments",
    "required-environments",
    "exclude-newer",
    "exclude-newer-package",
    "compile-bytecode",
    "link-mode",
    "config-settings",
    "config-settings-package",
    "extra-build-dependencies",
    "extra-build-variables",
    "concurrent-builds",
    "concurrent-downloads",
    "concurrent-installs",
    "allow-insecure-host",
    "native-tls",
    "offline",
    "no-cache",
    "cache-dir",
    "http-proxy",
    "https-proxy",
    "no-proxy",
    "publish-url",
    "check-url",
    "trusted-publishing",
    "python-install-mirror",
    "pypy-install-mirror",
    "python-downloads-json-url",
    "managed",
    "package",
    "workspace",
    "conflicts",
    "cache-keys",
    "build-backend",
    "pip",
    "preview",
    "torch-backend",
];

const PIP_KEY_ORDER: &[&str] = &[
    "",
    "python",
    "system",
    "break-system-packages",
    "target",
    "prefix",
    "index-url",
    "extra-index-url",
    "find-links",
    "no-index",
    "index-strategy",
    "keyring-provider",
    "no-binary",
    "no-binary-package",
    "only-binary",
    "only-binary-package",
    "no-build",
    "no-build-package",
    "no-build-isolation",
    "no-build-isolation-package",
    "resolution",
    "prerelease",
    "fork-strategy",
    "exclude-newer",
    "compile-bytecode",
    "link-mode",
    "config-settings",
    "allow-insecure-host",
    "native-tls",
    "offline",
    "no-cache",
    "cache-dir",
    "all-extras",
    "extra",
    "no-deps",
    "allow-empty-requirements",
    "reinstall",
    "reinstall-package",
    "upgrade",
    "upgrade-package",
    "python-platform",
    "python-version",
    "strict",
    "exclude-newer-package",
    "annotation-style",
    "custom-compile-command",
    "emit-build-options",
    "emit-find-links",
    "emit-index-annotation",
    "emit-index-url",
    "emit-marker-expression",
    "generate-hashes",
    "no-annotate",
    "no-emit-package",
    "no-header",
    "no-strip-extras",
    "no-strip-markers",
    "output-file",
    "universal",
];

fn has_key_value_entries(table: &[tombi_syntax::SyntaxElement]) -> bool {
    table.iter().any(|e| e.kind() == KEY_VALUE)
}

#[allow(clippy::too_many_lines)]
pub fn fix(tables: &mut Tables) {
    if let Some(table_elements) = tables.get("tool.uv") {
        let table = &mut table_elements.first().unwrap().borrow_mut();
        for_entries(table, &mut |key, entry| match key.as_str() {
            "allow-insecure-host"
            | "build-constraint-dependencies"
            | "constraint-dependencies"
            | "dev-dependencies"
            | "environments"
            | "exclude-dependencies"
            | "no-binary-package"
            | "no-build-isolation-package"
            | "no-build-package"
            | "no-proxy"
            | "no-sources-package"
            | "override-dependencies"
            | "reinstall-package"
            | "required-environments"
            | "upgrade-package"
            | "workspace.exclude"
            | "workspace.members"
            | "pip.allow-insecure-host"
            | "pip.extra"
            | "pip.no-binary-package"
            | "pip.no-build-isolation-package"
            | "pip.no-build-package"
            | "pip.no-emit-package"
            | "pip.only-binary-package"
            | "pip.reinstall-package"
            | "pip.upgrade-package" => {
                sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
            }
            _ => {}
        });
        reorder_table_keys(table, KEY_ORDER);
    }

    if let Some(sources_tables) = tables.get("tool.uv.sources") {
        for sources_ref in sources_tables {
            let sources_table = &mut sources_ref.borrow_mut();
            if has_key_value_entries(sources_table) {
                reorder_table_keys(sources_table, &[""]);
            }
        }
    }

    if let Some(pip_elements) = tables.get("tool.uv.pip") {
        let pip_table = &mut pip_elements.first().unwrap().borrow_mut();
        if has_key_value_entries(pip_table) {
            for_entries(pip_table, &mut |key, entry| match key.as_str() {
                "allow-insecure-host"
                | "extra"
                | "no-binary-package"
                | "no-build-isolation-package"
                | "no-build-package"
                | "no-emit-package"
                | "only-binary-package"
                | "reinstall-package"
                | "upgrade-package" => {
                    sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| {
                        natural_lexical_cmp(lhs, rhs)
                    });
                }
                _ => {}
            });
            reorder_table_keys(pip_table, PIP_KEY_ORDER);
        }
    }
}
