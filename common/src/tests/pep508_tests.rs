use crate::pep508::{MarkerExpr, Requirement};

fn format_requirement_helper(start: &str, keep_full_version: bool) -> String {
    Requirement::new(start)
        .unwrap()
        .normalize(keep_full_version)
        .to_string()
}

fn format_marker_helper(input: &str) -> String {
    MarkerExpr::new(input).unwrap().to_string()
}

#[test]
fn test_get_canonic_requirement_name_lowercase() {
    let result = Requirement::new("A").unwrap().canonical_name();
    insta::assert_snapshot!(result, @"a");
}

#[test]
fn test_get_canonic_requirement_name_replace_dot_with_dash() {
    let result = Requirement::new("a.b").unwrap().canonical_name();
    insta::assert_snapshot!(result, @"a-b");
}

#[test]
fn test_get_canonic_requirement_name_replace_underscore_with_dash() {
    let result = Requirement::new("a_b").unwrap().canonical_name();
    insta::assert_snapshot!(result, @"a-b");
}

#[test]
fn test_format_requirement_strip_version() {
    let start = r#"requests [security , tests] >= 2.0.0, == 2.8.* ; (os_name=="a" or os_name=='b') and os_name=='c' and python_version > "3.8""#;
    let got = format_requirement_helper(start, false);
    insta::assert_snapshot!(got.clone(), @"requests[security,tests]>=2,==2.8.*; (os_name=='a' or os_name=='b') and os_name=='c' and python_version>'3.8'");
    let stable = format_requirement_helper(&got, false);
    assert_eq!(stable, got, "formatting should remain stable");
}

#[test]
fn test_format_requirement_keep_version() {
    let start = r#"requests [security , tests] >= 2.0.0, == 2.8.* ; (os_name=="a" or os_name=='b') and os_name=='c' and python_version > "3.8""#;
    let got = format_requirement_helper(start, true);
    insta::assert_snapshot!(got.clone(), @"requests[security,tests]>=2.0.0,==2.8.*; (os_name=='a' or os_name=='b') and os_name=='c' and python_version>'3.8'");
    let stable = format_requirement_helper(&got, true);
    assert_eq!(stable, got, "formatting should remain stable");
}

#[test]
fn test_format_requirement_do_not_strip_tilda() {
    let start = "a~=3.0.0";
    let got = format_requirement_helper(start, false);
    insta::assert_snapshot!(got.clone(), @"a~=3.0.0");
    let stable = format_requirement_helper(&got, false);
    assert_eq!(stable, got, "formatting should remain stable");
}

#[test]
fn test_format_requirement_url() {
    let start =
        " pip   @   https://github.com/pypa/pip/archive/1.3.1.zip#sha1=da9234ee9982d4bbb3c72346a6de940a148ea686 ";
    let got = format_requirement_helper(start, true);
    insta::assert_snapshot!(got.clone(), @"pip @ https://github.com/pypa/pip/archive/1.3.1.zip#sha1=da9234ee9982d4bbb3c72346a6de940a148ea686");
    let stable = format_requirement_helper(&got, true);
    assert_eq!(stable, got, "formatting should remain stable");
}

#[test]
fn test_format_requirement_keep_rc_version() {
    let start = "a==5.2rc1";
    let got = format_requirement_helper(start, true);
    insta::assert_snapshot!(got.clone(), @"a==5.2rc1");
    let stable = format_requirement_helper(&got, true);
    assert_eq!(stable, got, "formatting should remain stable");
}

#[test]
fn test_format_requirement_pre_release() {
    let start = "pkg>=2.7.0rc1";
    let got = format_requirement_helper(start, false);
    insta::assert_snapshot!(got.clone(), @"pkg>=2.7.0rc1");
    let stable = format_requirement_helper(&got, false);
    assert_eq!(stable, got, "formatting should remain stable");
}

