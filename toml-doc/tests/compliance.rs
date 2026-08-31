//! What the parser makes of the invalid half of the TOML test suite.
//!
//! The suite spans two spec versions and `toml_parser` reads TOML 1.1, where optional seconds,
//! newlines inside an inline table, and `\\xHH` escapes are all things a file may say. So the bar
//! is the 1.1 half: every case that spec calls invalid has to be rejected, since a formatter that
//! rewrote a file no reader can read would hand back something that looks repaired while still
//! saying nothing.
//!
//! Nine fixtures are not UTF-8, so they never reach a `&str` API and sit outside the count.

use std::collections::HashSet;

#[test]
fn every_invalid_document_is_rejected() {
    let spec: HashSet<String> = toml_test_data::version("1.1.0")
        .map(|case| case.display().to_string())
        .collect();

    let read: Vec<String> = toml_test_data::invalid()
        .filter(|case| spec.contains(&case.name().display().to_string()))
        .filter(|case| str::from_utf8(case.fixture()).is_ok())
        .map(|case| case.name().display().to_string())
        .collect();
    // a filter that selected nothing would pass this test while reading no file at all
    assert!(read.len() > 250, "{}", read.len());

    let accepted: Vec<String> = toml_test_data::invalid()
        .filter(|case| read.contains(&case.name().display().to_string()))
        .filter(|case| str::from_utf8(case.fixture()).is_ok_and(|source| toml_doc::parse(source).is_ok()))
        .map(|case| case.name().display().to_string())
        .collect();

    assert_eq!(accepted, Vec::<String>::new());
}

/// TOML sets no limit on how deep a value nests, and a machine does: reading, writing and dropping
/// one all walk it by calling themselves, so a value past that says so rather than ending the
/// process.
#[test]
fn a_value_nested_deeper_than_this_reads_is_rejected() {
    let read = |depth: usize| {
        let source = format!("a = {}1{}\n", "[".repeat(depth), "]".repeat(depth));
        toml_doc::parse(&source)
            .map(|_| ())
            .map_err(|errors| errors[0].message.clone())
    };

    assert_eq!(read(toml_doc::NESTING), Ok(()));
    for depth in [toml_doc::NESTING + 1, 3_000, 12_000] {
        assert!(read(depth).is_err_and(|why| why.contains("nested deeper")), "{depth}");
    }
    // a bracket inside a string or a comment opens nothing
    let held = format!("a = \"{}\"\n# {}\n", "[".repeat(3_000), "[".repeat(3_000));
    assert!(toml_doc::parse(&held).is_ok());
}
