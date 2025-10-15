use common::taplo::formatter::{format_syntax, Options};
use common::taplo::parser::parse;
use common::taplo::syntax::SyntaxElement;
use indoc::indoc;
use rstest::rstest;

use crate::project::fix;
use common::table::Tables;

fn evaluate(
    start: &str,
    keep_full_version: bool,
    max_supported_python: (u8, u8),
    generate_python_version_classifiers: bool,
) -> String {
    let root_ast = parse(start).into_syntax().clone_for_update();
    let count = root_ast.children_with_tokens().count();
    let mut tables = Tables::from_ast(&root_ast);
    fix(
        &mut tables,
        keep_full_version,
        max_supported_python,
        (3, 9),
        generate_python_version_classifiers,
    );
    let entries = tables
        .table_set
        .iter()
        .flat_map(|e| e.borrow().clone())
        .collect::<Vec<SyntaxElement>>();
    root_ast.splice_children(0..count, entries);
    let opt = Options {
        column_width: 1,
        ..Options::default()
    };
    format_syntax(root_ast, opt)
}

#[rstest]
#[case::no_project(
        indoc ! {r""},
        "\n",
        false,
        (3, 9),
        true,
)]
#[case::project_requires_no_keep(
        indoc ! {r#"
    [project]
    dependencies=["a>=1.0.0", "b.c>=1.5.0"]
    "#},
        indoc ! {r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
    ]
    dependencies = [
      "a>=1",
      "b-c>=1.5",
    ]
    "#},
        false,
        (3, 9),
        true,
)]
#[case::project_requires_keep(
        indoc ! {r#"
    [project]
    dependencies=["a>=1.0.0", "b.c>=1.5.0"]
    "#},
        indoc ! {r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
    ]
    dependencies = [
      "a>=1.0.0",
      "b-c>=1.5.0",
    ]
    "#},
        true,
        (3, 9),
        true,
)]
#[case::project_requires_ge(
        indoc ! {r#"
    [project]
    requires-python = " >= 3.9"
    classifiers = [
      # comment license inline 1
      # comment license inline 2
      "License :: OSI Approved :: MIT License", # comment license post
      # comment 3.12 inline 1
      # comment 3.12 inline 2
      "Programming Language :: Python :: 3.12", # comment 3.12 post
      # comment 3.10 inline
      "Programming Language :: Python :: 3.10" # comment 3.10 post
      # extra 1
      # extra 2
      # extra 3
    ]
    "#},
        indoc ! {r#"
    [project]
    requires-python = ">=3.9"
    classifiers = [
      # comment license inline 1
      # comment license inline 2
      "License :: OSI Approved :: MIT License",      # comment license post
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      # comment 3.10 inline
      "Programming Language :: Python :: 3.10", # comment 3.10 post
      # extra 1
      # extra 2
      # extra 3
    ]
    "#},
        true,
        (3, 10),
        true,
)]
#[case::project_requires_gt(
        indoc ! {r#"
    [project]
    requires-python = " > 3.8"
    "#},
        indoc ! {r#"
    [project]
    requires-python = ">3.8"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
    ]
    "#},
        true,
        (3, 9),
        true,
)]
#[case::project_requires_eq(
        indoc ! {r#"
    [project]
    requires-python = " == 3.12"
    "#},
        indoc ! {r#"
    [project]
    requires-python = "==3.12"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.12",
    ]
    "#},
        true,
        (3, 9),
        true,
)]
#[case::project_sort_keywords(
        indoc ! {r#"
    [project]
    keywords = ["b", "A", "a-c", " c"]
    "#},
        indoc ! {r#"
    [project]
    keywords = [
      " c",
      "A",
      "a-c",
      "b",
    ]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
    ]
    "#},
        true,
        (3, 9),
        true,
)]
#[case::project_sort_dynamic(
        indoc ! {r#"
    [project]
    dynamic = ["b", "A", "a-c", " c", "a10", "a2"]
    "#},
        indoc ! {r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
    ]
    dynamic = [
      " c",
      "A",
      "a-c",
      "a2",
      "a10",
      "b",
    ]
    "#},
        true,
        (3, 9),
        true,
)]
#[case::project_name_norm(
        indoc ! {r#"
    [project]
    name = "a.b.c"
    "#},
        indoc ! {r#"
    [project]
    name = "a-b-c"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
    ]
    "#},
        true,
        (3, 9),
        true,
)]
#[case::project_name_literal(
        indoc ! {r"
    [project]
    name = 'a.b.c'
    "},
        indoc ! {r#"
    [project]
    name = "a-b-c"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
    ]
    "#},
        true,
        (3, 9),
        true,
)]
#[case::project_requires_gt_old(
        indoc ! {r#"
    [project]
    requires-python = " > 3.7"
    "#},
        indoc ! {r#"
    [project]
    requires-python = ">3.7"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.8",
      "Programming Language :: Python :: 3.9",
    ]
    "#},
        true,
        (3, 9),
        true,
)]
#[case::project_requires_range(
        indoc ! {r#"
    [project]
    requires-python=">=3.7,<3.13"
    "#},
        indoc ! {r#"
    [project]
    requires-python = ">=3.7,<3.13"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.7",
      "Programming Language :: Python :: 3.8",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: 3.11",
      "Programming Language :: Python :: 3.12",
    ]
    "#},
        true,
        (3, 9),
        true,
)]
#[case::project_requires_high_range(
        indoc ! {r#"
    [project]
    requires-python = "<=3.13,>3.10"
    "#},
        indoc ! {r#"
    [project]
    requires-python = "<=3.13,>3.10"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.11",
      "Programming Language :: Python :: 3.12",
      "Programming Language :: Python :: 3.13",
    ]
    "#},
        true,
        (3, 9),
        true,
)]
#[case::project_requires_range_neq(
        indoc ! {r#"
    [project]
    requires-python = "<=3.10,!=3.9,>=3.8"
    "#},
        indoc ! {r#"
    [project]
    requires-python = "<=3.10,!=3.9,>=3.8"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.8",
      "Programming Language :: Python :: 3.10",
    ]
    "#},
        true,
        (3, 13),
        true,
)]
#[case::project_description_whitespace(
        "[project]\ndescription = ' A  magic stuff \t is great\t\t.\r\n  Like  really  . Works on .rst and .NET :)\t\'\nrequires-python = '==3.12'",
        indoc ! {r#"
    [project]
    description = "A magic stuff is great. Like really. Works on .rst and .NET :)"
    requires-python = "==3.12"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.12",
    ]
    "#},
        true,
        (3, 13),
        true,
)]
#[case::project_description_multiline(
        indoc ! {r#"
    [project]
    requires-python = "==3.12"
    description = """
    A magic stuff is great.
        Like really.
    """
    "#},
        indoc ! {r#"
    [project]
    description = "A magic stuff is great. Like really."
    requires-python = "==3.12"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.12",
    ]
    "#},
        true,
        (3, 13),
        true,
)]
#[case::project_dependencies_with_double_quotes(
        indoc ! {r#"
    [project]
    dependencies = [
        'packaging>=20.0;python_version>"3.4"',
        "appdirs"
    ]
    requires-python = "==3.12"
    "#},
        indoc ! {r#"
    [project]
    requires-python = "==3.12"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.12",
    ]
    dependencies = [
      "appdirs",
      "packaging>=20.0; python_version>'3.4'",
    ]
    "#},
        true,
        (3, 13),
        true,
)]
#[case::project_platform_dependencies(
        indoc ! {r#"
    [project]
    dependencies = [
        'pyperclip; platform_system == "Darwin"',
        'pyperclip; platform_system == "Windows"',
        "appdirs"
    ]
    requires-python = "==3.12"
    "#},
        indoc ! {r#"
    [project]
    requires-python = "==3.12"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.12",
    ]
    dependencies = [
      "appdirs",
      "pyperclip; platform_system=='Darwin'",
      "pyperclip; platform_system=='Windows'",
    ]
    "#},
        true,
        (3, 13),
        true,
)]
#[case::project_opt_inline_dependencies(
        indoc ! {r#"
    [project]
    dependencies = ["packaging>=24"]
    optional-dependencies.test = ["pytest>=8.1.1",  "covdefaults>=2.3"]
    optional-dependencies.docs = ["sphinx-argparse-cli>=1.15", "Sphinx>=7.3.7"]
    requires-python = "==3.12"
    "#},
        indoc ! {r#"
    [project]
    requires-python = "==3.12"
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.12",
    ]
    dependencies = [
      "packaging>=24",
    ]
    optional-dependencies.docs = [
      "sphinx>=7.3.7",
      "sphinx-argparse-cli>=1.15",
    ]
    optional-dependencies.test = [
      "covdefaults>=2.3",
      "pytest>=8.1.1",
    ]
    "#},
        true,
        (3, 13),
        true,
)]
#[case::project_opt_dependencies(
        indoc ! {r#"
    [project.optional-dependencies]
    test = ["pytest>=8.1.1",  "covdefaults>=2.3"]
    docs = ["sphinx-argparse-cli>=1.15", "Sphinx>=7.3.7"]
    "#},
        indoc ! {r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
    ]
    optional-dependencies.docs = [
      "sphinx>=7.3.7",
      "sphinx-argparse-cli>=1.15",
    ]
    optional-dependencies.test = [
      "covdefaults>=2.3",
      "pytest>=8.1.1",
    ]
    "#},
        true,
        (3, 9),
        true,
)]
#[case::project_scripts_collapse(
        indoc ! {r#"
    [project.scripts]
    c = 'd'
    a = "b"
    "#},
        indoc ! {r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
    ]
    scripts.a = "b"
    scripts.c = "d"
    "#},
        true,
        (3, 9),
        true,
)]
#[case::project_entry_points_collapse(
        indoc ! {r#"
    [project]
    entry-points.tox = {"tox-uv" = "tox_uv.plugin", "tox" = "tox.plugin"}
    [project.scripts]
    virtualenv = "virtualenv.__main__:run_with_catch"
    [project.gui-scripts]
    hello-world = "timmins:hello_world"
    [project.entry-points."virtualenv.activate"]
    bash = "virtualenv.activation.bash:BashActivator"
    [project.entry-points]
    B = {base = "vehicle_crash_prevention.main:VehicleBase"}
    [project.entry-points."no_crashes.vehicle"]
    base = "vehicle_crash_prevention.main:VehicleBase"
    [project.entry-points.plugin-namespace]
    plugin-name1 = "pkg.subpkg1"
    plugin-name2 = "pkg.subpkg2:func"
    "#},
        indoc ! {r#"
    [project]
    classifiers = [
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
    ]
    scripts.virtualenv = "virtualenv.__main__:run_with_catch"
    gui-scripts.hello-world = "timmins:hello_world"
    entry-points.B.base = "vehicle_crash_prevention.main:VehicleBase"
    entry-points."no_crashes.vehicle".base = "vehicle_crash_prevention.main:VehicleBase"
    entry-points.plugin-namespace.plugin-name1 = "pkg.subpkg1"
    entry-points.plugin-namespace.plugin-name2 = "pkg.subpkg2:func"
    entry-points.tox.tox = "tox.plugin"
    entry-points.tox.tox-uv = "tox_uv.plugin"
    entry-points."virtualenv.activate".bash = "virtualenv.activation.bash:BashActivator"
    "#},
        true,
        (3, 9),
        true,
)]
#[case::project_preserve_implementation_classifiers(
        indoc ! {r#"
    [project]
    requires-python = ">=3.8"
    classifiers = [
      "License :: OSI Approved :: MIT License",
      "Topic :: Software Development :: Libraries :: Python Modules",
      "Programming Language :: Python :: Implementation :: CPython",
      "Programming Language :: Python :: Implementation :: PyPy",
    ]
    "#},
        indoc ! {r#"
    [project]
    requires-python = ">=3.8"
    classifiers = [
      "License :: OSI Approved :: MIT License",
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.8",
      "Programming Language :: Python :: 3.9",
      "Programming Language :: Python :: 3.10",
      "Programming Language :: Python :: Implementation :: CPython",
      "Programming Language :: Python :: Implementation :: PyPy",
      "Topic :: Software Development :: Libraries :: Python Modules",
    ]
    "#},
        true,
        (3, 10),
        true,
)]
#[case::remove_existing_python_classifiers(
    indoc! {r#"
    [project]
    classifiers = [
      "Topic :: Software Development :: Libraries :: Python Modules",
      "Programming Language :: Python :: 3 :: Only",
      "Programming Language :: Python :: 3.9",
      "License :: OSI Approved :: MIT License",
      "Programming Language :: Python :: 3.10",
    ]
    dependencies = ["a>=1.0.0"]
    "#},
    indoc! {r#"
    [project]
    classifiers = [
      "License :: OSI Approved :: MIT License",
      "Topic :: Software Development :: Libraries :: Python Modules",
    ]
    dependencies = [
      "a>=1.0.0",
    ]
    "#},
    true,
    (3, 10),
    false,
)]
#[case::missing_classifiers(
    indoc! {r#"
    [project]
    dependencies = ["a>=1.0.0"]
    "#},
    indoc! {r#"
    [project]
    dependencies = [
      "a>=1.0.0",
    ]
    "#},
    true,
    (3, 10),
    false,
)]
#[case::empty_classifiers(
    indoc! {r#"
    [project]
    classifiers = []
    dependencies = ["a>=1.0.0"]
    "#},
    indoc! {r#"
    [project]
    classifiers = [
    ]
    dependencies = [
      "a>=1.0.0",
    ]
    "#},
    true,
    (3, 10),
    false,
)]
#[case::preserve_non_python_classifiers(
    indoc! {r#"
    [project]
    classifiers = [
      "License :: OSI Approved :: MIT License",
      "Topic :: Software Development :: Libraries :: Python Modules"
    ]
    dependencies = ["a>=1.0.0"]
    "#},
    indoc! {r#"
    [project]
    classifiers = [
      "License :: OSI Approved :: MIT License",
      "Topic :: Software Development :: Libraries :: Python Modules",
    ]
    dependencies = [
      "a>=1.0.0",
    ]
    "#},
    true,
    (3, 10),
    false,
)]
fn test_format_project(
    #[case] start: &str,
    #[case] expected: &str,
    #[case] keep_full_version: bool,
    #[case] max_supported_python: (u8, u8),
    #[case] generate_python_version_classifiers: bool,
) {
    assert_eq!(
        evaluate(
            start,
            keep_full_version,
            max_supported_python,
            generate_python_version_classifiers
        ),
        expected
    );
}
