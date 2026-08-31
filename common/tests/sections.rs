//! Finding tables and ordering their keys.

use common::layout::Layout;
use common::sections;
use toml_doc::{Document, LineEnding};

fn parse(source: &str) -> Document<'_> {
    toml_doc::parse(source).expect("valid source")
}

/// Reordering leaves the separators where position put them, so the layout pass follows it just as
/// it does in the formatter.
fn written(document: &mut Document<'_>) -> String {
    Layout {
        column_width: 120,
        indent: 2,
        ending: LineEnding::Lf,
    }
    .apply(document);
    document.to_string()
}

#[test]
fn keys_follow_the_given_order_and_the_rest_go_alphabetically() {
    let mut document = parse("[tool.x]\nzebra = 1\nname = 2\nalpha = 3\nversion = 4\n");
    let section = sections::first(&mut document, "tool.x").expect("section");
    sections::reorder_keys(&mut section.entries, &["name", "version"]);

    assert_eq!(
        document.to_string(),
        "[tool.x]\nname = 2\nversion = 4\nalpha = 3\nzebra = 1\n"
    );
}

#[test]
fn a_named_key_pulls_its_dotted_children_along() {
    let mut document = parse("[tool.x]\nother = 1\nlint.select = 2\nlint.ignore = 3\n");
    let section = sections::first(&mut document, "tool.x").expect("section");
    sections::reorder_keys(&mut section.entries, &["lint"]);

    assert_eq!(
        document.to_string(),
        "[tool.x]\nlint.ignore = 3\nlint.select = 2\nother = 1\n"
    );
}

#[test]
fn entries_never_cross_a_group_marker() {
    let mut document = parse("[tool.x]\nb = 1\na = 2\n# Group: two\nz = 3\ny = 4\n");
    let section = sections::first(&mut document, "tool.x").expect("section");
    sections::reorder_keys(&mut section.entries, &[]);

    assert_eq!(
        document.to_string(),
        "[tool.x]\na = 2\nb = 1\n# Group: two\ny = 4\nz = 3\n"
    );
}

#[test]
fn a_moved_entry_takes_its_comment_with_it() {
    let mut document = parse("[tool.x]\n# about zebra\nzebra = 1\nalpha = 2\n");
    let section = sections::first(&mut document, "tool.x").expect("section");
    sections::reorder_keys(&mut section.entries, &[]);

    assert_eq!(document.to_string(), "[tool.x]\nalpha = 2\n# about zebra\nzebra = 1\n");
}

#[test]
fn a_repeated_header_names_several_sections() {
    let mut document = parse("[[tool.x]]\na = 1\n[[tool.x]]\na = 2\n");

    assert_eq!(sections::named(&mut document, "tool.x").len(), 2);
}

#[test]
fn renaming_a_key_leaves_its_value_alone() {
    let mut document = parse("[tool.x]\nold = 1\nkeep = 2\n");
    let section = sections::first(&mut document, "tool.x").expect("section");
    sections::rename_keys(&mut section.entries, &[("old", "new")]);

    assert_eq!(document.to_string(), "[tool.x]\nnew = 1\nkeep = 2\n");
}

#[test]
fn a_value_can_be_found_by_key() {
    let mut document = parse("[tool.x]\nwanted = 7\n");
    let section = sections::first(&mut document, "tool.x").expect("section");

    assert_eq!(
        sections::find(&mut section.entries, "wanted").map(|value| value.to_string()),
        Some("7".to_owned())
    );
}

#[test]
fn every_entry_is_visited_with_its_key() {
    let mut document = parse("[tool.x]\na = 1\nb.c = 2\n");
    let section = sections::first(&mut document, "tool.x").expect("section");
    let mut seen = Vec::new();
    sections::for_entries(section, |key, value| seen.push(format!("{key}={value}")));

    assert_eq!(seen, ["a=1", "b.c=2"]);
}

#[test]
fn a_recognised_inline_table_gets_its_keys_ordered() {
    let mut document = parse("a = [{ zebra = 1, module = 2, alpha = 3 }]\n");
    sections::reorder_inline_tables(
        &mut document,
        &[],
        &[sections::InlineSchema {
            discriminator: "module",
            key_order: &["module"],
        }],
    );

    assert_eq!(written(&mut document), "a = [ { module = 2, alpha = 3, zebra = 1 } ]\n");
}

#[test]
fn an_inline_table_without_the_discriminator_is_left_alone() {
    let source = "a = { zebra = 1, alpha = 2 }\n";
    let mut document = parse(source);
    sections::reorder_inline_tables(
        &mut document,
        &[],
        &[sections::InlineSchema {
            discriminator: "module",
            key_order: &["module"],
        }],
    );

    assert_eq!(written(&mut document), source);
}

#[test]
fn tables_land_in_the_order_they_are_named() {
    let mut document =
        parse("[demo]\nz = 1\n\n[project]\nname = \"x\"\n\n[tool.ruff.lint]\na = 1\n\n[tool.ruff]\nb = 2\n");
    sections::reorder_within(&mut document, &["project", "tool.ruff"], &["tool"], &|_| None);

    assert_eq!(
        written(&mut document),
        "[project]\nname = \"x\"\n\n[tool.ruff]\nb = 2\n\n[tool.ruff.lint]\na = 1\n\n[demo]\nz = 1\n"
    );
}

#[test]
fn a_sub_table_sits_where_its_name_sits_among_the_keys() {
    let mut document = parse("[tool.coverage.report]\na = 1\n\n[tool.coverage.run]\nb = 2\n");
    sections::reorder_within(&mut document, &["tool.coverage"], &["tool"], &|name| {
        (name == ["tool", "coverage"]).then(|| vec![String::from("run"), String::from("report")])
    });

    assert_eq!(
        written(&mut document),
        "[tool.coverage.run]\nb = 2\n\n[tool.coverage.report]\na = 1\n"
    );
}

