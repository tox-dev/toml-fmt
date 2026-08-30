use crate::pep508::marker::MarkerExpr;
use crate::pep508::version_op::{Number, Operator, VersionOp};
use regex::Regex;
use std::str::FromStr;
use std::sync::LazyLock;

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
}

/// Where the semicolon that opens an environment marker sits.
///
/// A URL holds no whitespace and may hold a semicolon of its own, so what separates a direct
/// reference from its marker is the space PEP 508 writes before it.
fn marker_break(raw: &str) -> Option<usize> {
    raw.char_indices()
        .find(|(at, held)| *held == ';' && (!raw[..*at].contains('@') || raw[..*at].ends_with(char::is_whitespace)))
        .map(|(at, _)| at)
}

/// The clauses a requirement names, with the parentheses PEP 508 allows around them.
///
/// A parenthesis opens a list and the matching one closes it; a lone one is text this parser cannot
/// read, and dropping it would write a requirement the file does not say.
fn read_specifiers(text: &str) -> Result<Vec<VersionOp>, String> {
    let inside = match text.strip_prefix('(') {
        Some(rest) => rest
            .strip_suffix(')')
            .ok_or_else(|| format!("Unclosed parenthesis: '{text}'"))?
            .trim(),
        None => text,
    };
    // a list may close with a comma, which names no clause of its own
    let inside = inside.strip_suffix(',').map_or(inside, str::trim_end);
    if inside.is_empty() || inside.contains(['(', ')']) {
        return Err(format!("The requirement names no version: '{text}'"));
    }
    inside
        .split(',')
        .map(|spec| VersionOp::new(spec).map_err(|why| format!("Invalid version specifier '{spec}': {why}")))
        .collect()
}

/// What a name writes between its words, all of which compare alike.
static SEPARATORS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[-_.]+").expect("static pattern"));

/// Whether the text names a distribution the way PEP 508 spells one: letters, digits and the
/// separators between them, opening and closing on one of the first two.
#[must_use]
pub fn names_a_distribution(name: &str) -> bool {
    name.starts_with(|held: char| held.is_ascii_alphanumeric())
        && name.ends_with(|held: char| held.is_ascii_alphanumeric())
        && name
            .chars()
            .all(|held| held.is_ascii_alphanumeric() || "._-".contains(held))
}

impl Requirement {
    pub fn new(raw: &str) -> Result<Self, String> {
        // a dependency is written on one line, so a break inside it is text no reader accepts
        if raw.contains(['\n', '\r']) {
            return Err(String::from("A requirement is written on one line"));
        }
        let (raw_req, marker_start) = match marker_break(raw) {
            Some(at) => (&raw[..at], Some(at + 1)),
            None => (raw, None),
        };

        let (req_part, url_start) = if let Some(idx) = raw_req.find("@") {
            (&raw_req[..idx], Some(idx + 1))
        } else {
            (raw_req, None)
        };

        // what follows the name, or the extras, is the versions it names and nothing else: text this
        // parser cannot read must not be dropped by reading only the part it can
        let (name, extras, constraints) = if let Some(start) = req_part.find('[') {
            let end = req_part[start..]
                .find(']')
                .map(|at| start + at)
                .ok_or("Unclosed extras bracket")?;
            let inside = req_part[start + 1..end].trim();
            // a list written empty names no extras, which is what leaving the brackets off says
            let extras: Vec<String> = if inside.is_empty() {
                Vec::new()
            } else {
                inside.split(',').map(|extra| extra.trim().to_string()).collect()
            };
            // an extra names a distribution, so text that does not is text this parser cannot read
            if !extras.iter().all(|extra| names_a_distribution(extra)) {
                return Err(format!("Invalid extras: '{}'", &req_part[start..=end]));
            }
            (&req_part[..start], extras, req_part[end + 1..].trim())
        } else {
            let name_end = req_part.find(|c: char| "=!<>~(".contains(c)).unwrap_or(req_part.len());
            (&req_part[..name_end], vec![], req_part[name_end..].trim())
        };
        if !names_a_distribution(name.trim()) {
            return Err(format!("Invalid name '{}'", name.trim()));
        }

        let version_or_url = if let Some(url_idx) = url_start {
            // a dependency names a version or a direct reference, never both, so text before the
            // `@` that is not the name is not something to drop
            if !constraints.is_empty() {
                return Err(format!("The requirement names a version and a URL: '{raw}'"));
            }
            let url = raw_req[url_idx..].trim();
            // a URL holds no whitespace, so what follows one is text this parser cannot read: a
            // marker written without the space PEP 508 asks for is left to the caller as written
            if url.contains(char::is_whitespace) {
                return Err(format!("Unexpected text after the URL: '{url}'"));
            }
            if url.is_empty() {
                return Err(String::from("The direct reference names no URL"));
            }
            Some(VersionOrUrl::Url(url.to_string()))
        } else if constraints.is_empty() {
            None
        } else {
            Some(VersionOrUrl::Versions(read_specifiers(constraints)?))
        };

        let marker = match marker_start {
            // `;` opens a marker, so a delimiter with nothing after it names one this cannot read
            Some(marker_idx) => {
                Some(MarkerExpr::new(raw[marker_idx..].trim()).map_err(|why| format!("Invalid marker: {why}"))?)
            }
            None => None,
        };
        Ok(Requirement {
            name: name.trim().to_string(),
            extras,
            version_or_url,
            marker,
        })
    }

