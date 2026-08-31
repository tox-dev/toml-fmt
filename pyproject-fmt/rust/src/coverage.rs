// Order mirrors coverage.py's configuration sections (run, paths, report, html, json, lcov, xml).
pub const KEY_ORDER: &[&str] = &[
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
    "run.debug",
    "run.debug_file",
    "run.disable_warnings",
    "run.plugins",
    "run.core",
    "run.patch",
    "run.sigterm",
    "run",
    "paths",
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
    "report",
    "html.directory",
    "html.title",
    "html.extra_css",
    "html.show_contexts",
    "html.skip_covered",
    "html.skip_empty",
    "html",
    "json.output",
    "json.pretty_print",
    "json.show_contexts",
    "json",
    "lcov.output",
    "lcov.line_checksums",
    "lcov",
    "xml.output",
    "xml.package_depth",
    "xml",
];

/// Whether what the name holds is a list of names, which sorts.
pub fn sorts(key: &str) -> bool {
    matches!(
        key,
        "run.source"
            | "run.source_pkgs"
            | "run.source_dirs"
            | "run.include"
            | "run.omit"
            | "run.concurrency"
            | "run.debug"
            | "run.disable_warnings"
            | "report.include"
            | "report.omit"
            | "report.exclude_lines"
            | "report.exclude_also"
            | "report.partial_branches"
            | "report.partial_also"
    )
}
