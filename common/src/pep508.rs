use std::fmt::Write;
use std::str::FromStr;

use pep508_rs::pep440_rs::Operator::TildeEqual;
use pep508_rs::{MarkerTree, Requirement, VersionOrUrl};

pub fn format_requirement(value: &str, keep_full_version: bool) -> String {
    let req = Requirement::from_str(value).unwrap();
    let mut result = req.name.to_string();
    if !req.extras.is_empty() {
        write!(&mut result, "[").unwrap();
        let extra_count = req.extras.len() - 1;
        for (at, extra) in req.extras.iter().enumerate() {
            write!(&mut result, "{extra}").unwrap();
            if extra_count != at {
                write!(&mut result, ",").unwrap();
            }
        }
        write!(&mut result, "]").unwrap();
    }
    if let Some(version_or_url) = req.version_or_url {
        match version_or_url {
            VersionOrUrl::VersionSpecifier(v) => {
                let extra_count = v.len() - 1;
                for (at, spec) in v.iter().enumerate() {
                    let mut spec_repr = format!("{spec}");
                    if !keep_full_version && spec.operator() != &TildeEqual {
                        loop {
                            let propose = spec_repr.strip_suffix(".0");
                            if propose.is_none() {
                                break;
                            }
                            spec_repr = propose.unwrap().to_string();
                        }
                    }
                    write!(&mut result, "{spec_repr}").unwrap();
                    if extra_count != at {
                        write!(&mut result, ",").unwrap();
                    }
                }
            }
            VersionOrUrl::Url(u) => {
                write!(&mut result, " @ {u}").unwrap();
            }
        }
    }
    if let Some(marker) = req.marker {
        write!(&mut result, "; ").unwrap();
        handle_marker(&marker, &mut result, false);
    }

    result
}

fn handle_marker(marker: &MarkerTree, result: &mut String, nested: bool) {
    match marker {
        MarkerTree::Expression(e) => {
            write!(result, "{}{}{}", e.l_value, e.operator, e.r_value).unwrap();
        }
        MarkerTree::And(a) => {
            handle_tree(result, nested, a, " and ");
        }
        MarkerTree::Or(a) => {
            handle_tree(result, nested, a, " or ");
        }
    }
}

fn handle_tree(result: &mut String, nested: bool, elements: &[MarkerTree], x: &str) {
    let len = elements.len() - 1;
    if nested && len > 0 {
        write!(result, "(").unwrap();
    }
    for (at, e) in elements.iter().enumerate() {
        handle_marker(e, result, true);
        if at != len {
            write!(result, "{x}").unwrap();
        }
    }
    if nested && len > 0 {
        write!(result, ")").unwrap();
    }
}

pub fn get_canonic_requirement_name(value: &str) -> String {
    let req = Requirement::from_str(value).unwrap();
    req.name.to_string()
}
