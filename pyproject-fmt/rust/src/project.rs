use std::cmp::Ordering;
use std::collections::HashSet;
use std::sync::LazyLock;

use lexical_sort::natural_lexical_cmp;
use regex::Regex;

use common::arrays::{dedupe_strings_in, map_strings, sort, sort_names_in, string_of};
use common::nesting::Width;
use common::pep508::{is_valid_version, Requirement};
use common::pep508::{Number, Operator, Version, VersionOp};
use common::sections;
use toml_doc::{
    Array, Comment, Document, Entry, InlineTable, Key, KeyValue, LineEnding, Member, Pad, Piece, Section, SectionKind,
    Trail, Trivia, Value,
};

use crate::TableFormatConfig;

pub const KEY_ORDER: &[&str] = &[
    "name",
    "version",
    "import-names",
    "import-namespaces",
    "description",
    "readme",
    "keywords",
    "license",
    "license-files",
    "maintainers",
    "authors",
    "requires-python",
    "classifiers",
    "dynamic",
    "dependencies",
    // these go at the end as they may be inline or exploded
    "optional-dependencies",
    "urls",
    "scripts",
    "gui-scripts",
    "entry-points",
];

/// Requirements sort by the name they install, so `Flask` and `flask-cors` land next to each other,
/// and by the whole line when two share a name.
fn normalize_and_sort_requirements(value: &mut Value<'_>, keep_full_version: bool) {
    let Value::Array(array) = value else { return };
    // a requirement this parser cannot read is left as the file wrote it
    map_strings(array, |text| {
        Requirement::new(text).map_or_else(
            |_| text.to_owned(),
            |found| found.normalize(keep_full_version).to_string(),
        )
    });
    sort(
        array,
        &|member| {
            let text = string_of(member)?;
            let name = Requirement::new(&text).map_or_else(|_| text.clone(), |found| found.canonical_name());
            Some((name, text))
        },
        &|left: &(String, String), right: &(String, String)| {
            natural_lexical_cmp(&left.0, &right.0).then_with(|| natural_lexical_cmp(&left.1, &right.1))
        },
    );
}

/// # Errors
///
/// Will return the offending value if `project.version` is not a valid PEP 440 version.
pub fn fix(
    document: &mut Document<'_>,
    keep_full_version: bool,
    max_supported_python: (u8, u8),
    min_supported_python: (u8, u8),
    generate_python_version_classifiers: bool,
    table_config: &TableFormatConfig,
) -> Result<(), String> {
    let key_order = &["name", "email"];

    for people in ["authors", "maintainers"] {
        let name = ["project", people].map(str::to_owned);
        if table_config.should_collapse(&name) {
            common::nesting::collapse_array_of_tables(document, &format!("project.{people}"), Width::unbounded());
        } else {
            expand_array_of_tables(document, &format!("project.{people}"), key_order);
        }
        // whichever form the file is left in, a person reads name first
        sections::for_array_elements(document, &name, key_order, &mut |_, _| {});
    }

    let path = sections::parse_name("project");
    static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r" \.(\W)").unwrap());
    expand_entry_points_inline_tables(document, &path);

    let mut invalid_version = None;
    sections::for_keys_under(document, &path, |key, value| match dispatch_on(key) {
        "name" => {
            // the project's name is one distribution name, not a dependency: text that names
            // anything else is left as the file wrote it
            common::strings::update(value, |text| {
                Requirement::canonical_name_of(text).unwrap_or_else(|_| text.to_owned())
            });
        }
        "version" => {
            if let Some(raw) = common::strings::text_of(value) {
                if !is_valid_version(&raw) {
                    invalid_version = Some(raw);
                }
            }
        }
        // the deprecated license table holds a path and the license text itself, neither of which is
        // an SPDX expression: only the string spelling names one
        "license" => {
            common::strings::update(value, |text| {
                // text that is not an expression is a license description the file wrote, and
                // uppercasing the words inside it would rewrite what it says
                if !names_a_license(text) {
                    return text.to_owned();
                }
                text.split_whitespace()
                    .map(|token| {
                        if ["and", "or", "with"]
                            .iter()
                            .any(|held| token.eq_ignore_ascii_case(held))
                        {
                            token.to_uppercase()
                        } else {
                            token.to_owned()
                        }
                    })
                    .collect::<Vec<String>>()
                    .join(" ")
            });
        }
        "description" => {
            common::strings::update(value, |text| {
                // the whole description is read as one run of words, so a line break says what a
                // space says and an empty line says nothing at all
                let words: Vec<&str> = text.split_whitespace().collect();
                RE.replace_all(&words.join(" "), ".$1").to_string()
            });
        }
        "requires-python" => {
            // whitespace between the clauses says nothing, while whitespace inside one is what
            // makes the text something PEP 440 does not read: taking it out would write a
            // constraint the file does not name
            common::strings::update(value, |text| {
                read_specifiers(text).map_or_else(|| text.to_owned(), |_| text.split_whitespace().collect())
            });
        }
        "dependencies" | "optional-dependencies" => {
            normalize_and_sort_requirements(value, keep_full_version);
        }
        "dynamic" => {
            sort_names_in(value);
        }
        "keywords" => {
            dedupe_strings_in(value, &|text| text.to_lowercase());
            sort_names_in(value);
        }
        "import-names" | "import-namespaces" => {
            if let Value::Array(array) = value {
                map_strings(array, |text| import_name(text).unwrap_or_else(|| text.to_owned()));
            }
            sort_names_in(value);
        }
        "classifiers" => {
            // a classifier is one of a fixed set of strings, so two that differ in case are an
            // invalid spelling beside a valid one rather than the same claim twice
            dedupe_strings_in(value, &ToOwned::to_owned);
            sort_names_in(value);
        }
        "authors" | "maintainers" => {}
        _ => {}
    });
    if let Some(raw) = invalid_version {
        return Err(format!("project.version `{raw}` is not a valid PEP 440 version"));
    }

    generate_classifiers(
        document,
        &path,
        max_supported_python,
        min_supported_python,
        generate_python_version_classifiers,
    );

    sections::for_keys_under(document, &path, |key, value| {
        if key == "classifiers" {
            dedupe_strings_in(value, &ToOwned::to_owned);
            sort_names_in(value);
        }
    });
    normalize_extra_names(document, &path);
    sections::reorder_under(document, &path, KEY_ORDER);
    Ok(())
}

