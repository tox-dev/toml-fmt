use common::array::sort_strings;
use common::table::{for_entries, reorder_table_keys, Tables};
use lexical_sort::natural_lexical_cmp;

const KEY_ORDER: &[&str] = &[
    "",
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

const WORKSPACE_KEY_ORDER: &[&str] = &[
    "",
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

pub fn fix(tables: &mut Tables) {
    if let Some(table_elements) = tables.get("tool.pixi") {
        let table = &mut table_elements.first().unwrap().borrow_mut();
        for_entries(table, &mut |key, entry| match key.as_str() {
            "workspace.channels" | "workspace.platforms" | "workspace.preview" | "workspace.build-variants-files" => {
                sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
            }
            _ => {}
        });
        reorder_table_keys(table, KEY_ORDER);
    }

    if let Some(workspace_elements) = tables.get("tool.pixi.workspace") {
        let workspace_table = &mut workspace_elements.first().unwrap().borrow_mut();
        for_entries(workspace_table, &mut |key, entry| match key.as_str() {
            "channels" | "platforms" | "preview" | "build-variants-files" => {
                sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
            }
            _ => {}
        });
        reorder_table_keys(workspace_table, WORKSPACE_KEY_ORDER);
    }
}