#[test]
fn a_header_with_nothing_under_it_stays_next_to_the_table_it_opens() {
    let mut document = parse("[tool.a]\n\n[tool.a.b]\nx = 1\n");
    sections::reorder_within(&mut document, &[], &["tool"], &|_| None);

    assert_eq!(written(&mut document), "[tool.a]\n[tool.a.b]\nx = 1\n");
}

/// The keys before the first header are their own table, which no name reaches.
#[test]
fn the_keys_before_the_first_header_are_their_own_table() {
    let mut document = parse("root = 1\n\n[\"\"]\nempty = 1\n");
    let mut seen: Vec<String> = Vec::new();
    sections::with_root_entries(&mut document, |entries| {
        seen.extend(entries.iter().map(|entry| entry.key_value.key.path()));
    });

    assert_eq!(seen, ["root"]);
}

/// A name selects the table the file gave it, the empty name included, so nothing a file can write
/// is out of reach.
#[test]
fn the_entries_of_a_named_table_can_be_visited() {
    let mut document = parse("root = 1\n\n[tool.a]\nx = 1\n\n[\"\"]\nempty = 1\n");
    let mut seen: Vec<String> = Vec::new();
    for name in ["tool.a", ""] {
        sections::with_entries(&mut document, name, |entries| {
            seen.extend(entries.iter().map(|entry| entry.key_value.key.path()));
        });
    }

    assert_eq!(seen, ["x", "empty"]);
    assert_eq!(sections::names(&document), ["tool.a", "\"\""]);
}

#[test]
fn a_group_marker_holds_the_keys_on_either_side_of_it() {
    let mut document = parse("[tool.a]\nz = 1\ny = 2\n# Group: later\nb = 3\na = 4\n");
    let section = sections::first(&mut document, "tool.a").expect("the table is written");
    sections::reorder_keys(&mut section.entries, &[]);

    assert_eq!(
        written(&mut document),
        "[tool.a]\ny = 2\nz = 1\n# Group: later\na = 4\nb = 3\n"
    );
}

#[test]
fn a_key_that_cannot_be_read_is_left_where_it_is() {
    let mut document = parse("[tool.a]\nb = 1\n");
    let section = sections::first(&mut document, "tool.a").expect("the table is written");
    sections::rename_keys(&mut section.entries, &[("b", "c")]);
    sections::for_entries(section, |key, _value| assert_eq!(key, "c"));

    assert_eq!(written(&mut document), "[tool.a]\nc = 1\n");
}

#[test]
fn a_table_with_nothing_in_it_reorders_to_itself() {
    let mut document = parse("[tool.a]\n");
    let section = sections::first(&mut document, "tool.a").expect("the table is written");
    sections::reorder_keys(&mut section.entries, &["b"]);

    assert_eq!(written(&mut document), "[tool.a]\n");
}

#[test]
fn an_empty_line_above_a_key_does_not_open_a_group() {
    let mut document = parse("[tool.a]\n\nz = 1\n\ny = 2\n");
    let section = sections::first(&mut document, "tool.a").expect("the table is written");
    sections::reorder_keys(&mut section.entries, &[]);

    assert_eq!(written(&mut document), "[tool.a]\ny = 2\nz = 1\n");
}

#[test]
fn an_inline_table_a_schema_recognizes_gets_its_keys_ordered() {
    let mut document = parse(
        "root = { name = \"x\", include-group = \"g\" }\n\n[tool.a]\nb = { name = \"y\", include-group = \"h\" }\n",
    );
    sections::reorder_inline_tables(
        &mut document,
        &[],
        &[sections::InlineSchema {
            discriminator: "include-group",
            key_order: &["include-group", "name"],
        }],
    );

    assert_eq!(
        written(&mut document),
        "root = { include-group = \"g\", name = \"x\" }\n\n[tool.a]\nb = { include-group = \"h\", name = \"y\" }\n"
    );
}

/// A `[fruit.physical]` written under a `[[fruit]]` belongs to that element. Sorting the two apart
/// would attach it to whichever element ended up above it, or say the same table twice.
#[test]
fn an_array_element_and_the_tables_under_it_move_as_one() {
    let source = concat!(
        "[[fruit]]\nname = \"apple\"\n",
        "[fruit.physical]\ncolor = \"red\"\n",
        "[[fruit.variety]]\nname = \"red delicious\"\n",
        "[[fruit]]\nname = \"banana\"\n",
        "[fruit.physical]\ncolor = \"yellow\"\n",
        "[[fruit.variety]]\nname = \"plantain\"\n",
    );
    let mut document = parse(source);
    sections::reorder_within(&mut document, &[], &[], &|_| None);
    let out = written(&mut document);

    assert!(out.parse::<toml::Table>().is_ok(), "{out}");
    assert_eq!(
        out,
        concat!(
            "[[fruit]]\nname = \"apple\"\n\n",
            "[fruit.physical]\ncolor = \"red\"\n\n",
            "[[fruit.variety]]\nname = \"red delicious\"\n\n",
            "[[fruit]]\nname = \"banana\"\n\n",
            "[fruit.physical]\ncolor = \"yellow\"\n\n",
            "[[fruit.variety]]\nname = \"plantain\"\n",
        )
    );
}

/// A table the order does not name keeps the place its group was first written in; the tables
/// under it still sort among themselves.
#[test]
fn a_table_no_order_names_keeps_the_place_the_file_gave_it() {
    let mut document = parse("[b]\nx = 1\n\n[a.deep]\ny = 2\n\n[a]\nz = 3\n");
    sections::reorder_within(&mut document, &[], &[], &|_| None);

    assert_eq!(written(&mut document), "[b]\nx = 1\n\n[a]\nz = 3\n\n[a.deep]\ny = 2\n");
}

