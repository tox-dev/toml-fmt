use common::taplo::formatter::{format_syntax, Options};
use common::taplo::parser::parse;
use indoc::indoc;
use rstest::rstest;

use crate::global::reorder_tables;
use common::table::Tables;

#[rstest]
#[case::reorder(
        indoc ! {r#"
        # comment
        requires = ["tox>=4.22"]

        [demo]
        desc = "demo"

        [env.type]
        description = "type"

        [env.docs]
        description = "docs"

        [env_run_base]
        description = "base"

    "#},
        indoc ! {r#"
        # comment
        requires = ["tox>=4.22"]

        [env_run_base]
        description = "base"

        [env.type]
        description = "type"

        [env.docs]
        description = "docs"

        [demo]
        desc = "demo"
    "#},
)]
fn test_reorder_table(#[case] start: &str, #[case] expected: &str) {
    let root_ast = parse(start).into_syntax().clone_for_update();
    let tables = Tables::from_ast(&root_ast);
    reorder_tables(&root_ast, &tables);
    let opt = Options {
        column_width: 120,
        ..Options::default()
    };
    let got = format_syntax(root_ast, opt);
    assert_eq!(got, expected);
}