/// `entry-points.group = { name = "target" }` says the same thing as `entry-points.group.name =
/// "target"`, and the dotted form is what the key order and the rest of the rules expect.
/// What a rule reads a key by: its first segment, except where the key names something under it
/// that the rule is not about, as the deprecated `license.file` and `license.text` are.
fn dispatch_on(key: &str) -> &str {
    match key {
        "license.file" | "license.text" => "",
        _ => key.split('.').next().unwrap_or_default(),
    }
}

fn expand_entry_points_inline_tables(document: &mut Document<'_>, path: &[String]) {
    sections::for_entry_runs(document, path, |entries, under| {
        expand_entry_points_in(entries, under, path);
    });
}

fn expand_entry_points_in(entries: &mut Vec<Entry<'_>>, under: &[String], path: &[String]) {
    let mut expanded: Vec<Entry<'_>> = Vec::new();
    for entry in std::mem::take(entries) {
        let named: Vec<String> = under.iter().chain(&entry.key_value.key.segments()).cloned().collect();
        let key = named
            .strip_prefix(path)
            .map_or_else(String::new, common::sections::dotted_name);
        let Value::InlineTable(table) = &entry.key_value.value else {
            expanded.push(entry);
            continue;
        };
        // the padding before the closing brace can hold a comment, and no dotted key has a place
        // for it, so the table stays as it is. A disabled entry is one the comment beside it speaks
        // for, and neither of the keys it would split into can carry that
        if !key.starts_with("entry-points.")
            || table.members.is_empty()
            || table.trailing.has_comment()
            || common::disabled::is_enabled_here(&entry)
        {
            expanded.push(entry);
            continue;
        }
        let last = table.members.len() - 1;
        for (index, member) in table.members.iter().enumerate() {
            let mut written = Entry {
                // what led the table leads the first key it becomes, and each member keeps the
                // comments written around it
                lead: if index == 0 {
                    entry.lead.clone()
                } else {
                    Trivia::default()
                },
                indent: entry.indent.clone(),
                key_value: KeyValue {
                    // the parent's key parts carry over as written, quoting included
                    key: Key::from_parts(
                        entry
                            .key_value
                            .key
                            .parts()
                            .iter()
                            .cloned()
                            .chain(member.item.key.parts().iter().cloned())
                            .collect(),
                    ),
                    pre_eq: " ".into(),
                    post_eq: " ".into(),
                    value: member.item.value.clone(),
                },
                trail: Trail {
                    ws: "".into(),
                    // the comment closing the table's line closes the last key it becomes
                    comment: (index == last).then(|| entry.trail.comment.clone()).flatten(),
                    ending: entry.trail.ending,
                },
            };
            for text in comments_around(member) {
                written.lead.pieces_mut().push(Piece::Comment {
                    indent: "".into(),
                    // a comment runs to the end of its line, so it takes one of its own however the
                    // file it came from ended
                    text,
                    ending: LineEnding::Lf,
                });
            }
            expanded.push(written);
        }
    }
    *entries = expanded;
}