#[test]
fn two_tools_no_order_names_keep_the_order_the_file_gave_them() {
    let mut document = parse(concat!(
        "[tool.zebra]\nb = 1\n\n[tool.zebra.sub]\nc = 2\n",
        "\n[tool.alpha]\nd = 3\n\n[tool.alpha.sub]\ne = 4\n",
    ));
    sections::reorder_within(&mut document, &[], &["tool"], &|_| None);

    assert_eq!(
        written(&mut document),
        concat!(
            "[tool.zebra]\nb = 1\n\n[tool.zebra.sub]\nc = 2\n",
            "\n[tool.alpha]\nd = 3\n\n[tool.alpha.sub]\ne = 4\n",
        )
    );
}

/// A `# Group:` marker holds a boundary that ordering must not cross, as it does for keys and
/// array members.
#[test]
fn tables_do_not_cross_a_group_marker() {
    let mut document = parse(concat!(
        "# Group: one\n[tool.zzz]\na = 1\n\n[tool.aaa]\nb = 2\n",
        "\n# Group: two\n[tool.yyy]\nc = 3\n\n[tool.bbb]\nd = 4\n",
    ));
    sections::reorder_within(&mut document, &["tool.aaa", "tool.bbb"], &["tool"], &|_| None);

    assert_eq!(
        written(&mut document),
        concat!(
            "# Group: one\n[tool.aaa]\nb = 2\n\n[tool.zzz]\na = 1\n",
            "\n# Group: two\n[tool.bbb]\nd = 4\n\n[tool.yyy]\nc = 3\n",
        )
    );
}

/// The two kinds of grouping compose: an array element and the tables under it stay one block
/// inside the partition its marker opens.
#[test]
fn a_group_marker_and_an_array_element_hold_together() {
    let source = concat!(
        "# Group: one\n[[fruit]]\nname = \"apple\"\n\n[fruit.physical]\ncolor = \"red\"\n",
        "\n# Group: two\n[tool.zzz]\na = 1\n\n[tool.aaa]\nb = 2\n",
    );
    let before: toml::Table = source.parse().expect("valid source");
    let mut document = parse(source);
    sections::reorder_within(&mut document, &[], &["tool"], &|_| None);
    let out = written(&mut document);

    assert_eq!(out.parse::<toml::Table>().expect("valid output"), before, "{out}");
    assert!(out.starts_with("# Group: one\n[[fruit]]"), "{out}");
    assert!(out.contains("# Group: two\n[tool.zzz]"), "{out}");
}

/// Renaming an older spelling on top of the canonical one would say the same key twice.
#[test]
fn renaming_leaves_a_key_alone_when_the_new_name_is_taken() {
    let mut document = parse("[tool.a]\nenvlist = [ \"one\" ]\nenv_list = [ \"two\" ]\n");
    let section = sections::first(&mut document, "tool.a").expect("the table is written");
    sections::rename_keys(&mut section.entries, &[("envlist", "env_list")]);
    let out = written(&mut document);

    assert!(out.parse::<toml::Table>().is_ok(), "{out}");
    assert_eq!(out, "[tool.a]\nenvlist = [ \"one\" ]\nenv_list = [ \"two\" ]\n");
}

#[test]
fn renaming_still_rewrites_a_name_nothing_else_holds() {
    let mut document = parse("[tool.a]\nenvlist = [ \"one\" ]\n");
    let section = sections::first(&mut document, "tool.a").expect("the table is written");
    sections::rename_keys(&mut section.entries, &[("envlist", "env_list")]);

    assert_eq!(written(&mut document), "[tool.a]\nenv_list = [ \"one\" ]\n");
}

/// A child table does not have to sit next to the element it belongs to; a later header still
/// resolves under the most recent matching element.
#[test]
fn an_array_child_keeps_its_element_across_an_unrelated_table() {
    let source = concat!(
        "[[fruit]]\nname = \"apple\"\n",
        "[other]\nx = 1\n",
        "[fruit.physical]\ncolor = \"red\"\n",
        "[[fruit]]\nname = \"banana\"\n",
    );
    let before: toml::Table = source.parse().expect("valid source");
    let mut document = parse(source);
    sections::reorder_within(&mut document, &[], &[], &|_| None);
    let out = written(&mut document);

    assert_eq!(out.parse::<toml::Table>().expect("valid output"), before, "{out}");
}

#[test]
fn nested_array_elements_keep_what_belongs_to_them() {
    let source = concat!(
        "[[fruit]]\nname = \"apple\"\n",
        "[[fruit.variety]]\nname = \"red delicious\"\n",
        "[fruit.variety.origin]\nplace = \"here\"\n",
        "[[fruit.variety]]\nname = \"granny smith\"\n",
        "[zzz]\nx = 1\n",
        "[fruit.variety.origin]\nplace = \"there\"\n",
        "[[fruit]]\nname = \"banana\"\n",
    );
    let before: toml::Table = source.parse().expect("valid source");
    let mut document = parse(source);
    sections::reorder_within(&mut document, &[], &[], &|_| None);
    let out = written(&mut document);

    assert_eq!(out.parse::<toml::Table>().expect("valid output"), before, "{out}");
}

/// `a.b` is the dotted path of two plain names; a segment the file quoted because it holds a dot
/// reads back in the form the file wrote it, which no rule written against a bare name can match.
#[test]
fn a_quoted_key_holding_a_dot_is_not_the_dotted_path() {
    let document = parse("a.b = 1\n\"a.b\" = 2\n'c.d' = 3\nplain = 4\n");
    let name = |at: usize| sections::dispatch_name(&document.root[at].key_value.key);

    assert_eq!(name(0), "a.b");
    assert_eq!(name(1), "\"a.b\"");
    assert_eq!(name(2), "\"c.d\"");
    assert_eq!(name(3), "plain");
}

