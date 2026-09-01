use indoc::indoc;

#[test]
fn test_format_build_systems_no_build_system() {
    let start = indoc! {r""};
    let res = format_build_systems_helper(start, false);
    insta::assert_snapshot!(res, @"");
}

#[test]
fn test_format_build_systems_build_system_requires_no_keep() {
    let start = indoc! {r#"
    [build-system]
    requires=["a>=1.0.0", "b.c>=1.5.0"]
    "#};
    let res = format_build_systems_helper(start, false);
    insta::assert_snapshot!(res, @r#"
    [build-system]
    requires = [ "a>=1", "b-c>=1.5" ]
    "#);
}

#[test]
fn test_format_build_systems_build_system_requires_keep() {
    let start = indoc! {r#"
    [build-system]
    requires=["a>=1.0.0", "b.c>=1.5.0"]
    "#};
    let res = format_build_systems_helper(start, true);
    insta::assert_snapshot!(res, @r#"
    [build-system]
    requires = [ "a>=1.0.0", "b-c>=1.5.0" ]
    "#);
}

#[test]
fn test_format_build_systems_join() {
    let start = indoc! {r#"
    [build-system]
    requires=["a"]
    build-backend = "hatchling.build"
    [[build-system.a]]
    name = "Hammer"
    [[build-system.a]]  # empty table within the array
    [[build-system.a]]
    name = "Nail"
    "#};
    let res = format_build_systems_helper(start, false);
    insta::assert_snapshot!(res, @r#"
    [build-system]
    build-backend = "hatchling.build"
    requires = [ "a" ]
    a = [
      { name = "Hammer" },
      # empty table within the array
      {},
      { name = "Nail" }
    ]
    "#);
}

#[test]
fn test_format_build_systems_issue_2_python_version_marker() {
    let start = indoc! {r#"
    [build-system]
    requires = [
      "cython==3.0.11",
      "numpy==1.22.2; python_version<'3.9'",
      "numpy>=2; python_version>='3.9'",
      "setuptools",
    ]
    "#};
    let res = format_build_systems_helper(start, true);
    insta::assert_snapshot!(res, @r#"
    [build-system]
    requires = [
      "cython==3.0.11",
      "numpy==1.22.2; python_version<'3.9'",
      "numpy>=2; python_version>='3.9'",
      "setuptools",
    ]
    "#);
}

/// The frontend puts these directories at the front of `sys.path` in the order they are listed, so
/// the one written first is the one Python searches first.
#[test]
fn test_format_build_systems_backend_path_keeps_its_order() {
    let start = indoc! {r#"
    [build-system]
    build-backend = "backend"
    backend-path = ["src", "lib", "another"]
    "#};
    let res = format_build_systems_helper(start, false);
    insta::assert_snapshot!(res, @r#"
    [build-system]
    build-backend = "backend"
    backend-path = [ "src", "lib", "another" ]
    "#);
}

#[test]
fn test_format_build_systems_setuptools_backend_keeps_constrained_wheel() {
    let start = indoc! {r#"
    [build-system]
    requires = ["setuptools", "wheel>=0.40"]
    build-backend = "setuptools.build_meta"
    "#};
    let res = format_build_systems_helper(start, false);
    insta::assert_snapshot!(res, @r#"
    [build-system]
    build-backend = "setuptools.build_meta"
    requires = [ "setuptools", "wheel>=0.40" ]
    "#);
}

#[test]
fn test_format_build_systems_setuptools_backend_keeps_wheel_with_marker() {
    let start = indoc! {r#"
    [build-system]
    requires = ["setuptools", "wheel; sys_platform=='win32'"]
    build-backend = "setuptools.build_meta"
    "#};
    let res = format_build_systems_helper(start, false);
    insta::assert_snapshot!(res, @r#"
    [build-system]
    build-backend = "setuptools.build_meta"
    requires = [ "setuptools", "wheel; sys_platform=='win32'" ]
    "#);
}

#[test]
fn test_format_build_systems_other_backend_keeps_wheel() {
    let start = indoc! {r#"
    [build-system]
    requires = ["hatchling", "wheel"]
    build-backend = "hatchling.build"
    "#};
    let res = format_build_systems_helper(start, false);
    insta::assert_snapshot!(res, @r#"
    [build-system]
    build-backend = "hatchling.build"
    requires = [ "hatchling", "wheel" ]
    "#);
}

#[test]
fn test_format_build_systems_no_backend_keeps_wheel() {
    let start = indoc! {r#"
    [build-system]
    requires = ["setuptools", "wheel"]
    "#};
    let res = format_build_systems_helper(start, false);
    insta::assert_snapshot!(res, @r#"
    [build-system]
    requires = [ "setuptools", "wheel" ]
    "#);
}

#[test]
fn test_format_build_systems_backend_path_keeps_wheel() {
    let start = indoc! {r#"
    [build-system]
    requires = ["setuptools", "wheel"]
    build-backend = "setuptools.build_meta"
    backend-path = ["_custom"]
    "#};
    let res = format_build_systems_helper(start, false);
    insta::assert_snapshot!(res, @r#"
    [build-system]
    build-backend = "setuptools.build_meta"
    requires = [ "setuptools", "wheel" ]
    backend-path = [ "_custom" ]
    "#);
}

#[test]
fn test_format_build_systems_no_setuptools_keeps_wheel() {
    let start = indoc! {r#"
    [build-system]
    requires = ["wheel"]
    build-backend = "setuptools.build_meta"
    "#};
    let res = format_build_systems_helper(start, false);
    insta::assert_snapshot!(res, @r#"
    [build-system]
    build-backend = "setuptools.build_meta"
    requires = [ "wheel" ]
    "#);
}

#[test]
fn test_format_build_systems_requires_not_an_array() {
    let start = indoc! {r#"
    [build-system]
    build-backend = "setuptools.build_meta"
    requires = "setuptools"
    "#};
    let res = format_build_systems_helper(start, false);
    insta::assert_snapshot!(res, @r#"
    [build-system]
    build-backend = "setuptools.build_meta"
    requires = "setuptools"
    "#);
}

#[test]
fn test_format_build_systems_backend_that_is_not_a_string() {
    let start = indoc! {r#"
    [build-system]
    build-backend = [ "setuptools.build_meta" ]
    requires = [ "setuptools", "wheel" ]
    "#};
    let res = format_build_systems_helper(start, false);
    insta::assert_snapshot!(res, @r#"
    [build-system]
    build-backend = [ "setuptools.build_meta" ]
    requires = [ "setuptools", "wheel" ]
    "#);
}

#[test]
fn test_format_build_systems_keeps_a_requirement_it_cannot_read() {
    let start = indoc! {r#"
    [build-system]
    requires = ["good >= 1.0.0", "!! not a requirement !!"]
    "#};
    insta::assert_snapshot!(format_build_systems_helper(start, false), @r#"
    [build-system]
    requires = [ "!! not a requirement !!", "good>=1" ]
    "#);
}

/// A build requirement is what the author says the build needs. No specifier proves which release a
/// resolver will pick, so a declared `wheel` stays where it was written.
#[test]
fn test_format_build_systems_keeps_a_declared_wheel() {
    for requires in [
        r#"["setuptools", "wheel"]"#,
        r#"["setuptools>=70.1", "wheel"]"#,
        r#"["setuptools<70", "wheel"]"#,
    ] {
        let start = format!("[build-system]\nbuild-backend = \"setuptools.build_meta\"\nrequires = {requires}\n");
        let result = format_build_systems_helper(&start, false);
        assert!(result.contains("\"wheel\""), "{result}");
    }
}

fn format_build_systems_helper(start: &str, keep_full_version: bool) -> String {
    super::evaluate_settings(
        start,
        &_pyproject_fmt::Settings {
            keep_full_version,
            ..super::default_settings()
        },
    )
}
