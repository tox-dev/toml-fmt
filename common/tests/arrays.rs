//! Reordering and rewriting the members of an array.

use common::arrays;
use common::layout::Layout;
use toml_doc::{Array, Document, LineEnding, Value};

fn parse(source: &str) -> Document<'_> {
    toml_doc::parse(source).expect("valid source")
}

/// Reordering leaves the separators where position put them, so the layout pass follows it just as
/// it does in the formatter.
fn written(document: &mut Document<'_>) -> String {
    Layout {
        column_width: 120,
        indent: 2,
        ending: LineEnding::Lf,
    }
    .apply(document);
    document.to_string()
}

fn with_array(source: &str, act: impl FnOnce(&mut Array<'_>)) -> String {
    let mut document = parse(source);
    let Value::Array(array) = &mut document.root[0].key_value.value else {
        panic!("the test source holds an array");
    };
    act(array);
    written(&mut document)
}

#[test]
fn deduping_leaves_a_member_that_is_not_a_string() {
    assert_eq!(
        with_array("a = [ 1, \"b\", \"b\", 1 ]\n", |array| arrays::dedupe_strings(
            array,
            &str::to_owned
        )),
        "a = [ 1, \"b\", 1 ]\n"
    );
}

/// A formatter that cannot read what a file says leaves it alone; dropping is what
/// [`retain_strings`] is for.
#[test]
fn a_mapping_never_drops_a_member() {
    assert_eq!(
        with_array("a = [ \"keep\", \"as written\" ]\n", |array| arrays::map_strings(
            array,
            |text| if text == "keep" {
                text.to_uppercase()
            } else {
                text.to_owned()
            }
        )),
        "a = [ \"KEEP\", \"as written\" ]\n"
    );
}

#[test]
fn deduping_a_value_that_is_not_an_array_leaves_it_alone() {
    let mut document = parse("a = \"one\"\n");
    arrays::dedupe_strings_in(&mut document.root[0].key_value.value, &str::to_owned);

    assert_eq!(document.to_string(), "a = \"one\"\n");
}

#[test]
fn strings_sort_by_the_key_they_map_to() {
    assert_eq!(
        with_array("a = [ \"Bee\", \"ant\" ]\n", |array| arrays::sort_strings(
            array,
            &str::to_lowercase,
            &str::cmp
        )),
        "a = [ \"ant\", \"Bee\" ]\n"
    );
}

#[test]
fn a_group_marker_holds_the_members_on_either_side_of_it() {
    assert_eq!(
        with_array(
            "a = [\n  \"z\",\n  \"y\",\n  # Group: later\n  \"b\",\n  \"a\",\n]\n",
            |array| arrays::sort_strings(array, &str::to_owned, &str::cmp)
        ),
        "a = [\n  \"y\",\n  \"z\",\n  # Group: later\n  \"a\",\n  \"b\",\n]\n"
    );
}

#[test]
fn a_trailing_comma_holds_the_array_open_across_a_sort() {
    assert_eq!(
        with_array("a = [ \"b\", \"a\", ]\n", |array| arrays::sort_strings(
            array,
            &str::to_owned,
            &str::cmp
        )),
        "a = [\n  \"a\",\n  \"b\",\n]\n"
    );
}

#[test]
fn a_rejected_string_is_dropped_with_its_comma() {
    assert_eq!(
        with_array("a = [ \"keep\", \"drop\", 1 ]\n", |array| {
            arrays::retain_strings(array, |text| text != "drop");
        }),
        "a = [ \"keep\", 1 ]\n"
    );
}

#[test]
fn sorting_a_value_that_is_not_an_array_leaves_it_alone() {
    let mut document = parse("a = \"one\"\n");
    arrays::sort_strings_in(&mut document.root[0].key_value.value, &str::to_owned, &str::cmp);

    assert_eq!(document.to_string(), "a = \"one\"\n");
}

#[test]
fn a_value_holding_an_array_sorts_and_dedupes_in_place() {
    let mut document = parse("a = [ \"b\", \"a\", \"b\" ]\n");
    let value = &mut document.root[0].key_value.value;
    arrays::sort_strings_in(value, &str::to_owned, &str::cmp);
    arrays::dedupe_strings_in(value, &str::to_owned);

    assert_eq!(written(&mut document), "a = [ \"a\", \"b\" ]\n");
}

#[test]
fn removing_the_last_member_takes_its_comma_with_it() {
    assert_eq!(
        with_array("a = [ \"keep\", \"drop\", ]\n", |array| {
            arrays::retain_strings(array, |text| text != "drop");
        }),
        "a = [ \"keep\" ]\n"
    );
}

#[test]
fn the_text_a_member_holds_reads_back_decoded() {
    let document = parse("a = [ \"one\", 2 ]\n");
    let Value::Array(array) = &document.root[0].key_value.value else {
        panic!("the test source holds an array");
    };

    assert_eq!(arrays::string_of(&array.members[0]), Some(String::from("one")));
    assert_eq!(arrays::string_of(&array.members[1]), None);
}

#[test]
fn an_empty_group_between_two_markers_sorts_nothing() {
    assert_eq!(
        with_array(
            "a = [\n  # Group: one\n  # Group: two\n  \"b\",\n  \"a\",\n]\n",
            |array| arrays::sort_strings(array, &str::to_owned, &str::cmp)
        ),
        "a = [\n  # Group: one\n  # Group: two\n  \"a\",\n  \"b\",\n]\n"
    );
}

#[test]
fn a_mapping_leaves_a_member_that_is_not_a_string() {
    assert_eq!(
        with_array("a = [ 1, \"b\" ]\n", |array| arrays::map_strings(
            array,
            str::to_uppercase
        )),
        "a = [ 1, \"B\" ]\n"
    );
}

#[test]
fn an_array_with_nothing_in_it_sorts_to_itself() {
    assert_eq!(
        with_array("a = []\n", |array| arrays::sort_strings(
            array,
            &str::to_owned,
            &str::cmp
        )),
        "a = []\n"
    );
}

/// A member the key function cannot read has nothing to sort by, and the array says what it says
/// by the order it was written in.
#[test]
fn an_array_holding_a_member_without_a_key_is_left_as_written() {
    assert_eq!(
        with_array("a = [ \"z\", 1, \"b\" ]\n", |array| arrays::sort_strings(
            array,
            &str::to_owned,
            &str::cmp
        )),
        "a = [ \"z\", 1, \"b\" ]\n"
    );
}

/// A list of names is what [`sort_names_in`] sorts, and a member that is not one says the list is
/// something else, which the order it was written in is part of.
#[test]
fn a_list_holding_more_than_names_is_left_as_written() {
    let mut document = parse("a = [ \"z\", { name = \"one\" }, \"b\" ]\n");
    arrays::sort_names_in(&mut document.root[0].key_value.value);

    assert_eq!(written(&mut document), "a = [ \"z\", { name = \"one\" }, \"b\" ]\n");
}

/// A comment can sit before the comma, after it, or on a line of its own above the member, and a
/// dropped member holds whichever ones the file wrote around it.
#[test]
fn a_dropped_member_hands_its_comments_to_what_follows() {
    let dropped = |source: &str| {
        let mut document = parse(source);
        let Value::Array(array) = &mut document.root[0].key_value.value else {
            panic!("the test source holds an array");
        };
        arrays::retain_strings(array, |text| text != "drop");
        let out = written(&mut document);
        assert!(toml_doc::parse(&out).is_ok(), "{out}");
        out
    };

    assert_eq!(
        dropped("a = [\n  \"keep\",\n  \"drop\", # after the comma\n  \"tail\",\n]\n"),
        "a = [\n  \"keep\",\n  # after the comma\n  \"tail\",\n]\n"
    );
    assert_eq!(
        dropped("a = [\n  \"keep\",\n  \"drop\" # before the comma\n  ,\n  \"tail\",\n]\n"),
        "a = [\n  \"keep\",\n  # before the comma\n  \"tail\",\n]\n"
    );
    assert_eq!(
        dropped("a = [\n  \"keep\",\n  # above\n  \"drop\",\n  \"tail\",\n]\n"),
        "a = [\n  \"keep\",\n  # above\n  \"tail\",\n]\n"
    );
}

#[test]
fn the_first_and_last_members_hand_their_comments_on_too() {
    let dropped = |source: &str| {
        let mut document = parse(source);
        let Value::Array(array) = &mut document.root[0].key_value.value else {
            panic!("the test source holds an array");
        };
        arrays::retain_strings(array, |text| text != "drop");
        let out = written(&mut document);
        assert!(toml_doc::parse(&out).is_ok(), "{out}");
        out
    };

    assert_eq!(
        dropped("a = [\n  \"drop\", # first\n  \"tail\",\n]\n"),
        "a = [\n  # first\n  \"tail\",\n]\n"
    );
    assert_eq!(
        dropped("a = [\n  \"keep\",\n  \"drop\", # last\n]\n"),
        "a = [\n  \"keep\"\n  # last\n]\n"
    );
}

#[test]
fn deduping_and_filtering_keep_the_comments_of_what_they_drop() {
    let mut document = parse("a = [\n  \"one\",\n  \"one\", # duplicate\n  \"two\",\n]\n");
    let Value::Array(array) = &mut document.root[0].key_value.value else {
        panic!("the test source holds an array");
    };
    arrays::dedupe_strings(array, &str::to_owned);
    assert_eq!(
        written(&mut document),
        "a = [\n  \"one\",\n  # duplicate\n  \"two\",\n]\n"
    );

    let mut document = parse("a = [\n  \"one\",\n  \"drop\", # dropped\n  \"two\",\n]\n");
    let Value::Array(array) = &mut document.root[0].key_value.value else {
        panic!("the test source holds an array");
    };
    arrays::retain_strings(array, |text| text != "drop");
    assert_eq!(
        written(&mut document),
        "a = [\n  \"one\",\n  # dropped\n  \"two\",\n]\n"
    );
}

/// The order almost every list in a formatted file reads in: what a name says rather than how it is
/// capitalized, and a number by its value rather than by its digits.
#[test]
fn a_list_of_names_sorts_by_what_the_names_say() {
    let mut document = parse("a = [ \"py10\", \"Beta\", \"py9\", \"alpha\" ]\n");
    arrays::sort_names_in(&mut document.root[0].key_value.value);

    assert_eq!(
        written(&mut document),
        "a = [ \"alpha\", \"Beta\", \"py9\", \"py10\" ]\n"
    );
}

/// A member whose place is part of what the file says keeps it, and only what is written between
/// two of those moves.
#[test]
fn members_sort_only_between_the_ones_that_stay() {
    let mut document = parse("a = [ \"c\", { include-group = \"x\" }, \"z\", \"b\", { include-group = \"y\" } ]\n");
    let Value::Array(array) = &mut document.root[0].key_value.value else {
        panic!("the test source holds an array");
    };
    arrays::sort_runs(
        array,
        &|member| matches!(&member.item, Value::InlineTable(_)),
        &|member| arrays::string_of(member),
        &|left: &String, right: &String| left.cmp(right),
    );

    assert_eq!(
        written(&mut document),
        "a = [ \"c\", { include-group = \"x\" }, \"b\", \"z\", { include-group = \"y\" } ]\n"
    );
}

/// A member the key function cannot read holds the place the file gave it, and the ones it can read
/// sort among the places that are left.
#[test]
fn a_member_with_nothing_to_sort_by_holds_its_place() {
    assert_eq!(
        with_array("a = [ \"c\", { held = 1 }, \"b\", \"a\" ]\n", |array| {
            arrays::sort_placed(array, &arrays::string_of, &Ord::cmp)
        }),
        "a = [ \"a\", { held = 1 }, \"b\", \"c\" ]\n"
    );
}

/// A run holding one name has nothing to sort it against, so it stays where it was written.
#[test]
fn a_run_holding_one_name_is_left_alone() {
    assert_eq!(
        with_array("a = [ { held = 1 }, \"b\" ]\n", |array| arrays::sort_placed(
            array,
            &arrays::string_of,
            &Ord::cmp
        )),
        "a = [ { held = 1 }, \"b\" ]\n"
    );
}

/// A `# Group:` marker names the group it opens, so the names on either side of it sort among
/// themselves and the marker stays on top of its group.
#[test]
fn members_that_hold_their_place_sort_within_their_own_group() {
    assert_eq!(
        with_array(
            "a = [\n  \"z\",\n  \"a\",\n  # Group: later\n  \"y\",\n  \"b\",\n]\n",
            |array| arrays::sort_placed(array, &arrays::string_of, &Ord::cmp)
        ),
        "a = [\n  \"a\",\n  \"z\",\n  # Group: later\n  \"b\",\n  \"y\",\n]\n"
    );
}
