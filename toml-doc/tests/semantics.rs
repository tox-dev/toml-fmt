//! What a document says, past what its grammar can rule out.
//!
//! The test suite covers the ways a value is written wrong that implementations trip over; these
//! are the ones it leaves out.

/// The bounds a date or a time is read against, on both sides of each one, where the conformance
/// corpus stops at the ones implementations trip over.
#[test]
fn a_moment_is_read_against_the_bounds_of_what_it_names() {
    let held = [
        "2024-02-29",
        "2023-02-28",
        "2024-12-31",
        "23:59:59",
        "23:59:60",
        "00:00:00.5",
    ];
    let unheld = [
        "2023-02-29",
        "2024-02-30",
        "2024-13-01",
        "2024-00-01",
        "2024-01-00",
        "2024-01-32",
        "24:00:00",
        "23:60:00",
        "23:59:61",
        "23:59:59.",
    ];

    let read = |moment: &str| toml_doc::parse(&format!("when = {moment}\n")).is_ok();

    assert_eq!(
        (
            held.into_iter().filter(|moment| !read(moment)).collect::<Vec<&str>>(),
            unheld.into_iter().filter(|moment| read(moment)).collect::<Vec<&str>>()
        ),
        (vec![], vec![])
    );
}

/// An offset is a time of its own, and the hour RFC 3339 gives it stops at 23.
#[test]
fn an_offset_is_read_against_the_bounds_of_what_it_names() {
    let tails = ["Z", "z", "+00:00", "-23:59", "+23:00"];
    let unheld = ["+7:00", "+07:0", "+24:00", "-24:00", "+23:60", "x", "+0700"];

    let read = |tail: &str| toml_doc::parse(&format!("when = 1979-05-27T07:32:00{tail}\n")).is_ok();

    assert_eq!(
        (
            tails.into_iter().filter(|tail| !read(tail)).collect::<Vec<&str>>(),
            unheld.into_iter().filter(|tail| read(tail)).collect::<Vec<&str>>()
        ),
        (vec![], vec![])
    );
}

#[test]
fn a_key_written_twice_is_reported_where_the_second_one_is() {
    let source = "[table]\nname = 1\nname = 2\n";

    let error = toml_doc::parse(source).expect_err("a key written twice");

    assert_eq!(
        (&source[error[0].span.clone()], error[0].message.as_str()),
        ("name", "`name` is written twice")
    );
}

/// A document holds a scalar as the text it was written as, so how wide a number is says nothing
/// about whether this model can hold it.
#[test]
fn a_number_is_held_however_wide_it_is_written() {
    let numbers = [
        "9223372036854775807",
        "-9223372036854775808",
        "9223372036854775808",
        "-9223372036854775809",
        "0x8000000000000000",
        "1e308",
        "1e9999",
        "1e-9999",
    ];

    let refused: Vec<&str> = numbers
        .into_iter()
        .filter(|number| toml_doc::parse(&format!("value = {number}\n")).is_err())
        .collect();

    assert_eq!(refused, Vec::<&str>::new());
}