#[test]
fn test_format_requirement_post_release() {
    let start = "pkg>=2.7.0.post1";
    let got = format_requirement_helper(start, false);
    insta::assert_snapshot!(got.clone(), @"pkg>=2.7.0.post1");
    let stable = format_requirement_helper(&got, false);
    assert_eq!(stable, got, "formatting should remain stable");
}

#[test]
fn test_format_requirement_dev_release() {
    let start = "pkg>=2.7.0.dev1";
    let got = format_requirement_helper(start, false);
    insta::assert_snapshot!(got.clone(), @"pkg>=2.7.0.dev1");
    let stable = format_requirement_helper(&got, false);
    assert_eq!(stable, got, "formatting should remain stable");
}

#[test]
fn test_format_requirement_local_version() {
    let start = "pkg>=2.7.0+abc";
    let got = format_requirement_helper(start, false);
    insta::assert_snapshot!(got.clone(), @"pkg>=2.7.0+abc");
    let stable = format_requirement_helper(&got, false);
    assert_eq!(stable, got, "formatting should remain stable");
}

#[test]
fn test_format_requirement_pre_post() {
    let start = "pkg>=2.7.0rc1.post2";
    let got = format_requirement_helper(start, false);
    insta::assert_snapshot!(got.clone(), @"pkg>=2.7.0rc1.post2");
    let stable = format_requirement_helper(&got, false);
    assert_eq!(stable, got, "formatting should remain stable");
}

#[test]
fn test_format_requirement_pre_dev() {
    let start = "pkg>=2.7.0rc1.dev3";
    let got = format_requirement_helper(start, false);
    insta::assert_snapshot!(got.clone(), @"pkg>=2.7.0rc1.dev3");
    let stable = format_requirement_helper(&got, false);
    assert_eq!(stable, got, "formatting should remain stable");
}

#[test]
fn test_format_requirement_pre_local() {
    let start = "pkg>=2.7.0rc1+abc";
    let got = format_requirement_helper(start, false);
    insta::assert_snapshot!(got.clone(), @"pkg>=2.7.0rc1+abc");
    let stable = format_requirement_helper(&got, false);
    assert_eq!(stable, got, "formatting should remain stable");
}

#[test]
fn test_format_requirement_post_dev_local() {
    let start = "pkg>=2.7.0.post2.dev3+abc";
    let got = format_requirement_helper(start, false);
    insta::assert_snapshot!(got.clone(), @"pkg>=2.7.0.post2.dev3+abc");
    let stable = format_requirement_helper(&got, false);
    assert_eq!(stable, got, "formatting should remain stable");
}

#[test]
fn test_format_requirement_all_segments() {
    let start = "pkg>=2.7.0rc1.post2.dev3+abc";
    let got = format_requirement_helper(start, false);
    insta::assert_snapshot!(got.clone(), @"pkg>=2.7.0rc1.post2.dev3+abc");
    let stable = format_requirement_helper(&got, false);
    assert_eq!(stable, got, "formatting should remain stable");
}

#[test]
fn test_format_requirement_pre_release_keep() {
    let start = "pkg>=2.7.0rc1";
    let got = format_requirement_helper(start, true);
    insta::assert_snapshot!(got.clone(), @"pkg>=2.7.0rc1");
    let stable = format_requirement_helper(&got, true);
    assert_eq!(stable, got, "formatting should remain stable");
}

#[test]
fn test_format_requirement_parentheses() {
    let start = "pkg (>=0.5.5,<0.6.1)";
    let got = format_requirement_helper(start, false);
    insta::assert_snapshot!(got.clone(), @"pkg>=0.5.5,<0.6.1");
    let stable = format_requirement_helper(&got, false);
    assert_eq!(stable, got, "formatting should remain stable");
}

#[test]
fn test_format_requirement_parentheses_extras() {
    let start = "pkg [extra] (>=0.5.5,<0.6.1)";
    let got = format_requirement_helper(start, false);
    insta::assert_snapshot!(got.clone(), @"pkg[extra]>=0.5.5,<0.6.1");
    let stable = format_requirement_helper(&got, false);
    assert_eq!(stable, got, "formatting should remain stable");
}