/// Whether the table carries a comment anywhere inside it.
fn commented(table: &InlineTable<'_>) -> bool {
    table.trailing.has_comment()
        || table
            .members
            .iter()
            .any(|inner| inner.lead.has_comment() || inner.trail.has_comment() || inner.after.has_comment())
}

/// The comments a multiline inline table wrote around one of its members, which the key that member
/// becomes takes with it.
fn comments_around<'a>(member: &Member<'a, KeyValue<'a>>) -> Vec<Comment<'a>> {
    member
        .lead
        .parts()
        .iter()
        .chain(member.trail.parts())
        .chain(member.after.parts())
        .filter_map(|part| match part {
            Pad::Comment(text) => Some(text.clone()),
            Pad::Space(_) | Pad::Newline(_) => None,
        })
        .collect()
}

/// The canonical spelling of the import name, or `None` where the text is not one.
///
/// [PEP 794](https://peps.python.org/pep-0794/) writes a dotted name of Python identifiers, and
/// lets one modifier follow it: the word `private`. Text that says anything else is left alone.
fn import_name(text: &str) -> Option<String> {
    let (name, modifier) = match text.split_once(';') {
        Some((name, rest)) => (name.trim(), Some(rest.trim())),
        None => (text.trim(), None),
    };
    if name.is_empty() || !name.split('.').all(is_an_identifier) {
        return None;
    }
    match modifier {
        None => Some(name.to_owned()),
        Some("private") => Some(format!("{name}; private")),
        Some(_) => None,
    }
}

/// Whether the text is one segment of a dotted import name, which Python spells as an identifier.
fn is_an_identifier(segment: &str) -> bool {
    let mut held = segment.chars();
    held.next().is_some_and(|first| first == '_' || first.is_alphabetic())
        && held.all(|held| held == '_' || held.is_alphanumeric())
}

/// Whether the text is an SPDX license expression: registered license identifiers joined by `AND`,
/// `OR` and `WITH`, with parentheses around whichever parts a file chooses to group.
///
/// Text that is not one is a license description the file wrote, and there is no telling prose
/// from an expression by shape alone: `MIT or later` is shaped like one and names no license.
fn names_a_license(text: &str) -> bool {
    spdx::Expression::parse(text).is_ok()
}

/// Whether the project leaves the classifiers, or what they are read from, to its build backend: a
/// field a file writes as dynamic is one it must not also write out.
fn names_dynamic(document: &mut Document<'_>, path: &[String]) -> bool {
    let mut held = false;
    sections::for_keys_under(document, path, |key, value| {
        if key != "dynamic" {
            return;
        }
        if let Value::Array(array) = value {
            held = held
                || array
                    .members
                    .iter()
                    .filter_map(common::arrays::string_of)
                    .any(|name| name == "classifiers" || name == "requires-python");
        }
    });
    held
}

fn generate_classifiers(
    document: &mut Document<'_>,
    path: &[String],
    max_supported_python: (u8, u8),
    min_supported_python: (u8, u8),
    generate_python_version_classifiers: bool,
) {
    if !generate_python_version_classifiers || names_dynamic(document, path) {
        return;
    }
    let (held, existing) = supported_minors_with_classifier(document, path, max_supported_python, min_supported_python);
    let Some(held) = held else {
        return;
    };
    let Some(existing) = existing else {
        // written as something other than a list of classifiers; a second key would say it twice
        if held.minors.is_empty() || names_classifiers(document, path) || !holds_a_project(document, path) {
            return;
        }
        let mut written = Array::default();
        apply_classifiers(&mut written, &held, &HashSet::new());
        write_classifiers(document, path, Value::Array(written));
        return;
    };
    // `existing` was read off the same array, so the key is there and holds one
    sections::for_keys_under(document, path, |key, value| {
        let Some(Value::Array(array)) = (key == "classifiers").then_some(value) else {
            return;
        };
        apply_classifiers(array, &held, &existing);
    });
}