#[test]
fn a_table_read_out_of_the_document_is_found_by_its_segments() {
    let mut document = parse("[env.\"3.13t\"]\nb = 1\n[env]\nc = 2\n");
    let name = document.sections[0].header.key.segments();
    let mut seen = Vec::new();
    sections::with_entries_of(&mut document, &name, |entries| {
        seen.extend(entries.iter().map(|entry| entry.key_value.key.path()));
    });

    assert_eq!(seen, ["b"]);
}

/// A name read out of the document is looked up and collected by its segments, so one the file
/// quoted because it holds a dot is the one name it wrote.
#[test]
fn names_read_out_of_the_document_keep_their_segments() {
    let mut document = parse(concat!(
        "[tool.x.\"a.b\"]\nk = 1\n",
        "[tool.x.plain]\nj = 2\n",
        "[tool.y]\ngroup.\"c.d\".deps = [ ]\ngroup.other.deps = [ ]\n",
    ));

    assert_eq!(sections::headers_below(&document, &["tool", "x"]), ["a.b", "plain"]);
    assert_eq!(sections::headers_below(&document, &["tool"]), ["x", "y"]);
    assert_eq!(sections::headers_below(&document, &["nothing"]), Vec::<String>::new());

    let entries = &sections::first(&mut document, "tool.y").expect("written").entries;
    assert_eq!(sections::keys_below(entries, &["group"]), ["c.d", "other"]);
    assert_eq!(sections::keys_below(entries, &["group", "c.d"]), ["deps"]);
    assert_eq!(sections::keys_below(entries, &["nothing"]), Vec::<String>::new());

    let name = ["tool", "x", "a.b"].map(str::to_owned);
    assert!(sections::first_of(&mut document, &name).is_some());
    assert!(sections::first_of(&mut document, &["tool".to_owned(), "x".to_owned(), "a".to_owned()]).is_none());
}

/// `tool."a.b"` and `tool.a` are different tools, so neither the ordering nor the spacing may read
/// one as the other.
#[test]
fn a_quoted_tool_name_is_its_own_group() {
    let source = concat!(
        "[tool.\"a.b\"]\nx = 1\n\n[tool.\"a.b\".child]\ny = 2\n",
        "\n[tool.a]\nz = 3\n\n[tool.a.child]\nw = 4\n",
    );
    let before: toml::Table = source.parse().expect("valid source");
    let mut document = parse(source);
    sections::reorder_within(&mut document, &[], &["tool"], &|_| None);
    let out = written(&mut document);

    assert_eq!(out.parse::<toml::Table>().expect("valid output"), before, "{out}");
    assert_eq!(
        sections::names(&document),
        ["tool.\"a.b\"", "tool.\"a.b\".child", "tool.a", "tool.a.child"]
    );
}

#[test]
fn every_section_under_a_repeated_name_is_reached() {
    let mut document = parse("[[a.b]]\nx = 1\n[[a.b]]\ny = 2\n[[a.c]]\nz = 3\n");
    let name = ["a", "b"].map(str::to_owned);

    assert_eq!(sections::named_of(&mut document, &name).len(), 2);
    assert_eq!(sections::named_of(&mut document, &["a".to_owned()]).len(), 0);
}

/// A setting names a table the way TOML does, so a quoted segment is the one name it holds.
#[test]
fn a_configured_name_reads_as_the_segments_it_names() {
    assert_eq!(sections::parse_name("a.b.c"), ["a", "b", "c"]);
    assert_eq!(sections::parse_name("a.\"b.c\""), ["a", "b.c"]);
    assert_eq!(sections::parse_name("a.'b.c'"), ["a", "b.c"]);
    assert_eq!(sections::parse_name("plain"), ["plain"]);
    // a name TOML cannot read as a key names no table, and the caller is told why
    assert!(sections::read_name("a.\"unclosed").is_err());
    assert!(sections::read_name("").is_err());
    // a name is the whole of what it says: text carrying its own key, comment or table says more
    assert!(sections::read_name("a = 1\nb").is_err());
    assert!(sections::read_name("a # held").is_err());
    assert!(sections::read_name("a = 0\n#").is_err());
    assert_eq!(
        sections::read_name("a.b"),
        Ok(vec![String::from("a"), String::from("b")])
    );
}

/// An element of an array of tables is the same element whichever way the file writes it, so one
/// rule reaches both.
#[test]
fn an_array_of_tables_reads_the_same_written_out_as_folded_in() {
    let order = &["", "name", "url"];
    let path = ["tool", "x", "source"].map(str::to_owned);
    let run = |source: &str| {
        let mut document = parse(source);
        sections::for_array_elements(&mut document, &path, order, &mut |key, value| {
            if key == "url" {
                *value = toml_doc::Value::Scalar(toml_doc::Repr::basic_string("seen"));
            }
        });
        document.to_string()
    };

    assert_eq!(
        run("[[tool.x.source]]\nurl = \"a\"\nname = \"b\"\n"),
        "[[tool.x.source]]\nname = \"b\"\nurl = \"seen\"\n"
    );
    assert_eq!(
        run("[tool.x]\nsource = [ { url = \"a\", name = \"b\" } ]\n"),
        "[tool.x]\nsource = [ {name = \"b\" , url = \"seen\" } ]\n"
    );
    assert_eq!(
        run("tool.x.source = [ { url = \"a\", name = \"b\" } ]\n"),
        "tool.x.source = [ {name = \"b\" , url = \"seen\" } ]\n"
    );
}

