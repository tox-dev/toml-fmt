use common::array::sort_strings;
use common::table::{for_entries, reorder_table_keys, Tables};
use lexical_sort::natural_lexical_cmp;

const KEY_ORDER: &[&str] = &[
    "",
    // === Run phase (data collection) ===
    "run",
    // Source selection (what to measure)
    "run.source",
    "run.source_pkgs",
    "run.source_dirs",
    // File filtering (include/omit together)
    "run.include",
    "run.omit",
    // Measurement options
    "run.branch",
    "run.cover_pylib",
    "run.timid",
    // Execution context
    "run.command_line",
    "run.concurrency",
    "run.context",
    "run.dynamic_context",
    // Data management
    "run.data_file",
    "run.parallel",
    "run.relative_files",
    // Plugins and extensions
    "run.plugins",
    // Debugging
    "run.debug",
    "run.debug_file",
    "run.disable_warnings",
    // Other
    "run.core",
    "run.patch",
    "run.sigterm",
    // === Paths (path mapping) ===
    "paths",
    // === Report phase (general reporting) ===
    "report",
    // Coverage threshold
    "report.fail_under",
    "report.precision",
    // File filtering (include/omit together)
    "report.include",
    "report.omit",
    "report.include_namespace_packages",
    // Line exclusion (exclude patterns together)
    "report.exclude_lines",
    "report.exclude_also",
    // Partial branch handling (partial together)
    "report.partial_branches",
    "report.partial_also",
    // Output control (skip together)
    "report.skip_covered",
    "report.skip_empty",
    "report.show_missing",
    // Formatting
    "report.format",
    "report.sort",
    // Error handling
    "report.ignore_errors",
    // === HTML output ===
    "html",
    "html.directory",
    "html.title",
    "html.extra_css",
    "html.show_contexts",
    "html.skip_covered",
    "html.skip_empty",
    // === JSON output ===
    "json",
    "json.output",
    "json.pretty_print",
    "json.show_contexts",
    // === LCOV output ===
    "lcov",
    "lcov.output",
    "lcov.line_checksums",
    // === XML output ===
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
        // Run phase arrays
        "run.source"
        | "run.source_pkgs"
        | "run.source_dirs"
        | "run.include"
        | "run.omit"
        | "run.concurrency"
        | "run.plugins"
        | "run.debug"
        | "run.disable_warnings"
        // Report phase arrays
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
