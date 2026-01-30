use rstest::rstest;

use crate::pep508::{MarkerExpr, Requirement};

#[rstest]
#[case::lowercase("A", "a")]
#[case::replace_dot_with_dash("a.b", "a-b")]
#[case::replace_underscore_with_dash("a_b", "a-b")]
fn test_get_canonic_requirement_name(#[case] start: &str, #[case] expected: &str) {
    assert_eq!(Requirement::new(start).unwrap().canonical_name(), expected);
}

#[rstest]
#[case::strip_version(
    r#"requests [security , tests] >= 2.0.0, == 2.8.* ; (os_name=="a" or os_name=='b') and os_name=='c' and python_version > "3.8""#,
    "requests[security,tests]>=2,==2.8.*; (os_name=='a' or os_name=='b') and os_name=='c' and python_version>'3.8'",
    false
)]
#[case::keep_version(
    r#"requests [security , tests] >= 2.0.0, == 2.8.* ; (os_name=="a" or os_name=='b') and os_name=='c' and python_version > "3.8""#,
    "requests[security,tests]>=2.0.0,==2.8.*; (os_name=='a' or os_name=='b') and os_name=='c' and python_version>'3.8'",
    true
)]
#[case::do_not_strip_tilda("a~=3.0.0", "a~=3.0.0", false)]
#[case::url(
    " pip   @   https://github.com/pypa/pip/archive/1.3.1.zip#sha1=da9234ee9982d4bbb3c72346a6de940a148ea686 ",
    "pip @ https://github.com/pypa/pip/archive/1.3.1.zip#sha1=da9234ee9982d4bbb3c72346a6de940a148ea686",
    true
)]
#[case::keep_rc_version("a==5.2rc1", "a==5.2rc1", true)]
#[case::pre_release("pkg>=2.7.0rc1", "pkg>=2.7.0rc1", false)]
#[case::post_release("pkg>=2.7.0.post1", "pkg>=2.7.0.post1", false)]
#[case::dev_release("pkg>=2.7.0.dev1", "pkg>=2.7.0.dev1", false)]
#[case::local_version("pkg>=2.7.0+abc", "pkg>=2.7.0+abc", false)]
#[case::pre_post("pkg>=2.7.0rc1.post2", "pkg>=2.7.0rc1.post2", false)]
#[case::pre_dev("pkg>=2.7.0rc1.dev3", "pkg>=2.7.0rc1.dev3", false)]
#[case::pre_local("pkg>=2.7.0rc1+abc", "pkg>=2.7.0rc1+abc", false)]
#[case::post_dev_local("pkg>=2.7.0.post2.dev3+abc", "pkg>=2.7.0.post2.dev3+abc", false)]
#[case::all_segments("pkg>=2.7.0rc1.post2.dev3+abc", "pkg>=2.7.0rc1.post2.dev3+abc", false)]
#[case::pre_release_keep("pkg>=2.7.0rc1", "pkg>=2.7.0rc1", true)]
#[case::parentheses("pkg (>=0.5.5,<0.6.1)", "pkg>=0.5.5,<0.6.1", false)]
#[case::parentheses_extras("pkg [extra] (>=0.5.5,<0.6.1)", "pkg[extra]>=0.5.5,<0.6.1", false)]
#[case::epoch("pkg>=1!2.0.0", "pkg>=1!2", false)]
#[case::alpha_label("pkg>=2.7alpha1", "pkg>=2.7a1", false)]
#[case::beta_label("pkg>=2.7beta2", "pkg>=2.7b2", false)]
#[case::preview_label("pkg>=2.7preview3", "pkg>=2.7rc3", false)]
#[case::pre_label("pkg>=2.7pre", "pkg>=2.7rc0", false)]
#[case::c_label("pkg>=2.7c1", "pkg>=2.7rc1", false)]
#[case::post_no_number("pkg>=2.7.post", "pkg>=2.7.post0", false)]
#[case::dev_no_number("pkg>=2.7.dev", "pkg>=2.7.dev0", false)]
#[case::name_only("requests", "requests", false)]
#[case::private_simple("requests; private", "requests; private", false)]
#[case::private_with_version("requests>=2.0; private", "requests>=2; private", false)]
#[case::private_with_marker(
    "requests>=2.0; os_name=='linux'; private",
    "requests>=2; os_name=='linux'; private",
    false
)]
#[case::private_spacing("requests ;  PRIVATE  ", "requests; private", false)]
fn test_format_requirement(#[case] start: &str, #[case] expected: &str, #[case] keep_full_version: bool) {
    let got = Requirement::new(start)
        .unwrap()
        .normalize(keep_full_version)
        .to_string();
    assert_eq!(got, expected);
    // formatting remains stable
    assert_eq!(
        Requirement::new(got.as_str())
            .unwrap()
            .normalize(keep_full_version)
            .to_string(),
        expected
    );
}

#[rstest]
#[case::simple_comparison("os_name == 'linux'", "os_name=='linux'")]
#[case::and_expr(
    "os_name == 'linux' and python_version > '3.8'",
    "os_name=='linux' and python_version>'3.8'"
)]
#[case::or_expr("os_name == 'linux' or os_name == 'darwin'", "os_name=='linux' or os_name=='darwin'")]
#[case::parentheses("(os_name == 'linux')", "(os_name=='linux')")]
#[case::ident_rhs("platform_machine == arm64", "platform_machine==arm64")]
#[case::in_operator("sys_platform in 'linux'", "sys_platformin'linux'")]
#[case::not_in_operator("sys_platform not in 'win32'", "sys_platformnot in'win32'")]
fn test_marker_expression(#[case] input: &str, #[case] expected: &str) {
    let marker = MarkerExpr::new(input).unwrap();
    assert_eq!(marker.to_string(), expected);
}

#[rstest]
#[case::unclosed_string("os_name == 'linux")]
#[case::unexpected_char("os_name == @value")]
#[case::trailing_tokens("os_name == 'linux' extra")]
#[case::missing_operator("os_name 'linux'")]
#[case::missing_identifier("== 'linux'")]
#[case::unclosed_paren("(os_name == 'linux'")]
#[case::missing_rhs("os_name ==")]
#[case::not_without_in("os_name not foo")]
fn test_marker_expression_errors(#[case] input: &str) {
    assert!(MarkerExpr::new(input).is_err());
}

#[rstest]
#[case::unclosed_extras("pkg[extra>=1.0")]
#[case::invalid_marker("pkg; @@@invalid")]
#[case::invalid_version_op("pkg>=1.0,&2.0")]
fn test_requirement_errors(#[case] input: &str) {
    assert!(Requirement::new(input).is_err());
}

#[test]
fn test_requirement_from_str() {
    let req: Requirement = "requests>=2.0".parse().unwrap();
    assert_eq!(req.to_string(), "requests>=2");
}

#[test]
fn test_requirement_empty_marker() {
    let req = Requirement::new("pkg>=1.0;").unwrap().normalize(false);
    assert_eq!(req.to_string(), "pkg>=1");
}

#[test]
fn test_invalid_epoch_falls_back() {
    let req = Requirement::new("pkg>=abc!1.0").unwrap().normalize(false);
    assert!(req.to_string().starts_with("pkg>="));
}