/// What is written at the name but is not an array of inline tables is left as it is.
#[test]
fn an_array_of_tables_rule_leaves_everything_else_alone() {
    let order = &["", "name"];
    let path = ["tool", "x", "source"].map(str::to_owned);
    let run = |source: &str| {
        let mut document = parse(source);
        sections::for_array_elements(&mut document, &path, order, &mut |_, _| {});
        document.to_string()
    };

    assert_eq!(run("[tool.x]\nsource = \"one\"\n"), "[tool.x]\nsource = \"one\"\n");
    assert_eq!(run("[tool.x]\nsource = [ 1, 2 ]\n"), "[tool.x]\nsource = [ 1, 2 ]\n");
    assert_eq!(
        run("[tool.y]\nsource = [ { b = 1 } ]\n"),
        "[tool.y]\nsource = [ { b = 1 } ]\n"
    );
    assert_eq!(
        run("[tool.x.other]\nsource = [ { b = 1 } ]\n"),
        "[tool.x.other]\nsource = [ { b = 1 } ]\n"
    );
}

/// The keys written under an ordered name hold the order the file gave them, since where each one
/// sits among the others is part of what it says.
#[test]
fn the_keys_of_an_ordered_name_hold_their_place() {
    let mut document = parse("[tool.x]\nz = 1\npaths.zulu = 2\npaths.alpha = 3\na = 4\n");
    let section = sections::first(&mut document, "tool.x").expect("section");
    sections::reorder_keys_within(&mut section.entries, &["paths"], &["paths"]);

    assert_eq!(
        written(&mut document),
        "[tool.x]\npaths.zulu = 2\npaths.alpha = 3\na = 4\nz = 1\n"
    );
}

/// A table written under an ordered name holds the place the file gave it among the tables beside
/// it, whatever it is called.
#[test]
fn the_tables_of_an_ordered_name_hold_their_place() {
    let mut document = parse(concat!(
        "[tool.x.hooks.zebra]\na = 1\n\n[tool.x.hooks.alpha]\nb = 2\n\n",
        "[tool.x.envs.demo.overrides.zebra]\nc = 3\n\n[tool.x.envs.demo.overrides.alpha]\nd = 4\n\n",
        "[tool.x.plain.zebra]\ne = 5\n\n[tool.x.plain.alpha]\nf = 6\n",
    ));
    sections::reorder_within_keeping(&mut document, &["tool.x"], &["tool"], &|_| None, &|_| {
        vec![String::from("hooks"), String::from("overrides")]
    });

    assert_eq!(
        written(&mut document),
        concat!(
            "[tool.x.envs.demo.overrides.zebra]\nc = 3\n\n[tool.x.envs.demo.overrides.alpha]\nd = 4\n\n",
            "[tool.x.hooks.zebra]\na = 1\n\n[tool.x.hooks.alpha]\nb = 2\n\n",
            "[tool.x.plain.alpha]\nf = 6\n\n[tool.x.plain.zebra]\ne = 5\n",
        )
    );
}

/// A key the file wrote as a comment says nothing to a rule reading what the file says.
#[test]
fn a_disabled_key_is_not_read_as_an_entry() {
    let mut seen = Vec::new();
    let source = "[tool.x]\n# a = 1\nb = 2\n";
    let mut document = parse(source);
    common::disabled::try_with_disabled_keys(&mut document, source, |document| {
        let section = sections::first(document, "tool.x").expect("section");
        sections::for_entries(section, |key, _value| seen.push(key.to_owned()));
        Ok(())
    })
    .expect("the pass wrote a document");

    assert_eq!(seen, ["b"]);
}

/// Reordering breaks up whatever grouping the empty lines marked, except around a disabled key: it
/// is a comment the file wrote, and the lines around it are part of what it says.
#[test]
fn reordering_keeps_the_lines_written_around_a_disabled_key() {
    let source = "[tool.x]\na = 1\n\n# b = 2\n";
    let mut document = parse(source);
    let formatted = common::disabled::try_with_disabled_keys(&mut document, source, |document| {
        let section = sections::first(document, "tool.x").expect("section");
        sections::reorder_keys(&mut section.entries, &[]);
        Ok(())
    })
    .expect("the pass wrote a document");

    assert_eq!(formatted, source);
}

/// Every value the document holds is reachable, wherever the file wrote it.
#[test]
fn every_value_the_document_holds_is_handed_over() {
    let mut document = parse("a = 1\n[tool.x]\nb = 2\nc = 3\n");

    let written: Vec<String> = sections::every_value(&mut document)
        .into_iter()
        .map(|value| value.to_string())
        .collect();

    assert_eq!(written, ["1", "2", "3"]);
}

/// TOML gives every spelling of a table the same name, so a rule reads the same keys whichever one
/// the file chose.
#[test]
fn the_keys_under_a_table_are_read_however_the_file_splits_its_path() {
    let seen = |source: &str| {
        let mut document = parse(source);
        let path = vec![String::from("tool"), String::from("black")];
        let mut held = Vec::new();
        sections::for_keys_under(&mut document, &path, |name, _| held.push(name.to_owned()));
        held
    };

    assert_eq!(seen("tool.black.line-length = 1\n"), ["line-length"]);
    assert_eq!(seen("[tool]\nblack.line-length = 1\n"), ["line-length"]);
    assert_eq!(seen("[tool.black]\nline-length = 1\n"), ["line-length"]);
    assert_eq!(seen("[tool.black.sub]\na = 1\n"), ["sub.a"]);
    assert_eq!(seen("tool.black = { line-length = 1 }\n"), ["line-length"]);
    assert_eq!(seen("tool = { black = { line-length = 1 } }\n"), ["line-length"]);
    assert_eq!(seen("[tool.other]\na = 1\n"), Vec::<String>::new());
    // a value written where the table belongs holds no key of its own
    assert_eq!(seen("tool.black = 1\n"), Vec::<String>::new());
    // a key the file wrote as a comment says nothing to a rule reading what the file says
    assert_eq!(seen("[tool.black]\n# a = 1\nb = 2\n"), ["b"]);
}

