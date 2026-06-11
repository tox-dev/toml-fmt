use common::array::{remove_strings, sort_strings, transform};
use common::pep508::Requirement;
use common::string::{get_string_token, load_text};
use common::table::{for_entries, reorder_table_keys, Tables};
use lexical_sort::{lexical_cmp, natural_lexical_cmp};
use tombi_syntax::SyntaxKind::{BASIC_STRING, LITERAL_STRING};
use tombi_syntax::SyntaxNode;

pub fn fix(tables: &Tables, keep_full_version: bool) {
    let table_element = tables.get("build-system");
    if table_element.is_none() {
        return;
    }
    let table = &mut table_element.unwrap().first().unwrap().borrow_mut();
    let mut backend = String::new();
    let mut has_backend_path = false;
    for_entries(table, &mut |key, entry| match key.as_str() {
        "build-backend" => {
            if let Some(token) = get_string_token(entry) {
                backend = load_text(token.text(), entry.kind());
            }
        }
        "backend-path" => {
            has_backend_path = true;
        }
        _ => {}
    });
    let drop_wheel = !has_backend_path
        && matches!(
            backend.as_str(),
            "setuptools.build_meta" | "setuptools.build_meta:__legacy__"
        );
    for_entries(table, &mut |key, entry| match key.as_str() {
        "requires" => {
            transform(entry, &|s| {
                Requirement::new(s).unwrap().normalize(keep_full_version).to_string()
            });
            if drop_wheel && requires_has_setuptools(entry) {
                remove_strings(entry, |s| {
                    Requirement::new(s).is_ok_and(|r| r.canonical_name() == "wheel" && r.is_name_only())
                });
            }
            sort_strings::<String, _, _>(
                entry,
                |s| Requirement::new(s.as_str()).unwrap().canonical_name(),
                &|lhs, rhs| natural_lexical_cmp(lhs, rhs),
            );
        }
        "backend-path" => {
            sort_strings::<String, _, _>(entry, |s| s.to_lowercase(), &|lhs, rhs| lexical_cmp(lhs, rhs));
        }
        _ => {}
    });
    reorder_table_keys(table, &["", "build-backend", "requires", "backend-path"]);
}

fn requires_has_setuptools(entry: &SyntaxNode) -> bool {
    entry
        .descendants()
        .filter(|n| matches!(n.kind(), BASIC_STRING | LITERAL_STRING))
        .any(|n| {
            get_string_token(&n).is_some_and(|token| {
                Requirement::new(&load_text(token.text(), n.kind())).is_ok_and(|r| r.canonical_name() == "setuptools")
            })
        })
}
