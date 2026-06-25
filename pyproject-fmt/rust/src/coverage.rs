use common::array::sort_strings;
use common::table::{for_entries, reorder_table_keys, Tables};
use lexical_sort::natural_lexical_cmp;

// Order mirrors coverage.py's configuration sections (run, paths, report, html, json, lcov, xml).
const KEY_ORDER: &[&str] = &[
    "",
    "run",
    "run.source",
    "run.source_pkgs",
    "run.source_dirs",
    "run.include",
    "run.omit",
    "run.branch",
    "run.cover_pylib",
    "run.timid",
    "run.command_line",
    "run.concurrency",
    "run.context",
    "run.dynamic_context",
    "run.data_file",
    "run.parallel",
    "run.relative_files",
    "run.plugins",
    "run.debug",
    "run.debug_file",
    "run.disable_warnings",
    "run.core",
    "run.patch",
    "run.sigterm",
    "paths",
    "report",
    "report.fail_under",
    "report.precision",
    "report.include",
    "report.omit",
    "report.include_namespace_packages",
    "report.exclude_lines",
    "report.exclude_also",
    "report.partial_branches",
    "report.partial_also",
    "report.skip_covered",
    "report.skip_empty",
    "report.show_missing",
    "report.format",
    "report.sort",
    "report.ignore_errors",
    "html",
    "html.directory",
    "html.title",
    "html.extra_css",
    "html.show_contexts",
    "html.skip_covered",
    "html.skip_empty",
    "json",
    "json.output",
    "json.pretty_print",
    "json.show_contexts",
    "lcov",
    "lcov.output",
    "lcov.line_checksums",
    "xml",
    "xml.output",
    "xml.package_depth",
];

pub fn fix(tables: &mut Tables) {
    let Some(table_elements) = tables.get("tool.coverage") else {
        return;
    };
    let table = &mut table_elements.first().unwrap().borrow_mut();
    for_entries(table, &mut |key, entry| match key.as_str() {
        "run.source"
        | "run.source_pkgs"
        | "run.source_dirs"
        | "run.include"
        | "run.omit"
        | "run.concurrency"
        | "run.plugins"
        | "run.debug"
        | "run.disable_warnings"
        | "report.include"
        | "report.omit"
        | "report.exclude_lines"
        | "report.exclude_also"
        | "report.partial_branches"
        | "report.partial_also" => {
            sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
        }
        _ => {}
    });
    reorder_table_keys(table, KEY_ORDER);
}
