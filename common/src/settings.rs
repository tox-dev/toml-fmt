//! Reading the settings a document holds for the formatter.
//!
//! The formatter reads TOML 1.1, so the settings written inside a file it accepts have to be read
//! by the same parser: a second reader of an older TOML would drop the file's own configuration on
//! a value it does not know, and format it as though none had been written.

use toml_doc::{Document, KeyValue, SectionKind, Value};

/// A value a settings table holds, in the forms a setting is written in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Setting {
    Text(String),
    Whole(i64),
    Truth(bool),
    List(Vec<Setting>),
    /// A table, which no setting is, kept so the caller can name the key that holds one.
    Table,
}

/// The settings written at `path`, in the order the file writes them, or `None` where the document
/// holds no such table.
///
/// The order is the file's own so that a caller reporting on a setting reports on the same one
/// every run.
///
/// # Errors
///
/// Returns the name of a setting written in a form no setting takes, or says that the settings
/// themselves are not one table.
pub fn read(document: &Document<'_>, path: &[String]) -> Result<Option<Vec<(String, Setting)>>, String> {
    let Some(entries) = entries_at(document, path)? else {
        return Ok(None);
    };
    entries
        .into_iter()
        .map(|(name, value)| {
            let read = match value {
                Some(value) => read_value(&name, value)?,
                None => Setting::Table,
            };
            Ok((name, read))
        })
        .collect::<Result<Vec<(String, Setting)>, String>>()
        .map(Some)
}

/// The name of every key the settings table holds, with its value where the key holds one and
/// `None` where it holds a table of its own.
type Named<'doc> = Vec<(String, Option<&'doc Value<'doc>>)>;

fn entries_at<'doc>(document: &'doc Document<'_>, path: &[String]) -> Result<Option<Named<'doc>>, String> {
    // a table the file repeats is a list of tables, and the settings written across its elements are
    // no one table however deep under it they sit
    for section in &document.sections {
        let named = section.header.key.segments();
        if section.header.kind == SectionKind::ArrayOfTables && path.starts_with(&named) {
            return Err(format!("{}: an array of tables holds no settings", path.join(".")));
        }
    }
    let mut held: Option<Named<'doc>> = None;
    for entry in &document.root {
        take(&mut held, path, &[], &entry.key_value)?;
    }
    for section in &document.sections {
        let under = section.header.key.segments();
        // a header names the table it opens whether or not the file wrote a key under it
        if let Some([name, ..]) = under.strip_prefix(path) {
            push(held.get_or_insert_with(Vec::new), name.clone(), None);
        }
        for entry in &section.entries {
            take(&mut held, path, &under, &entry.key_value)?;
        }
    }
    Ok(held)
}

/// Take from one key-value whatever it says about the settings, wherever the file wrote it: under a
/// header, in a dotted key, or inside a table written as a value.
fn take<'doc>(
    held: &mut Option<Named<'doc>>,
    path: &[String],
    under: &[String],
    key_value: &'doc KeyValue<'doc>,
) -> Result<(), String> {
    let named: Vec<String> = under.iter().cloned().chain(key_value.key.segments()).collect();
    if let Some(rest) = named.strip_prefix(path) {
        let found = held.get_or_insert_with(Vec::new);
        match rest {
            // the settings are written as the value of the key that names them
            [] => settings_inside(found, path, &key_value.value)?,
            [name] => push(found, name.clone(), Some(&key_value.value)),
            [name, ..] => push(found, name.clone(), None),
        }
        return Ok(());
    }
    // the settings sit inside this value, which is a table only where the file wrote one
    if path.starts_with(&named)
        && let Value::InlineTable(table) = &key_value.value
    {
        for member in &table.members {
            take(held, path, &named, &member.item)?;
        }
    }
    Ok(())
}

/// The settings a table written as a value holds.
fn settings_inside<'doc>(found: &mut Named<'doc>, path: &[String], value: &'doc Value<'doc>) -> Result<(), String> {
    let Value::InlineTable(table) = value else {
        return Err(format!("{}: the settings are not a table", path.join(".")));
    };
    for member in &table.members {
        let segments = member.item.key.segments();
        let name = segments.first().expect("a key names at least one part").clone();
        // a dotted name inside the table names a table of its own rather than a setting
        push(found, name, (segments.len() == 1).then_some(&member.item.value));
    }
    Ok(())
}

