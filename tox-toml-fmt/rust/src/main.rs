use std::collections::HashSet;
use std::string::String;

use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::FromPyObjectOwned;
#[cfg(feature = "extension-module")]
use pyo3::prelude::{PyModule, PyModuleMethods};
use pyo3::types::{PyAnyMethods, PyDict, PyDictMethods};
use pyo3::{pyclass, pymethods, Bound, PyResult};
#[cfg(feature = "extension-module")]
use pyo3::{pyfunction, pymodule, wrap_pyfunction, Python};

use toml_doc::Document;

use tox_rules::{
    fix_envs, fix_root, normalize_aliases, reorder_inline_tables, reorder_tables_with_pins, sort_env_list,
};

#[pyclass(frozen, get_all)]
pub struct Settings {
    pub column_width: usize,
    pub indent: usize,
    pub table_format: String,
    pub sub_table_spacing: String,
    pub separate_root_table: String,
    pub expand_tables: Vec<String>,
    pub collapse_tables: Vec<String>,
    pub skip_wrap_for_keys: Vec<String>,
    pub pin_envs: Vec<String>,
}

#[pymethods]
impl Settings {
    #[new]
    #[pyo3(signature = (**kwargs))]
    pub fn new(kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
        const NAMES: &[&str] = &[
            "column_width",
            "indent",
            "table_format",
            "sub_table_spacing",
            "separate_root_table",
            "expand_tables",
            "collapse_tables",
            "skip_wrap_for_keys",
            "pin_envs",
        ];
        let kwargs = required_keywords(kwargs, NAMES)?;
        let column_width = required(kwargs, "column_width")?;
        let indent = required(kwargs, "indent")?;
        let table_format = required(kwargs, "table_format")?;
        let sub_table_spacing = required(kwargs, "sub_table_spacing")?;
        let separate_root_table = required(kwargs, "separate_root_table")?;
        let expand_tables: Vec<String> = required(kwargs, "expand_tables")?;
        let collapse_tables: Vec<String> = required(kwargs, "collapse_tables")?;
        let skip_wrap_for_keys: Vec<String> = required(kwargs, "skip_wrap_for_keys")?;
        let pin_envs: Vec<String> = required(kwargs, "pin_envs")?;
        // a selector names a table the way TOML names one, so a file asking for a name no key
        // spells is told rather than formatted as though it had asked for nothing
        for (setting, held) in [("expand_tables", &expand_tables), ("collapse_tables", &collapse_tables)] {
            for name in held {
                if let Err(why) = common::sections::read_name(name) {
                    return Err(PyValueError::new_err(format!(
                        "{setting}: {name} is not a table name: {why}"
                    )));
                }
            }
        }
        // a pattern names a key and a pin names an environment, each of which the file writes as
        // something; neither says anything where the file wrote no name at all
        for (setting, held) in [("skip_wrap_for_keys", &skip_wrap_for_keys), ("pin_envs", &pin_envs)] {
            if held.iter().any(|name| name.trim().is_empty()) {
                return Err(PyValueError::new_err(format!(
                    "{setting}: a name is written there, not nothing"
                )));
            }
        }
        Ok(Self {
            column_width,
            indent,
            table_format,
            sub_table_spacing,
            separate_root_table,
            expand_tables,
            collapse_tables,
            skip_wrap_for_keys,
            pin_envs,
        })
    }
}

fn required_keywords<'py>(
    kwargs: Option<&'py Bound<'py, PyDict>>,
    names: &[&str],
) -> PyResult<&'py Bound<'py, PyDict>> {
    let kwargs = kwargs.ok_or_else(|| PyTypeError::new_err(format!("missing keyword argument: '{}'", names[0])))?;
    for key in kwargs.keys() {
        let name = key.extract::<String>()?;
        if !names.contains(&name.as_str()) {
            return Err(PyTypeError::new_err(format!("unexpected keyword argument: '{name}'")));
        }
    }
    Ok(kwargs)
}

fn required<'py, T: FromPyObjectOwned<'py>>(kwargs: &Bound<'py, PyDict>, name: &str) -> PyResult<T> {
    kwargs
        .get_item(name)
        .expect("Rust strings are valid Python dictionary keys")
        .ok_or_else(|| PyTypeError::new_err(format!("missing keyword argument: '{name}'")))?
        .extract()
        .map_err(Into::into)
}

pub type TableFormatConfig = common::shape::Tables;

/// The tables a user asked to fold or write out, read from the settings the wrapper handed over.
fn table_config(settings: &Settings) -> TableFormatConfig {
    TableFormatConfig::new(
        &settings.table_format,
        &settings.expand_tables,
        &settings.collapse_tables,
    )
}

/// The settings the source writes at `path`, read with the parser that reads the file itself.
#[cfg(feature = "extension-module")]
#[pyfunction]
fn settings_in<'py>(py: Python<'py>, content: &str, path: Vec<String>) -> PyResult<Option<Bound<'py, PyDict>>> {
    common::settings::settings_in(py, content, path)
}