/// Whether the file says there is a project at all: a header naming it or something under it, or a
/// key of its own the file wrote.
///
/// A key the file wrote as a comment declares nothing, so a file holding only one gains no key.
fn holds_a_project(document: &mut Document<'_>, path: &[String]) -> bool {
    if document
        .sections
        .iter()
        .any(|section| section.header.key.segments().starts_with(path))
    {
        return true;
    }
    // a table written as a value is the table it names, empty or not
    let mut held = false;
    sections::for_table_at(document, path, |_| held = true);
    sections::for_keys_under(document, path, |_, _| held = true);
    held
}

/// Whether the project names its classifiers at all, in whatever the key holds.
fn names_classifiers(document: &mut Document<'_>, path: &[String]) -> bool {
    let mut held = false;
    sections::for_keys_under(document, path, |key, _| held = held || key == "classifiers");
    held
}

/// Write the classifiers into the run of keys the project is already written in, so the file keeps
/// the shape its author gave it.
///
/// Only a run written at or above the project can hold a key of it: one under a table of its own
/// would say something about that table instead.
fn write_classifiers(document: &mut Document<'_>, path: &[String], value: Value<'static>) {
    // a table written as a value is closed where the file wrote it, so a key of it goes inside. A
    // path names one table, so what goes in goes in once
    let mut held = Some(value);
    sections::for_table_at(document, path, |table| {
        let value = held.take().expect("a path names one table");
        table
            .members
            .push(common::build::member(common::build::key_value("classifiers", value)));
    });
    let Some(value) = held else {
        return;
    };
    // the header the file wrote for the table is where a key of it belongs, whether or not it holds
    // one already
    if let Some(section) = document
        .sections
        .iter_mut()
        .find(|section| section.header.key.segments() == path)
    {
        push_classifiers(&mut section.entries, &[], value);
        return;
    }
    let mut written = Some(value);
    sections::for_entry_runs(document, path, |entries, under| {
        let Some(value) = written.take() else {
            return;
        };
        let Some(rest) = path.strip_prefix(under) else {
            written = Some(value);
            return;
        };
        // the keys of the project are written in this run, so the one this adds belongs with them
        let holds = sections::active(entries).any(|entry| {
            let named: Vec<String> = under.iter().chain(&entry.key_value.key.segments()).cloned().collect();
            named.strip_prefix(path).is_some_and(|tail| !tail.is_empty())
        });
        if !holds {
            written = Some(value);
            return;
        }
        push_classifiers(entries, rest, value);
    });
    // no run of the file writes a key of the project, so the key that says it goes before the first
    // header, where it names the whole path it belongs to
    if let Some(value) = written {
        push_classifiers(&mut document.root, path, value);
    }
}

fn push_classifiers(entries: &mut Vec<Entry<'_>>, under: &[String], value: Value<'static>) {
    let named: Vec<&str> = under.iter().map(String::as_str).chain(["classifiers"]).collect();
    let mut entry = common::build::entry("classifiers", value);
    entry.key_value.key = Key::new(named);
    entries.push(entry);
}

/// The set of `Programming Language :: Python` classifiers the declared range implies. Ones outside
/// the range go, ones inside that are missing arrive, and everything else is left alone.
fn apply_classifiers(array: &mut Array<'_>, held: &Supported, existing: &HashSet<String>) {
    let mut must_have: HashSet<String> = HashSet::new();
    // a project no Python 3 release satisfies is not a Python 3 project, and neither is one a
    // Python 2 release still runs
    if !held.minors.is_empty() && held.only_python_3 {
        must_have.insert(String::from("Programming Language :: Python :: 3 :: Only"));
    }
    must_have.extend(
        held.minors
            .iter()
            .map(|minor| format!("Programming Language :: Python :: 3.{minor}")),
    );

    common::arrays::retain_strings(array, |text| {
        !text.starts_with("Programming Language :: Python :: 3") || must_have.contains(text)
    });
    // a minor the range admits is one the file may keep, but only the ones it says most about are
    // worth writing down where the file does not name them
    let written: HashSet<String> = must_have
        .iter()
        .filter(|text| {
            text.strip_prefix("Programming Language :: Python :: 3.")
                .and_then(|minor| minor.parse::<u8>().ok())
                .is_none_or(|minor| minor >= held.write_from)
        })
        .cloned()
        .collect();
    let mut to_add: Vec<&str> = written.difference(existing).map(String::as_str).collect();
    to_add.sort_unstable();
    for add in to_add {
        array.members.push(common::build::member(common::build::string(add)));
    }
    // a trailing comma holds the array open, which is where a generated classifier list belongs
    array.trailing_comma = true;
}