    pub fn normalize(mut self, keep_full_version: bool) -> Self {
        self.name = self.canonical_name();
        if !keep_full_version && let Some(VersionOrUrl::Versions(ref mut specs)) = self.version_or_url {
            for version_op in specs.iter_mut() {
                // `~=` says what it says by the numbers it names, and trailing `.0` is only
                // redundant without pre/post/dev/local segments
                if version_op.op == Operator::Compatible {
                    continue;
                }
                // `===` is written from the text it was given, whatever this makes of it
                let Some(version) = version_op.version_mut() else {
                    continue;
                };
                if version.has_wildcard
                    || version.pre.is_some()
                    || version.post.is_some()
                    || version.dev.is_some()
                    || version.local.is_some()
                {
                    continue;
                }
                while version.release.len() > 1 && version.release.last().is_some_and(Number::is_zero) {
                    version.release.pop();
                }
            }
        }
        self
    }

    /// The version constraints the requirement names, if it names any.
    #[must_use]
    pub fn version_ops(&self) -> &[VersionOp] {
        match &self.version_or_url {
            Some(VersionOrUrl::Versions(specs)) => specs,
            _ => &[],
        }
    }

    pub fn is_name_only(&self) -> bool {
        self.extras.is_empty() && self.version_or_url.is_none() && self.marker.is_none()
    }

    /// The canonical spelling of a distribution name, which is what a name on its own is: `pkg[x]`
    /// or `pkg>=1` names a dependency, and neither is a name.
    ///
    /// # Errors
    ///
    /// Returns why the text does not name a distribution.
    pub fn canonical_name_of(name: &str) -> Result<String, String> {
        if !names_a_distribution(name.trim()) {
            return Err(format!("Invalid name '{}'", name.trim()));
        }
        Ok(SEPARATORS.replace_all(&name.trim().to_lowercase(), "-").into_owned())
    }

    pub fn canonical_name(&self) -> String {
        SEPARATORS.replace_all(&self.name.to_lowercase(), "-").into_owned()
    }
}

impl FromStr for Requirement {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Requirement::new(s).map(|req| req.normalize(false))
    }
}

impl Requirement {
    /// The requirement as PEP 508 spells one, with its extras and versions in order.
    fn spelled(&self) -> String {
        let mut written = self.name.clone();
        if !self.extras.is_empty() {
            let mut extras = self.extras.clone();
            extras.sort();
            written.push('[');
            written.push_str(&extras.join(","));
            written.push(']');
        }
        match &self.version_or_url {
            Some(VersionOrUrl::Versions(versions)) => {
                let spelled: Vec<String> = versions.iter().map(ToString::to_string).collect();
                written.push_str(&spelled.join(","));
            }
            Some(VersionOrUrl::Url(url)) => {
                written.push_str(" @ ");
                written.push_str(url);
            }
            None => {}
        }
        if let Some(marker) = &self.marker {
            // PEP 508 requires whitespace after a URL, otherwise the `;` is parsed as part of the URI
            written.push_str(if matches!(self.version_or_url, Some(VersionOrUrl::Url(_))) {
                " ; "
            } else {
                "; "
            });
            written.push_str(&marker.to_string());
        }
        written
    }
}

impl std::fmt::Display for Requirement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.spelled())
    }
}
