use indoc::indoc;
use rstest::rstest;
use taplo::formatter::{format_syntax, Options};
use taplo::parser::parse;
use taplo::syntax::SyntaxKind::{ENTRY, VALUE};

use crate::array::{sort_strings, transform};
use crate::pep508::Requirement;

#[rstest]
#[case::strip_micro_no_keep(
        indoc ! {r#"
    a=["maturin >= 1.5.0"]
    "#},
        indoc ! {r#"
    a = ["maturin>=1.5"]
    "#},
        false
)]
#[case::strip_micro_keep(
        indoc ! {r#"
    a=["maturin >= 1.5.0"]
    "#},
        indoc ! {r#"
    a = ["maturin>=1.5.0"]
    "#},
        true
)]
#[case::no_change(
        indoc ! {r#"
    a = [
    "maturin>=1.5.3",# comment here
    # a comment afterwards
    ]
    "#},
        indoc ! {r#"
    a = [
      "maturin>=1.5.3", # comment here
      # a comment afterwards
    ]
    "#},
        false
)]
#[case::ignore_non_string(
        indoc ! {r#"
    a=[{key="maturin>=1.5.0"}]
    "#},
        indoc ! {r#"
    a = [{ key = "maturin>=1.5.0" }]
    "#},
        false
)]
#[case::has_double_quote(
        indoc ! {r#"
    a=['importlib-metadata>=7.0.0;python_version<"3.8"']
    "#},
        indoc ! {r#"
    a = ["importlib-metadata>=7; python_version<'3.8'"]
    "#},
        false
)]
fn test_normalize_requirement(#[case] start: &str, #[case] expected: &str, #[case] keep_full_version: bool) {
    let root_ast = parse(start).into_syntax().clone_for_update();
    for children in root_ast.children_with_tokens() {
        if children.kind() == ENTRY {
            for entry in children.as_node().unwrap().children_with_tokens() {
                if entry.kind() == VALUE {
                    transform(entry.as_node().unwrap(), &|s| {
                        Requirement::new(s).unwrap().normalize(keep_full_version).to_string()
                    });
                }
            }
        }
    }
    let res = format_syntax(root_ast, Options::default());
    assert_eq!(expected, res);
}

#[rstest]
#[case::empty(
        indoc ! {r"
    a = []
    "},
        indoc ! {r"
    a = []
    "}
)]
#[case::single(
        indoc ! {r#"
    a = ["A"]
    "#},
        indoc ! {r#"
    a = ["A"]
    "#}
)]
#[case::newline_single(
        indoc ! {r#"
    a = ["A"]
    "#},
        indoc ! {r#"
    a = ["A"]
    "#}
)]
#[case::newline_single_comment(
        indoc ! {r#"
    a = [ # comment
      "A"
    ]
    "#},
        indoc ! {r#"
    a = [
      # comment
      "A",
    ]
    "#}
)]
#[case::double(
        indoc ! {r#"
    a = ["A", "B"]
    "#},
        indoc ! {r#"
    a = ["A", "B"]
    "#}
)]
#[case::increasing(
        indoc ! {r#"
    a=["B", "D",
       # C comment
       "C", # C trailing
       # A comment
       "A" # A trailing
      # extra
    ] # array comment
    "#},
        indoc ! {r#"
    a = [
      # A comment
      "A", # A trailing
      "B",
      # C comment
      "C", # C trailing
      "D",
      # extra
    ] # array comment
    "#}
)]
fn test_order_array(#[case] start: &str, #[case] expected: &str) {
    let root_ast = parse(start).into_syntax().clone_for_update();
    for children in root_ast.children_with_tokens() {
        if children.kind() == ENTRY {
            for entry in children.as_node().unwrap().children_with_tokens() {
                if entry.kind() == VALUE {
                    sort_strings::<String, _, _>(entry.as_node().unwrap(), |s| s.to_lowercase(), &|lhs, rhs| {
                        lhs.cmp(rhs)
                    });
                }
            }
        }
    }
    let opt = Options {
        column_width: 120,
        ..Options::default()
    };
    let res = format_syntax(root_ast, opt);
    assert_eq!(res, expected);
}

#[rstest]
#[case::reorder_no_trailing_comma(
        indoc ! {r#"a=["B","A"]"#},
        indoc ! {r#"a=["A","B"]"#}
)]
fn test_reorder_no_trailing_comma(#[case] start: &str, #[case] expected: &str) {
    let root_ast = parse(start).into_syntax().clone_for_update();
    for children in root_ast.children_with_tokens() {
        if children.kind() == ENTRY {
            for entry in children.as_node().unwrap().children_with_tokens() {
                if entry.kind() == VALUE {
                    sort_strings::<String, _, _>(entry.as_node().unwrap(), |s| s.to_lowercase(), &|lhs, rhs| {
                        lhs.cmp(rhs)
                    });
                }
            }
        }
    }
    let mut res = root_ast.to_string();
    res.retain(|x| !x.is_whitespace());
    assert_eq!(res, expected);
}
