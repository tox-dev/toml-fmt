use common::pep508::{MarkerExpr, Requirement, is_valid_version};

/// Two spellings of one distribution name are the same name, which is what decides where a
/// requirement sorts against the others.
#[test]
fn a_distribution_name_reads_the_same_however_it_is_spelled() {
    for (name, written, canonical) in [
        ("lowercase", "A", "a"),
        ("replace_dot_with_dash", "a.b", "a-b"),
        ("replace_underscore_with_dash", "a_b", "a-b"),
    ] {
        assert_eq!(
            Requirement::new(written).expect("a requirement").canonical_name(),
            canonical,
            "{name}"
        );
    }
}

/// A requirement is written in one form however the file spelled it, and reading that form back
/// says the same thing: what the formatter writes is what it reads.
#[test]
fn a_requirement_is_written_in_one_form() {
    for (name, start, keep_full_version, written) in [
        (
            "strip_version",
            r#"requests [security , tests] >= 2.0.0, == 2.8.* ; (os_name=="a" or os_name=='b') and os_name=='c' and python_version > "3.8""#,
            false,
            "requests[security,tests]>=2,==2.8.*; (os_name=='a' or os_name=='b') and os_name=='c' and python_version>'3.8'",
        ),
        (
            "keep_version",
            r#"requests [security , tests] >= 2.0.0, == 2.8.* ; (os_name=="a" or os_name=='b') and os_name=='c' and python_version > "3.8""#,
            true,
            "requests[security,tests]>=2.0.0,==2.8.*; (os_name=='a' or os_name=='b') and os_name=='c' and python_version>'3.8'",
        ),
        ("do_not_strip_tilda", "a~=3.0.0", false, "a~=3.0.0"),
        (
            "url_with_marker",
            "pytest-notebook @ git+https://github.com/x/pytest-notebook.git@master ; python_version>='3.10'",
            true,
            "pytest-notebook @ git+https://github.com/x/pytest-notebook.git@master ; python_version>='3.10'",
        ),
        ("keep_rc_version", "a==5.2rc1", true, "a==5.2rc1"),
        ("pre_release", "pkg>=2.7.0rc1", false, "pkg>=2.7.0rc1"),
        ("post_release", "pkg>=2.7.0.post1", false, "pkg>=2.7.0.post1"),
        ("dev_release", "pkg>=2.7.0.dev1", false, "pkg>=2.7.0.dev1"),
        ("local_version", "pkg==2.7.0+abc", false, "pkg==2.7.0+abc"),
        ("pre_post", "pkg>=2.7.0rc1.post2", false, "pkg>=2.7.0rc1.post2"),
        ("pre_dev", "pkg>=2.7.0rc1.dev3", false, "pkg>=2.7.0rc1.dev3"),
        ("pre_local", "pkg==2.7.0rc1+abc", false, "pkg==2.7.0rc1+abc"),
        (
            "post_dev_local",
            "pkg==2.7.0.post2.dev3+abc",
            false,
            "pkg==2.7.0.post2.dev3+abc",
        ),
        (
            "all_segments",
            "pkg==2.7.0rc1.post2.dev3+abc",
            false,
            "pkg==2.7.0rc1.post2.dev3+abc",
        ),
        ("pre_release_keep", "pkg>=2.7.0rc1", true, "pkg>=2.7.0rc1"),
        ("parentheses", "pkg (>=0.5.5,<0.6.1)", false, "pkg>=0.5.5,<0.6.1"),
        (
            "parentheses_extras",
            "pkg [extra] (>=0.5.5,<0.6.1)",
            false,
            "pkg[extra]>=0.5.5,<0.6.1",
        ),
        ("epoch", "pkg>=1!2.0.0", false, "pkg>=1!2"),
        ("alpha_label", "pkg>=2.7alpha1", false, "pkg>=2.7a1"),
        ("beta_label", "pkg>=2.7beta2", false, "pkg>=2.7b2"),
        ("preview_label", "pkg>=2.7preview3", false, "pkg>=2.7rc3"),
        ("pre_label", "pkg>=2.7pre", false, "pkg>=2.7rc0"),
        ("c_label", "pkg>=2.7c1", false, "pkg>=2.7rc1"),
        ("post_no_number", "pkg>=2.7.post", false, "pkg>=2.7.post0"),
        ("dev_no_number", "pkg>=2.7.dev", false, "pkg>=2.7.dev0"),
        ("name_only", "requests", false, "requests"),
    ] {
        let got = format_requirement_helper(start, keep_full_version);

        assert_eq!(got, written, "{name}");
        assert_eq!(
            format_requirement_helper(&got, keep_full_version),
            got,
            "{name} settled on its first pass"
        );
    }
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

/// A marker is written in one form however the file spelled it, so two files asking the same thing
/// read the same afterwards.
#[test]
fn a_marker_is_written_in_one_form() {
    for (name, written, expected) in [
        ("simple_comparison", "os_name == 'linux'", "os_name=='linux'"),
        (
            "and_expr",
            "os_name == 'linux' and python_version > '3.8'",
            "os_name=='linux' and python_version>'3.8'",
        ),
        (
            "or_expr",
            "os_name == 'linux' or os_name == 'darwin'",
            "os_name=='linux' or os_name=='darwin'",
        ),
        ("parentheses", "(os_name == 'linux')", "(os_name=='linux')"),
        ("in_operator", "sys_platform in 'linux'", "sys_platform in 'linux'"),
        (
            "not_in_operator",
            "sys_platform not in 'win32'",
            "sys_platform not in 'win32'",
        ),
        ("compatible_release", "python_version ~= '3.8'", "python_version~='3.8'"),
    ] {
        assert_eq!(format_marker_helper(written), expected, "{name}");
    }
}

/// PEP 508 names the variables a marker reads and quotes every other value, so a bare word on
/// either side is text this parser cannot read. A quoted value stands on either side.
#[test]
fn test_marker_operands_are_variables_or_quoted_values() {
    for input in [
        "platform_machine == arm64",
        "python_version = '3.10'",
        "unknown == 'x'",
        "python_version <> '3.10'",
    ] {
        assert!(MarkerExpr::new(input).is_err(), "{input}");
    }
    insta::assert_snapshot!(format_marker_helper("'3.10' == python_version"), @"'3.10'==python_version");
}

/// A marker the grammar does not read says nothing about where a requirement applies, so it is
/// rejected rather than read as far as it goes.
#[test]
fn a_marker_the_grammar_does_not_read_is_rejected() {
    for (name, written) in [
        ("unclosed_string", "os_name == 'linux"),
        ("unexpected_char", "os_name == @value"),
        ("trailing_tokens", "os_name == 'linux' extra"),
        ("missing_operator", "os_name 'linux'"),
        ("missing_identifier", "== 'linux'"),
        ("unclosed_paren", "(os_name == 'linux'"),
        ("missing_rhs", "os_name =="),
        ("not_without_in", "os_name not foo"),
    ] {
        assert!(MarkerExpr::new(written).is_err(), "{name}");
    }
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

/// `;` opens a marker, so a delimiter with nothing after it names one this parser cannot read.
#[test]
fn test_requirement_empty_marker() {
    assert!(Requirement::new("pkg>=1.0;").is_err());
}

/// A release number is read as the digits the file wrote, and a caller comparing it with a small
/// one reads the same answer whether or not it fits a machine integer.
#[test]
fn test_a_release_number_reads_as_the_digits_it_names() {
    let held = |raw: &str| {
        let requirement = Requirement::new(raw).unwrap();
        let clause = &requirement.version_ops()[0];
        clause.version().unwrap().release.first().unwrap().saturating()
    };

    assert_eq!(
        (held("pkg==7"), held("pkg==0"), held("pkg==18446744073709551616")),
        (7, 0, u64::MAX)
    );
}

/// A version this parser cannot write back out is one the caller leaves as the file wrote it, and
/// PEP 440 puts no limit on how large a number a release names.
#[test]
fn test_a_version_pep_440_does_not_read_is_not_a_requirement() {
    assert!(Requirement::new("pkg>=abc!1.0").is_err());

    let read = |raw: &str| Requirement::new(raw).unwrap().normalize(false).to_string();
    assert_eq!(read("pkg==18446744073709551616"), "pkg==18446744073709551616");
    assert_eq!(read("pkg==18446744073709551616!2"), "pkg==18446744073709551616!2");
    assert_eq!(
        read("pkg==1.0-18446744073709551616"),
        "pkg==1.0.post18446744073709551616"
    );
}

/// PEP 440 writes an implicit post release as the number alone, and allows a `v` before the
/// release, both of which say the same as the spelling written out.
#[test]
fn test_the_spellings_pep_440_allows_for_one_version() {
    let read = |raw: &str| Requirement::new(raw).unwrap().normalize(false).to_string();

    assert_eq!(
        (
            read("pkg==v1.2"),
            read("pkg==1.2-1"),
            read("pkg==1.2rev1"),
            read("pkg==1.2r1")
        ),
        (
            String::from("pkg==1.2"),
            String::from("pkg==1.2.post1"),
            String::from("pkg==1.2.post1"),
            String::from("pkg==1.2.post1"),
        )
    );
}

/// A bracket that closes before it opens names no extras, and neither does an empty one.
/// A bracket that closes before it opens names no extras. One written empty names none either,
/// which is what leaving the brackets off says.
#[test]
fn test_brackets_that_name_no_extras() {
    assert!(Requirement::new("pkg]x[extra").is_err());
    assert!(Requirement::new("pkg[a,,b]").is_err());
    assert_eq!(
        (
            Requirement::new("pkg[]").map(|found| found.normalize(false).to_string()),
            Requirement::new("pkg[ ]").map(|found| found.normalize(false).to_string()),
            Requirement::new("pkg>=1,").map(|found| found.normalize(false).to_string()),
            Requirement::new("pkg(>=1,)").map(|found| found.normalize(false).to_string()),
        ),
        (
            Ok(String::from("pkg")),
            Ok(String::from("pkg")),
            Ok(String::from("pkg>=1")),
            Ok(String::from("pkg>=1")),
        )
    );
}

#[test]
fn test_a_direct_reference_without_a_url_is_not_a_requirement() {
    assert!(Requirement::new("pkg @ ").is_err());
}

/// `===` compares the text it is given, so nothing about that text is rewritten and it need not be
/// a version at all.
#[test]
fn test_arbitrary_equality_keeps_the_text_it_was_given() {
    let read = |raw: &str| Requirement::new(raw).unwrap().normalize(false).to_string();

    assert_eq!(
        (read("Pkg===1.0.0"), read("Pkg===v1.0-1"), read("Pkg===foobar")),
        (
            String::from("pkg===1.0.0"),
            String::from("pkg===v1.0-1"),
            String::from("pkg===foobar"),
        )
    );
}

/// `===` holds text that may be no version at all, which is what the clause hands back.
#[test]
fn test_a_clause_names_the_version_it_compares_against() {
    let clause = |raw: &str| Requirement::new(raw).unwrap().version_ops()[0].version().is_some();

    assert_eq!(
        (clause("pkg>=1"), clause("pkg===1.0"), clause("pkg===foobar")),
        (true, true, false)
    );

    let literal = |raw: &str| Requirement::new(raw).unwrap().version_ops()[0].literal().to_owned();
    assert_eq!(
        (literal("pkg===v1.0"), literal("pkg>=1.0")),
        (String::from("v1.0"), String::from("1.0"))
    );
}

/// A parenthesis opens the list of versions and the matching one closes it, so a lone one is text
/// this parser cannot read rather than one to drop.
#[test]
fn test_a_version_list_the_parentheses_do_not_close_is_not_a_requirement() {
    for raw in ["pkg(>=1", "pkg>=1)", "pkg()", "pkg(foo)", "pkg[extra](>=1"] {
        assert!(Requirement::new(raw).is_err(), "{raw}");
    }
    assert_eq!(
        Requirement::new("pkg (>=1,<2)").unwrap().normalize(false).to_string(),
        "pkg>=1,<2"
    );
}

/// PEP 440 puts a wildcard only after `==` or `!=`, and a compatible release names at least two
/// numbers to hold the last one open.
#[test]
fn test_an_operator_and_a_version_pep_440_does_not_pair_is_not_a_requirement() {
    for raw in [
        "pkg=== foo bar",
        "pkg===foo/bar",
        "pkg>=1+local",
        "pkg~=1.2+local",
        "pkg>=1.*",
        "pkg<=1.*",
        "pkg~=1",
        "pkg>=",
        "pkg===",
        "pkg==1rc1.*",
        "pkg==1.post1.*",
        "pkg==1.dev1.*",
        "pkg==1+local.*",
    ] {
        assert!(Requirement::new(raw).is_err(), "{raw}");
    }
}

/// A requirement that names nothing but a package is the one a rule may rewrite freely; anything
/// the file said about it beyond the name is part of what it asks for.
#[test]
fn a_requirement_says_whether_it_is_a_name_and_nothing_else() {
    for (name, written, only) in [
        ("bare_name", "wheel", true),
        ("version_constraint", "wheel>=0.40", false),
        ("extras", "wheel[extra]", false),
        ("marker", "wheel; sys_platform=='win32'", false),
        ("url", "wheel @ https://example.com/wheel.whl", false),
    ] {
        assert_eq!(
            Requirement::new(written).expect("a requirement").is_name_only(),
            only,
            "{name}"
        );
    }
}

/// PEP 794 writes `private` after an import name, which is a different field from a dependency.
#[test]
fn test_a_private_modifier_is_not_a_dependency_marker() {
    assert!(Requirement::new("wheel; private").is_err());
}

/// A version says what PEP 440 says one says: what the spec spells out is read, and what it does
/// not is not a version however much it reads like one.
#[test]
fn a_version_is_read_the_way_the_spec_writes_one() {
    for (name, written, valid) in [
        ("plain_release", "1.9.0", true),
        ("calver_leading_zeros", "2026.08.10", true),
        ("surrounding_whitespace", " 1.9.0 ", true),
        ("v_prefix", "v1.9", true),
        ("epoch", "2!1.0", true),
        ("pre_release_spelling", "1.0-ALPHA.1", true),
        ("implicit_post_release", "1.0-1", true),
        ("all_segments", "v1.0.c1.post.dev+Ubuntu_007-x", true),
        ("large_numbers", "99999999999999999999.0", true),
        ("rejects_non_numeric_release", "1.9.xyz", false),
        ("rejects_empty", "", false),
        ("rejects_trailing_separator", "1.0.0-", false),
        ("rejects_wildcard", "1.0.*", false),
    ] {
        assert_eq!(is_valid_version(written), valid, "{name}");
    }
}

#[test]
fn test_normalize_spells_out_a_pre_release() {
    insta::assert_snapshot!(format_requirement_helper("pkg==1.0alpha", false), @"pkg==1.0a0");
}

/// A requirement this parser cannot read whole is one it does not read at all, so nothing a caller
/// writes back drops what the file said.
#[test]
fn test_a_requirement_is_read_whole_or_not_at_all() {
    let refused = ["", "@@@", "pkg[extra]junk", "pkg(>=1)junk", "pkg[extra", "-pkg", "pkg-"];

    let read: Vec<&str> = refused
        .into_iter()
        .filter(|raw| Requirement::new(raw).is_ok())
        .collect();

    assert_eq!(read, Vec::<&str>::new());
}

/// A marker holding text outside ASCII is read by the bytes it is written in, so what comes back is
/// what went in.
#[test]
fn test_a_marker_keeps_text_outside_ascii_whole() {
    let read = Requirement::new("pkg; os_name == 'café'").expect("a marker with an accent");

    assert_eq!(read.normalize(false).to_string(), "pkg; os_name=='café'");
}

/// A URL holds no whitespace and may hold a semicolon of its own, so what separates a direct
/// reference from its marker is the space written beside it.
/// A URL holds no whitespace and may hold a semicolon of its own, so what opens a marker beside one
/// is the space PEP 508 writes before it. `private` belongs to PEP 794 import names, not here.
#[test]
fn test_a_semicolon_inside_a_url_stays_in_the_url() {
    let read = |raw: &str| Requirement::new(raw).map(|found| found.normalize(false).to_string());

    assert_eq!(
        read("pkg @ https://example.test/a;keep=1"),
        Ok(String::from("pkg @ https://example.test/a;keep=1"))
    );
    assert_eq!(
        read("pkg @ https://example.test/a ; python_version<'3'"),
        Ok(String::from("pkg @ https://example.test/a ; python_version<'3'"))
    );
    for raw in [
        "pkg @ https://example.test/a; python_version<'3'",
        "pkg @ https://example.test/a ; private",
        "pkg; private",
        "pkg>=1;",
        "pkg>=1@https://example.test/a",
        "pkg[a]>=1@https://example.test/a",
        "pkg\n>=1",
    ] {
        assert!(Requirement::new(raw).is_err(), "{raw}");
    }
}

/// PEP 508 has no escape for the quote a marker value is written in, so the value is written in the
/// one it does not hold.
#[test]
fn test_a_marker_value_is_written_in_a_quote_it_does_not_hold() {
    let read = Requirement::new("pkg; os_name == \"Bob's OS\"").expect("a marker holding a quote");

    assert_eq!(read.normalize(false).to_string(), "pkg; os_name==\"Bob's OS\"");
}

/// A URL holds no whitespace, so text after one is a requirement this parser does not read.
#[test]
fn test_text_after_a_url_is_not_read_as_a_requirement() {
    let read = Requirement::new("pkg @ https://example.test/a and more");

    assert!(read.is_err(), "{read:?}");
}

/// The constraints a requirement names, which is what decides whether a version it allows is one
/// the caller can rely on.
#[test]
fn test_a_requirement_says_what_it_constrains() {
    let read = Requirement::new("pkg>=1,<2").expect("a bounded requirement");
    let plain = Requirement::new("pkg").expect("a name");

    assert_eq!(
        (
            read.version_ops()
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>(),
            plain.version_ops().len()
        ),
        (vec![String::from(">=1"), String::from("<2")], 0)
    );
}

/// A name on its own is what `project.name` holds, so anything naming a dependency is not one.
#[test]
fn test_a_distribution_name_is_read_on_its_own() {
    let read = |name: &str| Requirement::canonical_name_of(name);

    assert_eq!(
        (read("Pkg_Name"), read("pkg[feature]").is_err(), read("").is_err()),
        (Ok(String::from("pkg-name")), true, true)
    );
}

/// PEP 440 puts no limit on how many digits a release names, so the numbers compare and count as
/// the file wrote them rather than as a machine integer holds them.
#[test]
fn a_release_number_counts_every_digit_the_file_wrote() {
    let number = |digits: &str| common::pep508::Number::written(digits).expect("digits");

    assert!(number("18446744073709551616") > number("18446744073709551615"));
    assert!(number("9") < number("10"));
    assert_eq!(number("9").succ(), number("10"));
    assert_eq!(number("18446744073709551615").succ(), number("18446744073709551616"));
    assert_eq!(number("10").pred(), Some(number("9")));
    assert_eq!(number("0").pred(), None);
    assert_eq!(number("100").pred(), Some(number("99")));
    assert_eq!(common::pep508::Number::zero(), number("0"));
    assert!(common::pep508::Number::zero().is_zero());
}

/// A number is the digits PEP 440 counts: a leading zero names none of its own, and text that is
/// not digits names no number at all.
#[test]
fn text_that_is_not_a_number_the_file_wrote_is_none() {
    let written = common::pep508::Number::written;

    assert!(written("00").is_none());
    assert!(written("01").is_none());
    assert!(written("").is_none());
    assert!(written("1.0").is_none());
    assert!(written("1rc1").is_none());
    assert!(written("0").is_some());
}

/// PEP 508 sets no limit on how deeply a marker groups, and a machine does: a marker past what a
/// stack holds is text this cannot read, which leaves the requirement as the file wrote it.
#[test]
fn a_marker_grouped_deeper_than_a_stack_holds_is_not_one_this_reads() {
    let grouped = |depth: usize| {
        let marker = format!("{}python_version == '3.10'{}", "(".repeat(depth), ")".repeat(depth));
        common::pep508::Requirement::new(&format!("pkg; {marker}")).map(|held| held.to_string())
    };

    assert_eq!(
        grouped(256),
        Ok(format!(
            "pkg; {}python_version=='3.10'{}",
            "(".repeat(256),
            ")".repeat(256)
        ))
    );
    assert!(grouped(257).is_err());
    assert!(grouped(10_000).is_err());
}

/// A marker the grammar does not read is rejected wherever it stops reading, including inside a
/// run of `and`, a run of `or`, and the parentheses one of those sits in.
#[test]
fn a_marker_that_stops_reading_partway_is_rejected() {
    for written in [
        "os_name == 'a' and",
        "os_name == 'a' or",
        "os_name == 'a' and or",
        "(os_name == 'a' and)",
        "(and)",
    ] {
        assert!(MarkerExpr::new(written).is_err(), "{written}");
    }
}

fn format_requirement_helper(start: &str, keep_full_version: bool) -> String {
    Requirement::new(start)
        .unwrap()
        .normalize(keep_full_version)
        .to_string()
}

fn format_marker_helper(input: &str) -> String {
    MarkerExpr::new(input).unwrap().to_string()
}
