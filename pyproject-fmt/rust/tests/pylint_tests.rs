use super::evaluate_full;

fn evaluate(start: &str) -> String {
    evaluate_full(start)
}

#[test]
fn test_pylint_main_before_messages_control() {
    let start = indoc::indoc! {r#"
    [tool.pylint.format]
    max-line-length = 120

    [tool.pylint.messages_control]
    disable = ["C0114", "C0115"]

    [tool.pylint.main]
    jobs = 4
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pylint]
    main.jobs = 4
    messages_control.disable = [ "C0114", "C0115" ]
    format.max-line-length = 120
    "#);
}

#[test]
fn test_pylint_disable_enable_sorted() {
    let start = indoc::indoc! {r#"
    [tool.pylint.messages_control]
    disable = ["W0621", "C0114", "R0903", "C0115"]
    enable = ["W0612", "C0103"]
    "#};
    let result = evaluate(start);
    insta::assert_snapshot!(result, @r#"
    [tool.pylint]
    messages_control.disable = [ "C0114", "C0115", "R0903", "W0621" ]
    messages_control.enable = [ "C0103", "W0612" ]
    "#);
}

#[test]
fn test_pylint_idempotent() {
    let start = indoc::indoc! {r#"
    [tool.pylint.main]
    jobs = 4
    [tool.pylint.messages_control]
    disable = [ "C0114", "C0115" ]
    "#};
    let once = evaluate(start);
    let twice = evaluate(&once);
    assert_eq!(once, twice);
}

/// pylint imports each plug-in and registers it in the order they are listed, so what one hook adds
/// is there for the next.
#[test]
fn test_pylint_load_plugins_keeps_its_order() {
    let start = indoc::indoc! {r#"
    [tool.pylint]
    main.load-plugins = ["project.patches", "base.plugin"]
    main.ignore = ["zeta", "alpha"]
    "#};
    let result = evaluate(start);

    insta::assert_snapshot!(result, @r#"
    [tool.pylint]
    main.ignore = [ "alpha", "zeta" ]
    main.load-plugins = [ "project.patches", "base.plugin" ]
    "#);
}

/// pylint reads these into a mapping in the order they are listed, so the last rule for a module is
/// the one it recommends.
#[test]
fn test_pylint_preferred_modules_keeps_its_order() {
    let start = indoc::indoc! {r#"
    [tool.pylint]
    main.preferred-modules = ["json:z-last", "json:a-first"]
    "#};
    let result = evaluate(start);

    insta::assert_snapshot!(result, @r#"
    [tool.pylint]
    main.preferred-modules = [ "json:z-last", "json:a-first" ]
    "#);
}