#[test]
fn test_format_requirement_epoch() {
    let start = "pkg>=1!2.0.0";
    let got = format_requirement_helper(start, false);
    insta::assert_snapshot!(got.clone(), @"pkg>=1!2");
    let stable = format_requirement_helper(&got, false);
    assert_eq!(stable, got, "formatting should remain stable");
}

#[test]
fn test_format_requirement_alpha_label() {
    let start = "pkg>=2.7alpha1";
    let got = format_requirement_helper(start, false);
    insta::assert_snapshot!(got.clone(), @"pkg>=2.7a1");
    let stable = format_requirement_helper(&got, false);
    assert_eq!(stable, got, "formatting should remain stable");
}

#[test]
fn test_format_requirement_beta_label() {
    let start = "pkg>=2.7beta2";
    let got = format_requirement_helper(start, false);
    insta::assert_snapshot!(got.clone(), @"pkg>=2.7b2");
    let stable = format_requirement_helper(&got, false);
    assert_eq!(stable, got, "formatting should remain stable");
}

#[test]
fn test_format_requirement_preview_label() {
    let start = "pkg>=2.7preview3";
    let got = format_requirement_helper(start, false);
    insta::assert_snapshot!(got.clone(), @"pkg>=2.7rc3");
    let stable = format_requirement_helper(&got, false);
    assert_eq!(stable, got, "formatting should remain stable");
}

#[test]
fn test_format_requirement_pre_label() {
    let start = "pkg>=2.7pre";
    let got = format_requirement_helper(start, false);
    insta::assert_snapshot!(got.clone(), @"pkg>=2.7rc0");
    let stable = format_requirement_helper(&got, false);
    assert_eq!(stable, got, "formatting should remain stable");
}

#[test]
fn test_format_requirement_c_label() {
    let start = "pkg>=2.7c1";
    let got = format_requirement_helper(start, false);
    insta::assert_snapshot!(got.clone(), @"pkg>=2.7rc1");
    let stable = format_requirement_helper(&got, false);
    assert_eq!(stable, got, "formatting should remain stable");
}

#[test]
fn test_format_requirement_post_no_number() {
    let start = "pkg>=2.7.post";
    let got = format_requirement_helper(start, false);
    insta::assert_snapshot!(got.clone(), @"pkg>=2.7.post0");
    let stable = format_requirement_helper(&got, false);
    assert_eq!(stable, got, "formatting should remain stable");
}

#[test]
fn test_format_requirement_dev_no_number() {
    let start = "pkg>=2.7.dev";
    let got = format_requirement_helper(start, false);
    insta::assert_snapshot!(got.clone(), @"pkg>=2.7.dev0");
    let stable = format_requirement_helper(&got, false);
    assert_eq!(stable, got, "formatting should remain stable");
}

#[test]
fn test_format_requirement_name_only() {
    let start = "requests";
    let got = format_requirement_helper(start, false);
    insta::assert_snapshot!(got.clone(), @"requests");
    let stable = format_requirement_helper(&got, false);
    assert_eq!(stable, got, "formatting should remain stable");
}

#[test]
fn test_format_requirement_private_simple() {
    let start = "requests; private";
    let got = format_requirement_helper(start, false);
    insta::assert_snapshot!(got.clone(), @"requests; private");
    let stable = format_requirement_helper(&got, false);
    assert_eq!(stable, got, "formatting should remain stable");
}

#[test]
fn test_format_requirement_private_with_version() {
    let start = "requests>=2.0; private";
    let got = format_requirement_helper(start, false);
    insta::assert_snapshot!(got.clone(), @"requests>=2; private");
    let stable = format_requirement_helper(&got, false);
    assert_eq!(stable, got, "formatting should remain stable");
}