/// What the project says it supports: the Python 3 minor versions, the oldest of them worth
/// writing down, and whether Python 3 is the only major it admits.
struct Supported {
    minors: Vec<u8>,
    /// A range says how old a release may be only where it names a lower bound; where it does not,
    /// the configured minimum decides how far back the file is given classifiers, while every
    /// minor the range admits is still one the file may keep.
    write_from: u8,
    only_python_3: bool,
}

type SupportedWithClassifier = (Option<Supported>, Option<HashSet<String>>);

/// The Python 3 minor versions the project supports, and the classifiers it already names.
///
/// A minor version is supported when some release in that series satisfies every clause of
/// `requires-python`; text that is not a specifier set leaves the configured window as it is.
fn supported_minors_with_classifier(
    document: &mut Document<'_>,
    path: &[String],
    max_supported_python: (u8, u8),
    min_supported_python: (u8, u8),
) -> SupportedWithClassifier {
    let mut classifiers: Option<HashSet<String>> = None;
    let mut requires: Option<String> = None;
    sections::for_keys_under(document, path, |key, value| match key {
        "requires-python" => requires = common::strings::text_of(value),
        "classifiers" => {
            if let Value::Array(array) = value {
                classifiers = Some(array.members.iter().filter_map(string_of).collect());
            }
        }
        _ => {}
    });
    let Some(text) = requires else {
        // with nothing said about it, what the formatter was configured with stands
        let held = Supported {
            minors: (min_supported_python.1..=max_supported_python.1).collect(),
            write_from: min_supported_python.1,
            only_python_3: true,
        };
        return (Some(held), classifiers);
    };
    // a constraint this cannot read still says what the project supports, and the configured window
    // would say something else in its place
    let Some(clauses) = read_specifiers(&text) else {
        return (None, classifiers);
    };
    let held = Supported {
        minors: (0..=max_supported_python.1)
            .filter(|minor| series_holds_a_release(&clauses, 3, *minor))
            .collect(),
        write_from: if clauses.iter().any(names_a_lower_bound) {
            0
        } else {
            min_supported_python.1
        },
        // `3 :: Only` says no other major runs it, which a constraint admitting a Python 2 does not
        only_python_3: !(0..=PYTHON_2_MINORS).any(|minor| series_holds_a_release(&clauses, 2, minor)),
    };
    (Some(held), classifiers)
}

/// Whether the clause says how old a release may be, which is what gives the range a floor of its
/// own to stand on.
fn names_a_lower_bound(clause: &VersionOp) -> bool {
    matches!(
        clause.op,
        Operator::GreaterEqual
            | Operator::GreaterThan
            | Operator::Equal
            | Operator::ArbitraryEqual
            | Operator::Compatible
    )
}

/// Python 2 stopped at 2.7, so nothing above it names a release.
const PYTHON_2_MINORS: u8 = 7;

/// The clauses of a specifier set, or `None` when the text is not one this can read.
fn read_specifiers(text: &str) -> Option<Vec<VersionOp>> {
    text.split(',').map(|part| VersionOp::new(part).ok()).collect()
}

