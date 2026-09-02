//! Reading the settings a document holds for the formatter.

use common::settings::{self, Setting};

#[test]
fn a_document_writing_no_such_table_holds_no_settings() {
    assert_eq!(read("a = 1\n", &["tool", "fmt"]), Ok(None));
}

/// TOML gives a table written as a header and one written as dotted keys the same name, so the
/// settings are read out of either.
#[test]
fn settings_are_read_however_the_file_splits_their_path() {
    let held = Ok(Some(vec![(String::from("column_width"), Setting::Whole(30))]));

    assert_eq!(read("[tool.fmt]\ncolumn_width = 30\n", &["tool", "fmt"]), held);
    assert_eq!(read("tool.fmt.column_width = 30\n", &["tool", "fmt"]), held);
    assert_eq!(read("[tool]\nfmt.column_width = 30\n", &["tool", "fmt"]), held);
}

#[test]
fn a_setting_is_read_in_each_form_one_is_written_in() {
    assert_eq!(
        read(
            "[fmt]\ntext = \"held\"\nwhole = 12\nnegative = -3\nplus = +4\nhex = 0xff\noctal = 0o17\nbinary = 0b101\nspaced = 1_0\ntruth = true\nfalsehood = false\nlist = [ \"a\", 1 ]\n",
            &["fmt"]
        ),
        Ok(Some(vec![
            (String::from("text"), Setting::Text(String::from("held"))),
            (String::from("whole"), Setting::Whole(12)),
            (String::from("negative"), Setting::Whole(-3)),
            (String::from("plus"), Setting::Whole(4)),
            (String::from("hex"), Setting::Whole(255)),
            (String::from("octal"), Setting::Whole(15)),
            (String::from("binary"), Setting::Whole(5)),
            (String::from("spaced"), Setting::Whole(10)),
            (String::from("truth"), Setting::Truth(true)),
            (String::from("falsehood"), Setting::Truth(false)),
            (
                String::from("list"),
                Setting::List(vec![Setting::Text(String::from("a")), Setting::Whole(1)])
            ),
        ]))
    );
}

/// A table is not a setting, whichever way the file writes one, and the key that holds it is named
/// so the caller can report on it.
#[test]
fn a_table_the_settings_hold_is_no_setting_of_its_own() {
    let table = Ok(Some(vec![(String::from("deeper"), Setting::Table)]));

    assert_eq!(read("[tool.fmt.deeper]\na = 1\n", &["tool", "fmt"]), table);
    assert_eq!(read("[tool.fmt]\ndeeper.a = 1\n", &["tool", "fmt"]), table);
    assert_eq!(read("[tool.fmt]\ndeeper = { a = 1 }\n", &["tool", "fmt"]), table);
}

/// TOML writes a table as a header or as an inline value, and the settings are read out of either.
#[test]
fn the_settings_are_read_from_a_table_written_as_a_value() {
    let held = Ok(Some(vec![(String::from("column_width"), Setting::Whole(30))]));

    assert_eq!(read("[tool]\nfmt = { column_width = 30 }\n", &["tool", "fmt"]), held);
    assert_eq!(read("tool.fmt = { column_width = 30 }\n", &["tool", "fmt"]), held);
}

/// The settings are one table: a file writing something else where they belong has configured
/// nothing, and is told so rather than formatted with the defaults.
#[test]
fn settings_written_as_something_other_than_one_table_are_named() {
    assert_eq!(
        read("[tool]\nfmt = 30\n", &["tool", "fmt"]),
        Err(String::from("tool.fmt: the settings are not a table"))
    );
    assert_eq!(
        read("[[tool.fmt]]\ncolumn_width = 30\n", &["tool", "fmt"]),
        Err(String::from("tool.fmt: an array of tables holds no settings"))
    );
}

#[test]
fn a_value_written_in_a_form_no_setting_takes_is_named() {
    assert_eq!(
        read("[fmt]\nwhen = 12:30\n", &["fmt"]),
        Err(String::from("when: 12:30 is not a setting"))
    );
    assert_eq!(
        read("[fmt]\nheld = [ 12:30 ]\n", &["fmt"]),
        Err(String::from("held: 12:30 is not a setting"))
    );
}