/// A key written as part of a longer path sits among keys that say something else, so it moves only
/// past the ones under the table it belongs to.
#[test]
fn ordering_a_table_leaves_what_the_file_wrote_beside_it_alone() {
    let written = |source: &str| {
        let mut document = parse(source);
        let path = vec![String::from("tool"), String::from("black")];
        sections::reorder_under(&mut document, &path, &["line-length", "target-version"]);
        document.to_string()
    };

    assert_eq!(
        written("[tool.black]\ntarget-version = 1\nline-length = 2\n"),
        "[tool.black]\nline-length = 2\ntarget-version = 1\n"
    );
    assert_eq!(
        written("[tool]\nruff.a = 0\nblack.target-version = 1\nblack.line-length = 2\nruff.b = 3\n"),
        "[tool]\nruff.a = 0\nblack.line-length = 2\nblack.target-version = 1\nruff.b = 3\n"
    );
    assert_eq!(
        written("black.target-version = 1\n[tool]\n"),
        "black.target-version = 1\n[tool]\n"
    );
    assert_eq!(written("[other]\na = 1\n"), "[other]\na = 1\n");
}

/// A key the file wrote as a comment is ordered with the table it belongs to, and the blank lines
/// around it are part of what that comment says.
#[test]
fn a_disabled_key_is_ordered_with_the_table_it_was_written_in() {
    let source = "[tool.x]\n\nz = 1\n\n# a = 2\n";
    let mut document = parse(source);
    let formatted = common::disabled::try_with_disabled_keys(&mut document, source, |document| {
        sections::reorder_under(document, &["tool".to_owned(), "x".to_owned()], &["a", "z"]);
        Ok(())
    })
    .expect("the pass wrote a document");

    assert_eq!(formatted, "[tool.x]\n\n# a = 2\nz = 1\n");
}

/// A table written as a value is the table it names, so its keys are put in the same order as the
/// ones the file wrote out.
#[test]
fn the_keys_of_a_table_written_as_a_value_are_ordered_too() {
    let written = |source: &str| {
        let mut document = parse(source);
        let path = vec![String::from("tool"), String::from("black")];
        sections::reorder_under(&mut document, &path, &["line-length", "target-version"]);
        // the members carry the spacing the file wrote around them, which the layout writes again
        common::layout::Layout {
            column_width: 120,
            indent: 2,
            ending: toml_doc::LineEnding::Lf,
        }
        .apply(&mut document);
        document.to_string()
    };

    assert_eq!(
        written("tool.black = { target-version = 1, line-length = 2 }\n"),
        "tool.black = { line-length = 2, target-version = 1 }\n"
    );
    assert_eq!(
        written("tool = { black = { target-version = 1, line-length = 2 } }\n"),
        "tool = { black = { line-length = 2, target-version = 1 } }\n"
    );
}

/// An order names the keys it speaks for. A table below the one being ordered holds keys of its
/// own, and they keep the order the file gave them unless the order names one of them.
#[test]
fn a_table_the_order_says_nothing_about_keeps_what_the_file_wrote() {
    let written = |source: &str, order: &[&str]| {
        let mut document = parse(source);
        sections::reorder_under(&mut document, &[String::from("tool")], order);
        document.to_string()
    };

    // `lint.select` says where `select` sits inside `lint`
    assert_eq!(
        written("[tool.lint]\nignore = 1\nselect = 2\n", &["lint.select", "lint.ignore"]),
        "[tool.lint]\nselect = 2\nignore = 1\n"
    );
    // `authors` says where the authors sit and nothing about what one holds
    assert_eq!(
        written("[[tool.authors]]\nname = 1\nemail = 2\n", &["authors"]),
        "[[tool.authors]]\nname = 1\nemail = 2\n"
    );
    assert_eq!(
        written("[tool]\nreadme = { file = 1, content-type = 2 }\n", &["readme"]),
        "[tool]\nreadme = { file = 1, content-type = 2 }\n"
    );
}

/// Where each key sits among the ones under the same name is part of what it says, so a name the
/// caller keeps in order holds the sequence the file gave it.
#[test]
fn a_name_kept_in_order_holds_the_sequence_the_file_wrote() {
    let mut document = parse("[tool]\nblack.paths.z = 1\nblack.line-length = 2\nblack.paths.a = 3\n");
    let path = vec![String::from("tool"), String::from("black")];
    sections::reorder_under_keeping(&mut document, &path, &["paths", "line-length"], &["paths"]);

    assert_eq!(
        document.to_string(),
        "[tool]\nblack.paths.z = 1\nblack.paths.a = 3\nblack.line-length = 2\n"
    );
}

/// A rule that renames what a key names reaches it wherever the file wrote it.
#[test]
fn a_name_under_a_table_is_renamed_wherever_the_file_wrote_it() {
    let written = |source: &str| {
        let mut document = parse(source);
        let path = vec![String::from("project")];
        sections::for_names_under(&mut document, &path, |tail, key| {
            if tail == ["extras", "Held"] {
                key.parts_mut().last_mut().expect("a segment").set_name("held");
            }
        });
        document.to_string()
    };

    assert_eq!(written("[project]\nextras.Held = 1\n"), "[project]\nextras.held = 1\n");
    assert_eq!(written("[project.extras]\nHeld = 1\n"), "[project.extras]\nheld = 1\n");
    assert_eq!(written("project.extras.Held = 1\n"), "project.extras.held = 1\n");
    assert_eq!(written("other.extras.Held = 1\n"), "other.extras.Held = 1\n");
}

