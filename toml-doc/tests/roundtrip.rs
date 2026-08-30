//! Writing a parsed document back must reproduce the source byte for byte.

use std::fmt::Write as _;
use std::path::Path;

#[test]
fn valid_corpus_round_trips() {
    let mut report = String::new();
    let mut checked = 0_usize;
    for case in toml_test_data::valid() {
        let Ok(source) = str::from_utf8(case.fixture()) else {
            continue;
        };
        checked += 1;
        match toml_doc::parse(source) {
            Ok(document) => {
                let written = document.to_string();
                if written != source {
                    let _ = writeln!(report, "{}: {source:?} became {written:?}", case.name().display());
                }
            }
            Err(errors) => {
                let _ = writeln!(report, "{}: rejected as {}", case.name().display(), errors[0]);
            }
        }
    }
    assert!(checked > 250, "corpus shrank to {checked} cases");
    assert!(report.is_empty(), "{report}");
}

#[test]
fn repository_toml_files_round_trip() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate sits in the workspace");
    let mut report = String::new();
    let mut checked = 0_usize;
    for path in toml_files(root) {
        let source = std::fs::read_to_string(&path).expect("readable file");
        checked += 1;
        match toml_doc::parse(&source) {
            Ok(document) if document.to_string() == source => {}
            Ok(_) => {
                let _ = writeln!(report, "{}: written back differently", path.display());
            }
            Err(errors) => {
                let _ = writeln!(report, "{}: rejected as {}", path.display(), errors[0]);
            }
        }
    }
    assert!(checked > 5, "found only {checked} files to check");
    assert!(report.is_empty(), "{report}");
}

fn toml_files(root: &Path) -> Vec<std::path::PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            if path.is_dir() {
                if !matches!(name.to_str(), Some("target" | ".git" | ".tox" | "node_modules")) {
                    pending.push(path);
                }
            } else if path.extension().is_some_and(|extension| extension == "toml") {
                found.push(path);
            }
        }
    }
    found
}
