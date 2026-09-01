use _tox_toml_fmt::{format_toml, Settings};

mod disabled_tests;
mod doc_config_tests;
mod doc_getting_started_tests;
mod doc_usage_tests;
mod main_tests;

/// The settings a case runs under where it says nothing else about them.
pub fn default_settings() -> Settings {
    Settings {
        column_width: 80,
        indent: 2,
        table_format: String::from("short"),
        sub_table_spacing: String::new(),
        separate_root_table: String::from("\n"),
        expand_tables: vec![],
        collapse_tables: vec![],
        skip_wrap_for_keys: vec![],
        pin_envs: vec![],
    }
}

/// Run a documentation example the way the docs print one, at the width the docs are written to.
pub fn format_doc_example(start: &str) -> String {
    evaluate_settings(
        start,
        &Settings {
            column_width: 120,
            ..default_settings()
        },
    )
}

/// Run the whole formatter over the source and again over what it wrote, so every case says the
/// formatter settles on its first pass.
pub fn evaluate_settings(start: &str, settings: &Settings) -> String {
    let written = format_toml(start, settings).expect("the formatter accepts it");
    assert_valid_toml(&written);
    assert_eq!(
        format_toml(&written, settings).expect("the formatter reads its own output"),
        written,
        "the formatter settled on its first pass"
    );
    written
}

/// The output has to be TOML a parser accepts, whatever the formatting rules did to it.
///
/// An independent parser reads it back as well as our own, so a document that is well formed but
/// says something no TOML file can say, like a table defined twice, is caught here.
pub fn assert_valid_toml(written: &str) {
    assert!(
        toml_doc::parse(written).is_ok(),
        "the formatter wrote something that does not parse:\n{written}"
    );
    assert!(
        written.parse::<toml::Table>().is_ok(),
        "the formatter wrote something no TOML document can say:\n{written}"
    );
}