/// A rule that adds or splits entries works on the container the file wrote them in.
#[test]
fn every_run_of_entries_under_a_table_is_reached() {
    let mut document = parse("other.a = 1\n[project]\nb = 2\n[project.sub]\nc = 3\n[other.more]\nd = 4\n");
    let path = vec![String::from("project")];
    let mut seen = Vec::new();
    sections::for_entry_runs(&mut document, &path, |entries, under| {
        seen.push((under.to_vec(), entries.len()));
    });

    assert_eq!(
        seen,
        [
            (Vec::new(), 1),
            (vec![String::from("project")], 1),
            (vec![String::from("project"), String::from("sub")], 1),
        ]
    );
}

/// A table folded into its parent is written inside an array, and the path already says which table
/// it is, so its keys are ordered without it naming itself.
#[test]
fn the_tables_inside_an_array_are_ordered_by_the_path_that_holds_them() {
    let written = |source: &str| {
        let mut document = parse(source);
        let path = vec![String::from("tool"), String::from("x"), String::from("source")];
        sections::reorder_array_tables_at(&mut document, &path, &["name", "url"]);
        // the members carry the spacing the file wrote around them, which the layout writes again
        common::layout::Layout {
            column_width: 120,
            indent: 2,
            ending: toml_doc::LineEnding::Lf,
        }
        .apply(&mut document);
        document.to_string()
    };

    assert_eq!(
        written("[tool.x]\nsource = [ { url = \"u\", name = \"n\" } ]\n"),
        "[tool.x]\nsource = [ { name = \"n\", url = \"u\" } ]\n"
    );
    assert_eq!(
        written("tool.x.source = [ { url = \"u\", name = \"n\" } ]\n"),
        "tool.x.source = [ { name = \"n\", url = \"u\" } ]\n"
    );
    // an array holding anything else keeps what it holds, and so does a value that is not one
    assert_eq!(
        written("[tool.x]\nsource = [ 1, 2 ]\n"),
        "[tool.x]\nsource = [ 1, 2 ]\n"
    );
    assert_eq!(written("[tool.x]\nsource = 1\n"), "[tool.x]\nsource = 1\n");
    assert_eq!(written("[tool.x]\nother = 1\n"), "[tool.x]\nother = 1\n");
}

/// A table written as a value is reached wherever the file wrote it, and a value that is not one
/// holds nothing to reach.
#[test]
fn the_table_written_as_a_value_is_reached() {
    let seen = |source: &str| {
        let mut document = parse(source);
        let path = vec![String::from("tool"), String::from("x")];
        let mut held = Vec::new();
        sections::for_table_at(&mut document, &path, |table| held.push(table.members.len()));
        held
    };

    assert_eq!(seen("tool.x = { a = 1, b = 2 }\n"), [2]);
    assert_eq!(seen("[tool]\nx = {}\n"), [0]);
    assert_eq!(seen("[tool]\nx = 1\n"), Vec::<usize>::new());
    assert_eq!(seen("[tool.x]\na = 1\n"), Vec::<usize>::new());
}

/// A `# Group:` marker names the keys below it, so a key never crosses one, in a table the file
/// wrote as a value as much as in one it wrote out.
#[test]
fn a_member_never_crosses_a_group_marker() {
    let written = |source: &str| {
        let mut document = parse(source);
        let path = vec![String::from("tool"), String::from("x")];
        sections::reorder_under(&mut document, &path, &["a", "z"]);
        document.to_string()
    };

    assert_eq!(
        written("tool.x = { z = 1,\n# Group: later\na = 2 }\n"),
        "tool.x = { z = 1,\n# Group: later\na = 2 }\n"
    );
    assert_eq!(written("tool.x = { z = 1, a = 2 }\n"), "tool.x = {a = 2 , z = 1 }\n");
    // a table the order says nothing about keeps what the file wrote
    assert_eq!(written("other.y = { z = 1, a = 2 }\n"), "other.y = { z = 1, a = 2 }\n");
}

/// Every table written inside an array is ordered the same way, one authored group at a time.
#[test]
fn a_table_inside_an_array_keeps_its_groups() {
    let source = "tool.x.source = [ { url = \"u\",\n# Group: auth\nname = \"n\" } ]\n";
    let mut document = parse(source);
    let path = vec![String::from("tool"), String::from("x"), String::from("source")];
    sections::reorder_array_tables_at(&mut document, &path, &["name", "url"]);

    assert_eq!(document.to_string(), source);
}

/// An alias moves the name of a key wherever the file wrote it, and a file already spelling the
/// newer name keeps both as written.
#[test]
fn a_name_an_alias_moves_is_renamed_wherever_it_is_written() {
    let written = |source: &str| {
        let mut document = parse(source);
        let path = vec![String::from("env"), String::from("test")];
        let renamed = sections::rename_under(&mut document, &path, &[("setenv", "set_env")]);
        (document.to_string(), renamed)
    };

    assert_eq!(
        written("[env.test]\nsetenv = 1\n"),
        (
            String::from("[env.test]\nset_env = 1\n"),
            vec![(String::from("setenv"), String::from("set_env"))]
        )
    );
    assert_eq!(
        written("env.test.setenv = 1\n").0,
        String::from("env.test.set_env = 1\n")
    );
    assert_eq!(
        written("env = { test = { setenv = 1 } }\n").0,
        String::from("env = { test = { set_env = 1 } }\n")
    );
    // a file already writing the newer name keeps both, since one table says a name once
    assert_eq!(
        written("[env.test]\nsetenv = 1\nset_env = 2\n"),
        (String::from("[env.test]\nsetenv = 1\nset_env = 2\n"), Vec::new())
    );
    // an alias names one key of the table, so a name written below one is not the key it moves
    assert_eq!(
        written("[env.test]\nsetenv.A = 1\n").0,
        String::from("[env.test]\nsetenv.A = 1\n")
    );
}