/// Whether one release of Python `3.<minor>` satisfies every clause.
///
/// An interpreter of that series says its version as `major.minor.micro`, so the series is a window
/// over the micro versions one of them can have. Each clause narrows the window, and what is left is
/// what one interpreter can report.
fn series_holds_a_release(clauses: &[VersionOp], major: u64, minor: u8) -> bool {
    let mut window = Window::default();
    for clause in clauses {
        // `===` compares the text rather than the version it reads as, so an interpreter satisfies
        // it only where its own three numbers are written that way
        if clause.op == Operator::ArbitraryEqual {
            let Some(micro) = names_a_micro(clause.literal(), major, u64::from(minor)) else {
                return false;
            };
            window.only_micro(micro);
            continue;
        }
        // only `===` takes text no version reads from, and it was read above
        let version = clause.version().expect("the clause names a version");
        window.narrow(&clause.op, version, major, u64::from(minor));
    }
    window.holds_a_release()
}

/// The micro version the text says, where it says a release of that series the way an interpreter
/// writes its own version.
fn names_a_micro(literal: &str, major: u64, minor: u64) -> Option<Number> {
    Number::written(literal.strip_prefix(&format!("{major}.{minor}."))?)
}

/// What a release of one minor series may name as its micro version.
#[derive(Default)]
struct Window {
    low: Option<(Number, bool)>,
    high: Option<(Number, bool)>,
    excluded: HashSet<Number>,
    empty: bool,
}

/// What a wildcard's numbers match in one minor series.
enum Match {
    /// Every release of the series.
    Series,
    /// The one release naming this micro version.
    Micro(Number),
    /// No release of the series.
    None,
}

impl Window {
    fn narrow(&mut self, op: &Operator, version: &Version, major: u64, minor: u64) {
        // an ordinary Python release names no epoch, so a bound that names one is above every one of
        // them and a bound below it rules out none
        if version.epoch.as_ref().is_some_and(|epoch| !epoch.is_zero()) {
            match op {
                Operator::LessEqual | Operator::LessThan | Operator::NotEqual => {}
                _ => self.empty = true,
            }
            return;
        }
        // the series is named by two numbers, which Python writes small: one too large for a machine
        // integer names no series either way
        let named = (
            version.release.first().map_or(0, Number::saturating),
            version.release.get(1).map_or(0, Number::saturating),
        );
        let tail = version.release.get(2..).unwrap_or_default();
        match op {
            Operator::GreaterEqual | Operator::GreaterThan => match named.cmp(&(major, minor)) {
                Ordering::Greater => self.empty = true,
                Ordering::Less => {}
                Ordering::Equal => {
                    let inclusive = match compare_micro(&micro_of(tail), tail, version) {
                        Ordering::Greater => true,
                        Ordering::Equal => *op == Operator::GreaterEqual,
                        Ordering::Less => false,
                    };
                    self.at_least(micro_of(tail), inclusive);
                }
            },
            Operator::LessEqual | Operator::LessThan => match named.cmp(&(major, minor)) {
                Ordering::Less => self.empty = true,
                Ordering::Greater => {}
                Ordering::Equal => {
                    let inclusive = match compare_micro(&micro_of(tail), tail, version) {
                        Ordering::Less => true,
                        Ordering::Equal => *op == Operator::LessEqual,
                        Ordering::Greater => false,
                    };
                    self.at_most(micro_of(tail), inclusive);
                }
            },
            Operator::Equal | Operator::ArbitraryEqual => self.only(version, named, tail, major, minor),
            Operator::NotEqual => self.without(version, named, tail, major, minor),
            // `~=X.Y.Z` says `>=X.Y.Z` and `==X.Y.*`, so the series it names is the one a component
            // short
            Operator::Compatible => {
                match wildcard_match(&version.release[..version.release.len() - 1], major, minor) {
                    Match::None => {
                        self.empty = true;
                        return;
                    }
                    Match::Micro(micro) => self.only_micro(micro),
                    Match::Series => {}
                }
                match named.cmp(&(major, minor)) {
                    Ordering::Greater => self.empty = true,
                    Ordering::Less => {}
                    Ordering::Equal => {
                        let inclusive = compare_micro(&micro_of(tail), tail, version) != Ordering::Less;
                        self.at_least(micro_of(tail), inclusive);
                    }
                }
            }
        }
    }

    /// `==`, which names one release or, with a wildcard, everything the numbers open with.
    fn only(&mut self, version: &Version, named: (u64, u64), tail: &[Number], major: u64, minor: u64) {
        if version.has_wildcard {
            match wildcard_match(&version.release, major, minor) {
                Match::Series => {}
                Match::Micro(micro) => self.only_micro(micro),
                Match::None => self.empty = true,
            }
            return;
        }
        // no release of the series is the one a pre, dev, post or local version names
        if named != (major, minor) || compare_micro(&micro_of(tail), tail, version) != Ordering::Equal {
            self.empty = true;
            return;
        }
        self.only_micro(micro_of(tail));
    }

