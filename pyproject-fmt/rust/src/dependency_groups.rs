use std::cmp::Ordering;

use common::arrays::{map_strings, sort_runs, string_of};
use common::pep508::Requirement;
use common::sections;
use lexical_sort::natural_lexical_cmp;
use toml_doc::{Document, Value};

pub fn fix(document: &mut Document<'_>, keep_full_version: bool) {
    common::nesting::collapse(document, "dependency-groups");
    let path = sections::parse_name("dependency-groups");
    sections::for_keys_under(document, &path, |_key, value| {
        let Value::Array(array) = value else { return };
        // a requirement this parser cannot read is left as the file wrote it
        map_strings(array, |text| normalized(text, keep_full_version));
        // an `include-group` puts the group it names where it is written, so it stays there and only
        // the requirements written between two of them sort
        sort_runs(
            array,
            &|member| matches!(&member.item, Value::InlineTable(_)),
            &|member| {
                string_of(member).map(|text| {
                    let name = Requirement::new(&text).map_or_else(|_| text.clone(), |found| found.canonical_name());
                    (name, text)
                })
            },
            &compare_parts,
        );
    });

    sections::reorder_under(document, &path, &["dev", "test", "type", "docs"]);
}

fn normalized(text: &str, keep_full_version: bool) -> String {
    Requirement::new(text).map_or_else(
        |_| text.to_owned(),
        |found| found.normalize(keep_full_version).to_string(),
    )
}

/// The name the requirement installs leads, and the whole line settles a tie.
type GroupKey = (String, String);

fn compare_parts(left: &GroupKey, right: &GroupKey) -> Ordering {
    natural_lexical_cmp(&left.0, &right.0).then_with(|| natural_lexical_cmp(&left.1, &right.1))
}
