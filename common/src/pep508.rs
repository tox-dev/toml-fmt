use std::fmt::Write;
use std::str::FromStr;

use pep508_rs::pep440_rs::Operator::TildeEqual;
use pep508_rs::{Requirement, VerbatimUrl, VersionOrUrl};

pub fn format_requirement(value: &str, keep_full_version: bool) -> String {
    let req: Requirement<VerbatimUrl> = Requirement::from_str(value).unwrap();
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
    if req.marker.contents().is_some() {
        write!(&mut result, "; ").unwrap();
        write!(result, "{}", req.marker.try_to_string().unwrap()).unwrap();
    }
    result
}

pub fn get_canonic_requirement_name(value: &str) -> String {
    let req: Requirement<VerbatimUrl> = Requirement::from_str(value).unwrap();
    req.name.to_string()
}