/// The name each key of an environment table names, which is what a per-table rule matches on.
fn env_table(named: &[String]) -> Option<Vec<String>> {
    (named.len() > 2 && named[0] == "env").then(|| named[..2].to_vec())
}

#[test]
fn every_key_is_visited_with_the_whole_path_it_names() {
    let mut document = parse("env.a.deps = 1\n[env.b]\nset_env.X = 2\nrunner = { of = 3 }\n");
    let mut seen: Vec<String> = Vec::new();
    sections::for_key_paths_under(&mut document, &["env".to_owned()], |named, _| {
        seen.push(named.join("."));
    });

    assert_eq!(
        seen,
        ["env.a.deps", "env.b.set_env.X", "env.b.runner", "env.b.runner.of"]
    );
}

#[test]
fn one_pass_orders_the_keys_of_every_table_it_names() {
    let mut document = parse("[env.a]\ndeps = 1\nrunner = { of = 2, base = 3 }\n[env.b]\ndeps = 4\nrunner = 5\n");
    sections::reorder_tables_of(&mut document, &env_table, &["runner", "deps"], &[]);

    // the table a key holds is ordered by what the order names inside it, which here is nothing
    assert_eq!(
        document.to_string(),
        "[env.a]\nrunner = { of = 2, base = 3 }\ndeps = 1\n[env.b]\nrunner = 5\ndeps = 4\n"
    );
}

#[test]
fn the_keys_of_one_table_never_sort_against_another_table_written_beside_it() {
    // `held` names no environment, so nothing orders it against the keys of one
    let mut document = parse("held = 0\nenv.b.deps = 1\nenv.a.deps = 2\nenv.b.runner = 3\nenv.a.runner = 4\n");
    sections::reorder_tables_of(&mut document, &env_table, &["runner", "deps"], &[]);

    // each table sorts into the lines it already held, so interleaved tables stay interleaved
    assert_eq!(
        document.to_string(),
        "held = 0\nenv.b.runner = 3\nenv.a.runner = 4\nenv.b.deps = 1\nenv.a.deps = 2\n"
    );
}

#[test]
fn a_kept_order_holds_the_keys_of_the_table_it_names_in_place() {
    let mut document = parse("[env.a]\nset_env.B = 1\nset_env.A = 2\ndeps = 3\n");
    sections::reorder_tables_of(&mut document, &env_table, &["set_env", "deps"], &["set_env"]);

    assert_eq!(
        document.to_string(),
        "[env.a]\nset_env.B = 1\nset_env.A = 2\ndeps = 3\n"
    );
}

#[test]
fn an_alias_is_renamed_in_every_table_that_writes_it() {
    // `setenv` names a key of no environment at the root, and names a key of one inside `set_env`
    let mut document = parse("setenv = 0\n[env.a]\nsetenv = 1\nset_env.setenv = 4\n[env.b]\nsetenv = 2\nset_env = 3\n");
    let renamed = sections::rename_tables_of(&mut document, &env_table, &[("setenv", "set_env")]);

    assert_eq!(
        document.to_string(),
        "setenv = 0\n[env.a]\nset_env = 1\nset_env.setenv = 4\n[env.b]\nsetenv = 2\nset_env = 3\n"
    );
    assert_eq!(
        renamed,
        [(
            vec!["env".to_owned(), "a".to_owned()],
            "setenv".to_owned(),
            "set_env".to_owned()
        )]
    );
}

#[test]
fn a_table_of_names_sorts_on_both_sides() {
    let mut document = parse("[tool.x.extras]\nz = [ \"b\", \"a\" ]\na = [ \"d\", \"c\" ]\n");
    sections::sort_names_under(&mut document, "tool.x.extras");

    assert_eq!(
        written(&mut document),
        "[tool.x.extras]\na = [ \"c\", \"d\" ]\nz = [ \"a\", \"b\" ]\n"
    );
}

#[test]
fn a_table_of_names_the_file_never_wrote_is_left_alone() {
    let mut document = parse("[tool.x]\nz = 1\n");
    sections::sort_names_under(&mut document, "tool.x.extras");

    assert_eq!(document.to_string(), "[tool.x]\nz = 1\n");
}

/// A key that names the path exactly names nothing under it, so a rule reading what a table holds
/// passes over it.
#[test]
fn a_key_that_names_the_path_itself_holds_nothing_under_it() {
    let mut document = parse("env = { a = 2 }\n");
    let mut seen: Vec<String> = Vec::new();
    sections::for_key_paths_under(&mut document, &["env".to_owned()], |named, _| {
        seen.push(named.join("."));
    });

    assert_eq!(seen, ["env.a"]);
}

/// A rule about one table says nothing about the tables beside it, wherever the file wrote them.
#[test]
fn inline_tables_outside_the_named_one_are_left_alone() {
    let schemas = [sections::InlineSchema {
        discriminator: "replace",
        key_order: &["replace", "of"],
    }];
    let mut document = parse(
        "beside = { of = 1, replace = \"ref\" }\n[tool]\nheld = { of = 1, replace = \"ref\" }\n[other]\nheld = { of = 1, replace = \"ref\" }\n",
    );
    sections::reorder_inline_tables(&mut document, &["tool".to_owned()], &schemas);

    assert_eq!(
        written(&mut document),
        "beside = { of = 1, replace = \"ref\" }\n[tool]\nheld = { replace = \"ref\", of = 1 }\n[other]\nheld = { of = 1, replace = \"ref\" }\n"
    );
}

/// A table written over several keys is named once, whichever of its keys names it.
#[test]
fn a_table_named_by_several_keys_is_read_once() {
    let document = parse("[tool]\nx.a = 1\nx.b = 2\ny.c = 3\nplain = 4\n");

    assert_eq!(
        sections::keys_below(&document.sections[0].entries, &[]),
        ["x", "y", "plain"]
    );
}
