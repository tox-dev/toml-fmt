use common::arrays::{map_strings, sort_strings_in};
use common::pep508::Requirement;
use common::sections;
use lexical_sort::natural_lexical_cmp;
use toml_doc::{Document, Value};

pub fn fix(document: &mut Document<'_>, keep_full_version: bool) {
    let path = sections::parse_name("build-system");
    sections::for_keys_under(document, &path, |key, value| {
        if key != "requires" {
            return;
        }
        // a requirement this parser cannot read is left as the file wrote it
        map_strings_in(value, |text| {
            Requirement::new(text).map_or_else(
                |_| text.to_owned(),
                |found| found.normalize(keep_full_version).to_string(),
            )
        });
        sort_strings_in(
            value,
            &|text| Requirement::new(text).map_or_else(|_| text.to_owned(), |found| found.canonical_name()),
            &|left, right| natural_lexical_cmp(left, right),
        );
    });
    sections::reorder_under(document, &path, &["build-backend", "requires", "backend-path"]);
}

fn map_strings_in<F>(value: &mut Value<'_>, rewrite: F)
where
    F: FnMut(&str) -> String,
{
    if let Value::Array(array) = value {
        map_strings(array, rewrite);
    }
}