#[cfg(feature = "extension-module")]
#[pyfunction]
#[pyo3(name = "format_toml")]
fn format_toml_py(py: Python<'_>, content: &str, opt: &Settings) -> PyResult<String> {
    py.detach(|| format_toml(content, opt)).map_err(PyValueError::new_err)
}

/// # Errors
///
/// Will return a message describing why the content was rejected, or why what the formatter wrote
/// is not a document.
pub fn format_toml(content: &str, opt: &Settings) -> Result<String, String> {
    common::formatted(content, |document| {
        format_core(document, opt);
        Ok(())
    })
}

/// Whether the file wrote a comment above the header or beside it.
fn carries_a_comment(section: &toml_doc::Section<'_>) -> bool {
    section.header.trail.comment.is_some()
        || section
            .header
            .lead
            .pieces()
            .iter()
            .any(|piece| matches!(piece, toml_doc::Piece::Comment { .. }))
}

fn format_core(document: &mut Document<'_>, opt: &Settings) {
    let table_config = table_config(opt);

    common::strings::normalize_key_quotes(document);
    // a table the file wrote with nothing under it says that it is there, so it is not one of the
    // tables the write-out below empties
    let written_empty: HashSet<Vec<String>> = document
        .sections
        .iter()
        .filter(|section| section.entries.is_empty())
        .map(|section| section.header.key.segments())
        .collect();
    // `[env]` holding `name.key` entries says the same thing as `[env.name]`, and the latter is the
    // form every other rule here is written against
    common::nesting::expand(document, "env");
    for name in nesting_targets(document) {
        // a setting names a table of any depth, so both passes run over every target: the fold takes
        // the children a setting asks to fold, and the write-out takes the ones it asks to keep
        common::nesting::collapse_of(
            document,
            &name,
            &|sub| table_config.should_collapse(sub),
            common::nesting::Width {
                column: opt.column_width,
                indent: opt.indent,
            },
        );
        common::nesting::expand_where(document, &name, &|child| !table_config.should_collapse(child));
    }

    // writing a table out empties the one its dotted keys were written under, and a header left with
    // nothing under it says nothing the file did not already say. One the file wrote a comment on
    // says what that comment says, so it stays to carry it
    let targets: HashSet<Vec<String>> = nesting_targets(document).into_iter().collect();
    document.sections.retain(|section| {
        let segments = section.header.key.segments();
        !section.entries.is_empty()
            || written_empty.contains(&segments)
            || carries_a_comment(section)
            || (segments != ["env"] && !targets.contains(&segments))
    });

    normalize_aliases(document);
    fix_root(document);
    fix_envs(document);
    sort_env_list(document, &opt.pin_envs);
    reorder_inline_tables(document);
    reorder_tables_with_pins(document, &opt.pin_envs);

    common::shape::Written {
        column_width: opt.column_width,
        indent: opt.indent,
        separate_root_table: &opt.separate_root_table,
        sub_table_spacing: &opt.sub_table_spacing,
        table_format: &opt.table_format,
        skip_wrap_for_keys: &opt.skip_wrap_for_keys,
        nested_prefixes: &["env_base", "env"],
    }
    .apply(document);
}

/// The tables that fold their own sub-tables in: the two fixed bases and each environment.
///
/// `env` itself is not one of them, since `[env.name]` holds one environment and folding it into
/// `env` would flatten away the grouping the file is written around.
fn nesting_targets(document: &toml_doc::Document<'_>) -> Vec<Vec<String>> {
    let mut names: Vec<Vec<String>> = vec![vec![String::from("env_run_base")], vec![String::from("env_pkg_base")]];
    let mut seen: HashSet<Vec<String>> = names.iter().cloned().collect();
    for section in &document.sections {
        let segments = section.header.key.segments();
        // an environment's name is the one segment the file gave it, whatever it holds, and a
        // reusable base is one of them under whichever of the two names it was written
        if segments.len() > 1 && (segments[0] == "env" || segments[0] == "env_base") {
            let head = segments[..2].to_vec();
            if seen.insert(head.clone()) {
                names.push(head);
            }
        }
    }
    names
}

/// # Panics
///
/// If the module cannot take one of its own members, which says the interpreter has run out of
/// memory rather than anything about the module.
// Gated so sibling crates (pyproject-fmt) that pull this in as an rlib don't get a duplicate `PyInit__lib` symbol from
// pyo3 at link time.
#[cfg(feature = "extension-module")]
#[pymodule(gil_used = false)]
#[pyo3(name = "_lib")]
pub fn _lib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let held = "the module takes its own members";
    m.add_function(wrap_pyfunction!(format_toml_py, m).expect(held))
        .expect(held);
    m.add_function(wrap_pyfunction!(settings_in, m).expect(held))
        .expect(held);
    m.add_class::<Settings>().expect(held);
    Ok(())
}
