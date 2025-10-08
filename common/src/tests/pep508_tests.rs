use rstest::rstest;

use crate::pep508::Requirement;

#[rstest]
#[case::lowercase("A", "a")]
#[case::replace_dot_with_dash("a.b", "a-b")]
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
