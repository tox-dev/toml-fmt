use crate::pep508::marker::MarkerExpr;
use crate::pep508::version_op::{Operator, VersionOp};
use regex::Regex;
use std::fmt::Write;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
enum VersionOrUrl {
    Versions(Vec<VersionOp>),
    Url(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Requirement {
    name: String,
    extras: Vec<String>,
    version_or_url: Option<VersionOrUrl>,
    marker: Option<MarkerExpr>,
    private: bool,
}

impl Requirement {
    pub fn new(raw: &str) -> Result<Self, String> {
        // Check for ; private suffix (PEP 794)
        let (raw_without_private, private) = if let Some(idx) = raw.rfind(';') {
            let suffix = raw[idx + 1..].trim();
            if suffix.eq_ignore_ascii_case("private") {
                (&raw[..idx], true)
            } else {
                (raw, false)
            }
        } else {
            (raw, false)
        };

        // Extract the marker (find ';' separator)
        let (raw_req, marker_start) = if let Some(idx) = raw_without_private.find(';') {
            (&raw_without_private[..idx], Some(idx + 1))
        } else {
            (raw_without_private, None)
        };

        // Extract URL from remaining (find "@" separator)
        let (req_part, url_start) = if let Some(idx) = raw_req.find("@") {
            (&raw_req[..idx], Some(idx + 1))
        } else {
            (raw_req, None)
        };

        // Split extras (inside [...]) and extract name
        let (name, extras) = if let Some(start) = req_part.find('[') {
            let end = req_part.find(']').ok_or("Unclosed extras bracket")?;
            let name = &req_part[..start];
            let extras = req_part[start + 1..end]
                .split(',')
                .map(|e| e.trim().to_string())
                .collect();
            (name, extras)
        } else {
            // No extras, but need to find where version specifiers start
            let name_end = req_part.find(|c: char| "=!<>~(".contains(c)).unwrap_or(req_part.len());
            let name = &req_part[..name_end];
            (name, vec![])
        };

        // Parse version specifiers into Vec<VersionOp>
        let version_or_url = if let Some(url_idx) = url_start {
            let url = &raw_req[url_idx..];
            Some(VersionOrUrl::Url(url.trim().to_string()))
        } else if let Some(specs_start) = req_part.find(|c: char| "=!<>~".contains(c)) {
            let specs_end = req_part.find(|c: char| ")".contains(c)).unwrap_or(req_part.len());
            let specs = &req_part[specs_start..specs_end];
            let mut parsed = Vec::new();
            for spec in specs.split(',') {
                match VersionOp::new(spec) {
                    Ok(version_op) => parsed.push(version_op),
                    Err(e) => return Err(format!("Invalid version specifier '{}': {}", spec, e)),
                }
            }
            Some(VersionOrUrl::Versions(parsed))
        } else {
            None
        };

        let marker = if let Some(marker_idx) = marker_start {
            let marker_str = raw_without_private[marker_idx..].trim();
            if marker_str.is_empty() {
                None
            } else {
                Some(MarkerExpr::new(marker_str).map_err(|e| format!("Invalid marker: {e}"))?)
            }
        } else {
            None
        };
        Ok(Requirement {
            name: name.trim().to_string(),
            extras,
            version_or_url,
            marker,
            private,
        })
    }

    pub fn normalize(mut self, keep_full_version: bool) -> Self {
        self.name = self.canonical_name();
        if !keep_full_version {
            if let Some(VersionOrUrl::Versions(ref mut specs)) = self.version_or_url {
                for version_op in specs.iter_mut() {
                    // Only strip trailing .0 if there are no pre, post, dev, or local segments
                    if version_op.op != Operator::Compatible
                        && !version_op.version.has_wildcard
                        && version_op.version.pre.is_none()
                        && version_op.version.post.is_none()
                        && version_op.version.dev.is_none()
                        && version_op.version.local.is_none()
                    {
                        while version_op.version.release.len() > 1 && *version_op.version.release.last().unwrap() == 0 {
                            version_op.version.release.pop();
                        }
                    }
                }
            }
        }
        self
    }

    pub fn canonical_name(&self) -> String {
        Regex::new(r"[-_.]+")
            .unwrap()
            .replace_all(&self.name.to_lowercase(), "-")
            .into_owned()
    }
}

impl FromStr for Requirement {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Requirement::new(s).map(|req| req.normalize(false))
    }
}

impl std::fmt::Display for Requirement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = self.name.clone();
        if !self.extras.is_empty() {
            let mut extras_sorted = self.extras.clone();
            extras_sorted.sort();
            write!(&mut result, "[")?;
            let extra_count = extras_sorted.len() - 1;
            for (at, extra) in extras_sorted.iter().enumerate() {
                write!(&mut result, "{extra}")?;
                if extra_count != at {
                    write!(&mut result, ",")?;
                }
            }
            write!(&mut result, "]")?;
        }
        if let Some(version_or_url) = &self.version_or_url {
            match version_or_url {
                VersionOrUrl::Versions(v) => {
                    let extra_count = v.len() - 1;
                    for (at, version_op) in v.iter().enumerate() {
                        write!(&mut result, "{}", version_op)?;
                        if extra_count != at {
                            write!(&mut result, ",")?;
                        }
                    }
                }
                VersionOrUrl::Url(u) => {
                    write!(&mut result, " @ {u}")?;
                }
            }
        }
        if let Some(marker) = &self.marker {
            write!(&mut result, "; {marker}")?;
        }
        if self.private {
            write!(&mut result, "; private")?;
        }
        write!(f, "{}", result)
    }
}
