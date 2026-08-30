pub const KEY_ORDER: &[&str] = &[
    "workspace.name",
    "workspace.version",
    "workspace.description",
    "workspace.authors",
    "workspace.license",
    "workspace.license-file",
    "workspace.readme",
    "workspace.homepage",
    "workspace.repository",
    "workspace.documentation",
    "workspace.channels",
    "workspace.platforms",
    "workspace.channel-priority",
    "workspace.solve-strategy",
    "workspace.conda-pypi-map",
    "workspace.requires-pixi",
    "workspace.exclude-newer",
    "workspace.preview",
    "workspace.build-variants",
    "workspace.build-variants-files",
    "workspace",
    "dependencies",
    "host-dependencies",
    "build-dependencies",
    "run-dependencies",
    "constraints",
    "pypi-dependencies",
    "pypi-options",
    "dev",
    "system-requirements",
    "activation",
    "tasks",
    "target",
    "feature",
    "environments",
    "package",
];

pub const WORKSPACE_KEY_ORDER: &[&str] = &[
    "name",
    "version",
    "description",
    "authors",
    "license",
    "license-file",
    "readme",
    "homepage",
    "repository",
    "documentation",
    "channels",
    "platforms",
    "channel-priority",
    "solve-strategy",
    "conda-pypi-map",
    "requires-pixi",
    "exclude-newer",
    "preview",
    "build-variants",
    "build-variants-files",
];

/// Whether what the name holds is a list of names, which sorts.
///
/// Channels and variant files are read in the order they are listed, the first one winning, so what
/// they say depends on where each one sits. A platform written as a table names none this can sort
/// by, and pixi runs the first entry a host satisfies, so a list holding one is left as written.
pub fn sorts(key: &str) -> bool {
    matches!(key, "workspace.platforms" | "workspace.preview")
}

/// The same, for the names written under `workspace`.
pub fn sorts_in_workspace(key: &str) -> bool {
    matches!(key, "platforms" | "preview")
}