#[test]
fn test_format_requirement_private_with_marker() {
    let start = "requests>=2.0; os_name=='linux'; private";
    let got = format_requirement_helper(start, false);
    insta::assert_snapshot!(got.clone(), @"requests>=2; os_name=='linux'; private");
    let stable = format_requirement_helper(&got, false);
    assert_eq!(stable, got, "formatting should remain stable");
}

#[test]
fn test_format_requirement_private_spacing() {
    let start = "requests ;  PRIVATE  ";
    let got = format_requirement_helper(start, false);
    insta::assert_snapshot!(got.clone(), @"requests; private");
    let stable = format_requirement_helper(&got, false);
    assert_eq!(stable, got, "formatting should remain stable");
}

#[test]
fn test_marker_expression_simple_comparison() {
    let input = "os_name == 'linux'";
    let result = format_marker_helper(input);
    insta::assert_snapshot!(result, @"os_name=='linux'");
}

#[test]
fn test_marker_expression_and_expr() {
    let input = "os_name == 'linux' and python_version > '3.8'";
    let result = format_marker_helper(input);
    insta::assert_snapshot!(result, @"os_name=='linux' and python_version>'3.8'");
}

#[test]
fn test_marker_expression_or_expr() {
    let input = "os_name == 'linux' or os_name == 'darwin'";
    let result = format_marker_helper(input);
    insta::assert_snapshot!(result, @"os_name=='linux' or os_name=='darwin'");
}

#[test]
fn test_marker_expression_parentheses() {
    let input = "(os_name == 'linux')";
    let result = format_marker_helper(input);
    insta::assert_snapshot!(result, @"(os_name=='linux')");
}

#[test]
fn test_marker_expression_ident_rhs() {
    let input = "platform_machine == arm64";
    let result = format_marker_helper(input);
    insta::assert_snapshot!(result, @"platform_machine==arm64");
}

#[test]
fn test_marker_expression_in_operator() {
    let input = "sys_platform in 'linux'";
    let result = format_marker_helper(input);
    insta::assert_snapshot!(result, @"sys_platformin'linux'");
}

#[test]
fn test_marker_expression_not_in_operator() {
    let input = "sys_platform not in 'win32'";
    let result = format_marker_helper(input);
    insta::assert_snapshot!(result, @"sys_platformnot in'win32'");
}

#[test]
fn test_marker_expression_errors_unclosed_string() {
    assert!(MarkerExpr::new("os_name == 'linux").is_err());
}

#[test]
fn test_marker_expression_errors_unexpected_char() {
    assert!(MarkerExpr::new("os_name == @value").is_err());
}

#[test]
fn test_marker_expression_errors_trailing_tokens() {
    assert!(MarkerExpr::new("os_name == 'linux' extra").is_err());
}

#[test]
fn test_marker_expression_errors_missing_operator() {
    assert!(MarkerExpr::new("os_name 'linux'").is_err());
}

#[test]
fn test_marker_expression_errors_missing_identifier() {
    assert!(MarkerExpr::new("== 'linux'").is_err());
}

#[test]
fn test_marker_expression_errors_unclosed_paren() {
    assert!(MarkerExpr::new("(os_name == 'linux'").is_err());
}

#[test]
fn test_marker_expression_errors_missing_rhs() {
    assert!(MarkerExpr::new("os_name ==").is_err());
}

#[test]
fn test_marker_expression_errors_not_without_in() {
    assert!(MarkerExpr::new("os_name not foo").is_err());
}

#[test]
fn test_requirement_errors_unclosed_extras() {
    assert!(Requirement::new("pkg[extra>=1.0").is_err());
}

#[test]
fn test_requirement_errors_invalid_marker() {
    assert!(Requirement::new("pkg; @@@invalid").is_err());
}

#[test]
fn test_requirement_errors_invalid_version_op() {
    assert!(Requirement::new("pkg>=1.0,&2.0").is_err());
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
