use std::collections::HashSet;
use std::string::String;

use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::{FromPyObjectOwned, PyModule, PyModuleMethods};
use pyo3::types::{PyAnyMethods, PyDict, PyDictMethods};
use pyo3::{pyclass, pyfunction, pymethods, pymodule, wrap_pyfunction, Bound, PyResult, Python};

use crate::global::reorder_tables;
use toml_doc::Document;

mod build_system;
mod dependency_groups;
mod project;
mod rules;

mod autopep8;
mod bandit;
mod black;
mod bumpversion;
mod check_manifest;
mod cibuildwheel;
mod codespell;
mod commitizen;
mod coverage;
mod deptry;
mod djlint;
mod docformatter;
mod global;
mod hatch;
mod interrogate;
mod isort;
mod maturin;
mod mypy;
mod pdm;
mod pixi;
mod poetry;
mod pylint;
mod pyproject_fmt;
mod pyrefly;
mod pyright;
mod pytest;
mod ruff;
mod scikit_build;
mod semantic_release;
mod setuptools;
mod towncrier;
mod tox;
mod ty;
mod uv;
mod vulture;
mod yapf;

#[pyclass(frozen, get_all)]
pub struct Settings {
    pub column_width: usize,
    pub indent: usize,
    pub keep_full_version: bool,
    pub max_supported_python: (u8, u8),
    pub min_supported_python: (u8, u8),
    pub generate_python_version_classifiers: bool,
    pub table_format: String,
    pub sub_table_spacing: String,
    pub separate_root_table: String,
    pub expand_tables: Vec<String>,
    pub collapse_tables: Vec<String>,
    pub skip_wrap_for_keys: Vec<String>,
}

#[pymethods]
impl Settings {
    #[new]
    #[pyo3(signature = (**kwargs))]
    pub fn new(kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
        const NAMES: &[&str] = &[
            "column_width",
            "indent",
            "keep_full_version",
            "max_supported_python",
            "min_supported_python",
            "generate_python_version_classifiers",
            "table_format",
            "sub_table_spacing",
            "separate_root_table",
            "expand_tables",
            "collapse_tables",
            "skip_wrap_for_keys",
        ];
        let kwargs = required_keywords(kwargs, NAMES)?;
        let column_width = required(kwargs, "column_width")?;
        let indent = required(kwargs, "indent")?;
        let keep_full_version = required(kwargs, "keep_full_version")?;
        let max_supported_python: (u8, u8) = required(kwargs, "max_supported_python")?;
        let min_supported_python: (u8, u8) = required(kwargs, "min_supported_python")?;
        let generate_python_version_classifiers = required(kwargs, "generate_python_version_classifiers")?;
        let table_format = required(kwargs, "table_format")?;
        let sub_table_spacing = required(kwargs, "sub_table_spacing")?;
        let separate_root_table = required(kwargs, "separate_root_table")?;
        let expand_tables: Vec<String> = required(kwargs, "expand_tables")?;
        let collapse_tables: Vec<String> = required(kwargs, "collapse_tables")?;
        let skip_wrap_for_keys: Vec<String> = required(kwargs, "skip_wrap_for_keys")?;
        // the classifiers this generates name Python 3, so a bound naming another major says
        // nothing this can act on
        for (name, version) in [
            ("max_supported_python", max_supported_python),
            ("min_supported_python", min_supported_python),
        ] {
            if version.0 != 3 {
                return Err(PyValueError::new_err(format!(
                    "{name} names Python {}, and only Python 3 is supported",
                    version.0
                )));
            }
        }
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
        for (setting, held) in [("skip_wrap_for_keys", &skip_wrap_for_keys)] {
            if held.iter().any(|name| name.trim().is_empty()) {
                return Err(PyValueError::new_err(format!(
                    "{setting}: a name is written there, not nothing"
                )));
            }
        }
        Ok(Self {
            column_width,
            indent,
            keep_full_version,
            max_supported_python,
            min_supported_python,
            generate_python_version_classifiers,
            table_format,
            sub_table_spacing,
            separate_root_table,
            expand_tables,
            collapse_tables,
            skip_wrap_for_keys,
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
#[pyfunction]
fn settings_in<'py>(py: Python<'py>, content: &str, path: Vec<String>) -> PyResult<Option<Bound<'py, PyDict>>> {
    common::settings::settings_in(py, content, path)
}

#[pyfunction]
#[pyo3(name = "format_toml")]
fn format_toml_py(py: Python<'_>, content: &str, opt: &Settings) -> PyResult<String> {
    py.detach(|| format_toml(content, opt)).map_err(PyValueError::new_err)
}

/// # Errors
///
/// Will return a message describing why the content was rejected, e.g. an invalid `project.version`.
pub fn format_toml(content: &str, opt: &Settings) -> Result<String, String> {
    common::formatted(content, |document| format_core(document, opt))
}

fn format_core(document: &mut Document<'_>, opt: &Settings) -> Result<(), String> {
    let table_config = table_config(opt);

    common::strings::normalize_key_quotes(document);
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

    build_system::fix(document, opt.keep_full_version);
    project::fix(
        document,
        opt.keep_full_version,
        opt.max_supported_python,
        opt.min_supported_python,
        opt.generate_python_version_classifiers,
        &table_config,
    )?;
    dependency_groups::fix(document, opt.keep_full_version);
    rules::fix(document);
    for fix in [
        poetry::fix,
        mypy::fix,
        setuptools::fix,
        hatch::fix,
        pyright::fix,
        pdm::fix,
        cibuildwheel::fix,
    ] {
        fix(document);
    }
    tox::fix(
        document,
        &table_config,
        common::nesting::Width {
            column: opt.column_width,
            indent: opt.indent,
        },
    );
    towncrier::fix(document);

    reorder_tables(document);
    poetry::reorder_inline_tables(document);
    mypy::reorder_inline_tables(document);
    setuptools::reorder_inline_tables(document);
    tox::reorder_inline_tables(document);

    common::shape::Written {
        column_width: opt.column_width,
        indent: opt.indent,
        separate_root_table: &opt.separate_root_table,
        sub_table_spacing: &opt.sub_table_spacing,
        table_format: &opt.table_format,
        skip_wrap_for_keys: &opt.skip_wrap_for_keys,
        nested_prefixes: &["tool"],
    }
    .apply(document);

    Ok(())
}

/// Every table that could hold sub-tables: the two fixed roots and each tool that appears.
fn nesting_targets(document: &Document<'_>) -> Vec<Vec<String>> {
    let mut names = vec![vec![String::from("build-system")], vec![String::from("project")]];
    let mut seen: HashSet<Vec<String>> = names.iter().cloned().collect();
    for section in &document.sections {
        let segments = section.header.key.segments();
        // a tool's own name is one segment, whatever it holds
        if segments.len() > 1 && segments[0] == "tool" {
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