/// A number the setting cannot hold is not one, and neither is a value written with a fraction.
#[test]
fn a_number_the_reader_cannot_hold_is_not_a_setting() {
    assert!(read("[fmt]\nheld = 9223372036854775808\n", &["fmt"]).is_err());
    assert!(read("[fmt]\nheld = 1.5\n", &["fmt"]).is_err());
}

/// A name is read the way TOML reads it, so the quotes a file writes around one are no part of it.
#[test]
fn a_name_written_in_quotes_is_the_name_it_spells() {
    let held = Ok(Some(vec![(String::from("column_width"), Setting::Whole(30))]));

    assert_eq!(
        read("[tool]\nfmt = { \"column_width\" = 30 }\n", &["tool", "fmt"]),
        held
    );
    assert_eq!(read("[tool.fmt]\n\"column_width\" = 30\n", &["tool", "fmt"]), held);
    assert_eq!(read("[\"tool\".fmt]\ncolumn_width = 30\n", &["tool", "fmt"]), held);
}

/// A table written as a value holds the settings wherever it sits, and a dotted name inside one
/// names a table rather than a setting.
#[test]
fn the_settings_are_read_out_of_the_tables_a_value_holds() {
    assert_eq!(
        read("tool = { fmt = { column_width = 30 } }\n", &["tool", "fmt"]),
        Ok(Some(vec![(String::from("column_width"), Setting::Whole(30))]))
    );
    assert_eq!(
        read("tool = { fmt.column_width = 30 }\n", &["tool", "fmt"]),
        Ok(Some(vec![(String::from("column_width"), Setting::Whole(30))]))
    );
    assert_eq!(
        read("[tool.fmt]\nnested.value = 1\n", &["tool", "fmt"]),
        Ok(Some(vec![(String::from("nested"), Setting::Table)]))
    );
    assert_eq!(
        read("[tool]\nfmt = { nested.value = 1 }\n", &["tool", "fmt"]),
        Ok(Some(vec![(String::from("nested"), Setting::Table)]))
    );
    // a value on the way to the settings that is not a table holds none of them
    assert_eq!(read("tool = 1\n", &["tool", "fmt"]), Ok(None));
}

/// A table the file repeats is a list of tables, and what its elements write is no one table.
#[test]
fn a_list_of_tables_anywhere_above_the_settings_holds_none() {
    let repeated = Err(String::from("tool.fmt: an array of tables holds no settings"));

    assert_eq!(
        read("[[tool]]\nfmt = { column_width = 30 }\n", &["tool", "fmt"]),
        repeated
    );
    assert_eq!(
        read("[[tool]]\n[tool.fmt]\ncolumn_width = 30\n", &["tool", "fmt"]),
        repeated
    );
    assert_eq!(read("[[tool.fmt]]\ncolumn_width = 30\n", &["tool", "fmt"]), repeated);
}

/// A header names the table it opens, so a table below the settings is one of their names whether
/// or not the file wrote a key under it.
#[test]
fn a_table_below_the_settings_is_named_even_where_it_holds_nothing() {
    let table = Ok(Some(vec![(String::from("unknown"), Setting::Table)]));

    assert_eq!(read("[tool.fmt.unknown]\n", &["tool", "fmt"]), table);
    assert_eq!(read("[[tool.fmt.unknown]]\n", &["tool", "fmt"]), table);
}

/// The settings are a table, so a root key that names the table and holds something else says the
/// file wrote no settings the reader can take.
#[test]
fn settings_written_at_the_root_as_something_other_than_a_table_are_rejected() {
    assert_eq!(
        read("tool = 1\n", &["tool"]),
        Err(String::from("tool: the settings are not a table"))
    );
}

/// The same, for a table the file wrote inside a value.
#[test]
fn settings_inside_a_value_that_is_not_a_table_are_rejected() {
    assert_eq!(
        read("tool = { fmt = 1 }\n", &["tool", "fmt"]),
        Err(String::from("tool.fmt: the settings are not a table"))
    );
}

fn read(source: &str, path: &[&str]) -> Result<Option<Vec<(String, Setting)>>, String> {
    let document = toml_doc::parse(source).expect("valid source");
    let path: Vec<String> = path.iter().map(|part| (*part).to_owned()).collect();
    settings::read(&document, &path)
}