    /// `!=`, which rules out one release or, with a wildcard, everything the numbers open with.
    fn without(&mut self, version: &Version, named: (u64, u64), tail: &[Number], major: u64, minor: u64) {
        if version.has_wildcard {
            match wildcard_match(&version.release, major, minor) {
                Match::Series => self.empty = true,
                Match::Micro(micro) => {
                    self.excluded.insert(micro);
                }
                Match::None => {}
            }
            return;
        }
        // a version outside the series rules out nothing in it, and neither does one no release of
        // the series is written as
        if named == (major, minor) && compare_micro(&micro_of(tail), tail, version) == Ordering::Equal {
            self.excluded.insert(micro_of(tail));
        }
    }

    /// Hold the window to the one release naming this micro version.
    fn only_micro(&mut self, micro: Number) {
        self.at_least(micro.clone(), true);
        self.at_most(micro, true);
    }

    fn at_least(&mut self, bound: Number, inclusive: bool) {
        let held = match &self.low {
            Some((low, low_inclusive)) => match bound.cmp(low) {
                Ordering::Greater => true,
                Ordering::Equal => *low_inclusive && !inclusive,
                Ordering::Less => false,
            },
            None => true,
        };
        if held {
            self.low = Some((bound, inclusive));
        }
    }

    fn at_most(&mut self, bound: Number, inclusive: bool) {
        let held = match &self.high {
            Some((high, high_inclusive)) => match bound.cmp(high) {
                Ordering::Less => true,
                Ordering::Equal => *high_inclusive && !inclusive,
                Ordering::Greater => false,
            },
            None => true,
        };
        if held {
            self.high = Some((bound, inclusive));
        }
    }

    /// Whether an interpreter of the series fits the window: whether some micro version lies inside
    /// it and is not one the clauses rule out.
    fn holds_a_release(&self) -> bool {
        if self.empty {
            return false;
        }
        let mut candidate = match &self.low {
            Some((low, true)) => low.clone(),
            Some((low, false)) => low.succ(),
            None => Number::zero(),
        };
        let highest = match &self.high {
            Some((high, true)) => Some(high.clone()),
            Some((high, false)) => match high.pred() {
                Some(below) => Some(below),
                None => return false,
            },
            None => None,
        };
        // each turn either finds a release or passes one the clauses rule out, and they rule out
        // only so many
        loop {
            if highest.as_ref().is_some_and(|highest| candidate > *highest) {
                return false;
            }
            if !self.excluded.contains(&candidate) {
                return true;
            }
            candidate = candidate.succ();
        }
    }
}

/// The micro version the numbers name, which is zero where they name none.
fn micro_of(tail: &[Number]) -> Number {
    tail.first().cloned().unwrap_or_else(Number::zero)
}

/// Where an interpreter's micro version stands against a version of the same series.
///
/// A release the file names before the final one is under it, and one it names after is above it,
/// so the plain release sits on one side of the version however it was written.
fn compare_micro(micro: &Number, tail: &[Number], version: &Version) -> Ordering {
    let held = match tail.split_first() {
        Some((first, rest)) => micro.cmp(first).then_with(|| {
            if rest.iter().all(Number::is_zero) {
                Ordering::Equal
            } else {
                Ordering::Less
            }
        }),
        None => micro.cmp(&Number::zero()),
    };
    if held != Ordering::Equal {
        return held;
    }
    // PEP 440 reads the suffixes in this order: a pre release stays under the final one whatever
    // follows it, a post release stays above it, and a dev release on its own is under it
    if version.pre.is_some() {
        return Ordering::Greater;
    }
    if version.post.is_some() {
        return Ordering::Less;
    }
    if version.dev.is_some() {
        return Ordering::Greater;
    }
    if version.local.is_some() {
        return Ordering::Less;
    }
    Ordering::Equal
}