/// Hold the name where the file first wrote it, with the last thing the file said it holds.
fn push<'doc>(found: &mut Named<'doc>, name: String, value: Option<&'doc Value<'doc>>) {
    match found.iter_mut().find(|(held, _)| *held == name) {
        Some((_, held)) => *held = value,
        None => found.push((name, value)),
    }
}

fn read_value(name: &str, value: &Value<'_>) -> Result<Setting, String> {
    match value {
        Value::Scalar(repr) => {
            if repr.quoting().is_some() {
                let text = toml_doc::decode(repr).expect("the document was read before this");
                return Ok(Setting::Text(text));
            }
            match repr.text() {
                "true" => Ok(Setting::Truth(true)),
                "false" => Ok(Setting::Truth(false)),
                text => read_whole(text)
                    .map(Setting::Whole)
                    .ok_or_else(|| format!("{name}: {text} is not a setting")),
            }
        }
        Value::Array(array) => array
            .members
            .iter()
            .map(|member| read_value(name, &member.item))
            .collect::<Result<Vec<Setting>, String>>()
            .map(Setting::List),
        Value::InlineTable(_) => Ok(Setting::Table),
    }
}

/// A TOML integer, which writes its digits in any of four bases and may space them with `_`.
fn read_whole(text: &str) -> Option<i64> {
    let held: String = text.chars().filter(|held| *held != '_').collect();
    let (sign, digits) = match held.strip_prefix('-') {
        Some(rest) => (-1, rest.to_owned()),
        None => (1, held.strip_prefix('+').unwrap_or(&held).to_owned()),
    };
    let (radix, digits) = match digits.get(..2) {
        Some("0x") => (16, &digits[2..]),
        Some("0o") => (8, &digits[2..]),
        Some("0b") => (2, &digits[2..]),
        _ => (10, &digits[..]),
    };
    i64::from_str_radix(digits, radix).ok().map(|read| read * sign)
}

#[cfg(feature = "python")]
mod python {
    use pyo3::exceptions::{PySyntaxError, PyValueError};
    use pyo3::types::{PyDict, PyDictMethods, PyList, PyListMethods};
    use pyo3::{Bound, IntoPyObject, PyAny, PyResult, Python};

    use super::Setting;

    /// The settings the source writes at `path`, or `None` where it writes no such table.
    ///
    /// # Errors
    ///
    /// Raises `SyntaxError` where the source is not a document, which the formatter itself reports
    /// on, and `ValueError` where a setting is written in a form no setting takes.
    pub fn settings_in<'py>(py: Python<'py>, content: &str, path: Vec<String>) -> PyResult<Option<Bound<'py, PyDict>>> {
        let document = toml_doc::parse(content).map_err(|errors| PySyntaxError::new_err(errors[0].to_string()))?;
        let Some(held) = super::read(&document, &path).map_err(PyValueError::new_err)? else {
            return Ok(None);
        };
        let written = PyDict::new(py);
        for (name, value) in held {
            written.set_item(name, into_python(py, &value)?)?;
        }
        Ok(Some(written))
    }

    fn into_python<'py>(py: Python<'py>, value: &Setting) -> PyResult<Bound<'py, PyAny>> {
        match value {
            Setting::Text(text) => Ok(text.into_pyobject(py)?.into_any()),
            Setting::Whole(number) => Ok(number.into_pyobject(py)?.into_any()),
            Setting::Truth(truth) => Ok(truth.into_pyobject(py)?.to_owned().into_any()),
            // what a table holds is no part of the settings; the caller only has to name the key
            Setting::Table => Ok(PyDict::new(py).into_any()),
            Setting::List(held) => {
                let written = PyList::empty(py);
                for item in held {
                    written.append(into_python(py, item)?)?;
                }
                Ok(written.into_any())
            }
        }
    }
}

#[cfg(feature = "python")]
pub use python::settings_in;
