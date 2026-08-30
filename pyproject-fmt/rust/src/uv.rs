pub const KEY_ORDER: &[&str] = &[
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

pub const PIP_KEY_ORDER: &[&str] = &[
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

#[allow(clippy::too_many_lines)]
/// Whether what the name holds is a list of names, which sorts.
pub fn sorts(key: &str) -> bool {
    matches!(
        key,
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
            | "pip.upgrade-package"
    )
}

/// The same, for the names written under `pip`.
pub fn sorts_in_pip(key: &str) -> bool {
    matches!(
        key,
        "allow-insecure-host"
            | "extra"
            | "no-binary-package"
            | "no-build-isolation-package"
            | "no-build-package"
            | "no-emit-package"
            | "only-binary-package"
            | "reinstall-package"
            | "upgrade-package"
    )
}