/// What the numbers a wildcard opens with match in one minor series.
///
/// A prefix is met by a release that opens with it once both are written out to the same length, so
/// the zeros a prefix ends with match a release that leaves them unwritten.
fn wildcard_match(release: &[Number], major: u64, minor: u64) -> Match {
    if release.first().map_or(0, Number::saturating) != major {
        return Match::None;
    }
    let Some(named) = release.get(1) else {
        return Match::Series;
    };
    if named.saturating() != minor {
        return Match::None;
    }
    let Some((first, rest)) = release.get(2..).unwrap_or_default().split_first() else {
        return Match::Series;
    };
    if rest.iter().all(Number::is_zero) {
        Match::Micro(first.clone())
    } else {
        Match::None
    }
}

/// Extra names compare with `-`, `_` and `.` treated alike, so `My_Extra` and `my-extra` name one
/// extra; writing the normalized spelling keeps them from looking like two.
fn normalize_extra_names(document: &mut Document<'_>, path: &[String]) {
    static SEPARATORS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[-_.]+").expect("static pattern"));
    // two spellings of one extra are two keys until one of them is rewritten, and writing the same
    // key twice is something no TOML document can say
    let mut taken: HashSet<Vec<String>> = HashSet::new();
    sections::for_names_under(document, path, |tail, _| {
        taken.insert(tail.to_vec());
    });
    sections::for_names_under(document, path, |tail, key| {
        // the group's name is the one segment after the field, whatever characters it holds
        let [field, extra] = tail else {
            return;
        };
        // an extra is a distribution name, and text that is not one is left for the backend to
        // report rather than rewritten into something else that is not one either
        if field != "optional-dependencies" || !common::pep508::names_a_distribution(extra) {
            return;
        }
        let lowered = extra.to_lowercase();
        let normalized = SEPARATORS.replace_all(&lowered, "-");
        if *extra == normalized {
            return;
        }
        let written = vec![field.clone(), normalized.to_string()];
        if taken.contains(&written) {
            return;
        }
        key.parts_mut()
            .last_mut()
            .expect("a key names at least one segment")
            .set_name(&normalized);
        taken.insert(written);
    });
}

/// `authors = [{ name = "..." }]` written out as `[[project.authors]]`, which is the long form the
/// table_format setting asks for.
fn expand_array_of_tables(document: &mut Document<'_>, full_name: &str, key_order: &[&str]) {
    let (parent_name, field_name) = full_name.split_once('.').expect("the name carries its parent");
    if !common::sections::named(document, full_name).is_empty() {
        return;
    }
    let Some(parent) = sections::first(document, parent_name) else {
        return;
    };
    let Some(at) = parent
        .entries
        .iter()
        .position(|entry| entry.key_value.key.is_path(field_name))
    else {
        return;
    };
    let Value::Array(array) = &parent.entries[at].key_value.value else {
        return;
    };

    // every member has to have a written form, or the array stays as it is: writing out only the
    // ones that convert would drop the rest of what the file says. A comment inside the array has
    // no one place to go among the headers it becomes, and the one closing the entry's line has
    // none either.
    if array.trailing.has_comment() || parent.entries[at].trail.comment.is_some() {
        return;
    }
    let mut written: Vec<Section<'_>> = Vec::new();
    for member in &array.members {
        let Value::InlineTable(table) = &member.item else {
            return;
        };
        if member.lead.has_comment() || member.trail.has_comment() || member.after.has_comment() || commented(table) {
            return;
        }
        let mut fields: Vec<Entry<'_>> = table
            .members
            .iter()
            .map(|inner| {
                // the member's own segments carry over, so a quoted name stays quoted
                let mut written = common::build::entry("placeholder", inner.item.value.clone());
                written.key_value.key = inner.item.key.clone();
                written
            })
            .collect();
        fields.sort_by_cached_key(|entry| {
            let key = common::sections::dispatch_name(&entry.key_value.key);
            (
                key_order
                    .iter()
                    .position(|name| *name == key)
                    .unwrap_or(key_order.len()),
                key,
            )
        });
        written.push(common::build::section(full_name, SectionKind::ArrayOfTables, fields));
    }
    if written.is_empty() {
        return;
    }
    // what led the entry leads the first header it becomes
    let lead = parent.entries.remove(at).lead;
    written[0].header.lead = lead;
    let parent_at = document
        .sections
        .iter()
        .position(|section| section.header.key.is_path(parent_name))
        .expect("the parent was just borrowed");
    for (offset, section) in written.into_iter().enumerate() {
        document.sections.insert(parent_at + 1 + offset, section);
    }
}
