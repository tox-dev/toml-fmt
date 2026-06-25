use common::array::sort_strings;
use common::table::{for_entries, reorder_table_keys, Tables};
use lexical_sort::natural_lexical_cmp;

// Keys carry the ini_options. prefix: after collapse every key appears as ini_options.<name> under tool.pytest, its
// only standardized child.
const KEY_ORDER: &[&str] = &[
    "",
    "ini_options.minversion",
    "ini_options.required_plugins",
    "ini_options.testpaths",
    "ini_options.pythonpath",
    "ini_options.norecursedirs",
    "ini_options.collect_ignore",
    "ini_options.collect_ignore_glob",
    "ini_options.python_files",
    "ini_options.python_classes",
    "ini_options.python_functions",
    "ini_options.consider_namespace_packages",
    "ini_options.confcutdir",
    "ini_options.rootdir_fallback",
    "ini_options.addopts",
    "ini_options.usefixtures",
    "ini_options.markers",
    "ini_options.empty_parameter_set_mark",
    "ini_options.xfail_strict",
    "ini_options.disable_test_id_escaping_and_forfeit_all_rights_to_community_support",
    "ini_options.filterwarnings",
    "ini_options.doctest_encoding",
    "ini_options.doctest_optionflags",
    "ini_options.console_output_style",
    "ini_options.verbosity_assertions",
    "ini_options.verbosity_test_cases",
    "ini_options.truncation_limit_chars",
    "ini_options.truncation_limit_lines",
    "ini_options.log_auto_indent",
    "ini_options.log_format",
    "ini_options.log_date_format",
    "ini_options.log_level",
    "ini_options.log_cli",
    "ini_options.log_cli_level",
    "ini_options.log_cli_format",
    "ini_options.log_cli_date_format",
    "ini_options.log_file",
    "ini_options.log_file_level",
    "ini_options.log_file_format",
    "ini_options.log_file_mode",
    "ini_options.log_file_date_format",
    "ini_options.junit_suite_name",
    "ini_options.junit_family",
    "ini_options.junit_duration_report",
    "ini_options.junit_log_passing_tests",
    "ini_options.junit_logging",
    "ini_options.cache_dir",
    "ini_options.tmp_path_retention_count",
    "ini_options.tmp_path_retention_policy",
    "ini_options.enable_assertion_pass_hook",
    "ini_options.faulthandler_timeout",
    "ini_options",
];

// Set-semantics arrays only; addopts (CLI argv) and pythonpath (search order) excluded.
const SORT_ARRAYS: &[&str] = &[
    "ini_options.testpaths",
    "ini_options.norecursedirs",
    "ini_options.collect_ignore",
    "ini_options.collect_ignore_glob",
    "ini_options.python_files",
    "ini_options.python_classes",
    "ini_options.python_functions",
    "ini_options.markers",
    "ini_options.filterwarnings",
    "ini_options.doctest_optionflags",
    "ini_options.usefixtures",
    "ini_options.required_plugins",
];

pub fn fix(tables: &mut Tables) {
    let Some(elements) = tables.get("tool.pytest") else {
        return;
    };
    let table = &mut elements.first().unwrap().borrow_mut();
    for_entries(table, &mut |key, entry| {
        if SORT_ARRAYS.contains(&key.as_str()) {
            sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| natural_lexical_cmp(lhs, rhs));
        }
    });
    reorder_table_keys(table, KEY_ORDER);
}
